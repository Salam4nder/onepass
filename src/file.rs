use std::fs::OpenOptions;
use std::path::PathBuf;
use std::env;
use chacha20poly1305;
use std::fs::File;
use std::io::{self, Read};
use chacha20poly1305::Nonce;
use hmac_sha256;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce
};

pub const DELIMITER:         &str = "--||--";

const DEFAULT_DIR_NAME:      &str = ".onepass";
const DEFAULT_FILE_NAME:     &str = "main.txt";

pub fn path() -> PathBuf {
    let home_dir = env::var("HOME").unwrap();
    let mut path = PathBuf::from(home_dir);
    path.push(DEFAULT_DIR_NAME);
    path.push(DEFAULT_FILE_NAME);
    path
}

pub fn dir_path() -> PathBuf {
    let home_dir = env::var("HOME").unwrap();
    let mut path = PathBuf::from(home_dir);
    path.push(DEFAULT_DIR_NAME);
    path
}

pub fn create() -> io::Result<std::fs::File> {
    let path = path();

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
    let path = path();

    let file = OpenOptions::new()
        .read(true)
        .open(&path)?;

    Ok(file)
}

pub fn open_append() -> io::Result<std::fs::File> {
    let path = path();

    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(&path)?;

    Ok(file)
}

pub fn open_write() -> io::Result<std::fs::File> {
    let path = path();

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)?;

    Ok(file)
}

pub fn open_truncate() -> io::Result<std::fs::File> {
    let path = path();

    let file = OpenOptions::new()
        .read(true)
        .truncate(true)
        .write(true)
        .open(&path)?;

    Ok(file)
}

pub fn exists() -> bool {
    let path = path();

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
