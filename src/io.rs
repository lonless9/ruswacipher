use std::fs;
use std::path::Path;

use crate::error::Result;
use base64::Engine;

/// Read a file into a byte vector
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let data = fs::read(path)?;
    Ok(data)
}

/// Write bytes to a file
pub fn write_file<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<()> {
    fs::write(path, data)?;
    Ok(())
}

/// Read a key file and return the key bytes
pub fn read_key_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let content = fs::read_to_string(path)?;
    let content = content.trim();

    // Try to decode as hex first, then as base64
    if let Ok(key) = hex::decode(content) {
        Ok(key)
    } else if let Ok(key) = base64::engine::general_purpose::STANDARD.decode(content) {
        Ok(key)
    } else {
        // Assume it's raw bytes
        Ok(content.as_bytes().to_vec())
    }
}

/// Write a key to a file in hex format
pub fn write_key_file<P: AsRef<Path>>(path: P, key: &[u8]) -> Result<()> {
    let hex_key = hex::encode(key);
    fs::write(path, hex_key)?;
    Ok(())
}

/// Write a key to a file in the specified format
pub fn write_key_file_with_format<P: AsRef<Path>>(
    path: P,
    key: &[u8],
    format: &crate::cli::KeyFormat,
) -> Result<()> {
    let content = match format {
        crate::cli::KeyFormat::Hex => hex::encode(key),
        crate::cli::KeyFormat::Base64 => {
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key)
        }
        crate::cli::KeyFormat::Raw => {
            // For raw format, write binary data directly
            fs::write(path, key)?;
            return Ok(());
        }
    };

    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_write_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, World!";

        write_file(temp_file.path(), test_data).unwrap();
        let read_data = read_file(temp_file.path()).unwrap();

        assert_eq!(test_data, read_data.as_slice());
    }

    #[test]
    fn test_key_file_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_key = b"test_key_123456789012345678901234";

        write_key_file(temp_file.path(), test_key).unwrap();
        let read_key = read_key_file(temp_file.path()).unwrap();

        assert_eq!(test_key, read_key.as_slice());
    }

    #[test]
    fn test_write_key_file_with_format_hex() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_key = b"test_key_12345678901234567890123";

        write_key_file_with_format(temp_file.path(), test_key, &crate::cli::KeyFormat::Hex)
            .unwrap();
        let content = std::fs::read_to_string(temp_file.path()).unwrap();

        assert_eq!(content, hex::encode(test_key));
    }

    #[test]
    fn test_write_key_file_with_format_base64() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_key = b"test_key_12345678901234567890123";

        write_key_file_with_format(temp_file.path(), test_key, &crate::cli::KeyFormat::Base64)
            .unwrap();
        let content = std::fs::read_to_string(temp_file.path()).unwrap();

        assert_eq!(
            content,
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, test_key)
        );
    }

    #[test]
    fn test_write_key_file_with_format_raw() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_key = b"test_key_12345678901234567890123";

        write_key_file_with_format(temp_file.path(), test_key, &crate::cli::KeyFormat::Raw)
            .unwrap();
        let content = std::fs::read(temp_file.path()).unwrap();

        assert_eq!(content, test_key);
    }

    #[test]
    fn test_read_key_file_hex_format() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_key = b"test_key_12345678901234567890123";
        let hex_content = hex::encode(test_key);

        std::fs::write(temp_file.path(), hex_content).unwrap();
        let read_key = read_key_file(temp_file.path()).unwrap();

        assert_eq!(read_key, test_key);
    }

    #[test]
    fn test_read_key_file_base64_format() {
        let temp_file = NamedTempFile::new().unwrap();
        let test_key = b"test_key_12345678901234567890123";
        let base64_content =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, test_key);

        std::fs::write(temp_file.path(), base64_content).unwrap();
        let read_key = read_key_file(temp_file.path()).unwrap();

        assert_eq!(read_key, test_key);
    }
}
