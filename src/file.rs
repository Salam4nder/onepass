use std::env;
use std::str;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write, Seek};

use hmac_sha256;
use rand::rngs::OsRng;
use chacha20poly1305::AeadCore;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};

pub const DEFAULT_DIR_NAME:      &str = ".onepass";
pub const DEFAULT_FILE_NAME:     &str = "main.txt";

pub fn purge(custom: Option<&str>) -> io::Result<()> {
    std::fs::remove_file(path(custom)) 
}

/// Create the needed file for the application.
/// The path can be adjusted with parameters.
pub fn create(custom_path: Option<&str>) -> io::Result<std::fs::File> {
    let ver = "0.3";
    let path = path(custom_path);

    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)?;
    
    match custom_path {
        Some(p) => {println!("Initialized file at {}", p)},
        None    => {println!("Initialized file at ~/{}/{} {}", DEFAULT_DIR_NAME, DEFAULT_FILE_NAME, ver)}
    }

    Ok(file)
}

pub fn open_truncate(custom: Option<&str>) -> io::Result<std::fs::File> {
    let path = path(custom);

    let file = OpenOptions::new()
        .read(true)
        .truncate(true)
        .write(true)
        .open(&path)?;

    Ok(file)
}

pub fn exists(custom: Option<&str>) -> bool {
    let path = path(custom);
    Path::new(&path).exists()
}

pub fn encrypt(custom_path: Option<&str>, password: &str, content: String) -> Result<Vec<u8>, String>{
    let h = hmac_sha256::Hash::hash(password.as_bytes());

    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let cipher = match ChaCha20Poly1305::new_from_slice(&h) {
        Ok(c) => c,
        Err(err) => {
            return Err(err.to_string());
        }
    };
    let ciphertext = match cipher.encrypt(&nonce, content.as_ref()) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };

    let mut f = match open_truncate(custom_path) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    if let Err(err) = f.write_all(nonce.as_slice()) {
        return Err(err.to_string())
    }
    if let Err(err) = f.write_all(ciphertext.as_slice()) {
        return Err(err.to_string())
    }

    Ok(ciphertext)
}

pub fn decrypt(path: Option<&str>, password: &str) -> Result<String, String> {
    let h = hmac_sha256::Hash::hash(password.as_bytes());

    let cipher = match ChaCha20Poly1305::new_from_slice(&h) {
        Ok(c) => c,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mut f = match open(path) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    let data = match extract_data(&mut f) { 
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    let plaintext = match cipher.decrypt(&data.nonce, data.buf.as_ref()){
        Ok(v) => v,
        Err(err) => {
            let err_str = err.to_string();
            if err_str == "aead::Error" {
                return Err(String::from("Incorrect password - aborting."))
            } else {
                return Err(err_str)
            }
        }
    };
    match std::str::from_utf8(&plaintext) {
        Ok(v) => {
            let lines: Vec<_> = v.lines().collect();
            for v in lines {
                println!("{}", v)
            }
            return Ok(v.to_string())
        },
        Err(err) => return Err(err.to_string()),
    };
}

pub fn path(custom_path: Option<&str>) -> PathBuf {
    let home_dir = env::var("HOME").unwrap();
    let mut path = PathBuf::from(home_dir);
    if let Some(c) = custom_path {
        path.push(c);
    } else {
        path.push(DEFAULT_DIR_NAME);
        path.push(DEFAULT_FILE_NAME);
    }
    path
}

pub struct Data {
    pub nonce: Nonce,
    pub buf: Vec<u8>,
}

fn extract_data(f: &mut File) -> Result<Data, io::Error> {
    let mut nonce_buf = [0u8; 12];
    f.read_exact(&mut nonce_buf)?;

    let nonce = Nonce::from_slice(&nonce_buf).clone();

    f.seek(SeekFrom::Start(12))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    Ok(Data { nonce, buf })
}

fn open(custom: Option<&str>) -> io::Result<std::fs::File> {
    let path = path(custom);

    let file = OpenOptions::new()
        .read(true)
        .open(&path)?;

    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    struct Cleanup {file_name: String}

    impl Cleanup {
        fn path(&self) -> String {
            format!("{}/{}.txt", DEFAULT_DIR_NAME, self.file_name).as_str().to_string()
        }
    }

    impl Drop for Cleanup {
        fn drop(&mut self) {
            let file_path = format!("{}/{}.txt", DEFAULT_DIR_NAME, self.file_name);
            purge(Some(&file_path)).expect("cleaning up");
        }
    }

    #[test]
    fn test_create() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup{file_name: id.to_string()};
        let t_path = &cleanup.path();
        create(Some(t_path)).expect("creating");
    }

    #[test]
    fn test_extract() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup{file_name: id.to_string()};
        let t_path = &cleanup.path();
        create(Some(t_path)).expect("creating");
        let c = "content\ndelimiter\nsecret-stuff\n";
        encrypt(Some(t_path), "master_pw", c.to_string()).expect("encrypting");

        let mut o = open(Some(t_path)).expect("opening");
        let data = extract_data(&mut o).expect("extracting");

        assert!(data.buf.len() > 0);
        assert_eq!(data.nonce.len(), 12);
    }

    #[test]
    fn test_list() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup{file_name: id.to_string()};
        let t_path = &cleanup.path();

        let master_password = "my_master_pw";
        bootstrap(Some(t_path), master_password).expect("bootstrapping");

        let mut names = vec![];
        for i in 0..100 {
            let name = format!("{}-name", i);
            let user = format!("{}-user", i);
            let password = format!("{}-password", i);
            write(
                Some(t_path),
                master_password,
                resource::Instance{
                    name: name.clone(),
                    user,
                    password,
                },
            ).expect("writing");
            names.push(name);
        }

        let result = list(Some(t_path), master_password).expect("listing");
        for v in result {
            assert!(names.contains(&v))
        }
    }

    #[test]
    fn test_update() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup{file_name: id.to_string()};
        let t_path = &cleanup.path();

        let master_password = "my_master_pw";
        bootstrap(Some(t_path), master_password).expect("bootstrapping");

        let name = "twitter";
        let user = "user@mail.com";
        let password = "password";
        write(
            Some(t_path),
            master_password,
            resource::Instance{
                name: name.to_string(),
                user: user.to_string(),
                password: password.to_string(),
            },
        ).expect("writing");
        let result = get(Some(t_path), master_password, &name).expect("getting");
        assert_eq!(name, result.resource.name);
        assert_eq!(user, result.resource.user);
        assert_eq!(password, result.resource.password);

        let new_val: &str = "epic";
        if let Err(err) = update(Some(t_path), master_password, &name, resource::NAME, new_val) {
            panic!("updating: {}", err)
        }
        let result = get(Some(t_path), master_password, &new_val).expect("getting");
        assert_eq!(new_val, result.resource.name);
        assert_eq!(password, result.resource.password);
    }

    #[test]
    fn test_del() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup{file_name: id.to_string()};
        let t_path = &cleanup.path();

        let master_password = "my_master_pw";
        bootstrap(Some(t_path), master_password).expect("bootstrapping");

        let name = "twitter".to_string();
        let user = "user@mail.com";
        let password = "password";
        write(
            Some(t_path),
            master_password,
            resource::Instance{
                name: name.clone(),
                user: user.to_string(),
                password: password.to_string(),
            },
        ).expect("writing");
        let result = get(Some(t_path), master_password, &name).expect("getting");
        if let Err(err) = delete(Some(t_path), master_password, &result.resource.name) {
            panic!("{}", err)
        }

        let l = list(Some(t_path), master_password).expect("listing");
        assert_eq!(0, l.len())
    }
}
