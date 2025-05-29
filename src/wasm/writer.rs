use crate::error::Result;

pub struct WasmWriter;

impl WasmWriter {
    /// Write WASM data to a file
    pub fn write_wasm_file<P: AsRef<std::path::Path>>(path: P, data: &[u8]) -> Result<()> {
        crate::io::write_file(path, data)
    }

    /// Validate and write WASM data to a file
    pub fn write_validated_wasm_file<P: AsRef<std::path::Path>>(
        path: P,
        data: &[u8],
    ) -> Result<()> {
        // Validate the WASM data before writing
        crate::wasm::WasmParser::validate_wasm(data)?;
        Self::write_wasm_file(path, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_wasm_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_data = b"test wasm data";

        WasmWriter::write_wasm_file(temp_file.path(), test_data).unwrap();
        let written_data = crate::io::read_file(temp_file.path()).unwrap();

        assert_eq!(test_data, written_data.as_slice());
    }
}
