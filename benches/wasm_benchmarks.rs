use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ruswacipher::wasm::WasmParser;
use std::fs;
use std::hint::black_box;

fn create_test_wasm_data() -> Vec<u8> {
    // Create a minimal valid WASM module for testing
    // WASM magic number (0x00, 0x61, 0x73, 0x6D) + version (0x01, 0x00, 0x00, 0x00)
    let mut wasm_data = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    // Add a simple type section
    wasm_data.extend_from_slice(&[
        0x01, // Type section ID
        0x04, // Section size
        0x01, // Number of types
        0x60, // Function type
        0x00, // No parameters
        0x00, // No results
    ]);

    // Add a function section
    wasm_data.extend_from_slice(&[
        0x03, // Function section ID
        0x02, // Section size
        0x01, // Number of functions
        0x00, // Function 0 uses type 0
    ]);

    // Add a code section
    wasm_data.extend_from_slice(&[
        0x0A, // Code section ID
        0x04, // Section size
        0x01, // Number of function bodies
        0x02, // Function body size
        0x00, // No locals
        0x0B, // End instruction
    ]);

    wasm_data
}

fn create_large_wasm_data(size_multiplier: usize) -> Vec<u8> {
    let base_wasm = create_test_wasm_data();

    // Add padding to simulate larger WASM files
    let padding_size = size_multiplier * 1024; // KB
    let padding = vec![0x00; padding_size];

    // Insert padding as a custom section
    let mut large_wasm = base_wasm[..8].to_vec(); // Magic + version

    // Add custom section with padding
    large_wasm.push(0x00); // Custom section ID

    // Encode section size (padding + 1 byte for name length + name)
    let section_size = padding_size + 1 + 7; // 7 = "padding".len()
    if section_size < 128 {
        large_wasm.push(section_size as u8);
    } else {
        // Use LEB128 encoding for larger sizes
        let mut size = section_size;
        while size >= 128 {
            large_wasm.push((size & 0x7F) as u8 | 0x80);
            size >>= 7;
        }
        large_wasm.push(size as u8);
    }

    large_wasm.push(7); // Name length
    large_wasm.extend_from_slice(b"padding"); // Name
    large_wasm.extend_from_slice(&padding);

    // Add the rest of the original WASM
    large_wasm.extend_from_slice(&base_wasm[8..]);

    large_wasm
}

fn benchmark_wasm_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_validation");

    let test_wasm = create_test_wasm_data();

    group.bench_function("validate_small_wasm", |b| {
        b.iter(|| black_box(WasmParser::validate_wasm(black_box(&test_wasm)).unwrap()))
    });

    // Test with different sizes
    let sizes = vec![1, 10, 100]; // KB
    for size in sizes {
        let large_wasm = create_large_wasm_data(size);
        group.throughput(Throughput::Bytes(large_wasm.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("validate_wasm", format!("{}KB", size)),
            &large_wasm,
            |b, wasm_data| {
                b.iter(|| black_box(WasmParser::validate_wasm(black_box(wasm_data)).unwrap()))
            },
        );
    }

    group.finish();
}

fn benchmark_wasm_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_parsing");

    let test_wasm = create_test_wasm_data();

    group.bench_function("parse_small_wasm", |b| {
        b.iter(|| black_box(WasmParser::get_module_info(black_box(&test_wasm)).unwrap()))
    });

    // Test with different sizes
    let sizes = vec![1, 10, 100]; // KB
    for size in sizes {
        let large_wasm = create_large_wasm_data(size);
        group.throughput(Throughput::Bytes(large_wasm.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("parse_wasm", format!("{}KB", size)),
            &large_wasm,
            |b, wasm_data| {
                b.iter(|| black_box(WasmParser::get_module_info(black_box(wasm_data)).unwrap()))
            },
        );
    }

    group.finish();
}

fn benchmark_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");

    // Create test data
    let test_wasm = create_test_wasm_data();

    // Test reading from memory (simulating file read)
    group.bench_function("read_wasm_data", |b| {
        b.iter(|| black_box(test_wasm.clone()))
    });

    // Test with different sizes for memory operations
    let sizes = vec![1, 10, 100]; // KB
    for size in sizes {
        let large_wasm = create_large_wasm_data(size);
        group.throughput(Throughput::Bytes(large_wasm.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("clone_wasm_data", format!("{}KB", size)),
            &large_wasm,
            |b, wasm_data| b.iter(|| black_box(wasm_data.clone())),
        );
    }

    group.finish();
}

// Benchmark real WASM file if available
fn benchmark_real_wasm_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_wasm_file");

    // Try to load the test WASM file if it exists
    if let Ok(real_wasm) = fs::read("web/test.wasm") {
        group.throughput(Throughput::Bytes(real_wasm.len() as u64));

        group.bench_function("validate_real_wasm", |b| {
            b.iter(|| black_box(WasmParser::validate_wasm(black_box(&real_wasm)).unwrap()))
        });

        group.bench_function("parse_real_wasm", |b| {
            b.iter(|| black_box(WasmParser::get_module_info(black_box(&real_wasm)).unwrap()))
        });
    } else {
        // Skip real file benchmarks if file doesn't exist
        group.bench_function("no_real_wasm_file", |b| b.iter(|| black_box(())));
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_wasm_validation,
    benchmark_wasm_parsing,
    benchmark_file_operations,
    benchmark_real_wasm_file
);
criterion_main!(benches);
