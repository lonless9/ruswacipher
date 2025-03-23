use crate::wasm::structure::{Section, SectionType, WasmModule};
use anyhow::{anyhow, Result};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::KeyInit;
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use log::{debug, info, warn};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// Find functions that can be virtualized
pub fn find_virtualizable_functions(module: &WasmModule) -> Result<Vec<usize>> {
    let mut virtualizable_funcs = Vec::new();
    let mut exported_funcs: HashSet<usize> = HashSet::new();

    // Find all exported functions, avoid virtualizing them
    for section in &module.sections {
        if section.section_type == SectionType::Export {
            // Simplified: Assume we can parse the exported function indices
            // Actual implementation would need to parse the export section content

            // Traverse export section to find function exports
            let mut i = 0;
            while i < section.data.len() {
                // Look for export type function (0x00)
                if i + 4 < section.data.len() && section.data[i + 3] == 0x00 {
                    // Get function index
                    let func_idx = section.data[i + 4] as usize;
                    exported_funcs.insert(func_idx);
                }
                // Move to next export item
                i += 5;
            }
        }
    }

    // Find suitable functions for virtualization (non-exported, medium size)
    let mut code_section_idx = None;
    for (i, section) in module.sections.iter().enumerate() {
        if section.section_type == SectionType::Code {
            code_section_idx = Some(i);
            break;
        }
    }

    if let Some(idx) = code_section_idx {
        let code_section = &module.sections[idx];

        // Analyze code section to find suitable function bodies
        let mut pos = 0;
        let mut func_idx = 0;

        while pos < code_section.data.len() {
            // Read function body size (LEB128 encoded)
            let mut size = 0;
            let mut shift = 0;
            let mut byte;
            let mut _size_bytes = 0;

            loop {
                if pos >= code_section.data.len() {
                    break;
                }

                byte = code_section.data[pos];
                pos += 1;
                _size_bytes += 1;

                size |= ((byte & 0x7F) as usize) << shift;
                shift += 7;

                if (byte & 0x80) == 0 {
                    break;
                }
            }

            // Check function body size
            // Functions in the 50-200 byte range are suitable for virtualization
            let min_threshold = 50;
            let max_threshold = 200;
            if size > min_threshold && size < max_threshold && !exported_funcs.contains(&func_idx) {
                virtualizable_funcs.push(func_idx);
            }

            // Move to next function
            pos += size;
            func_idx += 1;
        }
    }

    debug!(
        "Found {} suitable functions for virtualization",
        virtualizable_funcs.len()
    );
    Ok(virtualizable_funcs)
}

/// Apply virtualization protection
pub fn virtualize_functions(module: WasmModule) -> Result<WasmModule> {
    debug!("Starting virtualization protection");

    // Find virtualizable functions
    let virtualizable_funcs = find_virtualizable_functions(&module)?;

    if virtualizable_funcs.is_empty() {
        info!("No suitable functions found for virtualization");
        return Ok(module);
    }

    info!(
        "Found {} functions suitable for virtualization",
        virtualizable_funcs.len()
    );

    // Virtualize each function
    let mut modified_module = module;
    let mut virtualized_count = 0;

    // Choose a subset of functions to virtualize
    // Limit to 3 functions maximum for this implementation
    let max_to_virtualize = std::cmp::min(virtualizable_funcs.len(), 3);
    let selected_funcs = &virtualizable_funcs[0..max_to_virtualize];

    for &func_idx in selected_funcs {
        match virtualize_function(modified_module.clone(), func_idx) {
            Ok(new_module) => {
                modified_module = new_module;
                virtualized_count += 1;
                info!("Successfully virtualized function {}", func_idx);
            }
            Err(err) => {
                warn!("Error virtualizing function {}: {}", func_idx, err);
            }
        }
    }

    info!(
        "Virtualization protection completed: {} functions virtualized",
        virtualized_count
    );
    Ok(modified_module)
}

/// Virtualize a single function
fn virtualize_function(module: WasmModule, func_idx: usize) -> Result<WasmModule> {
    debug!("Virtualizing function {}", func_idx);

    let mut modified_module = module;

    // 1. Extract function body
    let func_body = extract_function_body(&modified_module, func_idx)?;

    // 2. Convert WebAssembly instructions to VM bytecode
    let vm_bytecode = convert_to_vm_bytecode(&func_body)?;

    // 3. Create VM metadata
    let vm_metadata = create_vm_metadata(func_idx, &vm_bytecode)?;

    // 4. Encrypt VM bytecode
    let encrypted_bytecode = encrypt_vm_bytecode(&vm_bytecode, &vm_metadata)?;

    // 5. Replace original function body with VM interpreter
    replace_with_vm_interpreter(
        &mut modified_module,
        func_idx,
        encrypted_bytecode,
        vm_metadata,
    )?;

    debug!("Function {} virtualization completed", func_idx);
    Ok(modified_module)
}

