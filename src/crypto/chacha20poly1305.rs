use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};

use crate::crypto::traits::{Cipher, EncryptionResult};
use crate::error::{Result, RusWaCipherError};

pub struct ChaCha20Poly1305Cipher {
    cipher: ChaCha20Poly1305,
}

impl ChaCha20Poly1305Cipher {
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(RusWaCipherError::KeyManagement(format!(
                "ChaCha20-Poly1305 requires a 32-byte key, got {} bytes",
                key.len()
            )));
        }

        let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|e| {
            RusWaCipherError::Encryption(format!(
                "Failed to create ChaCha20-Poly1305 cipher: {:?}",
                e
            ))
        })?;

        Ok(Self { cipher })
    }
}

impl Cipher for ChaCha20Poly1305Cipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult> {
        // Generate a random nonce
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        // Encrypt the data
        let ciphertext = self.cipher.encrypt(&nonce, plaintext).map_err(|e| {
            RusWaCipherError::Encryption(format!("ChaCha20-Poly1305 encryption failed: {:?}", e))
        })?;

        Ok(EncryptionResult {
            iv: nonce.to_vec(),
            ciphertext,
        })
    }

    fn decrypt(&self, iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        if iv.len() != 12 {
            return Err(RusWaCipherError::Decryption(format!(
                "ChaCha20-Poly1305 requires a 12-byte nonce, got {} bytes",
                iv.len()
            )));
        }

        let nonce = Nonce::from_slice(iv);

        let plaintext = self.cipher.decrypt(nonce, ciphertext).map_err(|e| {
            RusWaCipherError::Decryption(format!("ChaCha20-Poly1305 decryption failed: {:?}", e))
        })?;

        Ok(plaintext)
    }

    fn iv_length(&self) -> usize {
        12 // ChaCha20-Poly1305 uses 12-byte nonces
    }

    fn key_length(&self) -> usize {
        32 // ChaCha20-Poly1305 uses 32-byte keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key_management::KeyManager;

    #[test]
    fn test_chacha20poly1305_encrypt_decrypt() {
        let key = KeyManager::generate_key(32).unwrap();
        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();

        let plaintext = b"Hello, ChaCha20-Poly1305!";

        // Encrypt
        let result = cipher.encrypt(plaintext).unwrap();
        assert_eq!(result.iv.len(), 12);
        assert_ne!(result.ciphertext, plaintext);

        // Decrypt
        let decrypted = cipher.decrypt(&result.iv, &result.ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chacha20poly1305_invalid_key_length() {
        let short_key = vec![0u8; 16];
        assert!(ChaCha20Poly1305Cipher::new(&short_key).is_err());

        let long_key = vec![0u8; 64];
        assert!(ChaCha20Poly1305Cipher::new(&long_key).is_err());
    }

    #[test]
    fn test_chacha20poly1305_invalid_nonce_length() {
        let key = KeyManager::generate_key(32).unwrap();
        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();

        let ciphertext = b"some encrypted data";
        let invalid_nonce = vec![0u8; 8]; // Wrong length

        assert!(cipher.decrypt(&invalid_nonce, ciphertext).is_err());
    }

    #[test]
    fn test_chacha20poly1305_different_keys_different_results() {
        let key1 = KeyManager::generate_key(32).unwrap();
        let key2 = KeyManager::generate_key(32).unwrap();

        let cipher1 = ChaCha20Poly1305Cipher::new(&key1).unwrap();
        let cipher2 = ChaCha20Poly1305Cipher::new(&key2).unwrap();

        let plaintext = b"Test message";

        let result1 = cipher1.encrypt(plaintext).unwrap();
        let result2 = cipher2.encrypt(plaintext).unwrap();

        // Different keys should produce different ciphertexts
        assert_ne!(result1.ciphertext, result2.ciphertext);

        // Each cipher should only decrypt its own ciphertext
        assert!(cipher1.decrypt(&result1.iv, &result1.ciphertext).is_ok());
        assert!(cipher2.decrypt(&result2.iv, &result2.ciphertext).is_ok());

        // Cross-decryption should fail
        assert!(cipher1.decrypt(&result2.iv, &result2.ciphertext).is_err());
        assert!(cipher2.decrypt(&result1.iv, &result1.ciphertext).is_err());
    }
}
