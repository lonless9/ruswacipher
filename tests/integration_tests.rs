use anyhow::Result;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;

use ruswacipher::crypto;
use ruswacipher::obfuscation::{
    self, add_dead_code, obfuscate_control_flow, obfuscate_wasm, rename_locals,
    split_large_functions, virtualize_functions, ObfuscationLevel,
};
use ruswacipher::wasm::parser::{parse_file, parse_wasm, serialize_wasm};
use ruswacipher::wasm::structure::{SectionType, WasmModule};

/// Execute the complete obfuscation test pipeline
#[test]
fn test_full_obfuscation_pipeline() -> Result<()> {
    println!("Starting comprehensive obfuscation test pipeline...");

    // Create temporary directory
    let temp_dir = tempdir()?;
    let input_file = Path::new("tests/samples/simple.wasm");
    let output_file = temp_dir.path().join("obfuscated_full.wasm");

    // Verify input file exists
    assert!(input_file.exists(), "Test sample file does not exist");

    // Load original WASM
    println!("Loading original WASM file...");
    let original_wasm = fs::read(input_file)?;
    let original_module = parse_wasm(&original_wasm)?;

    // Analyze original module
    analyze_module(&original_module, "Original Module");

    // Start timing
    let start_time = Instant::now();

    // Method 1: Apply all obfuscations at once using high-level API (fast path)
    println!("\nMethod 1: Applying all obfuscations at once using high-level API...");
    obfuscate_wasm(input_file, &output_file, ObfuscationLevel::High, None)?;

    // For encrypted output, the direct parsing will fail.
    // Let's also create a decrypted output for verification
    let key_file = output_file.with_extension("wasm.key");
    let decrypted_file = temp_dir.path().join("decrypted_full.wasm");
    ruswacipher::crypto::engine::decrypt_file(&output_file, &decrypted_file, &key_file)?;

    // Analyze obfuscation result using the decrypted file
    let obfuscated_module_1 = parse_file(&decrypted_file)?;
    analyze_module(&obfuscated_module_1, "After High-Level Obfuscation");

    // Method 2: Manually apply each obfuscation step (detailed path)
    println!("\nMethod 2: Manually applying obfuscation techniques step by step...");

    // Step 1: Variable renaming
    println!("Step 1: Applying variable renaming...");
    let module_after_rename = rename_locals(original_module.clone())?;
    analyze_module(&module_after_rename, "After Variable Renaming");

    // Step 2: Add dead code
    println!("Step 2: Adding dead code...");
    let module_after_dead_code = add_dead_code(module_after_rename)?;
    analyze_module(&module_after_dead_code, "After Adding Dead Code");

    // Step 3: Control flow obfuscation
    println!("Step 3: Applying control flow obfuscation...");
    let module_after_control_flow = obfuscate_control_flow(module_after_dead_code)?;
    analyze_module(&module_after_control_flow, "After Control Flow Obfuscation");

    // Step 4: Function splitting
    println!("Step 4: Applying function splitting...");
    let module_after_function_split = split_large_functions(module_after_control_flow)?;
    analyze_module(&module_after_function_split, "After Function Splitting");

    // Step 5: Function virtualization
    println!("Step 5: Applying function virtualization...");
    let fully_obfuscated_module = virtualize_functions(module_after_function_split)?;
    analyze_module(&fully_obfuscated_module, "After Function Virtualization");

    // Calculate total time
    let duration = start_time.elapsed();
    println!("\nTotal obfuscation time: {:?}", duration);

    // Save manually obfuscated result
    let output_file_manual = temp_dir.path().join("obfuscated_manual.wasm");
    let obfuscated_wasm = serialize_wasm(&fully_obfuscated_module)?;
    fs::write(&output_file_manual, &obfuscated_wasm)?;

    // Compare results of both methods
    println!("\nComparison of obfuscation methods:");
    println!(
        "One-time API generated file size: {} bytes",
        fs::metadata(&output_file)?.len()
    );
    println!(
        "Manual process generated file size: {} bytes",
        fs::metadata(&output_file_manual)?.len()
    );

    // Verify obfuscated WASM is valid
    println!("\nVerifying obfuscated WASM module...");

    // Check if export functions still exist
    let has_exports = fully_obfuscated_module
        .sections
        .iter()
        .any(|section| section.section_type == SectionType::Export);
    assert!(has_exports, "Export section missing after obfuscation");

    // Verify module can be reparsed
    let reparsed_module = parse_wasm(&obfuscated_wasm)?;
    assert!(
        !reparsed_module.sections.is_empty(),
        "Obfuscated module cannot be correctly parsed"
    );

    println!("Full obfuscation test pipeline completed!");
    Ok(())
}

/// Analyze WASM module and print information
fn analyze_module(module: &WasmModule, stage_name: &str) {
    println!("\n--- {} Analysis ---", stage_name);

    // Calculate code section size
    let code_size = module
        .sections
        .iter()
        .find(|s| s.section_type == SectionType::Code)
        .map_or(0, |s| s.data.len());

    // Calculate section count
    let section_count = module.sections.len();

    // Determine function count
    let func_count = module
        .sections
        .iter()
        .find(|s| s.section_type == SectionType::Function)
        .map_or(0, |s| s.data.len());

    // Print analysis results
    println!("Section count: {}", section_count);
    println!("Code section size: {} bytes", code_size);
    println!("Function count: {}", func_count);

    // List all sections
    println!("Section list:");
    for (i, section) in module.sections.iter().enumerate() {
        println!(
            "  {}: {:?} - {} bytes",
            i,
            section.section_type,
            section.data.len()
        );
    }
}

