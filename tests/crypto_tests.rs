use anyhow::Result;
use ruswacipher::crypto::{decrypt_data, encrypt_data, generate_key, load_key, save_key};
use std::fs;
use std::path::Path;

#[test]
fn test_key_generation_and_storage() -> Result<()> {
    // Generate a random key
    let key_length = 32; // 256-bit key
    let key = generate_key(key_length);

    // Verify key length
    assert_eq!(
        key.len(),
        key_length,
        "Generated key length should match specified length"
    );

    // Save key to temporary file
    let key_path = Path::new("target/test_key.bin");
    save_key(&key, key_path)?;

    // Load key from temporary file
    let loaded_key = load_key(key_path)?;

    // Verify loaded key matches original key
    assert_eq!(loaded_key, key, "Loaded key should match original key");

    // Clean up
    fs::remove_file(key_path)?;

    Ok(())
}

#[test]
fn test_aes_gcm_encryption_decryption() -> Result<()> {
    // Use AES-GCM algorithm
    let algorithm = "aes-gcm";

    // Test data
    let original_data = b"This is a secret message for encryption and decryption testing.";

    // Generate key
    let key = generate_key(32);

    // Encrypt data
    let encrypted = encrypt_data(original_data, &key, algorithm)?;

    // Verify encrypted data is different from original data
    assert_ne!(
        encrypted, original_data,
        "Encrypted data should be different from original data"
    );

    // Decrypt data
    let decrypted = decrypt_data(&encrypted, &key)?;

    // Verify decrypted data matches original data
    assert_eq!(
        decrypted, original_data,
        "Decrypted data should match original data"
    );

    Ok(())
}

#[test]
fn test_chacha20poly1305_encryption_decryption() -> Result<()> {
    // Use ChaCha20Poly1305 algorithm
    let algorithm = "chacha20poly1305";

    // Test data
    let original_data = b"This is a secret message for ChaCha20-Poly1305 testing.";

    // Generate key
    let key = generate_key(32);

    // Encrypt data
    let encrypted = encrypt_data(original_data, &key, algorithm)?;

    // Verify encrypted data is different from original data
    assert_ne!(
        encrypted, original_data,
        "Encrypted data should be different from original data"
    );

    // Decrypt data
    let decrypted = decrypt_data(&encrypted, &key)?;

    // Verify decrypted data matches original data
    assert_eq!(
        decrypted, original_data,
        "Decrypted data should match original data"
    );

    Ok(())
}

#[test]
fn test_wrong_key_decryption() -> Result<()> {
    // Use AES-GCM algorithm
    let algorithm = "aes-gcm";

    // Test data
    let original_data = b"This is a secret message that should not be decryptable with wrong key.";

    // Correct key
    let correct_key = generate_key(32);

    // Wrong key
    let wrong_key = generate_key(32);

    // Ensure keys are different
    assert_ne!(
        correct_key, wrong_key,
        "Correct key and wrong key should be different"
    );

    // Encrypt data
    let encrypted = encrypt_data(original_data, &correct_key, algorithm)?;

    // Try to decrypt with wrong key - this should fail
    let decryption_result = decrypt_data(&encrypted, &wrong_key);

    // Verify decryption fails
    assert!(
        decryption_result.is_err(),
        "Decryption with wrong key should fail"
    );

    Ok(())
}

#[test]
fn test_file_encryption_decryption() -> Result<()> {
    use ruswacipher::crypto::{decrypt_file, encrypt_file};

    // Prepare test file
    let input_path = Path::new("target/test_input.bin");
    let encrypted_path = Path::new("target/test_encrypted.bin");
    let decrypted_path = Path::new("target/test_decrypted.bin");
    let key_path = Path::new("target/test_crypto_key.bin");

    // Create test data
    let test_data = b"This is test data for file encryption and decryption.";
    fs::write(input_path, test_data)?;

    // Generate key and save it
    let key = generate_key(32);
    save_key(&key, key_path)?;

    // Encrypt file, using the generated key
    encrypt_file(input_path, encrypted_path, Some(key_path), "aes-gcm")?;

    // Verify encrypted file is created and different from original file
    assert!(encrypted_path.exists(), "Encrypted file should exist");
    let encrypted_data = fs::read(encrypted_path)?;
    assert_ne!(
        encrypted_data, test_data,
        "Encrypted file content should be different from original data"
    );

    // Decrypt file
    decrypt_file(encrypted_path, decrypted_path, key_path)?;

    // Verify decrypted file content matches original data
    let decrypted_data = fs::read(decrypted_path)?;
    assert_eq!(
        decrypted_data, test_data,
        "Decrypted file content should match original data"
    );

    // Clean up
    fs::remove_file(input_path)?;
    fs::remove_file(encrypted_path)?;
    fs::remove_file(decrypted_path)?;
    fs::remove_file(key_path)?;

    Ok(())
}

