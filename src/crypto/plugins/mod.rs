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

    // 注册ChaCha20-Poly1305插件
    let chacha_plugin = super::algorithms::chacha20poly1305::ChaCha20Poly1305Plugin::new();
    manager.register_plugin(Arc::new(chacha_plugin));
}

/// 通过插件加密数据
pub fn encrypt_with_plugin(data: &[u8], key: &[u8], plugin_name: &str) -> Result<Vec<u8>> {
    let manager = PLUGIN_MANAGER.lock().unwrap();

    if let Some(plugin) = manager.get_plugin(plugin_name) {
        plugin.encrypt(data, key)
    } else {
        anyhow::bail!("未找到插件: {}", plugin_name)
    }
}

/// 通过插件解密数据
pub fn decrypt_with_plugin(data: &[u8], key: &[u8], plugin_name: &str) -> Result<Vec<u8>> {
    let manager = PLUGIN_MANAGER.lock().unwrap();

    if let Some(plugin) = manager.get_plugin(plugin_name) {
        plugin.decrypt(data, key)
    } else {
        anyhow::bail!("未找到插件: {}", plugin_name)
    }
}

/// 加载自定义插件目录中的插件
pub fn load_custom_plugins() -> Result<()> {
    // 检查环境变量 RUSWACIPHER_PLUGIN_PATH
    let plugin_path = match env::var("RUSWACIPHER_PLUGIN_PATH") {
        Ok(path) => path,
        Err(_) => {
            debug!("未设置RUSWACIPHER_PLUGIN_PATH环境变量，跳过自定义插件加载");
            return Ok(());
        }
    };

    info!("从目录加载自定义插件: {}", plugin_path);

    // 检查目录是否存在
    let plugin_dir = Path::new(&plugin_path);
    if !plugin_dir.is_dir() {
        warn!("插件目录不存在或不是一个目录: {}", plugin_path);
        return Ok(()); // 不返回错误，因为这可能是正常的配置
    }

    // TODO: 这里是动态库加载的骨架，实际实现需要使用libloading等库
    info!("自定义插件加载尚未完全实现");

    // 在此框架中，我们将输出找到的文件用于日志显示
    for entry in fs::read_dir(plugin_dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    // 在Linux上寻找.so文件，在Windows上寻找.dll文件，在macOS上寻找.dylib文件
                    #[cfg(target_os = "linux")]
                    let valid_ext = ext == "so";

                    #[cfg(target_os = "windows")]
                    let valid_ext = ext == "dll";

                    #[cfg(target_os = "macos")]
                    let valid_ext = ext == "dylib";

                    if valid_ext {
                        info!("找到插件文件: {}", path.display());
                        // TODO: 使用libloading加载动态库
                    }
                }
            }
        }
    }

    Ok(())
}
