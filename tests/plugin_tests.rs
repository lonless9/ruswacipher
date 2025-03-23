use ruswacipher::crypto::plugins;
use tempfile::tempdir;
use std::path::Path;
use std::fs;

/// Initialize plugin system
fn init() {
    plugins::register_builtin_plugins();
}

#[test]
fn test_aes_gcm_plugin() {
    init();
    
    let data = b"Hello, WebAssembly encryption!";
    let key = [1u8; 32]; // 32字节密钥
    
    // Use AES-GCM plugin to encrypt
    let encrypted = plugins::encrypt_with_plugin(data, &key, "aes-gcm").unwrap();
    
    // Verify encrypted result is not equal to original
    assert_ne!(&encrypted[..], &data[..]);
    
    // Decrypt
    let decrypted = plugins::decrypt_with_plugin(&encrypted, &key, "aes-gcm").unwrap();
    
    // Verify decrypted result is equal to original
    assert_eq!(&decrypted[..], &data[..]);
}

#[test]
fn test_chacha20poly1305_plugin() {
    init();
    
    let data = b"Hello, WebAssembly encryption!";
    let key = [2u8; 32]; // 32字节密钥
    
    // Use ChaCha20-Poly1305 plugin to encrypt
    let encrypted = plugins::encrypt_with_plugin(data, &key, "chacha20poly1305").unwrap();
    
    // Verify encrypted result is not equal to original
    assert_ne!(&encrypted[..], &data[..]);
    
    // Decrypt
    let decrypted = plugins::decrypt_with_plugin(&encrypted, &key, "chacha20poly1305").unwrap();
    
    // Verify decrypted result is equal to original
    assert_eq!(&decrypted[..], &data[..]);
}

#[test]
fn test_plugin_with_files() {
    init();
    
    // Create temporary directory
    let temp_dir = tempdir().unwrap();
    let input_file = temp_dir.path().join("input.txt");
    let encrypted_file = temp_dir.path().join("encrypted.bin");
    let decrypted_file = temp_dir.path().join("decrypted.txt");
    
    // Write test data
    let test_data = b"This is a test of the plugin system with files!";
    fs::write(&input_file, test_data).unwrap();
    
    // Generate key
    let key = [3u8; 32];
    
    // Read input file
    let data = fs::read(&input_file).unwrap();
    
    // Use AES-GCM plugin to encrypt
    let encrypted = plugins::encrypt_with_plugin(&data, &key, "aes-gcm").unwrap();
    
    // Write encrypted file
    fs::write(&encrypted_file, &encrypted).unwrap();
    
    // Read encrypted file
    let encrypted_data = fs::read(&encrypted_file).unwrap();
    
    // Decrypt
    let decrypted = plugins::decrypt_with_plugin(&encrypted_data, &key, "aes-gcm").unwrap();
    
    // Write decrypted file
    fs::write(&decrypted_file, &decrypted).unwrap();
    
    // Verify decrypted content is equal to original
    let final_data = fs::read(&decrypted_file).unwrap();
    assert_eq!(&final_data[..], &test_data[..]);
} 