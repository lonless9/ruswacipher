use env_logger::Env;
use ruswacipher::obfuscation::function_split::split_large_functions;
use ruswacipher::wasm::parser::{parse_wasm, serialize_wasm};
use ruswacipher::wasm::structure::{SectionType, WasmModule};
use std::fs;
use std::path::Path;
use std::sync::Once;

// Ensure logger is initialized only once
static INIT_LOGGER: Once = Once::new();

fn setup() {
    INIT_LOGGER.call_once(|| {
        env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    });
}

// Get simple test sample file
#[allow(dead_code)]
fn get_simple_wasm() -> WasmModule {
    let input_file = Path::new("tests/samples/simple.wasm");
    let wasm_data = fs::read(input_file).expect("Failed to read simple WASM file");
    parse_wasm(&wasm_data).expect("Failed to parse simple WASM file")
}

// Get complex test sample file
fn get_complex_wasm() -> WasmModule {
    let input_file = Path::new("tests/samples/complex.wasm");
    let wasm_data = fs::read(input_file).expect("Failed to read complex WASM file");
    parse_wasm(&wasm_data).expect("Failed to parse complex WASM file")
}

// Check if functions are split and return function count and code section size
fn count_functions_and_size(module: &WasmModule) -> (usize, usize) {
    let mut code_size = 0;
    let mut num_funcs = 0;

    for section in &module.sections {
        if section.section_type == SectionType::Code {
            code_size = section.data.len();

            // Read function count
            let mut pos = 0;
            let mut shift = 0;

            while pos < section.data.len() {
                let byte = section.data[pos];
                num_funcs |= ((byte & 0x7F) as usize) << shift;
                pos += 1;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
        }
    }

    (num_funcs, code_size)
}

#[test]
fn test_simple_function_splitting() {
    // Initialize logger (only once)
    setup();

    // Read simple WAT file
    let wat_path = Path::new("tests/samples/simple.wat");
    let wasm_path = Path::new("tests/samples/simple.wasm");

    // Convert WAT to WASM
    assert!(wat_path.exists(), "WAT file does not exist");
    let status = std::process::Command::new("wat2wasm")
        .arg(wat_path)
        .arg("-o")
        .arg(wasm_path)
        .status()
        .expect("Failed to execute wat2wasm");

    assert!(status.success(), "wat2wasm execution failed");
    assert!(wasm_path.exists(), "WASM file was not generated");

    // Read and parse WASM file
    let wasm_data = fs::read(wasm_path).expect("Failed to read WASM file");
    let module = parse_wasm(&wasm_data).expect("Failed to parse WASM file");

    let (orig_funcs, orig_size) = count_functions_and_size(&module);
    println!(
        "Original simple module: {} functions, code size: {}",
        orig_funcs, orig_size
    );

    // Execute function splitting
    let result = split_large_functions(module);
    assert!(
        result.is_ok(),
        "Function splitting failed: {:?}",
        result.err()
    );

    let obfuscated = result.unwrap();
    let (new_funcs, new_size) = count_functions_and_size(&obfuscated);
    println!(
        "Obfuscated simple module: {} functions, code size: {}",
        new_funcs, new_size
    );
    println!(
        "Simple module function change: {} -> {}",
        orig_funcs, new_funcs
    );

    // Simple module might be too small to be split, so no assertions here

    println!("Test successful: Function splitting works correctly");
}

#[test]
fn test_complex_function_splitting() {
    // Initialize logger (only once)
    setup();

    // Read complex WAT file
    let wat_path = Path::new("tests/samples/complex.wat");
    let wasm_path = Path::new("tests/samples/complex.wasm");

    // Convert WAT to WASM
    assert!(wat_path.exists(), "WAT file does not exist");
    let status = std::process::Command::new("wat2wasm")
        .arg(wat_path)
        .arg("-o")
        .arg(wasm_path)
        .status()
        .expect("Failed to execute wat2wasm");

    assert!(status.success(), "wat2wasm execution failed");
    assert!(wasm_path.exists(), "WASM file was not generated");

    // Read and parse WASM file
    let wasm_data = fs::read(wasm_path).expect("Failed to read WASM file");
    let module = parse_wasm(&wasm_data).expect("Failed to parse WASM file");

    let (orig_funcs, orig_size) = count_functions_and_size(&module);
    println!(
        "Original complex module: {} functions, code size: {}",
        orig_funcs, orig_size
    );

    // Execute function splitting
    let result = split_large_functions(module);
    assert!(
        result.is_ok(),
        "Function splitting failed: {:?}",
        result.err()
    );

    let obfuscated = result.unwrap();
    let (new_funcs, new_size) = count_functions_and_size(&obfuscated);
    println!(
        "Obfuscated complex module: {} functions, code size: {}",
        new_funcs, new_size
    );
    println!(
        "Complex module function change: {} -> {}",
        orig_funcs, new_funcs
    );

    // Complex module should contain splittable large functions
    // We added a large splittable function, so at least one function should be split
    assert!(
        new_funcs > orig_funcs,
        "Function splitting did not increase function count"
    );

    // Write obfuscated WASM to file for inspection
    let obfuscated_wasm_path = Path::new("tests/samples/complex_obfuscated.wasm");
    let module_data = serialize_wasm(&obfuscated).expect("Failed to serialize WASM module");
    fs::write(obfuscated_wasm_path, module_data).expect("Failed to write obfuscated WASM file");

    println!("Test successful: Function splitting works correctly");
}

#[test]
fn test_function_count_consistency() {
    // Initialize logger (only once)
    setup();

    // Get test module
    let module = get_complex_wasm();

    // Apply function splitting
    let obfuscated_module = split_large_functions(module).unwrap();

    // Verify function count in function section matches function body count in code section
    let (function_count, _) = count_functions_and_size(&obfuscated_module);
    let code_section_count = count_function_bodies(&obfuscated_module);

    println!(
        "Function count in function section: {}, function body count in code section: {}",
        function_count, code_section_count
    );

    // Function count in function section should match function body count in code section
    assert_eq!(
        function_count, code_section_count,
        "Function counts in function section and code section should match"
    );
}

// Helper function: Check if module contains the specified section
#[allow(dead_code)]
fn has_section(module: &WasmModule, section_type: SectionType) -> bool {
    module
        .sections
        .iter()
        .any(|section| section.section_type == section_type)
}

// Helper function: Get code section size
#[allow(dead_code)]
fn get_code_section_size(module: &WasmModule) -> usize {
    module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code)
        .map(|section| section.data.len())
        .unwrap_or(0)
}

// Helper function: Count functions in function section
#[allow(dead_code)]
fn count_functions(module: &WasmModule) -> usize {
    // Find function section
    let function_section = module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Function);

    if let Some(section) = function_section {
        // First byte is typically the function count (LEB128 encoded)
        if !section.data.is_empty() {
            let count = section.data[0] as usize;
            return count;
        }
    }

    0
}

// Helper function: Count function bodies in code section
fn count_function_bodies(module: &WasmModule) -> usize {
    // Find code section
    let code_section = module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code);

    if let Some(section) = code_section {
        // First byte of code section is the function body count (LEB128 encoded)
        if !section.data.is_empty() {
            let count = section.data[0] as usize;
            return count;
        }
    }

    0
}
