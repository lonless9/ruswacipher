use std::path::Path;
use std::fs;
use anyhow::Result;
use ruswacipher::crypto::{encrypt_data, decrypt_data, generate_key, save_key, load_key};

#[test]
fn test_key_generation_and_storage() -> Result<()> {
    // 生成一个随机密钥
    let key_length = 32; // 256位密钥
    let key = generate_key(key_length);
    
    // 验证密钥长度
    assert_eq!(key.len(), key_length, "生成的密钥长度应为指定长度");
    
    // 保存密钥到临时文件
    let key_path = Path::new("target/test_key.bin");
    save_key(&key, key_path)?;
    
    // 读取密钥
    let loaded_key = load_key(key_path)?;
    
    // 验证读取的密钥与原始密钥相同
    assert_eq!(loaded_key, key, "加载的密钥应与原始密钥相同");
    
    // 清理
    fs::remove_file(key_path)?;
    
    Ok(())
}

#[test]
fn test_aes_gcm_encryption_decryption() -> Result<()> {
    // 使用AES-GCM算法
    let algorithm = "aes-gcm";
    
    // 测试数据
    let original_data = b"This is a secret message for encryption and decryption testing.";
    
    // 生成密钥
    let key = generate_key(32);
    
    // 加密数据
    let encrypted = encrypt_data(original_data, &key, algorithm)?;
    
    // 验证加密数据与原始数据不同
    assert_ne!(encrypted, original_data, "加密数据应与原始数据不同");
    
    // 解密数据
    let decrypted = decrypt_data(&encrypted, &key)?;
    
    // 验证解密结果与原始数据相同
    assert_eq!(decrypted, original_data, "解密数据应与原始数据相同");
    
    Ok(())
}

#[test]
fn test_chacha20poly1305_encryption_decryption() -> Result<()> {
    // 使用ChaCha20Poly1305算法
    let algorithm = "chacha20poly1305";
    
    // 测试数据
    let original_data = b"This is a secret message for ChaCha20-Poly1305 testing.";
    
    // 生成密钥
    let key = generate_key(32);
    
    // 加密数据
    let encrypted = encrypt_data(original_data, &key, algorithm)?;
    
    // 验证加密数据与原始数据不同
    assert_ne!(encrypted, original_data, "加密数据应与原始数据不同");
    
    // 解密数据
    let decrypted = decrypt_data(&encrypted, &key)?;
    
    // 验证解密结果与原始数据相同
    assert_eq!(decrypted, original_data, "解密数据应与原始数据相同");
    
    Ok(())
}

#[test]
fn test_wrong_key_decryption() -> Result<()> {
    // 使用AES-GCM算法
    let algorithm = "aes-gcm";
    
    // 测试数据
    let original_data = b"This is a secret message that should not be decryptable with wrong key.";
    
    // 正确的密钥
    let correct_key = generate_key(32);
    
    // 错误的密钥
    let wrong_key = generate_key(32);
    
    // 确保密钥不同
    assert_ne!(correct_key, wrong_key, "正确密钥与错误密钥应该不同");
    
    // 加密数据
    let encrypted = encrypt_data(original_data, &correct_key, algorithm)?;
    
    // 尝试使用错误的密钥解密 - 这应该会失败
    let decryption_result = decrypt_data(&encrypted, &wrong_key);
    
    // 验证解密失败
    assert!(decryption_result.is_err(), "使用错误密钥的解密应该失败");
    
    Ok(())
}

#[test]
fn test_file_encryption_decryption() -> Result<()> {
    use ruswacipher::crypto::{encrypt_file, decrypt_file};
    
    // 准备测试文件
    let input_path = Path::new("target/test_input.bin");
    let encrypted_path = Path::new("target/test_encrypted.bin");
    let decrypted_path = Path::new("target/test_decrypted.bin");
    let key_path = Path::new("target/test_crypto_key.bin");
    
    // 创建测试数据
    let test_data = b"This is test data for file encryption and decryption.";
    fs::write(input_path, test_data)?;
    
    // 生成密钥并保存
    let key = generate_key(32);
    save_key(&key, key_path)?;
    
    // 加密文件，使用已生成的密钥
    encrypt_file(input_path, encrypted_path, Some(key_path), "aes-gcm")?;
    
    // 验证加密文件已创建且与原始文件不同
    assert!(encrypted_path.exists(), "加密文件应该存在");
    let encrypted_data = fs::read(encrypted_path)?;
    assert_ne!(encrypted_data, test_data, "加密文件内容应与原始数据不同");
    
    // 解密文件
    decrypt_file(encrypted_path, decrypted_path, key_path)?;
    
    // 验证解密文件与原始文件内容相同
    let decrypted_data = fs::read(decrypted_path)?;
    assert_eq!(decrypted_data, test_data, "解密文件内容应与原始数据相同");
    
    // 清理
    fs::remove_file(input_path)?;
    fs::remove_file(encrypted_path)?;
    fs::remove_file(decrypted_path)?;
    fs::remove_file(key_path)?;
    
    Ok(())
}

