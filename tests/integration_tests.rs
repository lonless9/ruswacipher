use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

// Helper function to create a minimal valid WASM file for testing
fn create_test_wasm_file() -> NamedTempFile {
    let temp_file = NamedTempFile::new().unwrap();

    // Create a minimal valid WASM module
    let wasm_data = vec![
        0x00, 0x61, 0x73, 0x6D, // WASM magic number
        0x01, 0x00, 0x00, 0x00, // Version
        // Type section
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Function section
        0x03, 0x02, 0x01, 0x00, // Code section
        0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B,
    ];

    fs::write(temp_file.path(), wasm_data).unwrap();
    temp_file
}

#[test]
#[serial]
fn test_cli_encrypt_aes_gcm() {
    let temp_dir = TempDir::new().unwrap();
    let input_wasm = create_test_wasm_file();
    let output_file = temp_dir.path().join("encrypted.wasm");
    let key_file = temp_dir.path().join("test.key");

    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("encrypt")
        .arg("-i")
        .arg(input_wasm.path())
        .arg("-o")
        .arg(&output_file)
        .arg("-a")
        .arg("aes-gcm")
        .arg("--generate-key")
        .arg(&key_file);

    cmd.assert().success().stderr(predicate::str::contains(
        "Encryption completed successfully",
    ));

    // Verify files were created
    assert!(output_file.exists());
    assert!(key_file.exists());

    // Verify key file contains valid hex
    let key_content = fs::read_to_string(&key_file).unwrap();
    assert!(key_content.trim().chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(key_content.trim().len(), 64); // 32 bytes = 64 hex chars
}

#[test]
#[serial]
fn test_cli_encrypt_chacha20poly1305() {
    let temp_dir = TempDir::new().unwrap();
    let input_wasm = create_test_wasm_file();
    let output_file = temp_dir.path().join("encrypted.wasm");
    let key_file = temp_dir.path().join("test.key");

    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("encrypt")
        .arg("-i")
        .arg(input_wasm.path())
        .arg("-o")
        .arg(&output_file)
        .arg("-a")
        .arg("chacha20poly1305")
        .arg("--generate-key")
        .arg(&key_file);

    cmd.assert().success().stderr(predicate::str::contains(
        "Encryption completed successfully",
    ));

    // Verify files were created
    assert!(output_file.exists());
    assert!(key_file.exists());
}

#[test]
#[serial]
fn test_cli_encrypt_decrypt_round_trip() {
    let temp_dir = TempDir::new().unwrap();
    let input_wasm = create_test_wasm_file();
    let encrypted_file = temp_dir.path().join("encrypted.wasm");
    let decrypted_file = temp_dir.path().join("decrypted.wasm");
    let key_file = temp_dir.path().join("test.key");

    // Read original file content
    let original_content = fs::read(input_wasm.path()).unwrap();

    // Encrypt
    let mut encrypt_cmd = Command::cargo_bin("ruswacipher").unwrap();
    encrypt_cmd
        .arg("encrypt")
        .arg("-i")
        .arg(input_wasm.path())
        .arg("-o")
        .arg(&encrypted_file)
        .arg("-a")
        .arg("aes-gcm")
        .arg("--generate-key")
        .arg(&key_file);

    encrypt_cmd.assert().success();

    // Decrypt
    let mut decrypt_cmd = Command::cargo_bin("ruswacipher").unwrap();
    decrypt_cmd
        .arg("decrypt")
        .arg("-i")
        .arg(&encrypted_file)
        .arg("-o")
        .arg(&decrypted_file)
        .arg("-k")
        .arg(&key_file);

    decrypt_cmd.assert().success();

    // Verify decrypted content matches original
    let decrypted_content = fs::read(&decrypted_file).unwrap();
    assert_eq!(original_content, decrypted_content);
}

#[test]
#[serial]
fn test_cli_encrypt_with_hex_key() {
    let temp_dir = TempDir::new().unwrap();
    let input_wasm = create_test_wasm_file();
    let output_file = temp_dir.path().join("encrypted.wasm");
    let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("encrypt")
        .arg("-i")
        .arg(input_wasm.path())
        .arg("-o")
        .arg(&output_file)
        .arg("-a")
        .arg("aes-gcm")
        .arg("--key-hex")
        .arg(hex_key);

    cmd.assert().success();
    assert!(output_file.exists());
}

#[test]
#[serial]
fn test_cli_encrypt_with_base64_key() {
    let temp_dir = TempDir::new().unwrap();
    let input_wasm = create_test_wasm_file();
    let output_file = temp_dir.path().join("encrypted.wasm");
    // 32 bytes in base64
    let base64_key = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=";

    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("encrypt")
        .arg("-i")
        .arg(input_wasm.path())
        .arg("-o")
        .arg(&output_file)
        .arg("-a")
        .arg("aes-gcm")
        .arg("--key-base64")
        .arg(base64_key);

    cmd.assert().success();
    assert!(output_file.exists());
}

#[test]
#[serial]
fn test_cli_invalid_input_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("encrypted.wasm");
    let key_file = temp_dir.path().join("test.key");

    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("encrypt")
        .arg("-i")
        .arg("nonexistent.wasm")
        .arg("-o")
        .arg(&output_file)
        .arg("-a")
        .arg("aes-gcm")
        .arg("--generate-key")
        .arg(&key_file);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));
}

#[test]
#[serial]
fn test_cli_invalid_algorithm() {
    let input_wasm = create_test_wasm_file();
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("encrypted.wasm");
    let key_file = temp_dir.path().join("test.key");

    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("encrypt")
        .arg("-i")
        .arg(input_wasm.path())
        .arg("-o")
        .arg(&output_file)
        .arg("-a")
        .arg("invalid-algorithm")
        .arg("--generate-key")
        .arg(&key_file);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
#[serial]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("--help");

    cmd.assert().success().stdout(predicate::str::contains(
        "A Rust tool for encrypting and protecting WebAssembly modules",
    ));
}

#[test]
#[serial]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ruswacipher"));
}

#[test]
#[serial]
fn test_key_format_options() {
    let temp_dir = TempDir::new().unwrap();
    let input_wasm = create_test_wasm_file();
    let output_file = temp_dir.path().join("encrypted.wasm");

    // Test hex format (default)
    let hex_key_file = temp_dir.path().join("hex.key");
    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("encrypt")
        .arg("-i")
        .arg(input_wasm.path())
        .arg("-o")
        .arg(&output_file)
        .arg("-a")
        .arg("aes-gcm")
        .arg("--generate-key")
        .arg(&hex_key_file)
        .arg("--key-format")
        .arg("hex");

    cmd.assert().success();

    let hex_content = fs::read_to_string(&hex_key_file).unwrap();
    assert!(hex_content.trim().chars().all(|c| c.is_ascii_hexdigit()));

    // Test base64 format
    let base64_key_file = temp_dir.path().join("base64.key");
    let mut cmd = Command::cargo_bin("ruswacipher").unwrap();
    cmd.arg("encrypt")
        .arg("-i")
        .arg(input_wasm.path())
        .arg("-o")
        .arg(&output_file)
        .arg("-a")
        .arg("aes-gcm")
        .arg("--generate-key")
        .arg(&base64_key_file)
        .arg("--key-format")
        .arg("base64");

    cmd.assert().success();

    let base64_content = fs::read_to_string(&base64_key_file).unwrap();
    // Base64 should contain only valid base64 characters
    assert!(base64_content
        .trim()
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
}
