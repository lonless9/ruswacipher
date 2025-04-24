use crate::obfuscation::ObfuscationLevel;
use crate::obfuscation::{
    add_dead_code, obfuscate_control_flow, rename_locals, split_large_functions,
    virtualize_functions,
};
use crate::wasm::structure::WasmModule;
use anyhow::Result;
use log::{debug, info};

/// Apply specified level of obfuscation to WebAssembly module
pub fn apply_obfuscation(module: WasmModule, level: ObfuscationLevel) -> Result<WasmModule> {
    info!("Applying obfuscation, level: {:?}", level);

    // Get transformations for current level
    let transformations = get_transformations_for_level(level);

    // Apply all transformations in sequence
    let mut result_module = module;
    for transform in transformations {
        debug!("Applying obfuscation transformation: {}", transform.name);
        result_module = (transform.function)(result_module)?;
    }

    info!("Obfuscation completed");
    Ok(result_module)
}

/// Obfuscation transformation description
struct Transformation {
    name: &'static str,
    function: fn(WasmModule) -> Result<WasmModule>,
}

/// Get transformation set based on obfuscation level
fn get_transformations_for_level(level: ObfuscationLevel) -> Vec<Transformation> {
    match level {
        ObfuscationLevel::Low => vec![Transformation {
            name: "Local variable renaming",
            function: rename_locals,
        }],

        ObfuscationLevel::Medium => vec![
            Transformation {
                name: "Local variable renaming",
                function: rename_locals,
            },
            Transformation {
                name: "Add dead code",
                function: add_dead_code,
            },
        ],

        ObfuscationLevel::High => vec![
            Transformation {
                name: "Local variable renaming",
                function: rename_locals,
            },
            Transformation {
                name: "Add dead code",
                function: add_dead_code,
            },
            Transformation {
                name: "Control flow obfuscation",
                function: obfuscate_control_flow,
            },
            Transformation {
                name: "Function splitting",
                function: split_large_functions,
            },
            Transformation {
                name: "Virtualization protection",
                function: virtualize_functions,
            },
        ],
    }
}
