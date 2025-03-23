use crate::crypto::plugins::EncryptionPlugin;
use anyhow::{anyhow, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::Rng;

/// ChaCha20-Poly1305 encryption plugin implementation
pub struct ChaCha20Poly1305Plugin;

impl ChaCha20Poly1305Plugin {
    /// Create new ChaCha20-Poly1305 plugin instance
    pub fn new() -> Self {
        ChaCha20Poly1305Plugin
    }
}

impl EncryptionPlugin for ChaCha20Poly1305Plugin {
    fn name(&self) -> &str {
        "chacha20poly1305"
    }

    fn description(&self) -> &str {
        "ChaCha20-Poly1305 stream encryption and authentication algorithm"
    }

    fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        // Ensure key length is 32 bytes
        if key.len() != 32 {
            return Err(anyhow!("ChaCha20-Poly1305 requires 32 bytes key"));
        }

        // Create cipher
        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| anyhow!("Failed to create ChaCha20-Poly1305 cipher: {}", e))?;

        // Generate random 12-byte nonce
        let mut rng = rand::rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt data
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow!("ChaCha20-Poly1305 encryption failed: {}", e))?;

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

        // Ensure key length is 32 bytes
        if key.len() != 32 {
            return Err(anyhow!("ChaCha20-Poly1305 requires 32 bytes key"));
        }

        // Separate nonce and ciphertext
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        // Create cipher
        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| anyhow!("Failed to create ChaCha20-Poly1305 cipher: {}", e))?;

        // Decrypt data
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("ChaCha20-Poly1305 decryption failed: {}", e))?;

        Ok(plaintext)
    }
}
