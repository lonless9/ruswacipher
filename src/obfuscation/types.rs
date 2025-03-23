use crate::wasm::structure::WasmModule;
use anyhow::Result;
use log::debug;
use thiserror::Error;

/// Obfuscation error types
#[derive(Error, Debug)]
pub enum ObfuscationError {
    #[error("Unsupported obfuscation level: {0}")]
    UnsupportedLevel(u8),

    #[error("Obfuscation operation failed: {0}")]
    OperationFailed(String),
}

/// Obfuscation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObfuscationLevel {
    Low,
    Medium,
    High,
}

impl ObfuscationLevel {
    pub fn try_from_u8(level: u8) -> Result<Self, ObfuscationError> {
        match level {
            1 => Ok(ObfuscationLevel::Low),
            2 => Ok(ObfuscationLevel::Medium),
            3 => Ok(ObfuscationLevel::High),
            _ => Err(ObfuscationError::UnsupportedLevel(level)),
        }
    }
}

impl From<u8> for ObfuscationLevel {
    fn from(level: u8) -> Self {
        match ObfuscationLevel::try_from_u8(level) {
            Ok(level) => level,
            Err(_) => {
                debug!(
                    "Invalid obfuscation level: {}, using default level (Low)",
                    level
                );
                ObfuscationLevel::Low
            }
        }
    }
}

/// Define obfuscation transformation type
pub type Transformation = fn(WasmModule) -> Result<WasmModule>;
