use base64::Engine;
use rand::RngCore;

use crate::cli::KeyFormat;
use crate::error::{Result, RusWaCipherError};

pub struct KeyManager;

impl KeyManager {
    /// Generate a random key of the specified length
    pub fn generate_key(length: usize) -> Result<Vec<u8>> {
        let mut key = vec![0u8; length];
        rand::rng().fill_bytes(&mut key);
        Ok(key)
    }

    /// Validate that a key has the correct length
    pub fn validate_key_length(key: &[u8], expected_length: usize) -> Result<()> {
        if key.len() != expected_length {
            return Err(RusWaCipherError::KeyManagement(format!(
                "Invalid key length: expected {}, got {}",
                expected_length,
                key.len()
            )));
        }
        Ok(())
    }

    /// Generate a random IV of the specified length
    pub fn generate_iv(length: usize) -> Result<Vec<u8>> {
        let mut iv = vec![0u8; length];
        rand::rng().fill_bytes(&mut iv);
        Ok(iv)
    }

    /// Decode a key from hexadecimal string
    pub fn decode_hex_key(hex_key: &str) -> Result<Vec<u8>> {
        hex::decode(hex_key.trim())
            .map_err(|e| RusWaCipherError::KeyManagement(format!("Invalid hexadecimal key: {}", e)))
    }

    /// Decode a key from Base64 string
    pub fn decode_base64_key(base64_key: &str) -> Result<Vec<u8>> {
        base64::engine::general_purpose::STANDARD
            .decode(base64_key.trim())
            .map_err(|e| RusWaCipherError::KeyManagement(format!("Invalid Base64 key: {}", e)))
    }

    /// Encode a key to the specified format
    pub fn encode_key(key: &[u8], format: &KeyFormat) -> String {
        match format {
            KeyFormat::Hex => hex::encode(key),
            KeyFormat::Base64 => base64::engine::general_purpose::STANDARD.encode(key),
            KeyFormat::Raw => {
                // For raw format, we'll return a warning message since it's not printable
                format!(
                    "Raw binary key ({} bytes) - cannot display as text",
                    key.len()
                )
            }
        }
    }

    /// Validate key for a specific algorithm
    pub fn validate_key_for_algorithm(
        key: &[u8],
        algorithm: &crate::config::EncryptionAlgorithm,
    ) -> Result<()> {
        let expected_length = match algorithm {
            crate::config::EncryptionAlgorithm::AesGcm => 32, // AES-256
            crate::config::EncryptionAlgorithm::ChaCha20Poly1305 => 32, // ChaCha20
        };

        Self::validate_key_length(key, expected_length)?;
        Ok(())
    }
}

/// Generate a key for the specified algorithm
pub fn generate_key(algorithm: &crate::config::EncryptionAlgorithm) -> Result<Vec<u8>> {
    let key_length = match algorithm {
        crate::config::EncryptionAlgorithm::AesGcm => 32, // AES-256
        crate::config::EncryptionAlgorithm::ChaCha20Poly1305 => 32, // ChaCha20
    };

    KeyManager::generate_key(key_length)
}