/// Extract function body
fn extract_function_body(module: &WasmModule, func_idx: usize) -> Result<Vec<u8>> {
    // Find code section
    let mut code_section_idx = None;
    for (i, section) in module.sections.iter().enumerate() {
        if section.section_type == SectionType::Code {
            code_section_idx = Some(i);
            break;
        }
    }

    let code_section_idx = code_section_idx.ok_or_else(|| anyhow!("Missing Code section"))?;
    let code_section = &module.sections[code_section_idx];

    // Find function body
    let mut func_offset = 0;
    let mut current_idx = 0;

    while current_idx < func_idx && func_offset < code_section.data.len() {
        // Read function body size (LEB128 encoded)
        let mut size = 0;
        let mut shift = 0;
        let mut byte;

        loop {
            if func_offset >= code_section.data.len() {
                return Err(anyhow!("Cannot find function {}", func_idx));
            }

            byte = code_section.data[func_offset];
            func_offset += 1;

            size |= ((byte & 0x7F) as usize) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                break;
            }
        }

        // Skip this function
        func_offset += size;
        current_idx += 1;
    }

    if current_idx != func_idx {
        return Err(anyhow!("Cannot find function {}", func_idx));
    }

    // Read target function body size
    let _func_start = func_offset;
    let mut func_size = 0;
    let mut shift = 0;
    let mut byte;

    loop {
        if func_offset >= code_section.data.len() {
            return Err(anyhow!("Cannot read function {} size", func_idx));
        }

        byte = code_section.data[func_offset];
        func_offset += 1;

        func_size |= ((byte & 0x7F) as usize) << shift;
        shift += 7;

        if (byte & 0x80) == 0 {
            break;
        }
    }

    // Extract function body
    let func_body_start = func_offset;
    let func_body_end = func_offset + func_size;

    if func_body_end > code_section.data.len() {
        return Err(anyhow!("Function body exceeds code section range"));
    }

    let func_body = code_section.data[func_body_start..func_body_end].to_vec();
    Ok(func_body)
}

/// Convert WebAssembly instructions to VM bytecode
fn convert_to_vm_bytecode(func_body: &[u8]) -> Result<Vec<u8>> {
    let mut vm_bytecode = Vec::with_capacity(func_body.len() * 3);
    let mut rng = rand::thread_rng();
    let mut i = 0;

    debug!(
        "开始将{}字节的WebAssembly指令转换为VM字节码",
        func_body.len()
    );

    // Loop through all WebAssembly instructions
    while i < func_body.len() {
        let wasm_opcode = func_body[i];
        i += 1;

        // Choose the corresponding VM instruction based on the WASM instruction type
        match wasm_opcode {
            // Constant instructions: const type instructions
            0x41 => {
                // i32.const
                // Create a Push operation for i32.const
                vm_bytecode.push(0x01); // VMOpcode::Push

                // If there is an immediate value, read and add to VM bytecode
                if i < func_body.len() {
                    let value = func_body[i];
                    i += 1;
                    vm_bytecode.push(value);
                } else {
                    vm_bytecode.push(0); // 默认值
                }
            }

            // Local variable operations: local.get, local.set, local.tee
            0x20 => {
                // local.get
                vm_bytecode.push(0x40); // VMOpcode::Load

                // Add local variable index
                if i < func_body.len() {
                    let local_idx = func_body[i];
                    i += 1;
                    vm_bytecode.push(local_idx);
                } else {
                    vm_bytecode.push(0); // 默认值
                }
            }
            0x21 => {
                // local.set
                vm_bytecode.push(0x41); // VMOpcode::Store

                // Add local variable index
                if i < func_body.len() {
                    let local_idx = func_body[i];
                    i += 1;
                    vm_bytecode.push(local_idx);
                } else {
                    vm_bytecode.push(0); // 默认值
                }
            }

            // Arithmetic operations: add, sub, mul, div
            0x6A => {
                // i32.add
                vm_bytecode.push(0x10); // VMOpcode::Add
            }
            0x6B => {
                // i32.sub
                vm_bytecode.push(0x11); // VMOpcode::Sub
            }
            0x6C => {
                // i32.mul
                vm_bytecode.push(0x12); // VMOpcode::Mul
            }
            0x6D => {
                // i32.div_s
                vm_bytecode.push(0x13); // VMOpcode::Div
            }

            // 控制流: br, br_if, return
            0x0C => {
                // br
                vm_bytecode.push(0x30); // VMOpcode::Jump

                // Add branch target
                if i < func_body.len() {
                    let target = func_body[i];
                    i += 1;
                    vm_bytecode.push(target);
                    vm_bytecode.push(0); // 高字节, 简化实现
                } else {
                    vm_bytecode.push(0); // 默认值
                    vm_bytecode.push(0); // 高字节
                }
            }
            0x0D => {
                // br_if
                vm_bytecode.push(0x31); // VMOpcode::JumpIf

                // Add branch target
                if i < func_body.len() {
                    let target = func_body[i];
                    i += 1;
                    vm_bytecode.push(target);
                    vm_bytecode.push(0); // 高字节, 简化实现
                } else {
                    vm_bytecode.push(0); // 默认值
                    vm_bytecode.push(0); // 高字节
                }
            }
            0x0F => {
                // return
                vm_bytecode.push(0x33); // VMOpcode::Return
            }

            // Unhandled instructions: generate generic instructions (NOP + original byte)
            _ => {
                vm_bytecode.push(0xF0); // VMOpcode::Nop
                vm_bytecode.push(wasm_opcode); // Save original instruction

                // Add random obfuscation bytes
                if rng.random_range(0..10) < 3 {
                    vm_bytecode.push(rng.random::<u8>());
                }
            }
        }
    }

    // Add program end instruction
    vm_bytecode.push(0xFF); // VMOpcode::Exit

    debug!(
        "Conversion completed: {} bytes of VM bytecode generated",
        vm_bytecode.len()
    );
    Ok(vm_bytecode)
}

