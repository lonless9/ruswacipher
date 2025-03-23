use anyhow::Result;
use std::fs;
use std::path::Path;

// Import the module we're testing
use ruswacipher::wasm::{
    self,
    structure::{Section, SectionType, WasmModule},
};

#[test]
fn test_parse_and_load_wasm() -> Result<()> {
    // Test loading a WASM file
    let wasm_path = Path::new("tests/samples/simple.wasm");
    let module = wasm::load_module(wasm_path)?;

    // Verify that we have some sections
    assert!(
        !module.sections.is_empty(),
        "WASM module should have sections"
    );

    // Verify the WASM version
    assert_eq!(module.version, 1, "WASM version should be 1");

    Ok(())
}

#[test]
fn test_parse_and_serialize() -> Result<()> {
    // Test parsing and then serializing back
    let wasm_path = Path::new("tests/samples/simple.wasm");
    let original_data = fs::read(wasm_path)?;

    // Parse into a module
    let module = wasm::parser::parse_binary(&original_data)?;

    // Serialize back to binary
    let serialized_data = wasm::parser::serialize_wasm(&module)?;

    // The serialized data should match the original data (or at least be valid WASM)
    // We won't do an exact comparison since serialization might reorder some sections
    // but we can at least verify it's valid WASM by parsing it again
    let _reparsed_module = wasm::parser::parse_binary(&serialized_data)?;

    Ok(())
}

#[test]
fn test_custom_sections() -> Result<()> {
    // Create a new WASM module
    let mut module = WasmModule::new();

    // Add a custom section
    let custom_data = vec![1, 2, 3, 4];
    wasm::parser::add_custom_section(&mut module, "test_section", custom_data.clone())?;

    // Verify the section was added
    let custom_sections = module.get_custom_sections("test_section");
    assert_eq!(custom_sections.len(), 1, "Should have one custom section");
    assert_eq!(
        custom_sections[0].data, custom_data,
        "Custom section data should match"
    );

    // Remove the custom section
    let removed = wasm::parser::remove_custom_section(&mut module, "test_section")?;
    assert!(removed, "Custom section should be removed");

    // Verify the section was removed
    let custom_sections = module.get_custom_sections("test_section");
    assert!(
        custom_sections.is_empty(),
        "Custom section should be removed"
    );

    Ok(())
}

#[test]
fn test_add_section() -> Result<()> {
    // Create a new WASM module
    let mut module = WasmModule::new();

    // Add a section
    let section = Section {
        section_type: SectionType::Custom,
        name: Some("test_section".to_string()),
        data: vec![1, 2, 3, 4],
    };

    module.add_section(section);

    // Verify the section was added
    assert_eq!(module.sections.len(), 1, "Module should have one section");
    assert_eq!(
        module.sections[0].section_type,
        SectionType::Custom,
        "Section type should be Custom"
    );
    assert_eq!(
        module.sections[0].name.as_ref().unwrap(),
        "test_section",
        "Section name should match"
    );
    assert_eq!(
        module.sections[0].data,
        vec![1, 2, 3, 4],
        "Section data should match"
    );

    Ok(())
}

#[test]
fn test_write_and_read_roundtrip() -> Result<()> {
    // Create a module with a custom section
    let mut module = WasmModule::new();
    wasm::parser::add_custom_section(&mut module, "test_section", vec![1, 2, 3, 4])?;

    // Write to a temporary file
    let temp_file = Path::new("target/test_wasm_file.wasm");
    wasm::save_module(&module, temp_file)?;

    // Read it back
    let loaded_module = wasm::load_module(temp_file)?;

    // Verify the section is still there
    let custom_sections = loaded_module.get_custom_sections("test_section");
    assert_eq!(
        custom_sections.len(),
        1,
        "Should have one custom section after roundtrip"
    );
    assert_eq!(
        custom_sections[0].data,
        vec![1, 2, 3, 4],
        "Custom section data should match after roundtrip"
    );

    // Clean up
    fs::remove_file(temp_file)?;

    Ok(())
}

#[test]
fn test_custom_section_in_real_wasm() -> Result<()> {
    // Load an existing WASM file
    let wasm_path = Path::new("tests/samples/simple.wasm");
    let original_data = fs::read(wasm_path)?;
    let mut module = wasm::parser::parse_binary(&original_data)?;

    // Add a custom section
    let section_name = "test_metadata";
    let section_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    wasm::parser::add_custom_section(&mut module, section_name, section_data.clone())?;

    // Verify custom section was added
    let custom_sections = module.get_custom_sections(section_name);
    assert_eq!(custom_sections.len(), 1, "Should have one custom section");
    assert_eq!(
        custom_sections[0].data, section_data,
        "Custom section data should match"
    );

    // Serialize to binary
    let serialized = wasm::parser::serialize_wasm(&module)?;

    // Parse again and check if section is still there
    let reparsed_module = wasm::parser::parse_binary(&serialized)?;
    let custom_sections = reparsed_module.get_custom_sections(section_name);
    assert_eq!(
        custom_sections.len(),
        1,
        "Should still have the custom section after serialization"
    );
    assert_eq!(
        custom_sections[0].data, section_data,
        "Custom section data should still match after serialization"
    );

    // Now remove the custom section
    let removed = wasm::parser::remove_custom_section(&mut module, section_name)?;
    assert!(removed, "Custom section should be removed");

    // Verify it's gone
    let custom_sections = module.get_custom_sections(section_name);
    assert!(
        custom_sections.is_empty(),
        "Custom section should be removed"
    );

    Ok(())
}
