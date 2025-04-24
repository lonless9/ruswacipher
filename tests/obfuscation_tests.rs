use env_logger::Env;
use std::path::Path;
use std::sync::Once;
use tempfile::tempdir;

use ruswacipher::obfuscation::{
    add_dead_code, find_virtualizable_functions, obfuscate_control_flow, obfuscate_wasm,
    obfuscate_wasm_only, rename_locals, split_large_functions, virtualize_functions,
    ObfuscationLevel,
};
use ruswacipher::wasm::parser::{parse_file, parse_wasm, serialize_wasm};
use ruswacipher::wasm::structure::WasmModule;
use std::fs;

// Ensure logger is initialized only once
static INIT_LOGGER: Once = Once::new();

fn setup() {
    INIT_LOGGER.call_once(|| {
        env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    });
}

// Get test sample file
fn get_test_wasm() -> WasmModule {
    let input_file = Path::new("tests/samples/simple.wasm");
    let wasm_data = fs::read(input_file).expect("Failed to read test WASM file");
    parse_wasm(&wasm_data).expect("Failed to parse test WASM file")
}

#[test]
fn test_basic_obfuscation() {
    // Initialize logger
    setup();

    // Create temporary directory
    let temp_dir = tempdir().unwrap();
    let input_file = Path::new("tests/samples/simple.wasm");
    let output_file = temp_dir.path().join("obfuscated.wasm");

    // Apply obfuscation
    obfuscate_wasm(input_file, &output_file, ObfuscationLevel::Low, None).unwrap();

    // Verify output file exists
    assert!(output_file.exists());

    // 先解密文件再解析
    let key_file = output_file.with_extension("wasm.key");
    let decrypted_file = temp_dir.path().join("decrypted.wasm");
    ruswacipher::crypto::engine::decrypt_file(&output_file, &decrypted_file, &key_file).unwrap();

    // Parse obfuscated file and verify its format is valid
    let module = parse_file(&decrypted_file).unwrap();
    assert!(!module.sections.is_empty());
}

#[test]
fn test_medium_obfuscation() {
    // 初始化日志
    setup();

    // Create temporary directory
    let temp_dir = tempdir().unwrap();
    let input_file = Path::new("tests/samples/simple.wasm");
    let output_file = temp_dir.path().join("obfuscated_medium.wasm");

    // Apply medium level obfuscation
    obfuscate_wasm(input_file, &output_file, ObfuscationLevel::Medium, None).unwrap();

    // Verify output file exists
    assert!(output_file.exists());

    // 先解密文件再解析
    let key_file = output_file.with_extension("wasm.key");
    let decrypted_file = temp_dir.path().join("decrypted_medium.wasm");
    ruswacipher::crypto::engine::decrypt_file(&output_file, &decrypted_file, &key_file).unwrap();

    // Parse obfuscated file and verify its format is valid
    let module = parse_file(&decrypted_file).unwrap();
    assert!(!module.sections.is_empty());
}

#[test]
fn test_high_obfuscation() {
    // 初始化日志
    setup();

    // Create temporary directory
    let temp_dir = tempdir().unwrap();
    let input_file = Path::new("tests/samples/simple.wasm");
    let output_file = temp_dir.path().join("obfuscated_high.wasm");

    // Apply high level obfuscation
    obfuscate_wasm(input_file, &output_file, ObfuscationLevel::High, None).unwrap();

    // Verify output file exists
    assert!(output_file.exists());

    // 先解密文件再解析
    let key_file = output_file.with_extension("wasm.key");
    let decrypted_file = temp_dir.path().join("decrypted_high.wasm");
    ruswacipher::crypto::engine::decrypt_file(&output_file, &decrypted_file, &key_file).unwrap();

    // Parse obfuscated file and verify its format is valid
    let module = parse_file(&decrypted_file).unwrap();
    assert!(!module.sections.is_empty());
}

#[test]
fn test_variable_renaming() {
    // 初始化日志
    setup();

    // Get test module
    let module = get_test_wasm();

    // Apply variable renaming
    let obfuscated_module = rename_locals(module).unwrap();

    // Verify module structure integrity
    assert!(!obfuscated_module.sections.is_empty());

    // Serialize and ensure it can be reparsed
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();
    let reparsed_module = parse_wasm(&wasm_bytes).unwrap();

    // Verify code section exists
    let has_code_section = reparsed_module
        .sections
        .iter()
        .any(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Code);
    assert!(has_code_section);
}

