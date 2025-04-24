use crate::wasm::structure::{SectionType, WasmModule};
use anyhow::Result;
use hmac::{Hmac, Mac};
use log::{debug, info, trace};
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::Sha256;

/// Rename local variables
pub fn rename_locals(module: WasmModule) -> Result<WasmModule> {
    debug!("Executing local variable renaming");
    trace!("Module before renaming local variables: {:?}", module);

    let mut modified_module = module;

    // Find Code section and modify
    for section in &mut modified_module.sections {
        if section.section_type == SectionType::Code {
            // Traverse code section, find local variable definitions and rename them
            // This requires understanding local variable definitions in WASM binary format
            // Actual implementation needs to parse WASM code section and make modifications

            // Below is a simplified example, actual implementation needs more detailed WASM code section parsing
            let mut new_data = section.data.clone();

            // Find local variable declarations
            for i in 0..new_data.len().saturating_sub(2) {
                // Local variable declaration pattern (simplified version, actual more complex)
                if is_local_decl_pattern(&new_data[i..i + 3]) {
                    // Replace variable name (actually need to modify variable index table)
                    scramble_local_name(&mut new_data, i);
                }
            }

            section.data = new_data;
        }
    }

    info!("Local variable renaming completed");
    Ok(modified_module)
}

/// Generate random identifier
#[allow(dead_code)]
pub fn generate_random_identifier(length: usize) -> String {
    let mut rng = rand::rng();

    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789"
        .chars()
        .collect();

    let mut id = String::with_capacity(length);

    // Ensure identifier starts with a letter
    id.push(chars[rng.random_range(0..52)]);

    // Add remaining characters
    for _ in 1..length {
        id.push(chars[rng.random_range(0..chars.len())]);
    }

    id
}

/// Check if it's a local variable declaration pattern
fn is_local_decl_pattern(bytes: &[u8]) -> bool {
    // WebAssembly local variable declarations usually consist of the following byte sequences:
    // 1. 0x41: i32.const - used for indexing
    // 2. Variable index (LEB128 encoded)
    // 3. 0x21: set_local instruction

    if bytes.len() < 3 {
        return false;
    }

    // Check if it's a local variable declaration pattern
    // Note: This is still a simplified version, but more accurate than before
    if bytes[0] == 0x41 {
        // i32.const
        // Skip LEB128 encoded number
        let mut i = 1;
        while i < bytes.len() && (bytes[i] & 0x80) != 0 {
            i += 1;
        }
        i += 1; // Skip last LEB128 byte

        // Check if it's set_local (0x21) instruction
        if i < bytes.len() && (bytes[i] == 0x21 || bytes[i] == 0x22 || bytes[i] == 0x23) {
            return true; // This is a local variable declaration
        }
    }

    // Check get_local (0x20), tee_local (0x22) etc instructions
    if bytes[0] == 0x20 || bytes[0] == 0x22 {
        return true;
    }

    false
}

/// Scramble local variable name
fn scramble_local_name(data: &mut [u8], pos: usize) {
    if pos + 2 >= data.len() {
        return;
    }

    // Use cryptographically secure way to generate new variable index
    let mut hmac = Hmac::<Sha256>::new_from_slice(b"RusWaCipher_LocalVar_Key").unwrap();

    // Add position information and original bytes as HMAC input
    hmac.update(&pos.to_le_bytes());
    hmac.update(&data[pos..pos + 3]);

    // Use HMAC result and use it as seed
    let result = hmac.finalize().into_bytes();
    let seed = u32::from_le_bytes([result[0], result[1], result[2], result[3]]);
    let mut rng = StdRng::seed_from_u64(seed as u64);

    // 获取原始的局部变量索引值
    let original_var_index = match data[pos] {
        0x20..=0x23 => {
            if pos + 1 < data.len() && (data[pos + 1] & 0x80) == 0 {
                data[pos + 1] as usize
            } else {
                // 多字节LEB128编码，我们不应该修改这些
                return;
            }
        }
        _ => return, // 不处理其他类型的指令
    };

    // 保持局部变量索引在有效范围内 (0-3)
    // 在WebAssembly中，局部变量索引必须有效，否则会导致验证错误
    let max_local_var = 3; // 安全的最大局部变量索引

    match data[pos] {
        // 局部变量指令
        0x20..=0x23 => {
            // get_local, set_local, tee_local instructions
            // 只处理单字节变量索引以确保安全
            if pos + 1 < data.len() && (data[pos + 1] & 0x80) == 0 {
                // 生成新的变量索引，确保在有效范围内
                // 为了更安全，我们只使用原始索引值或更小的值
                data[pos + 1] =
                    rng.random_range(0..=std::cmp::min(original_var_index, max_local_var)) as u8;
            }
        }
        _ => {} // 不处理其他类型的指令
    }
}
