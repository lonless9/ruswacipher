use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Application configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Default encryption algorithm
    pub default_algorithm: String,
    /// Whether to enable obfuscation
    pub enable_obfuscation: bool,
    /// Whether to save keys to file
    pub save_keys: bool,
    /// Key directory
    pub key_directory: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_algorithm: "aes-gcm".to_string(),
            enable_obfuscation: false,
            save_keys: true,
            key_directory: "./keys".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Cannot read configuration file: {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Cannot parse configuration file: {}", path.display()))
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Cannot create configuration directory: {}",
                    parent.display()
                )
            })?;
        }

        let content =
            serde_json::to_string_pretty(self).with_context(|| "Cannot serialize configuration")?;

        fs::write(path, content)
            .with_context(|| format!("Cannot write configuration file: {}", path.display()))?;

        Ok(())
    }
}
