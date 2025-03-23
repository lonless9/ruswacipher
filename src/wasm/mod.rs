pub mod parser;
pub mod structure;

use anyhow::Result;
use std::path::Path;
use structure::WasmModule;

/// Load WASM module from file
pub fn load_module(path: &Path) -> Result<WasmModule> {
    parser::parse_file(path)
}

/// Save WASM module to file
pub fn save_module(module: &WasmModule, path: &Path) -> Result<()> {
    module.write_to_file(path)
}
