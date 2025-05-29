use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::config::{DecryptionConfig, EncryptionAlgorithm, EncryptionConfig};
use crate::error::Result;

#[derive(Debug, Clone, ValueEnum)]
pub enum KeyFormat {
    /// Hexadecimal format (default)
    Hex,
    /// Base64 format
    Base64,
    /// Raw binary format
    Raw,
}

#[derive(Parser)]
#[command(name = "ruswacipher")]
#[command(about = "A Rust tool for encrypting and protecting WebAssembly modules")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encrypt a WASM file
    Encrypt {
        /// Input WASM file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output encrypted file path
        #[arg(short, long)]
        output: PathBuf,

        /// Encryption algorithm to use
        #[arg(short, long, default_value = "aes-gcm")]
        algorithm: EncryptionAlgorithm,

        /// Key file path (if not provided, a new key will be generated)
        #[arg(short, long)]
        key: Option<PathBuf>,

        /// Key in hexadecimal format (alternative to --key)
        #[arg(long, conflicts_with = "key")]
        key_hex: Option<String>,

        /// Key in Base64 format (alternative to --key)
        #[arg(long, conflicts_with_all = ["key", "key_hex"])]
        key_base64: Option<String>,

        /// Generate a new key and save it to this file
        #[arg(long)]
        generate_key: Option<PathBuf>,

        /// Format for generated key output
        #[arg(long, default_value = "hex")]
        key_format: KeyFormat,
    },

    /// Decrypt a WASM file
    Decrypt {
        /// Input encrypted file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output decrypted WASM file path
        #[arg(short, long)]
        output: PathBuf,

        /// Key file path
        #[arg(short, long)]
        key: PathBuf,
    },
}

impl Commands {
    pub fn to_encryption_config(&self) -> Result<EncryptionConfig> {
        match self {
            Commands::Encrypt {
                input,
                output,
                algorithm,
                key,
                key_hex,
                key_base64,
                generate_key,
                key_format,
            } => {
                let generate_key_flag = generate_key.is_some();
                Ok(EncryptionConfig {
                    algorithm: algorithm.clone(),
                    input_file: input.clone(),
                    output_file: output.clone(),
                    key_file: key.clone(),
                    key_hex: key_hex.clone(),
                    key_base64: key_base64.clone(),
                    generate_key: generate_key_flag,
                    key_output_file: generate_key.clone(),
                    key_format: key_format.clone(),
                })
            }
            _ => Err(crate::error::RusWaCipherError::InvalidInput(
                "Not an encryption command".to_string(),
            )),
        }
    }

    pub fn to_decryption_config(&self) -> Result<DecryptionConfig> {
        match self {
            Commands::Decrypt { input, output, key } => Ok(DecryptionConfig {
                input_file: input.clone(),
                output_file: output.clone(),
                key_file: key.clone(),
            }),
            _ => Err(crate::error::RusWaCipherError::InvalidInput(
                "Not a decryption command".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_key_format_values() {
        // Test that KeyFormat enum has expected variants
        let _hex = KeyFormat::Hex;
        let _base64 = KeyFormat::Base64;
        let _raw = KeyFormat::Raw;
    }

    #[test]
    fn test_encryption_config_from_encrypt_command() {
        let command = Commands::Encrypt {
            input: PathBuf::from("input.wasm"),
            output: PathBuf::from("output.wasm.enc"),
            algorithm: crate::config::EncryptionAlgorithm::AesGcm,
            key: Some(PathBuf::from("key.txt")),
            key_hex: None,
            key_base64: None,
            generate_key: None,
            key_format: KeyFormat::Hex,
        };

        let config = command.to_encryption_config().unwrap();
        assert!(matches!(
            config.algorithm,
            crate::config::EncryptionAlgorithm::AesGcm
        ));
        assert_eq!(config.input_file, PathBuf::from("input.wasm"));
        assert_eq!(config.output_file, PathBuf::from("output.wasm.enc"));
        assert_eq!(config.key_file, Some(PathBuf::from("key.txt")));
        assert!(!config.generate_key);
    }

    #[test]
    fn test_encryption_config_with_generate_key() {
        let command = Commands::Encrypt {
            input: PathBuf::from("input.wasm"),
            output: PathBuf::from("output.wasm.enc"),
            algorithm: crate::config::EncryptionAlgorithm::ChaCha20Poly1305,
            key: None,
            key_hex: None,
            key_base64: None,
            generate_key: Some(PathBuf::from("generated.key")),
            key_format: KeyFormat::Base64,
        };

        let config = command.to_encryption_config().unwrap();
        assert!(matches!(
            config.algorithm,
            crate::config::EncryptionAlgorithm::ChaCha20Poly1305
        ));
        assert!(config.generate_key);
        assert_eq!(config.key_output_file, Some(PathBuf::from("generated.key")));
        assert!(matches!(config.key_format, KeyFormat::Base64));
    }

    #[test]
    fn test_encryption_config_with_hex_key() {
        let command = Commands::Encrypt {
            input: PathBuf::from("input.wasm"),
            output: PathBuf::from("output.wasm.enc"),
            algorithm: crate::config::EncryptionAlgorithm::AesGcm,
            key: None,
            key_hex: Some("0123456789abcdef".to_string()),
            key_base64: None,
            generate_key: None,
            key_format: KeyFormat::Hex,
        };

        let config = command.to_encryption_config().unwrap();
        assert_eq!(config.key_hex, Some("0123456789abcdef".to_string()));
        assert!(config.key_base64.is_none());
        assert!(config.key_file.is_none());
    }

    #[test]
    fn test_encryption_config_with_base64_key() {
        let command = Commands::Encrypt {
            input: PathBuf::from("input.wasm"),
            output: PathBuf::from("output.wasm.enc"),
            algorithm: crate::config::EncryptionAlgorithm::AesGcm,
            key: None,
            key_hex: None,
            key_base64: Some("SGVsbG8gV29ybGQ=".to_string()),
            generate_key: None,
            key_format: KeyFormat::Hex,
        };

        let config = command.to_encryption_config().unwrap();
        assert_eq!(config.key_base64, Some("SGVsbG8gV29ybGQ=".to_string()));
        assert!(config.key_hex.is_none());
        assert!(config.key_file.is_none());
    }

    #[test]
    fn test_decryption_config_from_decrypt_command() {
        let command = Commands::Decrypt {
            input: PathBuf::from("input.wasm.enc"),
            output: PathBuf::from("output.wasm"),
            key: PathBuf::from("key.txt"),
        };

        let result = command.to_decryption_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.input_file, PathBuf::from("input.wasm.enc"));
        assert_eq!(config.output_file, PathBuf::from("output.wasm"));
        assert_eq!(config.key_file, PathBuf::from("key.txt"));
    }

    #[test]
    fn test_encryption_config_from_decrypt_command_fails() {
        let command = Commands::Decrypt {
            input: PathBuf::from("input.wasm.enc"),
            output: PathBuf::from("output.wasm"),
            key: PathBuf::from("key.txt"),
        };

        let result = command.to_encryption_config();
        assert!(result.is_err());
    }
}