/// Resolve key from various sources (file, hex, base64) based on configuration
pub fn resolve_key(config: &crate::config::EncryptionConfig) -> Result<Option<Vec<u8>>> {
    // Priority: key_hex > key_base64 > key_file
    if let Some(hex_key) = &config.key_hex {
        let key = KeyManager::decode_hex_key(hex_key)?;
        KeyManager::validate_key_for_algorithm(&key, &config.algorithm)?;
        return Ok(Some(key));
    }

    if let Some(base64_key) = &config.key_base64 {
        let key = KeyManager::decode_base64_key(base64_key)?;
        KeyManager::validate_key_for_algorithm(&key, &config.algorithm)?;
        return Ok(Some(key));
    }

    if let Some(key_file) = &config.key_file {
        let key = crate::io::read_key_file(key_file)?;
        KeyManager::validate_key_for_algorithm(&key, &config.algorithm)?;
        return Ok(Some(key));
    }

    // No key source provided
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = KeyManager::generate_key(32).unwrap();
        assert_eq!(key.len(), 32);

        // Generate another key and ensure they're different
        let key2 = KeyManager::generate_key(32).unwrap();
        assert_ne!(key, key2);
    }

    #[test]
    fn test_validate_key_length() {
        let key = vec![0u8; 32];
        assert!(KeyManager::validate_key_length(&key, 32).is_ok());
        assert!(KeyManager::validate_key_length(&key, 16).is_err());
    }

    #[test]
    fn test_generate_iv() {
        let iv = KeyManager::generate_iv(12).unwrap();
        assert_eq!(iv.len(), 12);

        // Generate another IV and ensure they're different
        let iv2 = KeyManager::generate_iv(12).unwrap();
        assert_ne!(iv, iv2);
    }

    #[test]
    fn test_generate_key_for_algorithm() {
        let aes_key = generate_key(&crate::config::EncryptionAlgorithm::AesGcm).unwrap();
        assert_eq!(aes_key.len(), 32);

        let chacha_key =
            generate_key(&crate::config::EncryptionAlgorithm::ChaCha20Poly1305).unwrap();
        assert_eq!(chacha_key.len(), 32);
    }

    #[test]
    fn test_decode_hex_key() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let decoded = KeyManager::decode_hex_key(hex_key).unwrap();
        assert_eq!(decoded.len(), 32);
        assert_eq!(hex::encode(&decoded), hex_key);
    }

    #[test]
    fn test_decode_hex_key_invalid() {
        let invalid_hex = "invalid_hex_string";
        assert!(KeyManager::decode_hex_key(invalid_hex).is_err());
    }

    #[test]
    fn test_decode_base64_key() {
        let key_bytes = vec![0u8; 32];
        let base64_key = base64::engine::general_purpose::STANDARD.encode(&key_bytes);
        let decoded = KeyManager::decode_base64_key(&base64_key).unwrap();
        assert_eq!(decoded, key_bytes);
    }

    #[test]
    fn test_decode_base64_key_invalid() {
        let invalid_base64 = "invalid_base64!@#$";
        assert!(KeyManager::decode_base64_key(invalid_base64).is_err());
    }

    #[test]
    fn test_encode_key_formats() {
        let key = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];

        // Test hex encoding
        let hex_encoded = KeyManager::encode_key(&key, &KeyFormat::Hex);
        assert_eq!(hex_encoded, "0123456789abcdef");

        // Test base64 encoding
        let base64_encoded = KeyManager::encode_key(&key, &KeyFormat::Base64);
        assert_eq!(
            base64_encoded,
            base64::engine::general_purpose::STANDARD.encode(&key)
        );

        // Test raw format (should return info message)
        let raw_encoded = KeyManager::encode_key(&key, &KeyFormat::Raw);
        assert!(raw_encoded.contains("Raw binary key"));
        assert!(raw_encoded.contains("8 bytes"));
    }

    #[test]
    fn test_validate_key_for_algorithm() {
        let valid_key = vec![0u8; 32];
        let invalid_key = vec![0u8; 16];

        // Test AES-GCM
        assert!(KeyManager::validate_key_for_algorithm(
            &valid_key,
            &crate::config::EncryptionAlgorithm::AesGcm
        )
        .is_ok());
        assert!(KeyManager::validate_key_for_algorithm(
            &invalid_key,
            &crate::config::EncryptionAlgorithm::AesGcm
        )
        .is_err());

        // Test ChaCha20-Poly1305
        assert!(KeyManager::validate_key_for_algorithm(
            &valid_key,
            &crate::config::EncryptionAlgorithm::ChaCha20Poly1305
        )
        .is_ok());
        assert!(KeyManager::validate_key_for_algorithm(
            &invalid_key,
            &crate::config::EncryptionAlgorithm::ChaCha20Poly1305
        )
        .is_err());
    }

    #[test]
    fn test_resolve_key_hex() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let config = crate::config::EncryptionConfig {
            algorithm: crate::config::EncryptionAlgorithm::AesGcm,
            input_file: std::path::PathBuf::new(),
            output_file: std::path::PathBuf::new(),
            key_file: None,
            key_hex: Some(hex_key.to_string()),
            key_base64: None,
            generate_key: false,
            key_output_file: None,
            key_format: KeyFormat::Hex,
        };

        let resolved_key = resolve_key(&config).unwrap().unwrap();
        assert_eq!(resolved_key.len(), 32);
        assert_eq!(hex::encode(&resolved_key), hex_key);
    }

    #[test]
    fn test_resolve_key_base64() {
        let key_bytes = vec![0u8; 32];
        let base64_key = base64::engine::general_purpose::STANDARD.encode(&key_bytes);
        let config = crate::config::EncryptionConfig {
            algorithm: crate::config::EncryptionAlgorithm::AesGcm,
            input_file: std::path::PathBuf::new(),
            output_file: std::path::PathBuf::new(),
            key_file: None,
            key_hex: None,
            key_base64: Some(base64_key),
            generate_key: false,
            key_output_file: None,
            key_format: KeyFormat::Base64,
        };

        let resolved_key = resolve_key(&config).unwrap().unwrap();
        assert_eq!(resolved_key, key_bytes);
    }

    #[test]
    fn test_resolve_key_none() {
        let config = crate::config::EncryptionConfig {
            algorithm: crate::config::EncryptionAlgorithm::AesGcm,
            input_file: std::path::PathBuf::new(),
            output_file: std::path::PathBuf::new(),
            key_file: None,
            key_hex: None,
            key_base64: None,
            generate_key: false,
            key_output_file: None,
            key_format: KeyFormat::Hex,
        };

        let resolved_key = resolve_key(&config).unwrap();
        assert!(resolved_key.is_none());
    }
}
