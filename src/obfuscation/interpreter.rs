use crate::obfuscation::vm::{VMOpcode, from_byte};
use anyhow::{Result, anyhow};
use log::debug;

/// 执行VM字节码的解释器
pub fn execute_vm_bytecode(
    bytecode: &[u8],     // VM指令字节码
    metadata: &[u8],     // 元数据（包含解密信息等）
    initial_stack: &[i32], // 初始栈（通常包含函数参数）
    _memory: &mut [u8]    // 内存区域
) -> Result<i32> {       // 返回执行结果
    // 初始化VM状态
    let mut pc = 0;             // 程序计数器
    let mut stack: Vec<i32> = Vec::with_capacity(64);
    let mut running = true;

    // 加载初始栈值
    for &value in initial_stack {
        stack.push(value);
    }

    // 解密字节码（如果需要）
    let decrypted_bytecode = decrypt_bytecode(bytecode, metadata)?;

    // 主执行循环
    while running && pc < decrypted_bytecode.len() {
        // 获取当前指令
        let opcode = decrypted_bytecode[pc];
        pc += 1;

        // 执行指令
        match from_byte(opcode) {
            Some(VMOpcode::Push) => {
                if pc < decrypted_bytecode.len() {
                    let value = decrypted_bytecode[pc] as i32;
                    pc += 1;
                    stack.push(value);
                }
            },
            Some(VMOpcode::Pop) => {
                if stack.pop().is_none() {
                    return Err(anyhow!("Stack underflow on Pop"));
                }
            },
            Some(VMOpcode::Add) => {
                if stack.len() < 2 {
                    return Err(anyhow!("Stack underflow on Add"));
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a + b);
            },
            Some(VMOpcode::Sub) => {
                if stack.len() < 2 {
                    return Err(anyhow!("Stack underflow on Sub"));
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a - b);
            },
            Some(VMOpcode::Mul) => {
                if stack.len() < 2 {
                    return Err(anyhow!("Stack underflow on Mul"));
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a * b);
            },
            Some(VMOpcode::Div) => {
                if stack.len() < 2 {
                    return Err(anyhow!("Stack underflow on Div"));
                }
                let b = stack.pop().unwrap();
                if b == 0 {
                    return Err(anyhow!("Division by zero"));
                }
                let a = stack.pop().unwrap();
                stack.push(a / b);
            },
            Some(VMOpcode::Exit) => {
                running = false;
            },
            Some(VMOpcode::Jump) => {
                if pc < decrypted_bytecode.len() {
                    let target = decrypted_bytecode[pc] as usize;
                    pc = target;
                }
            },
            Some(VMOpcode::JumpIf) => {
                if pc < decrypted_bytecode.len() && !stack.is_empty() {
                    let target = decrypted_bytecode[pc] as usize;
                    pc += 1;
                    
                    let condition = stack.pop().unwrap();
                    if condition != 0 {
                        pc = target;
                    }
                }
            },
            // 添加其他指令支持...
            _ => {
                debug!("Unknown opcode: 0x{:02X}", opcode);
            }
        }
    }

    // 返回结果（栈顶或默认值）
    Ok(stack.pop().unwrap_or(42))
}

/// 解密VM字节码
fn decrypt_bytecode(encrypted: &[u8], metadata: &[u8]) -> Result<Vec<u8>> {
    let mut decrypted = Vec::with_capacity(encrypted.len());
    
    // 从元数据提取密钥信息
    let key_size = 16;
    let key_offset = 8; // 假设前8字节是其他元数据
    if metadata.len() < key_offset + key_size {
        return Err(anyhow!("Invalid metadata format"));
    }
    
    let key = &metadata[key_offset..key_offset + key_size];
    
    // 解密过程（与加密过程相反）
    for (i, &byte) in encrypted.iter().enumerate() {
        let key_byte = key[i % key.len()];
        
        // 1. 字节旋转（右移4位）（与加密中的左移4位相反）
        let mut decrypted_byte = byte.rotate_left(4);
        
        // 2. 字节替换恢复
        decrypted_byte = match decrypted_byte & 0x0F {
            0x0F => (decrypted_byte & 0xF0) | 0x00,
            0x00 => decrypted_byte | 0x0F,
            _ => decrypted_byte,
        };
        
        // 3. XOR操作
        decrypted_byte ^= key_byte;
        
        decrypted.push(decrypted_byte);
    }
    
    Ok(decrypted)
} 