pub mod cli;
pub mod wasm;
pub mod crypto;
pub mod runtime;
pub mod obfuscation;

// Re-export commonly used features
pub use crypto::{encrypt_file, decrypt_file};
pub use wasm::{load_module, save_module}; 