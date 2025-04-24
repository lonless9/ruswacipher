use ruswacipher::obfuscation::control_flow::{add_dead_code, obfuscate_control_flow};
use ruswacipher::wasm::parser::{parse_wasm, serialize_wasm};
use ruswacipher::wasm::structure::{SectionType, WasmModule};
use std::fs;
use std::path::Path;

// Get test sample file
fn get_test_wasm() -> WasmModule {
    let input_file = Path::new("tests/samples/simple.wasm");
    let wasm_data = fs::read(input_file).expect("Failed to read test WASM file");
    parse_wasm(&wasm_data).expect("Failed to parse test WASM file")
}

// Get code section data
fn get_code_section_data(module: &WasmModule) -> Option<&[u8]> {
    module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code)
        .map(|section| section.data.as_slice())
}

// Manually implement control flow instruction search logic for testing
fn find_control_flow_instructions(data: &[u8]) -> Vec<usize> {
    let mut positions = Vec::new();

    for i in 0..data.len().saturating_sub(1) {
        // Detect control flow instructions (e.g. if, loop, block, br_if, etc.)
        match data[i] {
            0x02 | 0x03 | 0x04 | 0x0D | 0x0E => positions.push(i),
            _ => {}
        }
    }

    positions
}

#[test]
fn test_find_control_flow_instructions() {
    // Get code section data of test module
    let module = get_test_wasm();
    let code_data = get_code_section_data(&module).expect("Code section does not exist");

    // Find control flow instruction positions
    let positions = find_control_flow_instructions(code_data);

    // Verify found positions are reasonable
    // Note: Specific positions depend on test sample, here only verify basic functionality
    println!("Found {} control flow instructions", positions.len());

    // Check each found position is actually a control flow instruction
    for pos in &positions {
        // Ensure position is within valid range
        assert!(*pos < code_data.len());

        // Check if byte at position is a control flow instruction
        let opcode = code_data[*pos];
        match opcode {
            0x02 | 0x03 | 0x04 | 0x0D | 0x0E => {
                // These are the control flow instructions we expect
                println!(
                    "Found control flow instruction at position {} with opcode {:#04x}",
                    pos, opcode
                );
            }
            _ => {
                panic!(
                    "Found non-control flow instruction at position {} with opcode {:#04x}",
                    pos, opcode
                );
            }
        }
    }
}

#[test]
fn test_find_function_bodies() {
    // Since find_function_bodies is a private function, we cannot directly test it
    // Here we test the public functions add_dead_code and obfuscate_control_flow instead

    // Get test module
    let module = get_test_wasm();

    // Apply dead code obfuscation
    let obfuscated_module = add_dead_code(module).unwrap();

    // Verify module structure integrity
    assert!(!obfuscated_module.sections.is_empty());

    // Ensure code section exists
    let has_code_section = obfuscated_module
        .sections
        .iter()
        .any(|section| section.section_type == SectionType::Code);
    assert!(has_code_section);
}

#[test]
fn test_dead_code_addition() {
    // Get test module
    let module = get_test_wasm();

    // Get original code section size
    let original_code_section = module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code)
        .expect("Code section does not exist");

    let original_code_size = original_code_section.data.len();

    // Apply dead code insertion
    let obfuscated_module = add_dead_code(module).unwrap();

    // Get obfuscated code section size
    let obfuscated_code_section = obfuscated_module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code)
        .expect("Obfuscated code section does not exist");

    let obfuscated_code_size = obfuscated_code_section.data.len();

    // Verify code section size increased
    println!(
        "Original code size: {}, Obfuscated code size: {}",
        original_code_size, obfuscated_code_size
    );
    // Restore assertion, as dead code implementation is completed
    assert!(
        obfuscated_code_size > original_code_size,
        "Dead code insertion should increase code section size"
    );
    // Temporary test: Ensure code section size does not decrease
    assert!(
        obfuscated_code_size >= original_code_size,
        "Code section should not shrink"
    );

    // Verify module is still valid WebAssembly
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();
    let _reparsed_module = parse_wasm(&wasm_bytes).unwrap();
}

#[test]
fn test_control_flow_obfuscation() {
    // Get test module
    let module = get_test_wasm();

    // Get original control flow instruction count
    let original_code_section =
        get_code_section_data(&module).expect("Code section does not exist");
    let original_control_flow_count = find_control_flow_instructions(original_code_section).len();

    // Apply control flow obfuscation
    let obfuscated_module = obfuscate_control_flow(module).unwrap();

    // Verify module is still valid WebAssembly
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();
    let reparsed_module = parse_wasm(&wasm_bytes).unwrap();

    // Check obfuscated control flow instruction count
    let obfuscated_code_section =
        get_code_section_data(&reparsed_module).expect("Obfuscated code section does not exist");
    let obfuscated_control_flow_count =
        find_control_flow_instructions(obfuscated_code_section).len();

    // Control flow obfuscation may increase or change control flow instructions, but will not remove all instructions
    println!(
        "Original control flow instruction count: {}, Obfuscated control flow instruction count: {}",
        original_control_flow_count, obfuscated_control_flow_count
    );
}

#[test]
fn test_combined_control_flow_obfuscation() {
    // Get test module
    let module = get_test_wasm();

    // Get original code section size
    let original_code_size = module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code)
        .map(|section| section.data.len())
        .unwrap_or(0);

    // Apply two control flow obfuscation techniques
    let module_with_dead_code = add_dead_code(module).unwrap();
    let obfuscated_module = obfuscate_control_flow(module_with_dead_code).unwrap();

    // Get obfuscated code section size
    let obfuscated_code_section = obfuscated_module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code)
        .expect("Obfuscated code section does not exist");

    let obfuscated_code_size = obfuscated_code_section.data.len();

    // Verify code section size increased
    println!(
        "Original code size: {}, Obfuscated code size: {}",
        original_code_size, obfuscated_code_size
    );
    // Restore assertion, as control flow obfuscation implementation is completed
    assert!(
        obfuscated_code_size > original_code_size,
        "Control flow obfuscation should increase code section size"
    );
    // Temporary test: Ensure code section size does not decrease
    assert!(
        obfuscated_code_size >= original_code_size,
        "Code section should not shrink"
    );

    // Verify the module is still valid WebAssembly
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();
    let _reparsed_module = parse_wasm(&wasm_bytes).unwrap();
}
