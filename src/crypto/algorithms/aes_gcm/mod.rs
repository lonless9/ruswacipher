use crate::crypto::plugins::EncryptionPlugin;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use rand::Rng;

/// AES-GCM encryption plugin implementation
pub struct AesGcmPlugin;

impl AesGcmPlugin {
    /// Create new AES-GCM plugin instance
    pub fn new() -> Self {
        AesGcmPlugin
    }
}

impl EncryptionPlugin for AesGcmPlugin {
    fn name(&self) -> &str {
        "aes-gcm"
    }

    fn description(&self) -> &str {
        "AES-GCM (Galois/Counter Mode) 256-bit encryption algorithm"
    }

    fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        // Ensure key length is 32 bytes (256 bits)
        if key.len() != 32 {
            return Err(anyhow!("AES-GCM requires 32 bytes (256-bit) key"));
        }

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| anyhow!("Failed to create AES-GCM cipher: {}", e))?;

        // Generate random 12-byte nonce
        let mut rng = rand::rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt without associated data
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow!("AES-GCM encryption failed: {}", e))?;

        // Concatenate nonce and ciphertext
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    fn decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        // Check if input data length is at least nonce length
        if data.len() < 12 {
            return Err(anyhow!("Encrypted data length is too short"));
        }

        // Ensure key length is 32 bytes (256 bits)
        if key.len() != 32 {
            return Err(anyhow!("AES-GCM requires 32 bytes (256-bit) key"));
        }

        // Separate nonce and ciphertext
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| anyhow!("Failed to create AES-GCM cipher: {}", e))?;

        // Decrypt without associated data
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("AES-GCM decryption failed: {}", e))?;

        Ok(plaintext)
    }
}