#[test]
fn test_wasm_file_encryption_decryption() -> Result<()> {
    use ruswacipher::crypto::{encrypt_file, decrypt_file};
    use ruswacipher::wasm::load_module;
    
    // 准备路径
    let original_wasm_path = Path::new("tests/samples/simple.wasm");
    let encrypted_path = Path::new("target/test_encrypted.wasm");
    let decrypted_path = Path::new("target/test_decrypted.wasm");
    let key_path = Path::new("target/test_wasm_key.bin");
    
    // 生成密钥并保存
    let key = generate_key(32);
    save_key(&key, key_path)?;
    
    // 确保原始WASM文件存在
    assert!(original_wasm_path.exists(), "测试需要样本WASM文件");
    
    // 先尝试解析原始WASM文件
    let original_module = load_module(original_wasm_path)?;
    
    // 加密WASM文件
    encrypt_file(original_wasm_path, encrypted_path, Some(key_path), "aes-gcm")?;
    
    // 验证加密文件已创建且与原始文件不同
    assert!(encrypted_path.exists(), "加密文件应该存在");
    let original_data = fs::read(original_wasm_path)?;
    let encrypted_data = fs::read(encrypted_path)?;
    assert_ne!(encrypted_data, original_data, "加密文件内容应与原始数据不同");
    
    // 解密WASM文件
    decrypt_file(encrypted_path, decrypted_path, key_path)?;
    
    // 验证解密文件已创建
    assert!(decrypted_path.exists(), "解密文件应该存在");
    
    // 尝试解析解密后的WASM文件 - 如果解析成功，说明解密正确
    let decrypted_module = load_module(decrypted_path)?;
    
    // 验证解密后的模块与原始模块有相同数量的节
    assert_eq!(
        decrypted_module.sections.len(), 
        original_module.sections.len(), 
        "解密后的WASM模块应有与原始模块相同数量的节"
    );
    
    // 验证模块版本号一致
    assert_eq!(
        decrypted_module.version,
        original_module.version,
        "解密后的WASM模块版本应与原始模块相同"
    );
    
    // 详细验证每个节的类型和内容
    for (i, (orig_section, decrypted_section)) in original_module.sections.iter().zip(decrypted_module.sections.iter()).enumerate() {
        // 验证节类型相同
        assert_eq!(
            orig_section.section_type as u8,
            decrypted_section.section_type as u8,
            "节 #{} 类型不匹配", i
        );
        
        // 验证节名称相同（如果是自定义节）
        if orig_section.section_type == ruswacipher::wasm::structure::SectionType::Custom {
            assert_eq!(
                orig_section.name,
                decrypted_section.name,
                "自定义节 #{} 名称不匹配", i
            );
        }
        
        // 验证节数据相同
        assert_eq!(
            orig_section.data.len(),
            decrypted_section.data.len(),
            "节 #{} 数据长度不匹配", i
        );
        
        assert_eq!(
            orig_section.data,
            decrypted_section.data,
            "节 #{} 数据内容不匹配", i
        );
    }
    
    // 直接比较二进制内容
    let original_binary = fs::read(original_wasm_path)?;
    let decrypted_binary = fs::read(decrypted_path)?;
    assert_eq!(
        original_binary,
        decrypted_binary,
        "解密后的WASM二进制内容与原始内容不匹配"
    );
    
    // 清理
    fs::remove_file(encrypted_path)?;
    fs::remove_file(decrypted_path)?;
    fs::remove_file(key_path)?;
    
    Ok(())
} 