use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::RngCore;
use ruswacipher::crypto::algorithms::{AesGcmCipher, ChaCha20Poly1305Cipher, Cipher};

/// AES-GCM加密基准测试
fn bench_aes_gcm_encrypt(c: &mut Criterion) {
    // 生成随机密钥
    let mut key = vec![0u8; 32];
    rand::rng().fill_bytes(&mut key);

    // 生成随机数据
    let mut data = vec![0u8; 1024 * 1024]; // 1MB
    rand::rng().fill_bytes(&mut data);

    // 创建加密器
    let cipher = AesGcmCipher::new(&key).unwrap();

    // 基准测试
    c.bench_function("aes_gcm_encrypt_1mb", |b| {
        b.iter(|| {
            let _ = black_box(cipher.encrypt(black_box(&data)));
        })
    });
}

/// ChaCha20-Poly1305加密基准测试
fn bench_chacha20poly1305_encrypt(c: &mut Criterion) {
    // 生成随机密钥
    let mut key = vec![0u8; 32];
    rand::rng().fill_bytes(&mut key);

    // 生成随机数据
    let mut data = vec![0u8; 1024 * 1024]; // 1MB
    rand::rng().fill_bytes(&mut data);

    // 创建加密器
    let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();

    // 基准测试
    c.bench_function("chacha20poly1305_encrypt_1mb", |b| {
        b.iter(|| {
            let _ = black_box(cipher.encrypt(black_box(&data)));
        })
    });
}

/// AES-GCM解密基准测试
fn bench_aes_gcm_decrypt(c: &mut Criterion) {
    // 生成随机密钥
    let mut key = vec![0u8; 32];
    rand::rng().fill_bytes(&mut key);

    // 生成随机数据
    let mut data = vec![0u8; 1024 * 1024]; // 1MB
    rand::rng().fill_bytes(&mut data);

    // 创建加密器并加密数据
    let cipher = AesGcmCipher::new(&key).unwrap();
    let encrypted = cipher.encrypt(&data).unwrap();

    // 基准测试
    c.bench_function("aes_gcm_decrypt_1mb", |b| {
        b.iter(|| {
            let _ = black_box(cipher.decrypt(black_box(&encrypted)));
        })
    });
}

/// ChaCha20-Poly1305解密基准测试
fn bench_chacha20poly1305_decrypt(c: &mut Criterion) {
    // 生成随机密钥
    let mut key = vec![0u8; 32];
    rand::rng().fill_bytes(&mut key);

    // 生成随机数据
    let mut data = vec![0u8; 1024 * 1024]; // 1MB
    rand::rng().fill_bytes(&mut data);

    // 创建加密器并加密数据
    let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();
    let encrypted = cipher.encrypt(&data).unwrap();

    // 基准测试
    c.bench_function("chacha20poly1305_decrypt_1mb", |b| {
        b.iter(|| {
            let _ = black_box(cipher.decrypt(black_box(&encrypted)));
        })
    });
}

// 定义基准测试组
criterion_group!(
    benches,
    bench_aes_gcm_encrypt,
    bench_chacha20poly1305_encrypt,
    bench_aes_gcm_decrypt,
    bench_chacha20poly1305_decrypt
);
criterion_main!(benches);
