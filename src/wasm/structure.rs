use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// WebAssembly module structure
#[derive(Debug, Default, Clone)]
pub struct WasmModule {
    /// WASM version
    pub version: u32,
    /// Module sections list
    pub sections: Vec<Section>,
}

/// WASM section types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    DataCount,
}

impl SectionType {
    /// Get the ID corresponding to the section type
    pub fn to_id(&self) -> u8 {
        match self {
            SectionType::Custom => 0,
            SectionType::Type => 1,
            SectionType::Import => 2,
            SectionType::Function => 3,
            SectionType::Table => 4,
            SectionType::Memory => 5,
            SectionType::Global => 6,
            SectionType::Export => 7,
            SectionType::Start => 8,
            SectionType::Element => 9,
            SectionType::Code => 10,
            SectionType::Data => 11,
            SectionType::DataCount => 12,
        }
    }
}

/// WASM section structure
#[derive(Debug, Clone)]
pub struct Section {
    /// Section type
    pub section_type: SectionType,
    /// Section name (only for custom sections)
    pub name: Option<String>,
    /// Section data
    pub data: Vec<u8>,
}

impl WasmModule {
    /// Create a new empty module
    pub fn new() -> Self {
        WasmModule {
            version: 1, // WebAssembly version 1
            sections: Vec::new(),
        }
    }

    /// Get custom sections
    pub fn get_custom_sections(&self, name: &str) -> Vec<&Section> {
        self.sections
            .iter()
            .filter(|section| {
                section.section_type == SectionType::Custom
                    && section.name.as_ref().map_or(false, |n| n == name)
            })
            .collect()
    }

    /// Add a new section
    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    /// Write module to file
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)
            .with_context(|| format!("Cannot create file: {}", path.display()))?;

        // Use the serialize_wasm function from parser module to get the binary data
        let wasm_data = crate::wasm::parser::serialize_wasm(self)?;

        // Write the serialized data to file
        file.write_all(&wasm_data)
            .with_context(|| "Failed to write WASM data")?;

        Ok(())
    }
}
