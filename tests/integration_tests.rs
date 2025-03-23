use std::path::Path;
use tempfile::tempdir;
use anyhow::Result;
use std::fs;
use std::time::Instant;

use ruswacipher::obfuscation::{
    obfuscate_wasm, ObfuscationLevel, 
    rename_locals, add_dead_code, obfuscate_control_flow, 
    split_large_functions, virtualize_functions
};
use ruswacipher::wasm::parser::{parse_file, parse_wasm, serialize_wasm};
use ruswacipher::wasm::structure::{WasmModule, SectionType};

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
    
    // Analyze one-step obfuscation result
    let obfuscated_module_1 = parse_file(&output_file)?;
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
    println!("One-time API generated file size: {} bytes", fs::metadata(&output_file)?.len());
    println!("Manual process generated file size: {} bytes", fs::metadata(&output_file_manual)?.len());
    
    // Verify obfuscated WASM is valid
    println!("\nVerifying obfuscated WASM module...");
    
    // Check if export functions still exist
    let has_exports = fully_obfuscated_module.sections.iter()
        .any(|section| section.section_type == SectionType::Export);
    assert!(has_exports, "Export section missing after obfuscation");
    
    // Verify module can be reparsed
    let reparsed_module = parse_wasm(&obfuscated_wasm)?;
    assert!(!reparsed_module.sections.is_empty(), "Obfuscated module cannot be correctly parsed");
    
    println!("Full obfuscation test pipeline completed!");
    Ok(())
}

/// Analyze WASM module and print information
fn analyze_module(module: &WasmModule, stage_name: &str) {
    println!("\n--- {} Analysis ---", stage_name);
    
    // Calculate code section size
    let code_size = module.sections.iter()
        .find(|s| s.section_type == SectionType::Code)
        .map_or(0, |s| s.data.len());
    
    // Calculate section count
    let section_count = module.sections.len();
    
    // Determine function count
    let func_count = module.sections.iter()
        .find(|s| s.section_type == SectionType::Function)
        .map_or(0, |s| s.data.len());
    
    // Print analysis results
    println!("Section count: {}", section_count);
    println!("Code section size: {} bytes", code_size);
    println!("Function count: {}", func_count);
    
    // List all sections
    println!("Section list:");
    for (i, section) in module.sections.iter().enumerate() {
        println!("  {}: {:?} - {} bytes", i, section.section_type, section.data.len());
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
    for level in [ObfuscationLevel::Low, ObfuscationLevel::Medium, ObfuscationLevel::High] {
        let output_file = temp_dir.path().join(format!("obfuscated_{:?}.wasm", level));
        println!("\nApplying {:?} level obfuscation...", level);
        
        // Apply obfuscation
        let start = Instant::now();
        obfuscate_wasm(input_file, &output_file, level, None)?;
        let duration = start.elapsed();
        
        // Get file size
        let file_size = fs::metadata(&output_file)?.len();
        
        // Verify generated file
        let obfuscated_module = parse_file(&output_file)?;
        
        // Print results
        println!("Obfuscation level: {:?}", level);
        println!("Processing time: {:?}", duration);
        println!("File size: {} bytes", file_size);
        println!("Section count: {}", obfuscated_module.sections.len());
        
        // Verify all necessary sections exist
        assert!(obfuscated_module.sections.iter().any(|s| s.section_type == SectionType::Code),
               "Code section missing after obfuscation");
        assert!(obfuscated_module.sections.iter().any(|s| s.section_type == SectionType::Function),
               "Function section missing after obfuscation");
        assert!(obfuscated_module.sections.iter().any(|s| s.section_type == SectionType::Export),
               "Export section missing after obfuscation");
    }
    
    println!("\nComprehensive security obfuscation test completed!");
    Ok(())
} 