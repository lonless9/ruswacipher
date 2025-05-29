use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AesGcm,
    ChaCha20Poly1305,
}

impl std::fmt::Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionAlgorithm::AesGcm => write!(f, "aes-gcm"),
            EncryptionAlgorithm::ChaCha20Poly1305 => write!(f, "chacha20poly1305"),
        }
    }
}

impl std::str::FromStr for EncryptionAlgorithm {
    type Err = crate::error::RusWaCipherError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aes-gcm" | "aesgcm" => Ok(EncryptionAlgorithm::AesGcm),
            "chacha20poly1305" | "chacha20-poly1305" => Ok(EncryptionAlgorithm::ChaCha20Poly1305),
            _ => Err(crate::error::RusWaCipherError::InvalidInput(format!(
                "Unknown encryption algorithm: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub key_file: Option<PathBuf>,
    pub key_hex: Option<String>,
    pub key_base64: Option<String>,
    pub generate_key: bool,
    pub key_output_file: Option<PathBuf>,
    pub key_format: crate::cli::KeyFormat,
}

#[derive(Debug, Clone)]
pub struct DecryptionConfig {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub key_file: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_algorithm_display() {
        assert_eq!(EncryptionAlgorithm::AesGcm.to_string(), "aes-gcm");
        assert_eq!(
            EncryptionAlgorithm::ChaCha20Poly1305.to_string(),
            "chacha20poly1305"
        );
    }

    #[test]
    fn test_encryption_algorithm_from_str() {
        assert!(matches!(
            "aes-gcm".parse::<EncryptionAlgorithm>().unwrap(),
            EncryptionAlgorithm::AesGcm
        ));
        assert!(matches!(
            "aesgcm".parse::<EncryptionAlgorithm>().unwrap(),
            EncryptionAlgorithm::AesGcm
        ));
        assert!(matches!(
            "chacha20poly1305".parse::<EncryptionAlgorithm>().unwrap(),
            EncryptionAlgorithm::ChaCha20Poly1305
        ));
        assert!(matches!(
            "chacha20-poly1305".parse::<EncryptionAlgorithm>().unwrap(),
            EncryptionAlgorithm::ChaCha20Poly1305
        ));
    }

    #[test]
    fn test_encryption_algorithm_from_str_case_insensitive() {
        assert!(matches!(
            "AES-GCM".parse::<EncryptionAlgorithm>().unwrap(),
            EncryptionAlgorithm::AesGcm
        ));
        assert!(matches!(
            "ChaCha20Poly1305".parse::<EncryptionAlgorithm>().unwrap(),
            EncryptionAlgorithm::ChaCha20Poly1305
        ));
        assert!(matches!(
            "CHACHA20-POLY1305".parse::<EncryptionAlgorithm>().unwrap(),
            EncryptionAlgorithm::ChaCha20Poly1305
        ));
    }

    #[test]
    fn test_encryption_algorithm_from_str_invalid() {
        assert!("invalid".parse::<EncryptionAlgorithm>().is_err());
        assert!("".parse::<EncryptionAlgorithm>().is_err());
        assert!("aes".parse::<EncryptionAlgorithm>().is_err());
    }

    #[test]
    fn test_encryption_config_creation() {
        use crate::cli::KeyFormat;
        use std::path::PathBuf;

        let config = EncryptionConfig {
            algorithm: EncryptionAlgorithm::AesGcm,
            input_file: PathBuf::from("input.wasm"),
            output_file: PathBuf::from("output.wasm.enc"),
            key_file: Some(PathBuf::from("key.txt")),
            key_hex: None,
            key_base64: None,
            generate_key: false,
            key_output_file: None,
            key_format: KeyFormat::Hex,
        };

        assert!(matches!(config.algorithm, EncryptionAlgorithm::AesGcm));
        assert_eq!(config.input_file, PathBuf::from("input.wasm"));
        assert_eq!(config.output_file, PathBuf::from("output.wasm.enc"));
        assert_eq!(config.key_file, Some(PathBuf::from("key.txt")));
        assert!(!config.generate_key);
    }

    #[test]
    fn test_decryption_config_creation() {
        use std::path::PathBuf;

        let config = DecryptionConfig {
            input_file: PathBuf::from("input.wasm.enc"),
            output_file: PathBuf::from("output.wasm"),
            key_file: PathBuf::from("key.txt"),
        };

        assert_eq!(config.input_file, PathBuf::from("input.wasm.enc"));
        assert_eq!(config.output_file, PathBuf::from("output.wasm"));
        assert_eq!(config.key_file, PathBuf::from("key.txt"));
    }
}