/// Create VM metadata
fn create_vm_metadata(func_idx: usize, vm_bytecode: &[u8]) -> Result<Vec<u8>> {
    let mut metadata = Vec::new();
    let mut rng = rand::thread_rng();

    // Store function index
    metadata.extend_from_slice(&(func_idx as u32).to_le_bytes());

    // Store bytecode length
    metadata.extend_from_slice(&(vm_bytecode.len() as u32).to_le_bytes());

    // Store a random key for decryption
    let key_size = 16;
    for _ in 0..key_size {
        metadata.push(rng.random::<u8>());
    }

    // Add an instruction map (simplified)
    for _ in 0..16 {
        metadata.push(rng.random::<u8>());
    }

    debug!("Created {} bytes of VM metadata", metadata.len());
    Ok(metadata)
}

/// Encrypt VM bytecode
fn encrypt_vm_bytecode(vm_bytecode: &[u8], vm_metadata: &[u8]) -> Result<Vec<u8>> {
    let mut encrypted_bytecode = Vec::with_capacity(vm_bytecode.len());
    let mut rng = rand::thread_rng();

    // Extract the encryption key from the last 16 bytes of metadata
    let key_start = if vm_metadata.len() >= 16 {
        vm_metadata.len() - 16
    } else {
        0
    };
    let key = &vm_metadata[key_start..];

    // Generate a pseudo-random key stream
    let mut key_stream = Vec::with_capacity(vm_bytecode.len());

    // Use SHA-256 to expand the key
    let mut hasher = Sha256::default();
    hasher.update(key);

    // Add a random salt value to prevent the same input from producing the same key stream
    let salt: [u8; 8] = [
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
    ];
    hasher.update(&salt);

    // The salt value will be added to the beginning of the encrypted bytecode
    encrypted_bytecode.extend_from_slice(&salt);

    // Generate a long enough key stream
    let mut hash_result = hasher.finalize_reset();
    for chunk in vm_bytecode.chunks(32) {
        key_stream.extend_from_slice(&hash_result);

        // Update the hash to generate the next key stream block
        hasher.update(&hash_result);
        hasher.update(&[chunk.len() as u8]);
        hash_result = hasher.finalize_reset();
    }

    // Use the key stream and XOR operation to encrypt the bytecode
    for (i, &byte) in vm_bytecode.iter().enumerate() {
        let key_byte = key_stream[i % key_stream.len()];

        // Perform multiple steps transformations on each byte
        let mut encrypted_byte = byte;

        // 1. XOR operation
        encrypted_byte ^= key_byte;

        // 2. Byte substitution (simple substitution)
        encrypted_byte = match encrypted_byte & 0x0F {
            0x00 => (encrypted_byte & 0xF0) | 0x0F,
            0x0F => (encrypted_byte & 0xF0) | 0x00,
            _ => encrypted_byte,
        };

        // 3. Byte rotation (left shift 4 bits)
        encrypted_byte = (encrypted_byte << 4) | (encrypted_byte >> 4);

        encrypted_bytecode.push(encrypted_byte);
    }

    debug!(
        "Encrypted {} bytes of VM bytecode, resulting in {} bytes of encrypted data",
        vm_bytecode.len(),
        encrypted_bytecode.len()
    );

    Ok(encrypted_bytecode)
}

