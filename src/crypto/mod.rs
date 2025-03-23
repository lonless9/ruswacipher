pub mod engine;
pub mod algorithms;
pub mod plugins;

use anyhow::Result;
use std::path::Path;

/// Encryption and decryption function wrappers
pub use engine::{encrypt_data, decrypt_data, generate_key, save_key, load_key};

/// Encrypt WASM file
pub fn encrypt_file(input: &Path, output: &Path, key_file: Option<&Path>, algorithm: &str) -> Result<()> {
    engine::encrypt_file(input, output, key_file, algorithm)
}

/// Decrypt WASM file
pub fn decrypt_file(input: &Path, output: &Path, key_file: &Path) -> Result<()> {
    engine::decrypt_file(input, output, key_file)
} 