use std::fs::OpenOptions;
use std::path::PathBuf;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use chacha20poly1305::AeadCore;
use hmac_sha256;
use rand::rngs::OsRng;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use crate::input;
use crate::resource;

pub const DELIMITER:         &str = "--||--";

const DEFAULT_DIR_NAME:      &str = ".onepass";
const DEFAULT_FILE_NAME:     &str = "main.txt";

pub struct GetResponse {
    pub copied: bool,
    pub instance: resource::Instance,
}

pub fn update(
    custom: Option<&str>,
    password: &str,
    name: &str,
    key: &str,
    val: &str,
) -> Result<(), String> {
    let mut f = match open(custom) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };

    let data = match extract_data(&mut f) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    let decrypted_content = decrypt(
        password,
        data.buf,
        data.nonce,
    )?;

    let mut data = vec![];
    let mut target_idx: usize = 0;
    let lines: Vec<&str> = decrypted_content.lines().collect();
    for (i, _) in lines.iter().enumerate() {
        if lines[i] == "\n" {
            continue
        }
        if lines[i] == input::RESERVED_RESOURCE && lines[i+1] == name {
            match key {
                resource::NAME     => {target_idx = i+1},
                resource::USER     => {target_idx = i+2},
                resource::PASSWORD => {target_idx = i+3},
                _                  => { return Err("bad resource key".to_string()) },
            }
        }
        if target_idx != 0 && i == target_idx {
            data.push(String::from(val));
        } else {
            data.push(String::from(lines[i]));
        }
    }
    if target_idx == 0 {
        return Err("resource not found".to_string())
    }
    let new_nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let encrypted_content = encrypt(password, &data.join("\n"), new_nonce)?;
    let mut truncated_file = match open_truncate(custom) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    if let Err(err) = truncated_file.write_all(&new_nonce.to_vec()) {
        return Err(err.to_string())
    };
    if let Err(err) = truncated_file.write_all(&encrypted_content) {
        return Err(err.to_string())
    };
    Ok(())
}

pub fn list(custom: Option<&str>, password: &str) -> Result<Vec<String>, String> {
    let mut f = match open(custom) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    let data = match extract_data(&mut f) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    let decrypted_content = decrypt(password, data.buf, data.nonce)?;
    let lines: Vec<&str> = decrypted_content.lines().collect();
    let mut result: Vec<String> = vec![];
    for (i, _) in decrypted_content.lines().into_iter().enumerate(){
        if lines[i] == input::RESERVED_RESOURCE {
            result.push(String::from(lines[i+1]))
        }
    }
    Ok(result)
}

pub fn get(custom: Option<&str>, name: &str, pw: &str) -> Result<GetResponse, String> {
    let mut f = match open(custom) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    let data = match extract_data(&mut f) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    let decrypted_content = decrypt(
        pw,
        data.buf,
        data.nonce,
    )?;

    let mut ctx: ClipboardContext = match ClipboardProvider::new() {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };
    let (user, password): (String, String);
    let lines: Vec<&str> = decrypted_content.lines().collect();
    for (i, _) in lines.iter().enumerate() {
        println!("{}", lines[i]);
        if lines[i] == input::RESERVED_RESOURCE && lines[i+1] == name {
            if let (Some(next), Some(next_next)) = (lines.get(i + 2), lines.get(i + 3)) {
                let mut copied = true;
                user = next.to_string();
                password = next_next.to_string();
                if let Err(err) = ctx.set_contents(password.to_owned()) {
                    println!("copying to clipboard: {}", err);
                    copied = false;
                };
                return Ok(
                    GetResponse {
                        instance: resource::Instance{
                            name: name.to_string(),
                            user,
                            password,
                        },
                        copied,
                    }
                );
            } else {
                return Err("uncomplete resource is less than 3 lines".to_string());
            }
        }
    }
    return Err("resource not found".to_string())
}

