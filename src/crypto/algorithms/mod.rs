pub mod aes_gcm;
pub mod chacha20poly1305;

use anyhow::Result;
use std::fmt::Debug;
use std::marker::Send;
use std::marker::Sync;

/// Encryption algorithm interface
pub trait Cipher: Send + Sync + Debug {
    /// Encrypt data
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt data
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Algorithm name
    fn algorithm_name(&self) -> &'static str;

    /// Algorithm description
    fn description(&self) -> &'static str;
}

/// Create a cipher for the specified algorithm
pub fn create_cipher(algorithm: &str, key: &[u8]) -> Result<Box<dyn Cipher>> {
    match algorithm {
        "aes-gcm" => Ok(Box::new(AesGcmCipher::new(key)?)),
        "chacha20poly1305" => Ok(Box::new(ChaCha20Poly1305Cipher::new(key)?)),
        _ => Err(anyhow::anyhow!(
            "Unsupported encryption algorithm: {}",
            algorithm
        )),
    }
}

/// AES-GCM encryption implementation
#[derive(Debug)]
pub struct AesGcmCipher {
    key: Vec<u8>,
}

impl AesGcmCipher {
    /// Create a new AES-GCM cipher
    pub fn new(key: &[u8]) -> Result<Self> {
        // Ensure key length is 32 bytes (256 bits)
        if key.len() != 32 {
            anyhow::bail!("AES-GCM requires a 32-byte (256-bit) key");
        }

        Ok(Self { key: key.to_vec() })
    }
}

impl Cipher for AesGcmCipher {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Use plugin implementation for encryption
        crate::crypto::plugins::encrypt_with_plugin(data, &self.key, "aes-gcm")
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Use plugin implementation for decryption
        crate::crypto::plugins::decrypt_with_plugin(data, &self.key, "aes-gcm")
    }

    fn algorithm_name(&self) -> &'static str {
        "aes-gcm"
    }

    fn description(&self) -> &'static str {
        "AES-GCM (Galois/Counter Mode) 256-bit encryption algorithm"
    }
}

/// ChaCha20-Poly1305 encryption implementation
#[derive(Debug)]
pub struct ChaCha20Poly1305Cipher {
    key: Vec<u8>,
}

impl ChaCha20Poly1305Cipher {
    /// Create a new ChaCha20-Poly1305 cipher
    pub fn new(key: &[u8]) -> Result<Self> {
        // Ensure key length is 32 bytes
        if key.len() != 32 {
            anyhow::bail!("ChaCha20-Poly1305 requires a 32-byte key");
        }

        Ok(Self { key: key.to_vec() })
    }
}

impl Cipher for ChaCha20Poly1305Cipher {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Use plugin implementation for encryption
        crate::crypto::plugins::encrypt_with_plugin(data, &self.key, "chacha20poly1305")
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Use plugin implementation for decryption
        crate::crypto::plugins::decrypt_with_plugin(data, &self.key, "chacha20poly1305")
    }

    fn algorithm_name(&self) -> &'static str {
        "chacha20poly1305"
    }

    fn description(&self) -> &'static str {
        "ChaCha20-Poly1305 stream cipher and authentication algorithm"
    }
}
