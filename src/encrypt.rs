use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce
};
use hmac_sha256;

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
