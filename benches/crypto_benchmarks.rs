use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ruswacipher::config::EncryptionAlgorithm;
use ruswacipher::crypto::{AesGcmCipher, ChaCha20Poly1305Cipher, Cipher, KeyManager};
use std::hint::black_box;

fn benchmark_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_generation");

    group.bench_function("aes_gcm_key", |b| {
        b.iter(|| black_box(KeyManager::generate_key(32).unwrap()))
    });

    group.bench_function("chacha20poly1305_key", |b| {
        b.iter(|| black_box(KeyManager::generate_key(32).unwrap()))
    });

    group.bench_function("algorithm_specific_aes", |b| {
        b.iter(|| {
            black_box(ruswacipher::crypto::generate_key(&EncryptionAlgorithm::AesGcm).unwrap())
        })
    });

    group.bench_function("algorithm_specific_chacha", |b| {
        b.iter(|| {
            black_box(
                ruswacipher::crypto::generate_key(&EncryptionAlgorithm::ChaCha20Poly1305).unwrap(),
            )
        })
    });

    group.finish();
}

fn benchmark_encryption_decryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption_decryption");

    // Test data sizes
    let sizes = vec![1024, 8192, 65536, 1048576]; // 1KB, 8KB, 64KB, 1MB

    for size in sizes {
        let data = vec![0u8; size];
        group.throughput(Throughput::Bytes(size as u64));

        // AES-GCM benchmarks
        let aes_key = KeyManager::generate_key(32).unwrap();
        let aes_cipher = AesGcmCipher::new(&aes_key).unwrap();

        group.bench_with_input(
            BenchmarkId::new("aes_gcm_encrypt", size),
            &data,
            |b, data| b.iter(|| black_box(aes_cipher.encrypt(black_box(data)).unwrap())),
        );

        let encrypted_aes = aes_cipher.encrypt(&data).unwrap();
        group.bench_with_input(
            BenchmarkId::new("aes_gcm_decrypt", size),
            &encrypted_aes,
            |b, encrypted| {
                b.iter(|| {
                    black_box(
                        aes_cipher
                            .decrypt(black_box(&encrypted.iv), black_box(&encrypted.ciphertext))
                            .unwrap(),
                    )
                })
            },
        );

        // ChaCha20-Poly1305 benchmarks
        let chacha_key = KeyManager::generate_key(32).unwrap();
        let chacha_cipher = ChaCha20Poly1305Cipher::new(&chacha_key).unwrap();

        group.bench_with_input(
            BenchmarkId::new("chacha20poly1305_encrypt", size),
            &data,
            |b, data| b.iter(|| black_box(chacha_cipher.encrypt(black_box(data)).unwrap())),
        );

        let encrypted_chacha = chacha_cipher.encrypt(&data).unwrap();
        group.bench_with_input(
            BenchmarkId::new("chacha20poly1305_decrypt", size),
            &encrypted_chacha,
            |b, encrypted| {
                b.iter(|| {
                    black_box(
                        chacha_cipher
                            .decrypt(black_box(&encrypted.iv), black_box(&encrypted.ciphertext))
                            .unwrap(),
                    )
                })
            },
        );
    }

    group.finish();
}

fn benchmark_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_trip");

    let sizes = vec![1024, 8192, 65536];

    for size in sizes {
        let data = vec![0u8; size];
        group.throughput(Throughput::Bytes(size as u64));

        // AES-GCM round trip
        let aes_key = KeyManager::generate_key(32).unwrap();
        let aes_cipher = AesGcmCipher::new(&aes_key).unwrap();

        group.bench_with_input(
            BenchmarkId::new("aes_gcm_round_trip", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let encrypted = aes_cipher.encrypt(black_box(data)).unwrap();
                    black_box(
                        aes_cipher
                            .decrypt(&encrypted.iv, &encrypted.ciphertext)
                            .unwrap(),
                    )
                })
            },
        );

        // ChaCha20-Poly1305 round trip
        let chacha_key = KeyManager::generate_key(32).unwrap();
        let chacha_cipher = ChaCha20Poly1305Cipher::new(&chacha_key).unwrap();

        group.bench_with_input(
            BenchmarkId::new("chacha20poly1305_round_trip", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let encrypted = chacha_cipher.encrypt(black_box(data)).unwrap();
                    black_box(
                        chacha_cipher
                            .decrypt(&encrypted.iv, &encrypted.ciphertext)
                            .unwrap(),
                    )
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_key_generation,
    benchmark_encryption_decryption,
    benchmark_round_trip
);
criterion_main!(benches);