pub fn write(custom: Option<&str>, password: &str, r: resource::Instance) -> Result<(), String> {
    let mut f = match open(custom) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    let data = match extract_data(&mut f) { 
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    let mut content = decrypt(password, data.buf, data.nonce)?;
    for line in content.lines() {
        if line.trim() == r.name.trim() {
            return Err("resource already exists".to_string())
        }
    }

    let mut truncated = match open_truncate(custom) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    if let Err(err) = truncated.write_all(&nonce.to_vec()) {
        return Err(err.to_string())
    };
    content.push_str("\n");
    content.push_str(&r.to_string());
    let encrypted = encrypt(password, &content, nonce)?;
    if let Err(err) = truncated.write_all(&encrypted) {
        return Err(err.to_string())
    };
    Ok(())
}

pub fn bootstrap(custom: Option<&str>, password: &str) -> Result<(), String> {
    let mut f = match create(custom) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };

    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    if let Err(err) = f.write_all(&nonce.to_vec()) {
        return Err(err.to_string())
    };
    let mut content = String::from("\n");
    content.push_str(DELIMITER);
    content.push_str("\n");

    let encrypted_content = encrypt(password, &content, nonce)?;
    if let Err(err) = f.write_all(&encrypted_content) {
        return Err(err.to_string())
    };
    Ok(())
}

pub fn purge(custom: Option<&str>) -> io::Result<()> {
    std::fs::remove_file(file_path(custom)) 
}

pub fn file_path(custom: Option<&str>) -> PathBuf {
    let home_dir = env::var("HOME").unwrap();
    let mut path = PathBuf::from(home_dir);
    if let Some(c) = custom {
        path.push(c);
    } else {
        path.push(DEFAULT_DIR_NAME);
        path.push(DEFAULT_FILE_NAME);
    }
    path
}

/// Create the needed file to init the engine.
/// The path can be adjusted with parameters.
pub fn create(custom: Option<&str>) -> io::Result<std::fs::File> {
    let path = file_path(custom);

    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)?;

    Ok(file)
}

pub fn open(custom: Option<&str>) -> io::Result<std::fs::File> {
    let path = file_path(custom);

    let file = OpenOptions::new()
        .read(true)
        .open(&path)?;

    Ok(file)
}

pub fn open_append(custom: Option<&str>) -> io::Result<std::fs::File> {
    let path = file_path(custom);

    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(&path)?;

    Ok(file)
}

pub fn open_truncate(custom: Option<&str>) -> io::Result<std::fs::File> {
    let path = file_path(custom);

    let file = OpenOptions::new()
        .read(true)
        .truncate(true)
        .write(true)
        .open(&path)?;

    Ok(file)
}

pub fn exists(custom: Option<&str>) -> bool {
    let path = file_path(custom);

    if let Err(err) = OpenOptions::new().read(true).open(&path) {
        if err.kind() == io::ErrorKind::NotFound {
            return false;
        } else {
            return true;
        }
    };
    true
}

pub struct Data {
    pub nonce: Nonce,
    pub buf: Vec<u8>,
}

pub fn extract_data(f: &mut File) -> Result<Data, io::Error> {
    // 12-byte buffer for the nonce.
    let mut nonce_buf = [0u8; 12];
    f.read_exact(&mut nonce_buf)?;
    let nonce = Nonce::from_slice(&nonce_buf).clone();

    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    Ok(Data { nonce, buf })
}

pub fn encrypt(key: &str, content: &str, nonce: chacha20poly1305::Nonce) -> Result<Vec<u8>, String>{
    let h = hmac_sha256::Hash::hash(key.as_bytes());

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
    Ok(ciphertext)
}

pub fn decrypt(key: &str, content: Vec<u8>, nonce: chacha20poly1305::Nonce) -> Result<String, String> {
    let h = hmac_sha256::Hash::hash(key.as_bytes());

    let cipher = match ChaCha20Poly1305::new_from_slice(&h) {
        Ok(c) => c,
        Err(err) => {
            return Err(err.to_string());
        }
    };
    let plaintext = match cipher.decrypt(&nonce, content.as_ref()){
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };
    match std::str::from_utf8(&plaintext) {
        Ok(v) => return Ok(v.to_string()),
        Err(err) => return Err(err.to_string()),
    };
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
    fn encrypt_decrypt() {
        let content = "content\ndelimiter\nsecret-stuff";
        let key = "masterPassword";
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        let encrypted_content = encrypt(key, content, nonce).expect("encrypting");
        let decrypted_content = decrypt(
            key,
            encrypted_content,
            nonce,
        ).expect("decrypting");

        assert_eq!(content, decrypted_content);
    }

    #[test]
    fn test_bootstrap() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup{file_name: id.to_string()};
        let t_path = &cleanup.path();
        let master_password = "my_master_pw";
        bootstrap(Some(t_path), master_password).expect("bootstrapping");
    }

    #[test]
    fn test_get() {
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
        let result = get(Some(t_path), &name, master_password).expect("getting");

        assert_eq!(name, result.instance.name);
        assert_eq!(user, result.instance.user);
        assert_eq!(password, result.instance.password);
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
}
