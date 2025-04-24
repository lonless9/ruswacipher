pub mod cli;
pub mod crypto;
pub mod obfuscation;
pub mod runtime;
pub mod wasm;

// Re-export commonly used features
pub use crypto::{decrypt_file, encrypt_file};
pub use obfuscation::obfuscate_wasm_only;
pub use wasm::{load_module, save_module};
