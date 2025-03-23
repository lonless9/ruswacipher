use ruswacipher::obfuscation::virtualization::{
    find_virtualizable_functions, virtualize_functions,
};
use ruswacipher::wasm::parser::{parse_wasm, serialize_wasm};
use ruswacipher::wasm::structure::{SectionType, WasmModule};
use std::fs;

// Get test sample file
fn get_test_wasm() -> WasmModule {
    let wasm_data = fs::read("tests/samples/simple.wasm").expect("无法读取测试WASM文件");
    parse_wasm(&wasm_data).expect("无法解析测试WASM文件")
}

#[test]
fn test_find_virtualizable_functions() {
    // Get test module
    let module = get_test_wasm();

    // Find virtualizable functions
    let virtualizable_funcs = find_virtualizable_functions(&module).unwrap();

    // Verify operation does not crash and returns results
    println!(
        "Found {} virtualizable functions",
        virtualizable_funcs.len()
    );
}

#[test]
fn test_virtualization() {
    // Get test module
    let module = get_test_wasm();

    // Save original function and code section count
    let original_func_count = count_functions(&module);
    let original_code_size = get_code_section_size(&module);

    println!("Original module function count: {}", original_func_count);
    println!("Original module code size: {}", original_code_size);

    // Apply function virtualization
    let obfuscated_module = virtualize_functions(module).unwrap();

    // Verify module still valid
    assert!(has_code_section(&obfuscated_module));

    // Get new function and code section count
    let new_func_count = count_functions(&obfuscated_module);
    let new_code_size = get_code_section_size(&obfuscated_module);

    println!("Virtualized module function count: {}", new_func_count);
    println!("Virtualized module code size: {}", new_code_size);

    // Verify module can be serialized and reparsed
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();
    let reparsed_module = parse_wasm(&wasm_bytes).unwrap();

    // Ensure code section exists
    assert!(has_code_section(&reparsed_module));
}

// Helper function: Check if module has code section
fn has_code_section(module: &WasmModule) -> bool {
    module
        .sections
        .iter()
        .any(|section| section.section_type == SectionType::Code)
}

// Helper function: Count functions in module
fn count_functions(module: &WasmModule) -> usize {
    let function_section = module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Function);

    if let Some(section) = function_section {
        // Each function in the function section occupies one byte, so the size is equal to the number of functions
        return section.data.len();
    }
    0
}

// Helper function: Get code section size
fn get_code_section_size(module: &WasmModule) -> usize {
    let code_section = module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code);

    if let Some(section) = code_section {
        return section.data.len();
    }
    0
}
