use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce
};
use hmac_sha256;

struct EncryptedContent {
    content: Vec<u8>,
    nonce:  chacha20poly1305::Nonce,
}

impl EncryptedContent {
    fn new(content: Vec<u8>, nonce: chacha20poly1305::Nonce) -> EncryptedContent {
        EncryptedContent { content , nonce }
    }
}

pub fn encrypt(key: &str, content: &str) -> Result<EncryptedContent, String>{
    let h = hmac_sha256::Hash::hash(key.as_bytes());

    let cipher = match ChaCha20Poly1305::new_from_slice(&h) {
        Ok(c) => c,
        Err(err) => {
            println!("got fucked, {}", err.to_string());
            return Err(err.to_string());
        }
    };
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = match cipher.encrypt(&nonce, content.as_ref()) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };
    Ok(EncryptedContent::new(ciphertext, nonce))
}

pub fn decrypt(key: &str, content: Vec<u8>, nonce: chacha20poly1305::Nonce) -> Result<String, String> {
    let h = hmac_sha256::Hash::hash(key.as_bytes());

    let cipher = match ChaCha20Poly1305::new_from_slice(&h) {
        Ok(c) => c,
        Err(err) => {
            println!("got fucked, {}", err.to_string());
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
        let content = "lmao, epic!";
        let key = "masterPassword";

        let encrypted_content = encrypt(key, content).expect("encrypting");
        println!("{}", encrypted_content);
        let decrypted_content = decrypt(
            key,
            encrypted_content.content,
            encrypted_content.nonce,
        ).expect("decrypting");

        assert_eq!(content, decrypted_content);
    }
}
