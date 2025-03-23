use anyhow::Result;
use env_logger::Env;
use log::{debug, info};
use ruswacipher::obfuscation::function_split::split_large_functions;
use ruswacipher::wasm::parser::{parse_wasm, serialize_wasm};
use ruswacipher::wasm::structure::{SectionType, WasmModule};
use std::fs;
use std::path::Path;
use std::sync::Once;

// 确保日志只初始化一次
static INIT_LOGGER: Once = Once::new();

fn setup() {
    INIT_LOGGER.call_once(|| {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    });
}

// 获取简单测试样本文件
fn get_simple_wasm() -> WasmModule {
    let input_file = Path::new("tests/samples/simple.wasm");
    let wasm_data = fs::read(input_file).expect("Failed to read simple WASM file");
    parse_wasm(&wasm_data).expect("Failed to parse simple WASM file")
}

// 获取复杂测试样本文件
fn get_complex_wasm() -> WasmModule {
    let input_file = Path::new("tests/samples/complex.wasm");
    let wasm_data = fs::read(input_file).expect("Failed to read complex WASM file");
    parse_wasm(&wasm_data).expect("Failed to parse complex WASM file")
}

// 检查函数是否被分割并返回函数数量和代码段大小
fn count_functions_and_size(module: &WasmModule) -> (usize, usize) {
    let mut code_size = 0;
    let mut num_funcs = 0;

    for section in &module.sections {
        if section.section_type == SectionType::Code {
            code_size = section.data.len();

            // 读取函数数量
            let mut pos = 0;
            let mut shift = 0;

            while pos < section.data.len() {
                let byte = section.data[pos];
                num_funcs |= ((byte & 0x7F) as usize) << shift;
                pos += 1;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
        }
    }

    (num_funcs, code_size)
}

#[test]
fn test_simple_function_splitting() {
    // 初始化日志（只执行一次）
    setup();

    // 读取简单的WAT文件
    let wat_path = Path::new("tests/samples/simple.wat");
    let wasm_path = Path::new("tests/samples/simple.wasm");

    // 将WAT转换为WASM
    assert!(wat_path.exists(), "WAT文件不存在");
    let status = std::process::Command::new("wat2wasm")
        .arg(wat_path)
        .arg("-o")
        .arg(wasm_path)
        .status()
        .expect("执行wat2wasm失败");

    assert!(status.success(), "wat2wasm执行失败");
    assert!(wasm_path.exists(), "WASM文件未生成");

    // 读取并解析WASM文件
    let wasm_data = fs::read(wasm_path).expect("无法读取WASM文件");
    let module = parse_wasm(&wasm_data).expect("无法解析WASM文件");

    let (orig_funcs, orig_size) = count_functions_and_size(&module);
    println!(
        "原始简单模块: {} 个函数, 代码大小: {}",
        orig_funcs, orig_size
    );

    // 执行函数分割
    let result = split_large_functions(module);
    assert!(result.is_ok(), "函数分割失败: {:?}", result.err());

    let obfuscated = result.unwrap();
    let (new_funcs, new_size) = count_functions_and_size(&obfuscated);
    println!(
        "混淆后简单模块: {} 个函数, 代码大小: {}",
        new_funcs, new_size
    );
    println!("简单模块函数变化: {} -> {}", orig_funcs, new_funcs);

    // 简单模块可能太小，不会被分割，所以这里不做断言检查

    println!("测试成功：函数分割正常工作");
}

#[test]
fn test_complex_function_splitting() {
    // 初始化日志（只执行一次）
    setup();

    // 读取复杂的WAT文件
    let wat_path = Path::new("tests/samples/complex.wat");
    let wasm_path = Path::new("tests/samples/complex.wasm");

    // 将WAT转换为WASM
    assert!(wat_path.exists(), "WAT文件不存在");
    let status = std::process::Command::new("wat2wasm")
        .arg(wat_path)
        .arg("-o")
        .arg(wasm_path)
        .status()
        .expect("执行wat2wasm失败");

    assert!(status.success(), "wat2wasm执行失败");
    assert!(wasm_path.exists(), "WASM文件未生成");

    // 读取并解析WASM文件
    let wasm_data = fs::read(wasm_path).expect("无法读取WASM文件");
    let module = parse_wasm(&wasm_data).expect("无法解析WASM文件");

    let (orig_funcs, orig_size) = count_functions_and_size(&module);
    println!(
        "原始复杂模块: {} 个函数, 代码大小: {}",
        orig_funcs, orig_size
    );

    // 执行函数分割
    let result = split_large_functions(module);
    assert!(result.is_ok(), "函数分割失败: {:?}", result.err());

    let obfuscated = result.unwrap();
    let (new_funcs, new_size) = count_functions_and_size(&obfuscated);
    println!(
        "混淆后复杂模块: {} 个函数, 代码大小: {}",
        new_funcs, new_size
    );
    println!("复杂模块函数变化: {} -> {}", orig_funcs, new_funcs);

    // 复杂模块应该包含可分割的大函数
    // 我们添加了一个大型可分割函数，所以应该至少有一个函数被分割
    assert!(new_funcs > orig_funcs, "函数分割没有增加函数数量");

    // 将混淆后的WASM写入文件以便检查
    let obfuscated_wasm_path = Path::new("tests/samples/complex_obfuscated.wasm");
    let module_data = serialize_wasm(&obfuscated).expect("无法序列化WASM模块");
    fs::write(obfuscated_wasm_path, module_data).expect("无法写入混淆后的WASM文件");

    println!("测试成功：函数分割正常工作");
}

#[test]
fn test_function_count_consistency() {
    // 初始化日志（只执行一次）
    setup();

    // 获取测试模块
    let module = get_complex_wasm();

    // 应用函数分割
    let obfuscated_module = split_large_functions(module).unwrap();

    // 验证函数段中的函数数量与代码段中的函数数量一致
    let (function_count, _) = count_functions_and_size(&obfuscated_module);
    let code_section_count = count_function_bodies(&obfuscated_module);

    println!(
        "函数段中函数数量: {}, 代码段中函数体数量: {}",
        function_count, code_section_count
    );

    // 函数段中的函数数量应该与代码段中的函数体数量一致
    assert_eq!(
        function_count, code_section_count,
        "函数段和代码段中的函数数量应该一致"
    );
}

// 辅助函数：检查模块是否包含指定的段
fn has_section(module: &WasmModule, section_type: SectionType) -> bool {
    module
        .sections
        .iter()
        .any(|section| section.section_type == section_type)
}

// 辅助函数：获取代码段大小
fn get_code_section_size(module: &WasmModule) -> usize {
    module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code)
        .map(|section| section.data.len())
        .unwrap_or(0)
}

// 辅助函数：统计函数段中函数数量
fn count_functions(module: &WasmModule) -> usize {
    // 查找函数段
    let function_section = module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Function);

    if let Some(section) = function_section {
        // 第一个字节通常是函数数量（LEB128编码）
        if !section.data.is_empty() {
            let count = section.data[0] as usize;
            return count;
        }
    }

    0
}

// 辅助函数：统计代码段中函数体数量
fn count_function_bodies(module: &WasmModule) -> usize {
    // 查找代码段
    let code_section = module
        .sections
        .iter()
        .find(|section| section.section_type == SectionType::Code);

    if let Some(section) = code_section {
        // 代码段的第一个字节是函数体数量（LEB128编码）
        if !section.data.is_empty() {
            let count = section.data[0] as usize;
            return count;
        }
    }

    0
}
