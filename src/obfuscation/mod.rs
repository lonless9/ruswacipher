pub mod basic;
pub mod control_flow;
pub mod engine;
pub mod function_split;
pub mod interpreter;
pub mod types;
pub mod variable_obfuscation;
pub mod virtualization;
pub mod vm;

use crate::wasm;
use crate::wasm::structure::WasmModule;
use anyhow::Result;
use log::info;
use std::path::Path;

// Re-export main types from types
pub use types::{ObfuscationError, ObfuscationLevel};

// Re-export main functions from various modules
pub use basic::{apply_transformations, get_transformations};
pub use control_flow::{add_dead_code, obfuscate_control_flow};
pub use function_split::split_large_functions;
pub use interpreter::execute_vm_bytecode;
pub use variable_obfuscation::rename_locals;
pub use virtualization::{find_virtualizable_functions, virtualize_functions};

// Export functions from engine module
pub use engine::apply_obfuscation;

/// Obfuscate a WASM module with the specified level
pub fn obfuscate(module: WasmModule, level: ObfuscationLevel) -> Result<WasmModule> {
    info!("Starting WASM module obfuscation, level: {:?}", level);

    // Get obfuscation transformations
    let transformations = get_transformations(level);

    // Apply all transformations
    let obfuscated_module = apply_transformations(module, &transformations)?;

    info!("WASM module obfuscation completed");
    Ok(obfuscated_module)
}

/// Apply default system obfuscation transformations
pub fn apply_default_obfuscation(
    module: WasmModule,
    level: ObfuscationLevel,
) -> Result<WasmModule> {
    info!("Applying default obfuscation, level: {:?}", level);

    // Apply system default obfuscation methods
    let result = match level {
        ObfuscationLevel::Low => {
            // Low-level obfuscation: only rename local variables
            rename_locals(module)
        }
        ObfuscationLevel::Medium => {
            // Medium-level obfuscation: renaming + adding dead code
            let module = rename_locals(module)?;
            add_dead_code(module)
        }
        ObfuscationLevel::High => {
            // High-level obfuscation: all available obfuscation techniques
            let module = rename_locals(module)?;
            let module = add_dead_code(module)?;
            let module = obfuscate_control_flow(module)?;
            let module = split_large_functions(module)?;
            virtualize_functions(module)
        }
    };

    info!("Default obfuscation applied successfully");
    result
}

/// Obfuscate a WASM file without encryption (new function)
///
/// # Parameters
///
/// * `input_file` - Path to the input WASM file
/// * `output_file` - Path to the output obfuscated WASM file
/// * `level` - Obfuscation level to apply
pub fn obfuscate_wasm_only(
    input_file: &Path,
    output_file: &Path,
    level: ObfuscationLevel,
) -> Result<()> {
    info!(
        "Starting WASM file obfuscation only (without encryption): {} -> {}",
        input_file.display(),
        output_file.display()
    );

    // 1. Parse WASM file
    let wasm_data = std::fs::read(input_file)?;
    let module = wasm::parser::parse_wasm(&wasm_data)?;

    // 2. Apply obfuscation
    let obfuscated_module = obfuscate(module, level)?;

    // 3. Serialize back to binary
    let obfuscated_data = wasm::parser::serialize_wasm(&obfuscated_module)?;

    // 4. Write to output file
    std::fs::write(output_file, &obfuscated_data)?;

    info!("WASM file obfuscation completed (no encryption applied)");
    Ok(())
}

/// Obfuscate a WASM file and encrypt the result (renamed from previous obfuscate_wasm)
///
/// # Parameters
///
/// * `input_file` - Path to the input WASM file
/// * `output_file` - Path to the output obfuscated and encrypted WASM file
/// * `level` - Obfuscation level to apply
/// * `algorithm` - Optional encryption algorithm to use (defaults to "aes-gcm" if None)
pub fn obfuscate_and_encrypt_wasm(
    input_file: &Path,
    output_file: &Path,
    level: ObfuscationLevel,
    algorithm: Option<&str>,
) -> Result<()> {
    info!(
        "Starting WASM file obfuscation and encryption: {} -> {}",
        input_file.display(),
        output_file.display()
    );
    if let Some(alg) = algorithm {
        info!("Using encryption algorithm: {}", alg);
    }

    // 1. Parse WASM file
    let wasm_data = std::fs::read(input_file)?;
    let module = wasm::parser::parse_wasm(&wasm_data)?;

    // 2. Apply obfuscation
    let obfuscated_module = obfuscate(module, level)?;

    // 3. Serialize back to binary
    let obfuscated_data = wasm::parser::serialize_wasm(&obfuscated_module)?;

    // 4. Encrypt the result with specified algorithm or default
    let algorithm_str = algorithm.unwrap_or("aes-gcm");
    let key = crate::crypto::engine::generate_key(32); // Generate a random key

    let encrypted_data =
        crate::crypto::engine::encrypt_data(&obfuscated_data, &key, algorithm_str)?;

    // 5. Write to output file
    std::fs::write(output_file, &encrypted_data)?;

    // 6. Save key to file - use the same naming convention as in tests
    let key_path = output_file.with_extension("wasm.key");
    crate::crypto::engine::save_key(&key, &key_path)?;
    info!("Generated key and saved to: {}", key_path.display());

    info!("WASM file obfuscation and encryption completed");
    Ok(())
}

/// For backward compatibility - calls obfuscate_and_encrypt_wasm
///
/// # Parameters
///
/// * `input_file` - Path to the input WASM file
/// * `output_file` - Path to the output obfuscated and encrypted WASM file
/// * `level` - Obfuscation level to apply
/// * `algorithm` - Optional encryption algorithm to use (defaults to "aes-gcm" if None)
pub fn obfuscate_wasm(
    input_file: &Path,
    output_file: &Path,
    level: ObfuscationLevel,
    algorithm: Option<&str>,
) -> Result<()> {
    obfuscate_and_encrypt_wasm(input_file, output_file, level, algorithm)
}

/// Get description for specified obfuscation level
pub fn get_level_description(level: ObfuscationLevel) -> &'static str {
    match level {
        ObfuscationLevel::Low => "Basic obfuscation - Local variable renaming",
        ObfuscationLevel::Medium => "Medium obfuscation - Adding redundant code and control flow obfuscation",
        ObfuscationLevel::High => "Advanced obfuscation - High-intensity obfuscation including code splitting and virtualization",
    }
}