/// More complex security obfuscation test, testing various types of WASM files
#[test]
fn test_comprehensive_obfuscation() -> Result<()> {
    // Create temporary directory
    let temp_dir = tempdir()?;
    let input_file = Path::new("tests/samples/simple.wasm");

    println!("Executing comprehensive security obfuscation test...");

    // Test different obfuscation levels
    for level in [
        ObfuscationLevel::Low,
        ObfuscationLevel::Medium,
        ObfuscationLevel::High,
    ] {
        let output_file = temp_dir.path().join(format!("obfuscated_{:?}.wasm", level));
        println!("\nApplying {:?} level obfuscation...", level);

        // Apply obfuscation
        let start = Instant::now();
        obfuscate_wasm(input_file, &output_file, level, None)?;
        let duration = start.elapsed();

        // Get file size
        let file_size = fs::metadata(&output_file)?.len();

        // Decrypt the file for verification
        let key_file = output_file.with_extension("wasm.key");
        let decrypted_file = temp_dir.path().join(format!("decrypted_{:?}.wasm", level));
        ruswacipher::crypto::engine::decrypt_file(&output_file, &decrypted_file, &key_file)?;

        // Verify generated file
        let obfuscated_module = parse_file(&decrypted_file)?;

        // Print results
        println!("Obfuscation level: {:?}", level);
        println!("Processing time: {:?}", duration);
        println!("File size: {} bytes", file_size);
        println!("Section count: {}", obfuscated_module.sections.len());

        // Verify all necessary sections exist
        assert!(
            obfuscated_module
                .sections
                .iter()
                .any(|s| s.section_type == SectionType::Code),
            "Code section missing after obfuscation"
        );
        assert!(
            obfuscated_module
                .sections
                .iter()
                .any(|s| s.section_type == SectionType::Function),
            "Function section missing after obfuscation"
        );
        assert!(
            obfuscated_module
                .sections
                .iter()
                .any(|s| s.section_type == SectionType::Export),
            "Export section missing after obfuscation"
        );
    }

    println!("\nComprehensive security obfuscation test completed!");
    Ok(())
}

/// Add a new test function to test the correct obfuscation and encryption pipeline
#[test]
fn test_correct_obfuscation_encryption_pipeline() -> Result<()> {
    println!("Testing the correct obfuscation and encryption pipeline...");

    // Create temporary directory
    let temp_dir = tempdir()?;
    let input_file = Path::new("tests/samples/simple.wasm");
    let obfuscated_file = temp_dir.path().join("obfuscated.wasm");
    let encrypted_file = temp_dir.path().join("encrypted.wasm");

    // Verify input file exists
    assert!(input_file.exists(), "Test sample file does not exist");

    // Load original WASM
    println!("Loading original WASM file...");
    let original_wasm = fs::read(input_file)?;
    let original_module = parse_wasm(&original_wasm)?;
    analyze_module(&original_module, "Original Module");

    // Step 1: Applying obfuscation
    println!("Step 1: Applying obfuscation...");
    // Use direct module parsing to apply obfuscation
    let wasm_data = fs::read(input_file)?;
    let module = parse_wasm(&wasm_data)?;
    let obfuscated_module = obfuscation::obfuscate(module, ObfuscationLevel::High)?;
    let obfuscated_data = serialize_wasm(&obfuscated_module)?;
    fs::write(&obfuscated_file, &obfuscated_data)?;

    // Verify obfuscated file
    let obfuscated_module = parse_file(&obfuscated_file)?;
    analyze_module(&obfuscated_module, "After Obfuscation");

    // Step 2: Encrypting obfuscated file
    println!("Step 2: Encrypting obfuscated file...");

    // Generate random key and save to temporary file
    let key = crypto::engine::generate_key(32);
    let key_file_path = temp_dir.path().join("encryption.key");
    crypto::engine::save_key(&key, &key_file_path)?;

    // Encrypt obfuscated file using key
    let obfuscated_data = fs::read(&obfuscated_file)?;
    let encrypted_data = crypto::engine::encrypt_data(&obfuscated_data, &key, "aes-gcm")?;
    fs::write(&encrypted_file, &encrypted_data)?;

    // Verify encrypted file exists
    assert!(encrypted_file.exists(), "Encrypted file does not exist");
    println!(
        "File size after encryption: {} bytes",
        fs::metadata(&encrypted_file)?.len()
    );

    // Step 3: Decrypting file
    println!("Step 3: Decrypting file...");
    let decrypted_file = temp_dir.path().join("decrypted.wasm");
    crypto::decrypt_file(&encrypted_file, &decrypted_file, &key_file_path)?;

    // Verify decrypted file
    let decrypted_module = parse_file(&decrypted_file)?;
    analyze_module(&decrypted_module, "After Decryption");

    // Verify decrypted file matches obfuscated file
    let obfuscated_data = fs::read(&obfuscated_file)?;
    let decrypted_data = fs::read(&decrypted_file)?;
    assert_eq!(
        obfuscated_data, decrypted_data,
        "Decrypted data should match obfuscated data"
    );

    // Verify decrypted file is a valid WASM module
    assert!(
        decrypted_module
            .sections
            .iter()
            .any(|s| s.section_type == SectionType::Code),
        "Code section missing after pipeline"
    );
    assert!(
        decrypted_module
            .sections
            .iter()
            .any(|s| s.section_type == SectionType::Export),
        "Export section missing after pipeline"
    );

    println!("Correct obfuscation and encryption pipeline test completed!");
    Ok(())
}