#[test]
fn test_wasm_file_encryption_decryption() -> Result<()> {
    use ruswacipher::crypto::{decrypt_file, encrypt_file};
    use ruswacipher::wasm::load_module;

    // Prepare paths
    let original_wasm_path = Path::new("tests/samples/simple.wasm");
    let encrypted_path = Path::new("target/test_encrypted.wasm");
    let decrypted_path = Path::new("target/test_decrypted.wasm");
    let key_path = Path::new("target/test_wasm_key.bin");

    // Generate key and save it
    let key = generate_key(32);
    save_key(&key, key_path)?;

    // Ensure original WASM file exists
    assert!(
        original_wasm_path.exists(),
        "Test requires sample WASM file"
    );

    // Try to parse original WASM file
    let original_module = load_module(original_wasm_path)?;

    // Encrypt WASM file
    encrypt_file(
        original_wasm_path,
        encrypted_path,
        Some(key_path),
        "aes-gcm",
    )?;

    // Verify encrypted file is created and different from original file
    assert!(encrypted_path.exists(), "Encrypted file should exist");
    let original_data = fs::read(original_wasm_path)?;
    let encrypted_data = fs::read(encrypted_path)?;
    assert_ne!(
        encrypted_data, original_data,
        "Encrypted file content should be different from original data"
    );

    // Decrypt WASM file
    decrypt_file(encrypted_path, decrypted_path, key_path)?;

    // Verify decrypted file is created
    assert!(decrypted_path.exists(), "Decrypted file should exist");

    // Try to parse decrypted WASM file - if parsing succeeds, decryption is correct
    let decrypted_module = load_module(decrypted_path)?;

    // Verify decrypted module has the same number of sections as the original module
    assert_eq!(
        decrypted_module.sections.len(),
        original_module.sections.len(),
        "Decrypted WASM module should have the same number of sections as the original module"
    );

    // Verify module version is consistent
    assert_eq!(
        decrypted_module.version, original_module.version,
        "Decrypted WASM module version should be the same as the original module"
    );

    // Verify each section type and content
    for (i, (orig_section, decrypted_section)) in original_module
        .sections
        .iter()
        .zip(decrypted_module.sections.iter())
        .enumerate()
    {
        // Verify section type is consistent
        assert_eq!(
            orig_section.section_type as u8, decrypted_section.section_type as u8,
            "Section #{} type does not match",
            i
        );

        // Verify section name is consistent (if it's a custom section)
        if orig_section.section_type == ruswacipher::wasm::structure::SectionType::Custom {
            assert_eq!(
                orig_section.name, decrypted_section.name,
                "Custom section #{} name does not match",
                i
            );
        }

        // Verify section data is consistent
        assert_eq!(
            orig_section.data.len(),
            decrypted_section.data.len(),
            "Section #{} data length does not match",
            i
        );

        assert_eq!(
            orig_section.data, decrypted_section.data,
            "Section #{} data content does not match",
            i
        );
    }

    // Compare binary content directly
    let original_binary = fs::read(original_wasm_path)?;
    let decrypted_binary = fs::read(decrypted_path)?;
    assert_eq!(
        original_binary, decrypted_binary,
        "Decrypted WASM binary content does not match original content"
    );

    // Clean up
    fs::remove_file(encrypted_path)?;
    fs::remove_file(decrypted_path)?;
    fs::remove_file(key_path)?;

    Ok(())
}
