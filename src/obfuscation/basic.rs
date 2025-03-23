use crate::obfuscation::control_flow::{add_dead_code, obfuscate_control_flow};
use crate::obfuscation::function_split::split_large_functions;
use crate::obfuscation::types::{ObfuscationLevel, Transformation};
use crate::obfuscation::variable_obfuscation::rename_locals;
use crate::wasm::structure::WasmModule;
use anyhow::Result;

/// Apply multiple obfuscation transformations
pub fn apply_transformations(
    module: WasmModule,
    transformations: &[Transformation],
) -> Result<WasmModule> {
    transformations
        .iter()
        .try_fold(module, |m, transform| transform(m))
}

/// Get transformations for the specified obfuscation level
pub fn get_transformations(level: ObfuscationLevel) -> Vec<Transformation> {
    match level {
        ObfuscationLevel::Low => vec![rename_locals],
        ObfuscationLevel::Medium => vec![rename_locals, add_dead_code],
        ObfuscationLevel::High => vec![
            rename_locals,
            add_dead_code,
            obfuscate_control_flow,
            split_large_functions,
        ],
    }
}