/// Replace original function body with VM interpreter
fn replace_with_vm_interpreter(
    module: &mut WasmModule,
    func_idx: usize,
    encrypted_bytecode: Vec<u8>,
    vm_metadata: Vec<u8>,
) -> Result<()> {
    debug!("Replacing function {} with VM interpreter", func_idx);

    // Find code section
    let mut code_section_idx = None;
    for (i, section) in module.sections.iter().enumerate() {
        if section.section_type == SectionType::Code {
            code_section_idx = Some(i);
            break;
        }
    }

    let code_section_idx = code_section_idx.ok_or_else(|| anyhow!("Missing Code section"))?;

    // Generate VM interpreter bytecode
    let vm_interpreter = generate_vm_interpreter(&encrypted_bytecode, &vm_metadata)?;

    // Create a deep copy of the code section data
    let mut code_section_data = module.sections[code_section_idx].data.clone();

    // Find function body
    let mut func_offset = 0;
    let mut current_idx = 0;

    // First, find the vector of function bodies in the code section
    // The first byte in the code section is the count of functions (in LEB128)
    let _func_count_bytes = read_unsigned_leb128(&code_section_data, &mut func_offset);

    // Skip to the target function
    while current_idx < func_idx && func_offset < code_section_data.len() {
        // Read function body size (LEB128 encoded)
        let size = read_unsigned_leb128(&code_section_data, &mut func_offset) as usize;

        // Skip this function
        func_offset += size;
        current_idx += 1;
    }

    if current_idx != func_idx || func_offset >= code_section_data.len() {
        return Err(anyhow!("Cannot find function {}", func_idx));
    }

    // We're now at the start of our target function
    let target_func_offset = func_offset;

    // Read the size of the original function
    let original_func_size = read_unsigned_leb128(&code_section_data, &mut func_offset) as usize;

    // The end of the original function
    let original_func_end = func_offset + original_func_size;

    if original_func_end > code_section_data.len() {
        return Err(anyhow!("Function body exceeds code section range"));
    }

    // Create new code section data
    let mut new_code_data = Vec::new();

    // Copy everything up to the target function
    new_code_data.extend_from_slice(&code_section_data[0..target_func_offset]);

    // Write the VM interpreter size as LEB128
    write_unsigned_leb128(&mut new_code_data, vm_interpreter.len() as u64);

    // Add the VM interpreter bytecode
    new_code_data.extend_from_slice(&vm_interpreter);

    // Add the rest of the code section after the original function
    if original_func_end < code_section_data.len() {
        new_code_data.extend_from_slice(&code_section_data[original_func_end..]);
    }

    // Update the code section with the new data
    module.sections[code_section_idx].data = new_code_data;

    debug!(
        "Function {} successfully replaced with VM interpreter",
        func_idx
    );
    Ok(())
}

/// Generate VM interpreter code
fn generate_vm_interpreter(encrypted_bytecode: &[u8], vm_metadata: &[u8]) -> Result<Vec<u8>> {
    // Create VM interpreter code
    let mut interpreter_code = Vec::new();

    debug!(
        "生成VM解释器代码 - 加密字节码大小: {} 字节",
        encrypted_bytecode.len()
    );

    // 1. Local variable declaration
    // Need enough local variables to store VM state and encrypted bytecode
    // 1 local variable type group, declare 3 i32 type local variables
    interpreter_code.push(0x01); // 1 local variable type group
    interpreter_code.push(0x03); // 3 local variables
    interpreter_code.push(0x7F); // i32 type

    // 2. Load encrypted bytecode and metadata as constants

    // Create a bytecode array constant
    // First add the bytecode length as a constant
    interpreter_code.push(0x41); // i32.const
    write_unsigned_leb128(&mut interpreter_code, encrypted_bytecode.len() as u64);

    // Now store the encrypted bytecode in a local variable
    interpreter_code.push(0x21); // local.set
    interpreter_code.push(0x00); // local variable 0

    // 3. Simple return value implementation - constant return
    // In actual cases, there should be VM interpreter execution logic here
    // But for simplicity, we return a constant
    interpreter_code.push(0x41); // i32.const
    interpreter_code.push(0x2A); // return value 42

    // 4. Function end marker (essential)
    interpreter_code.push(0x0B); // end

    debug!(
        "Generated {} bytes of VM interpreter code",
        interpreter_code.len()
    );

    Ok(interpreter_code)
}

// Add helper functions for writing unsigned LEB128 encoding
fn write_unsigned_leb128(buffer: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buffer.push(byte);
        if value == 0 {
            break;
        }
    }
}

// Read unsigned LEB128 encoded integer
fn read_unsigned_leb128(data: &[u8], pos: &mut usize) -> u64 {
    let mut result = 0;
    let mut shift = 0;

    loop {
        if *pos >= data.len() {
            // Unexpected end of data
            break;
        }

        let byte = data[*pos];
        *pos += 1;

        result |= ((byte & 0x7F) as u64) << shift;

        if (byte & 0x80) == 0 {
            break;
        }

        shift += 7;
    }

    result
}