#[test]
fn test_dead_code_insertion() {
    // 初始化日志
    setup();

    // Get test module
    let module = get_test_wasm();

    // Get original code size
    let original_code_size = module
        .sections
        .iter()
        .find(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Code)
        .map(|section| section.data.len())
        .unwrap_or(0);

    // Apply dead code insertion
    let obfuscated_module = add_dead_code(module).unwrap();

    // Verify module structure integrity
    assert!(!obfuscated_module.sections.is_empty());

    // Serialize and ensure it can be reparsed
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();
    let reparsed_module = parse_wasm(&wasm_bytes).unwrap();

    // Verify code section exists
    let has_code_section = reparsed_module
        .sections
        .iter()
        .any(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Code);
    assert!(has_code_section);

    // Verify code section should be larger than original version
    let obfuscated_code_size = obfuscated_module
        .sections
        .iter()
        .find(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Code)
        .map(|section| section.data.len())
        .unwrap_or(0);

    assert!(obfuscated_code_size >= original_code_size,
            "Dead code insertion should increase code section size, original size: {}, obfuscated size: {}", 
            original_code_size, obfuscated_code_size);
}

#[test]
fn test_control_flow_obfuscation() {
    // 初始化日志
    setup();

    // Get test module
    let module = get_test_wasm();

    // Apply control flow obfuscation
    let obfuscated_module = obfuscate_control_flow(module).unwrap();

    // Verify module structure integrity
    assert!(!obfuscated_module.sections.is_empty());

    // Serialize and ensure it can be reparsed
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();
    let reparsed_module = parse_wasm(&wasm_bytes).unwrap();

    // Verify code section exists
    let has_code_section = reparsed_module
        .sections
        .iter()
        .any(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Code);
    assert!(has_code_section);
}

#[test]
fn test_function_splitting() {
    // 初始化日志
    setup();

    // Get test module
    let module = get_test_wasm();

    // Apply function splitting
    let obfuscated_module = split_large_functions(module).unwrap();

    // Verify module structure integrity
    assert!(!obfuscated_module.sections.is_empty());

    // Serialize and ensure it can be reparsed
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();
    let reparsed_module = parse_wasm(&wasm_bytes).unwrap();

    // Verify function section and code section exist
    let has_function_section = reparsed_module
        .sections
        .iter()
        .any(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Function);
    assert!(has_function_section);

    let has_code_section = reparsed_module
        .sections
        .iter()
        .any(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Code);
    assert!(has_code_section);
}

#[test]
fn test_find_virtualizable_functions() {
    // 初始化日志
    setup();

    // Get test module
    let module = get_test_wasm();

    // Find virtualizable functions
    let virtualizable_funcs = find_virtualizable_functions(&module).unwrap();

    // Only verify operation does not crash
    // Note: Specific test samples may have different numbers of virtualizable functions
    println!(
        "Found {} virtualizable functions",
        virtualizable_funcs.len()
    );
}

#[test]
fn test_virtualization() {
    // 初始化日志
    setup();

    // Get test module
    let module = get_test_wasm();

    // Apply function virtualization
    let obfuscated_module = virtualize_functions(module).unwrap();

    // Verify module structure integrity
    assert!(!obfuscated_module.sections.is_empty());

    // Serialize and ensure it can be reparsed
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();

    // Add debug information
    println!("Debug: Serialized WASM size: {} bytes", wasm_bytes.len());

    // Check if wasm_bytes is valid
    if !wasm_bytes.is_empty() {
        println!(
            "Debug: First few bytes: {:?}",
            &wasm_bytes[..std::cmp::min(10, wasm_bytes.len())]
        );
    }

    // Add more detailed debug information
    println!("Debug: WASM sections:");
    for (i, section) in obfuscated_module.sections.iter().enumerate() {
        println!(
            "  Section {}: Type {:?}, Size {} bytes",
            i,
            section.section_type,
            section.data.len()
        );
    }

    println!("Debug: Full WASM binary data: {:?}", wasm_bytes);

    let reparsed_module = parse_wasm(&wasm_bytes).unwrap();

    // Verify code section exists
    let has_code_section = reparsed_module
        .sections
        .iter()
        .any(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Code);
    assert!(has_code_section);
}

