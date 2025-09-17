use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use sha2::{Digest, Sha256};

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    pub fn new(key: &[u8]) -> Result<Self> {
        let key = Self::derive_key(key);
        let cipher = Aes256Gcm::new(&key);
        Ok(Self { cipher })
    }

    fn derive_key(input: &[u8]) -> Key<Aes256Gcm> {
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        *Key::<Aes256Gcm>::from_slice(&result)
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let mut result = Vec::new();
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(anyhow::anyhow!("Invalid ciphertext"));
        }

        let (nonce, encrypted) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce);

        self.cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let crypto = Crypto::new(b"test-key-12345").unwrap();
        let plaintext = b"Hello, World!";

        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }
}