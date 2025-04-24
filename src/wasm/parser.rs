use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use wasmparser::{Parser, Payload, Validator, WasmFeatures};

use super::structure::{Section, SectionType, WasmModule};

/// Parse WASM file
pub fn parse_file(path: &Path) -> Result<WasmModule> {
    let wasm_bytes =
        fs::read(path).with_context(|| format!("Cannot read WASM file: {}", path.display()))?;

    parse_binary(&wasm_bytes)
}

/// Parse WASM binary data
pub fn parse_binary(data: &[u8]) -> Result<WasmModule> {
    // Validate WASM binary
    let features = WasmFeatures::default();
    let mut validator = Validator::new_with_features(features);
    validator
        .validate_all(data)
        .with_context(|| "Invalid WASM binary data")?;

    // Parse WASM structure
    let mut module = WasmModule::default();

    for payload in Parser::new(0).parse_all(data) {
        let payload = payload.with_context(|| "Failed to parse WASM binary")?;
        process_payload(&mut module, &payload, data)?;
    }

    Ok(module)
}

/// Process WASM section payload
fn process_payload(module: &mut WasmModule, payload: &Payload, data: &[u8]) -> Result<()> {
    match payload {
        Payload::Version { num, .. } => {
            module.version = *num as u32;
        }
        Payload::CustomSection(section) => {
            module.sections.push(Section {
                section_type: SectionType::Custom,
                name: Some(section.name().to_string()),
                data: section.data().to_vec(),
            });
        }
        // Use macros to handle similar section types
        section @ Payload::TypeSection(_) => {
            add_section(module, section, SectionType::Type, data);
        }
        section @ Payload::ImportSection(_) => {
            add_section(module, section, SectionType::Import, data);
        }
        section @ Payload::FunctionSection(_) => {
            add_section(module, section, SectionType::Function, data);
        }
        section @ Payload::TableSection(_) => {
            add_section(module, section, SectionType::Table, data);
        }
        section @ Payload::MemorySection(_) => {
            add_section(module, section, SectionType::Memory, data);
        }
        section @ Payload::GlobalSection(_) => {
            add_section(module, section, SectionType::Global, data);
        }
        section @ Payload::ExportSection(_) => {
            add_section(module, section, SectionType::Export, data);
        }
        Payload::StartSection { func: _, range } => {
            add_section_from_range(module, range, SectionType::Start, data);
        }
        section @ Payload::ElementSection(_) => {
            add_section(module, section, SectionType::Element, data);
        }
        Payload::CodeSectionStart {
            count: _, range, ..
        } => {
            add_section_from_range(module, range, SectionType::Code, data);
        }
        section @ Payload::DataSection(_) => {
            add_section(module, section, SectionType::Data, data);
        }
        Payload::DataCountSection { count: _, range } => {
            add_section_from_range(module, range, SectionType::DataCount, data);
        }
        // Ignore other parts like function bodies, data items, etc., as they are already included in their respective sections
        _ => {}
    }

    Ok(())
}

/// Add section based on section type
fn add_section(module: &mut WasmModule, payload: &Payload, section_type: SectionType, data: &[u8]) {
    let range = match payload {
        Payload::TypeSection(reader) => reader.range(),
        Payload::ImportSection(reader) => reader.range(),
        Payload::FunctionSection(reader) => reader.range(),
        Payload::TableSection(reader) => reader.range(),
        Payload::MemorySection(reader) => reader.range(),
        Payload::GlobalSection(reader) => reader.range(),
        Payload::ExportSection(reader) => reader.range(),
        Payload::ElementSection(reader) => reader.range(),
        Payload::DataSection(reader) => reader.range(),
        _ => return, // Should not reach here
    };

    add_section_from_range(module, &range, section_type, data);
}

/// Add section from range
fn add_section_from_range(
    module: &mut WasmModule,
    range: &std::ops::Range<usize>,
    section_type: SectionType,
    data: &[u8],
) {
    let section_data = extract_section_data(data, range);
    module.sections.push(Section {
        section_type,
        name: None,
        data: section_data,
    });
}

/// Extract section data from raw binary data
fn extract_section_data(data: &[u8], range: &std::ops::Range<usize>) -> Vec<u8> {
    data[range.clone()].to_vec()
}

/// Add custom section to WASM module
pub fn add_custom_section(module: &mut WasmModule, name: &str, data: Vec<u8>) -> Result<()> {
    let section = Section {
        section_type: SectionType::Custom,
        name: Some(name.to_string()),
        data,
    };

    module.add_section(section);
    Ok(())
}

/// Remove custom section with specified name from WASM module
pub fn remove_custom_section(module: &mut WasmModule, name: &str) -> Result<bool> {
    let original_count = module.sections.len();

    module.sections.retain(|section| {
        section.section_type != SectionType::Custom
            || section.name.as_ref().is_none_or(|n| n != name)
    });

    Ok(original_count != module.sections.len())
}

/// Parse WASM binary data and return WasmModule structure
pub fn parse_wasm(data: &[u8]) -> Result<WasmModule> {
    parse_binary(data)
}

/// Serialize WasmModule structure to WASM binary data
pub fn serialize_wasm(module: &WasmModule) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // Write WASM magic number
    buffer.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]);

    // Write version number
    buffer.extend_from_slice(&module.version.to_le_bytes());

    // Write all sections
    for section in &module.sections {
        // Get section data with proper structure
        let section_data = prepare_section_data(section)?;
        buffer.extend_from_slice(&section_data);
    }

    Ok(buffer)
}

/// Prepare section data for serialization
fn prepare_section_data(section: &Section) -> Result<Vec<u8>> {
    let mut section_buffer = Vec::new();

    // Write section type ID
    let section_id = section.section_type.to_id();
    section_buffer.push(section_id);

    // For custom sections, prepare name field
    let mut content_buffer = Vec::new();
    if section.section_type == SectionType::Custom {
        if let Some(name) = &section.name {
            // Write name length (LEB128 encoded)
            write_unsigned_leb128(&mut content_buffer, name.len() as u64);
            // Write name
            content_buffer.extend_from_slice(name.as_bytes());
        } else {
            // Custom sections must have a name
            write_unsigned_leb128(&mut content_buffer, 0); // Empty name
        }
    }

    // Add section data
    content_buffer.extend_from_slice(&section.data);

    // Write section size (LEB128 encoded)
    write_unsigned_leb128(&mut section_buffer, content_buffer.len() as u64);

    // Add content to the section buffer
    section_buffer.extend_from_slice(&content_buffer);

    Ok(section_buffer)
}

/// Encode unsigned integer in LEB128 format
fn write_unsigned_leb128(buffer: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buffer.push(byte);
        if value == 0 {
            break;
        }
    }
}
