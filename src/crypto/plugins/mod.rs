use anyhow::Result;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Encryption plugin interface
pub trait EncryptionPlugin: Send + Sync {
    /// Get plugin name
    fn name(&self) -> &str;

    /// Get plugin description
    fn description(&self) -> &str;

    /// Encrypt data
    fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt data
    fn decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>>;
}

/// Plugin manager
#[derive(Default)]
pub struct PluginManager {
    plugins: HashMap<String, Arc<dyn EncryptionPlugin>>,
}

// Global plugin manager singleton
lazy_static::lazy_static! {
    static ref PLUGIN_MANAGER: Mutex<PluginManager> = Mutex::new(PluginManager::new());
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
        }
    }

    /// Register plugin
    pub fn register_plugin(&mut self, plugin: Arc<dyn EncryptionPlugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    /// Get plugin
    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn EncryptionPlugin>> {
        self.plugins.get(name).cloned()
    }

    /// Get all plugin names
    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}

/// Get global plugin manager
pub fn get_plugin_manager() -> &'static Mutex<PluginManager> {
    &PLUGIN_MANAGER
}

/// Register built-in plugins
pub fn register_builtin_plugins() {
    let mut manager = PLUGIN_MANAGER.lock().unwrap();

    // Register AES-GCM plugin
    let aes_plugin = super::algorithms::aes_gcm::AesGcmPlugin::new();
    manager.register_plugin(Arc::new(aes_plugin));

    // Register ChaCha20-Poly1305 plugin
    let chacha_plugin = super::algorithms::chacha20poly1305::ChaCha20Poly1305Plugin::new();
    manager.register_plugin(Arc::new(chacha_plugin));
}

/// Encrypt data with plugin
pub fn encrypt_with_plugin(data: &[u8], key: &[u8], plugin_name: &str) -> Result<Vec<u8>> {
    let manager = PLUGIN_MANAGER.lock().unwrap();

    if let Some(plugin) = manager.get_plugin(plugin_name) {
        plugin.encrypt(data, key)
    } else {
        anyhow::bail!("Plugin not found: {}", plugin_name)
    }
}

/// Decrypt data with plugin
pub fn decrypt_with_plugin(data: &[u8], key: &[u8], plugin_name: &str) -> Result<Vec<u8>> {
    let manager = PLUGIN_MANAGER.lock().unwrap();

    if let Some(plugin) = manager.get_plugin(plugin_name) {
        plugin.decrypt(data, key)
    } else {
        anyhow::bail!("Plugin not found: {}", plugin_name)
    }
}

/// Load custom plugins from directory
pub fn load_custom_plugins() -> Result<()> {
    // Check environment variable RUSWACIPHER_PLUGIN_PATH
    let plugin_path = match env::var("RUSWACIPHER_PLUGIN_PATH") {
        Ok(path) => path,
        Err(_) => {
            debug!("RUSWACIPHER_PLUGIN_PATH environment variable not set, skipping custom plugin loading");
            return Ok(());
        }
    };

    info!("Loading custom plugins from directory: {}", plugin_path);

    // Check if directory exists
    let plugin_dir = Path::new(&plugin_path);
    if !plugin_dir.is_dir() {
        warn!(
            "Plugin directory does not exist or is not a directory: {}",
            plugin_path
        );
        return Ok(()); // Don't return error, as this may be a normal configuration
    }

    // List found files for logging purposes
    for entry in (fs::read_dir(plugin_dir)?).flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                // Look for .so files on Linux, .dll files on Windows, .dylib files on macOS
                #[cfg(target_os = "linux")]
                let valid_ext = ext == "so";

                #[cfg(target_os = "windows")]
                let valid_ext = ext == "dll";

                #[cfg(target_os = "macos")]
                let valid_ext = ext == "dylib";

                if valid_ext {
                    info!("Found plugin file: {}", path.display());

                    // Use libloading to load dynamic library
                    unsafe {
                        match libloading::Library::new(path.as_path()) {
                            Ok(lib) => {
                                // Get plugin creation function
                                let create_plugin: libloading::Symbol<
                                    fn() -> Box<dyn EncryptionPlugin>,
                                > = match lib.get(b"create_plugin") {
                                    Ok(func) => func,
                                    Err(e) => {
                                        warn!("Unable to get plugin creation function, skipping plugin: {} - {}", path.display(), e);
                                        continue;
                                    }
                                };

                                // Call creation function to get plugin instance
                                let plugin = create_plugin();
                                let plugin_name = plugin.name().to_string();

                                info!(
                                    "Successfully loaded plugin: {} - {}",
                                    plugin_name,
                                    plugin.description()
                                );

                                // Register plugin
                                let mut manager = PLUGIN_MANAGER.lock().unwrap();
                                manager.register_plugin(Arc::from(plugin));

                                // Don't release the library, as the plugin needs to be used continuously
                                // Store the library somewhere to prevent it from being dropped (simplified handling here)
                                std::mem::forget(lib);
                            }
                            Err(e) => {
                                warn!("Unable to load plugin library: {} - {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
