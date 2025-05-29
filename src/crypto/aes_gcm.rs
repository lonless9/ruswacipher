use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};

use crate::crypto::traits::{Cipher, EncryptionResult};
use crate::crypto::KeyManager;
use crate::error::{Result, RusWaCipherError};

pub struct AesGcmCipher {
    cipher: Aes256Gcm,
}

impl AesGcmCipher {
    /// Create a new AES-GCM cipher with the provided key
    pub fn new(key: &[u8]) -> Result<Self> {
        KeyManager::validate_key_length(key, 32)?;

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        Ok(AesGcmCipher { cipher })
    }

    /// Create a new AES-GCM cipher with a randomly generated key
    pub fn new_with_random_key() -> Result<(Self, Vec<u8>)> {
        let key = KeyManager::generate_key(32)?;
        let cipher = Self::new(&key)?;
        Ok((cipher, key))
    }
}

impl Cipher for AesGcmCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult> {
        // Generate a random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt the data
        let ciphertext = self.cipher.encrypt(&nonce, plaintext).map_err(|e| {
            RusWaCipherError::Encryption(format!("AES-GCM encryption failed: {:?}", e))
        })?;

        Ok(EncryptionResult {
            iv: nonce.to_vec(),
            ciphertext,
        })
    }

    fn decrypt(&self, iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        if iv.len() != 12 {
            return Err(RusWaCipherError::Decryption(
                "Invalid IV length for AES-GCM: expected 12 bytes".to_string(),
            ));
        }

        let nonce = Nonce::from_slice(iv);

        let plaintext = self.cipher.decrypt(nonce, ciphertext).map_err(|e| {
            RusWaCipherError::Decryption(format!("AES-GCM decryption failed: {:?}", e))
        })?;

        Ok(plaintext)
    }

    fn iv_length(&self) -> usize {
        12 // AES-GCM standard nonce length
    }

    fn key_length(&self) -> usize {
        32 // AES-256 key length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_encrypt_decrypt() {
        let key = KeyManager::generate_key(32).unwrap();
        let cipher = AesGcmCipher::new(&key).unwrap();

        let plaintext = b"Hello, World! This is a test message.";

        // Encrypt
        let result = cipher.encrypt(plaintext).unwrap();
        assert_eq!(result.iv.len(), 12);
        assert!(!result.ciphertext.is_empty());
        assert_ne!(result.ciphertext, plaintext);

        // Decrypt
        let decrypted = cipher.decrypt(&result.iv, &result.ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_with_random_key() {
        let (cipher, key) = AesGcmCipher::new_with_random_key().unwrap();
        assert_eq!(key.len(), 32);

        let plaintext = b"Test message";
        let result = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&result.iv, &result.ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = vec![0u8; 16];
        assert!(AesGcmCipher::new(&short_key).is_err());
    }

    #[test]
    fn test_invalid_iv_length() {
        let key = KeyManager::generate_key(32).unwrap();
        let cipher = AesGcmCipher::new(&key).unwrap();

        let ciphertext = b"dummy ciphertext";
        let invalid_iv = vec![0u8; 8]; // Wrong length

        assert!(cipher.decrypt(&invalid_iv, ciphertext).is_err());
    }

    #[test]
    fn test_encryption_result_serialization() {
        let key = KeyManager::generate_key(32).unwrap();
        let cipher = AesGcmCipher::new(&key).unwrap();

        let plaintext = b"Test serialization";
        let result = cipher.encrypt(plaintext).unwrap();

        // Serialize
        let serialized = result.serialize();
        assert_eq!(serialized.len(), result.iv.len() + result.ciphertext.len());

        // Deserialize
        let deserialized = EncryptionResult::deserialize(&serialized, cipher.iv_length()).unwrap();
        assert_eq!(deserialized.iv, result.iv);
        assert_eq!(deserialized.ciphertext, result.ciphertext);

        // Decrypt using deserialized data
        let decrypted = cipher
            .decrypt(&deserialized.iv, &deserialized.ciphertext)
            .unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
