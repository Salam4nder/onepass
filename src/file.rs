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

use crate::resource;

pub const DELIMITER:         &str = "--||--";

const DEFAULT_DIR_NAME:      &str = ".onepass";
const DEFAULT_FILE_NAME:     &str = "main.txt";

/// TEST_DIR_NAME is used in tests.
const TEST_DIR_NAME:      &str = ".onepass_test";
/// TEST_FILE_NAME is used in tests.
const TEST_FILE_NAME:      &str = ".main_test.txt";

/// OpParams can be used to alter the default file path on basic
/// file operations. Can be created with `OpParams::default() otherwise` 
pub struct OpParams {
    pub testing: bool,
    pub custom_path: Option<String>
}

impl Default for OpParams {
    fn default() -> OpParams {
        OpParams { testing: false, custom_path: None }
    }
}

// pub fn write(params: OpParams, pw: String, r: resource::Instance) {
// }

pub fn bootstrap(params: OpParams, pw: String) -> Result<(), String> {
    let mut f = match create(params) {
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

    let encrypted_content = encrypt(&pw, &content, nonce)?;
    if let Err(err) = f.write_all(&encrypted_content) {
        return Err(err.to_string())
    };
    Ok(())
}

pub fn purge(params: OpParams) -> io::Result<()> {
    std::fs::remove_dir_all(dir_path(params.testing)) 
}

pub fn file_path(test: bool) -> PathBuf {
    let home_dir = env::var("HOME").unwrap();
    let mut path = PathBuf::from(home_dir);
    if test {
        path.push(TEST_DIR_NAME);
        path.push(TEST_FILE_NAME);
    } else {
        path.push(DEFAULT_DIR_NAME);
        path.push(DEFAULT_FILE_NAME);
    }
    path
}

pub fn dir_path(test: bool) -> PathBuf {
    let home_dir = env::var("HOME").unwrap();
    let mut path = PathBuf::from(home_dir);
    if test {
        path.push(TEST_DIR_NAME);
    } else {
        path.push(DEFAULT_DIR_NAME);
    }
    path
}

/// Create the needed file to init the engine.
/// The path can be adjusted with parameters.
pub fn create(params: OpParams) -> io::Result<std::fs::File> {
    let path = file_path(params.testing);

    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)?;

    Ok(file)
}

pub fn open() -> io::Result<std::fs::File> {
    let path = file_path(/* test */ false);

    let file = OpenOptions::new()
        .read(true)
        .open(&path)?;

    Ok(file)
}

pub fn open_append() -> io::Result<std::fs::File> {
    let path = file_path(/* test */ false);

    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(&path)?;

    Ok(file)
}

pub fn open_truncate() -> io::Result<std::fs::File> {
    let path = file_path(/* test */ false);

    let file = OpenOptions::new()
        .read(true)
        .truncate(true)
        .write(true)
        .open(&path)?;

    Ok(file)
}

pub fn exists() -> bool {
    let path = file_path(/* test */ false);

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
    use chacha20poly1305::AeadCore;
    use rand::rngs::OsRng;

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
}
