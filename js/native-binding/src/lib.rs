#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::*;
use ruswacipher::obfuscation::{obfuscate_wasm as wasm_obfuscate, ObfuscationLevel};
use std::path::Path;

/// WebAssembly Obfuscation Level (Node.js Binding)
#[napi]
pub enum ObfuscationLevel {
  /// Low level obfuscation
  Low = 0,
  /// Medium level obfuscation
  Medium = 1,
  /// High level obfuscation
  High = 2,
}

impl From<ObfuscationLevel> for ruswacipher::obfuscation::ObfuscationLevel {
  fn from(level: ObfuscationLevel) -> Self {
    match level {
      ObfuscationLevel::Low => ruswacipher::obfuscation::ObfuscationLevel::Low,
      ObfuscationLevel::Medium => ruswacipher::obfuscation::ObfuscationLevel::Medium,
      ObfuscationLevel::High => ruswacipher::obfuscation::ObfuscationLevel::High,
    }
  }
}

/// Encryption Algorithm Type (Node.js Binding)
#[napi]
pub enum EncryptionAlgorithm {
  /// AES-GCM algorithm
  AesGcm = 0,
  /// ChaCha20-Poly1305 algorithm
  ChaCha20Poly1305 = 1,
  /// Support all algorithms
  All = 2,
}

/// Convert JavaScript encryption algorithm enum to algorithm name string
fn algorithm_to_string(algorithm: EncryptionAlgorithm) -> &'static str {
  match algorithm {
    EncryptionAlgorithm::AesGcm => "aes-gcm",
    EncryptionAlgorithm::ChaCha20Poly1305 => "chacha20poly1305",
    EncryptionAlgorithm::All => "all",
  }
}

/// Obfuscate WebAssembly file
///
/// Use RusWaCipher to obfuscate and protect WebAssembly binary files
///
/// @param input_path - Input file path
/// @param output_path - Output file path
/// @param level - Obfuscation level
/// @param algorithm - Encryption algorithm (optional, defaults to AES-GCM)
/// @returns Success status
#[napi]
pub fn obfuscate_wasm(
  input_path: String,
  output_path: String,
  level: ObfuscationLevel,
  #[napi(ts_arg_type = "EncryptionAlgorithm?")] algorithm: Option<EncryptionAlgorithm>,
) -> Result<bool> {
  let input = Path::new(&input_path);
  let output = Path::new(&output_path);

  // Check if input file exists
  if !input.exists() {
    return Err(Error::new(
      Status::InvalidArg,
      format!("Input file does not exist: {}", input_path),
    ));
  }

  // Get algorithm type, default is AES-GCM
  let algorithm_str = match algorithm {
    Some(alg) => algorithm_to_string(alg),
    None => "aes-gcm",
  };

  // Call Rust library function, passing algorithm parameter
  match wasm_obfuscate(input, output, level.into(), Some(algorithm_str)) {
    Ok(_) => Ok(true),
    Err(e) => Err(Error::new(
      Status::GenericFailure,
      format!("Failed to obfuscate WASM: {}", e),
    )),
  }
}

/// Get RusWaCipher library version
#[napi]
pub fn get_version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
} 