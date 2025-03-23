use anyhow::{Result, Context, anyhow};
use std::fs;
use std::path::Path;
use rand::Rng;

// Import plugin system
use super::plugins;

/// Encrypt WASM file
pub fn encrypt_file(input: &Path, output: &Path, key_file: Option<&Path>, algorithm: &str) -> Result<()> {
    let data = fs::read(input)
        .with_context(|| format!("Cannot read input file: {}", input.display()))?;

    // Get or generate key
    let key = match key_file {
        Some(path) => load_key(path)?,
        None => {
            let key = generate_key(32);
            // Save to same directory as output
            let key_path = output.with_file_name(format!(
                "{}.key", 
                output.file_stem().unwrap_or_default().to_string_lossy()
            ));
            save_key(&key, &key_path)?;
            println!("Generated key and saved to: {}", key_path.display());
            key
        }
    };

    // Use encryption plugin to encrypt data
    let encrypted = encrypt_data(&data, &key, algorithm)?;

    // Write encrypted data
    fs::write(output, encrypted)
        .with_context(|| format!("Cannot write to output file: {}", output.display()))?;

    Ok(())
}

/// Decrypt WASM file
pub fn decrypt_file(input: &Path, output: &Path, key_file: &Path) -> Result<()> {
    let data = fs::read(input)
        .with_context(|| format!("Cannot read encrypted file: {}", input.display()))?;

    let key = load_key(key_file)?;

    // Use encryption plugin to decrypt data
    let decrypted = decrypt_data(&data, &key)
        .with_context(|| "Decryption failed")?;

    fs::write(output, decrypted)
        .with_context(|| format!("Cannot write to decrypted file: {}", output.display()))?;

    Ok(())
}

/// Encrypt data with specified algorithm
pub fn encrypt_data(data: &[u8], key: &[u8], algorithm: &str) -> Result<Vec<u8>> {
    // Initialize plugin system (if needed)
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        plugins::register_builtin_plugins();
        let _ = plugins::load_custom_plugins();
    });
    
    // Use plugin to encrypt
    let encrypted = plugins::encrypt_with_plugin(data, key, algorithm)?;
    
    // Create compatible header format for JavaScript runtime
    // Get encryption algorithm information
    let mut header_map = serde_json::Map::new();
    header_map.insert("algorithm".to_string(), serde_json::Value::String(algorithm.to_string()));
    
    // For AES-GCM algorithm, nonce is in the first 12 bytes
    // For ChaCha20Poly1305 algorithm, nonce is in the first 12 bytes
    if algorithm == "aes-gcm" || algorithm == "chacha20poly1305" {
        if encrypted.len() >= 12 {
            // Add nonce to header
            let nonce = &encrypted[..12];
            let nonce_values: Vec<u8> = nonce.to_vec();
            header_map.insert("nonce".to_string(), serde_json::Value::Array(
                nonce_values.into_iter().map(|b| serde_json::Value::Number(serde_json::Number::from(b))).collect()
            ));
            
            // Generate JSON header
            let header_json = serde_json::Value::Object(header_map);
            let header_string = serde_json::to_string(&header_json)?;
            let header_bytes = header_string.as_bytes();
            
            // Create new output format:
            // 4-byte header length + JSON header + encrypted data (excluding first 12 bytes nonce as it's already in header)
            let mut result = Vec::new();
            
            // Write 4-byte header length (little endian)
            let header_len = header_bytes.len() as u32;
            result.extend_from_slice(&header_len.to_le_bytes());
            
            // Write header
            result.extend_from_slice(header_bytes);
            
            // Write encrypted data (skip first 12 bytes nonce as it's already in header)
            result.extend_from_slice(&encrypted[12..]);
            
            return Ok(result);
        }
    }
    
    // For other algorithms or error cases, use old format
    // Add algorithm identifier header
    let mut result = Vec::with_capacity(algorithm.len() + 1 + encrypted.len());
    result.push(algorithm.len() as u8);  // Algorithm name length
    result.extend_from_slice(algorithm.as_bytes());  // Algorithm name
    result.append(&mut encrypted.clone());  // Encrypted data
    
    Ok(result)
}

/// Decrypt data
pub fn decrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(anyhow!("Encrypted data is empty"));
    }
    
    // Initialize plugin system (if needed) - moved to beginning to ensure all paths can use plugins
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        plugins::register_builtin_plugins();
        let _ = plugins::load_custom_plugins();
    });
    
    // First try to parse new format (JavaScript compatible format)
    if data.len() >= 4 {
        // Try to read header length (4 bytes, little endian)
        let header_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        
        if data.len() >= 4 + header_len {
            // Get JSON header
            let header_bytes = &data[4..(4 + header_len)];
            
            // Try to parse JSON
            if let Ok(header_text) = std::str::from_utf8(header_bytes) {
                if let Ok(header_json) = serde_json::from_str::<serde_json::Value>(header_text) {
                    // Check if header is valid
                    if let Some(algorithm) = header_json.get("algorithm").and_then(|v| v.as_str()) {
                        if let Some(nonce_array) = header_json.get("nonce").and_then(|v| v.as_array()) {
                            // Extract nonce
                            let mut nonce = Vec::with_capacity(nonce_array.len());
                            for value in nonce_array {
                                if let Some(byte) = value.as_u64() {
                                    nonce.push(byte as u8);
                                } else {
                                    return Err(anyhow!("Invalid nonce value"));
                                }
                            }
                            
                            // Get encrypted data
                            let encrypted_data = &data[(4 + header_len)..];
                            
                            // For AES-GCM and ChaCha20Poly1305, need to add nonce back to encrypted data
                            if algorithm == "aes-gcm" || algorithm == "chacha20poly1305" {
                                let mut complete_data = Vec::with_capacity(nonce.len() + encrypted_data.len());
                                complete_data.extend_from_slice(&nonce);
                                complete_data.extend_from_slice(encrypted_data);
                                
                                // Use plugin to decrypt
                                return plugins::decrypt_with_plugin(&complete_data, key, algorithm);
                            }
                            
                            // For other algorithms, decrypt directly using plugin
                            return plugins::decrypt_with_plugin(encrypted_data, key, algorithm);
                        }
                    }
                }
            }
        }
    }
    
    // If unable to parse new format, try old format
    // Read algorithm identifier
    let algo_len = data[0] as usize;
    if data.len() < 1 + algo_len {
        return Err(anyhow!("Invalid encrypted data format"));
    }
    
    let algorithm = std::str::from_utf8(&data[1..1+algo_len])
        .map_err(|_| anyhow!("Invalid algorithm name (UTF-8)"))?;
    
    // Get encrypted data portion
    let encrypted_data = &data[1+algo_len..];
    
    // Use plugin to decrypt
    plugins::decrypt_with_plugin(encrypted_data, key, algorithm)
}

/// Generate random key
pub fn generate_key(length: usize) -> Vec<u8> {
    let mut key = vec![0u8; length];
    rand::rng().fill(&mut key[..]);
    key
}

/// Save key to file
pub fn save_key(key: &[u8], path: &Path) -> Result<()> {
    let base64_key = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key);
    fs::write(path, &base64_key)
        .with_context(|| format!("Cannot save key to file: {}", path.display()))?;
    Ok(())
}

/// Load key from file
pub fn load_key(path: &Path) -> Result<Vec<u8>> {
    let base64_key = fs::read_to_string(path)
        .with_context(|| format!("Cannot read key file: {}", path.display()))?;
    
    let key = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_key.trim())
        .with_context(|| "Key file contains invalid Base64 data")?;
    
    Ok(key)
} 