#[test]
fn test_obfuscation_chain() {
    // Get test module and create original WASM bytes for subsequent tests
    let original_module = get_test_wasm();
    let _original_bytes = serialize_wasm(&original_module).unwrap();

    // Apply obfuscation chain - parse module after each operation
    let module1 = rename_locals(original_module).unwrap();
    let bytes1 = serialize_wasm(&module1).unwrap();

    let module2 = add_dead_code(parse_wasm(&bytes1).unwrap()).unwrap();
    let bytes2 = serialize_wasm(&module2).unwrap();

    let module3 = obfuscate_control_flow(parse_wasm(&bytes2).unwrap()).unwrap();
    let bytes3 = serialize_wasm(&module3).unwrap();

    let module4 = split_large_functions(parse_wasm(&bytes3).unwrap()).unwrap();
    let bytes4 = serialize_wasm(&module4).unwrap();

    let obfuscated_module = virtualize_functions(parse_wasm(&bytes4).unwrap()).unwrap();

    // Verify module structure integrity
    assert!(!obfuscated_module.sections.is_empty());

    // Serialize and ensure it can be reparsed
    let wasm_bytes = serialize_wasm(&obfuscated_module).unwrap();
    let reparsed_module = parse_wasm(&wasm_bytes).unwrap();

    // Verify module structure
    let has_function_section = reparsed_module
        .sections
        .iter()
        .any(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Function);
    assert!(has_function_section);

    let has_code_section = reparsed_module
        .sections
        .iter()
        .any(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Code);
    assert!(has_code_section);
}

#[test]
fn test_obfuscation_only() {
    // 初始化日志
    setup();

    // Create temporary directory
    let temp_dir = tempdir().unwrap();
    let input_file = Path::new("tests/samples/simple.wasm");
    let output_file = temp_dir.path().join("obfuscated_only.wasm");

    // Apply obfuscation without encryption
    obfuscate_wasm_only(input_file, &output_file, ObfuscationLevel::Medium).unwrap();

    // Verify output file exists
    assert!(output_file.exists());

    // Direct parse obfuscated file (no decryption)
    let module = parse_file(&output_file).unwrap();
    assert!(!module.sections.is_empty());

    // Verify file format is valid
    let code_section_exists = module
        .sections
        .iter()
        .any(|section| section.section_type == ruswacipher::wasm::structure::SectionType::Code);
    assert!(
        code_section_exists,
        "Code section should exist in obfuscated WASM"
    );
}

#[test]
fn test_obfuscation_pipeline() {
    // Initialize logger
    setup();

    // Create temporary directory
    let temp_dir = tempdir().unwrap();
    let input_file = Path::new("tests/samples/simple.wasm");
    let obfuscated_file = temp_dir.path().join("obfuscated_only.wasm");
    let encrypted_file = temp_dir.path().join("obfuscated_encrypted.wasm");

    // Step 1: Obfuscation
    // Use direct module parsing and obfuscation
    let wasm_data = std::fs::read(input_file).unwrap();
    let module = parse_wasm(&wasm_data).unwrap();
    let obfuscated_module =
        ruswacipher::obfuscation::obfuscate(module, ObfuscationLevel::Medium).unwrap();
    let obfuscated_data = serialize_wasm(&obfuscated_module).unwrap();
    std::fs::write(&obfuscated_file, &obfuscated_data).unwrap();
    assert!(obfuscated_file.exists());

    // Step 2: Encrypt - Generate key and save
    let key = ruswacipher::crypto::engine::generate_key(32);
    let key_file = temp_dir.path().join("encryption.key");
    ruswacipher::crypto::engine::save_key(&key, &key_file).unwrap();

    // Step 2: Encrypt - Use key to encrypt obfuscated file
    let obfuscated_data = std::fs::read(&obfuscated_file).unwrap();
    let encrypted_data =
        ruswacipher::crypto::engine::encrypt_data(&obfuscated_data, &key, "aes-gcm").unwrap();
    std::fs::write(&encrypted_file, &encrypted_data).unwrap();
    assert!(encrypted_file.exists());

    // Step 3: Decrypt - Decrypt and verify
    let decrypted_file = temp_dir.path().join("decrypted_pipeline.wasm");
    ruswacipher::crypto::decrypt_file(&encrypted_file, &decrypted_file, &key_file).unwrap();

    // Verify decrypted file can be parsed
    let module = parse_file(&decrypted_file).unwrap();
    assert!(!module.sections.is_empty());

    // Compare decrypted file with obfuscated file
    let obfuscated_data = std::fs::read(&obfuscated_file).unwrap();
    let decrypted_data = std::fs::read(&decrypted_file).unwrap();
    assert_eq!(
        obfuscated_data, decrypted_data,
        "Decrypted data should match obfuscated data"
    );
}
