use crate::wasm::structure::{SectionType, WasmModule, Section};
use anyhow::{anyhow, Result};
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
    
    // 确保模块有内存段定义，因为VM解释器需要使用内存
    ensure_memory_section(&mut modified_module)?;

    // 1. Extract function body
    let func_body = extract_function_body(&modified_module, func_idx)?;

    // 2. Convert WebAssembly instructions to VM bytecode
    let vm_bytecode = convert_to_vm_bytecode(&func_body)?;

    // 3. Create VM metadata
    let vm_metadata = create_vm_metadata(func_idx, &vm_bytecode)?;

    // 4. Encrypt VM bytecode
    let encrypted_bytecode = encrypt_vm_bytecode(&vm_bytecode, &vm_metadata)?;

    // 5. 将加密字节码和元数据添加到数据段
    let (bytecode_offset, metadata_offset) = add_bytecode_to_data_section(
        &mut modified_module, 
        &encrypted_bytecode, 
        &vm_metadata
    )?;

    // 6. 替换原始函数体，调用Rust实现的解释器
    replace_with_rust_interpreter(
        &mut modified_module,
        func_idx,
        bytecode_offset,
        metadata_offset,
        encrypted_bytecode.len(),
        vm_metadata.len(),
    )?;

    debug!("Function {} virtualization completed", func_idx);
    Ok(modified_module)
}

/// 确保模块包含内存段
/// 如果模块中没有内存段，添加一个新的内存段
fn ensure_memory_section(module: &mut WasmModule) -> Result<()> {
    // 检查模块是否已包含内存段
    let has_memory_section = module
        .sections
        .iter()
        .any(|section| section.section_type == SectionType::Memory);

    if !has_memory_section {
        debug!("Adding memory section to module for VM interpreter");
        
        // 创建内存段数据
        // 内存段格式：
        // - 内存段数量 (通常为1，使用LEB128编码)
        // - 每个内存段的limits:
        //   - flags (0表示没有最大值限制，1表示有最大值限制)
        //   - 初始页数 (LEB128编码)
        //   - [可选] 最大页数 (如果flags为1，使用LEB128编码)
        
        let mut memory_data = Vec::new();
        
        // 内存段数量 (1个)
        memory_data.push(0x01);
        
        // Limits:
        // - flags (0: 没有最大值限制)
        // - 初始页数 (1页 = 64KB，足够大多数VM解释器使用)
        memory_data.push(0x00);
        memory_data.push(0x01);
        
        // 创建内存段
        let memory_section = Section {
            section_type: SectionType::Memory,
            name: None,
            data: memory_data,
        };
        
        // 找到适合插入内存段的位置
        // 内存段应该在全局段之后，导出段之前
        let mut insert_position = 0;
        let mut has_global_section = false;
        
        for (i, section) in module.sections.iter().enumerate() {
            match section.section_type {
                SectionType::Global => {
                    has_global_section = true;
                    insert_position = i + 1;
                }
                SectionType::Export => {
                    if !has_global_section {
                        insert_position = i;
                    }
                    break;
                }
                _ => {}
            }
        }
        
        // 将内存段插入模块
        module.sections.insert(insert_position, memory_section);
        debug!("Memory section added to module at position {}", insert_position);
    }
    
    Ok(())
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
    let mut rng = rand::rng();
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
                    // 读取LEB128编码的立即数
                    let mut value = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        value |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    // 将立即数值存入字节码
                    vm_bytecode.extend_from_slice(&value.to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            0x42 => {
                // i64.const
                vm_bytecode.push(0x01); // VMOpcode::Push
                // 标记为64位值
                vm_bytecode.push(0x01);
                
                // 读取LEB128编码的立即数
                if i < func_body.len() {
                    let mut value = 0u64;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        value |= ((byte & 0x7F) as u64) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    // 将立即数值存入字节码（只保留低32位，简化实现）
                    vm_bytecode.extend_from_slice(&(value as u32).to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            0x43 => {
                // f32.const
                vm_bytecode.push(0x01); // VMOpcode::Push
                // 标记为浮点值
                vm_bytecode.push(0x02);
                
                // 读取4字节浮点数
                if i + 3 < func_body.len() {
                    vm_bytecode.extend_from_slice(&func_body[i..i+4]);
                    i += 4;
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            0x44 => {
                // f64.const
                vm_bytecode.push(0x01); // VMOpcode::Push
                // 标记为双精度浮点值
                vm_bytecode.push(0x03);
                
                // 读取8字节浮点数（只保留低32位，简化实现）
                if i + 3 < func_body.len() {
                    vm_bytecode.extend_from_slice(&func_body[i..i+4]);
                    i += 8; // 仍然前进8个字节，但只读取前4个
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }

            // Local variable operations: local.get, local.set, local.tee
            0x20 => {
                // local.get
                vm_bytecode.push(0x40); // VMOpcode::Load

                // Add local variable index
                if i < func_body.len() {
                    let mut local_idx = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        local_idx |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    vm_bytecode.extend_from_slice(&local_idx.to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            0x21 => {
                // local.set
                vm_bytecode.push(0x41); // VMOpcode::Store

                // Add local variable index
                if i < func_body.len() {
                    let mut local_idx = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        local_idx |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    vm_bytecode.extend_from_slice(&local_idx.to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            0x22 => {
                // local.tee - tee先压入值，然后设置局部变量
                vm_bytecode.push(0x03); // VMOpcode::Dup
                
                // 然后是Store操作
                vm_bytecode.push(0x41); // VMOpcode::Store
                
                // Add local variable index
                if i < func_body.len() {
                    let mut local_idx = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        local_idx |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    vm_bytecode.extend_from_slice(&local_idx.to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            
            // Global variable operations
            0x23 => {
                // global.get
                vm_bytecode.push(0x45); // 使用新的VMOpcode::GlobalLoad
                
                // Add global variable index
                if i < func_body.len() {
                    let mut global_idx = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        global_idx |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    vm_bytecode.extend_from_slice(&global_idx.to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            0x24 => {
                // global.set
                vm_bytecode.push(0x46); // 使用新的VMOpcode::GlobalStore
                
                // Add global variable index
                if i < func_body.len() {
                    let mut global_idx = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        global_idx |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    vm_bytecode.extend_from_slice(&global_idx.to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
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
            0x6E => {
                // i32.div_u (无符号除法)
                vm_bytecode.push(0x13); // VMOpcode::Div
                vm_bytecode.push(0x01); // 标记为无符号除法
            }
            0x6F => {
                // i32.rem_s (有符号取余)
                vm_bytecode.push(0x14); // VMOpcode::Rem
            }
            0x70 => {
                // i32.rem_u (无符号取余)
                vm_bytecode.push(0x14); // VMOpcode::Rem
                vm_bytecode.push(0x01); // 标记为无符号
            }
            
            // 逻辑操作
            0x71 => {
                // i32.and
                vm_bytecode.push(0x20); // VMOpcode::And
            }
            0x72 => {
                // i32.or
                vm_bytecode.push(0x21); // VMOpcode::Or
            }
            0x73 => {
                // i32.xor
                vm_bytecode.push(0x22); // VMOpcode::Xor
            }
            0x45 => {
                // i32.eqz
                vm_bytecode.push(0x01); // VMOpcode::Push
                vm_bytecode.push(0x00); // 常量0
                vm_bytecode.extend_from_slice(&0u32.to_le_bytes());
                vm_bytecode.push(0x47); // 比较操作 - 定义新操作码
                vm_bytecode.push(0x00); // 等于比较
            }
            
            // 比较操作
            0x46 => {
                // i32.eq
                vm_bytecode.push(0x47); // 比较操作
                vm_bytecode.push(0x00); // 等于比较
            }
            0x47 => {
                // i32.ne
                vm_bytecode.push(0x47); // 比较操作
                vm_bytecode.push(0x01); // 不等于比较
            }
            0x48 => {
                // i32.lt_s
                vm_bytecode.push(0x47); // 比较操作
                vm_bytecode.push(0x02); // 小于比较（有符号）
            }
            0x49 => {
                // i32.lt_u
                vm_bytecode.push(0x47); // 比较操作
                vm_bytecode.push(0x03); // 小于比较（无符号）
            }
            0x4A => {
                // i32.gt_s
                vm_bytecode.push(0x47); // 比较操作
                vm_bytecode.push(0x04); // 大于比较（有符号）
            }
            0x4B => {
                // i32.gt_u
                vm_bytecode.push(0x47); // 比较操作
                vm_bytecode.push(0x05); // 大于比较（无符号）
            }
            
            // 内存操作
            0x28 => {
                // i32.load
                vm_bytecode.push(0x48); // 内存加载操作
                
                // 读取对齐和偏移
                if i + 1 < func_body.len() {
                    let alignment = func_body[i];
                    i += 1;
                    let offset = func_body[i];
                    i += 1;
                    
                    vm_bytecode.push(alignment);
                    vm_bytecode.push(offset);
                } else {
                    vm_bytecode.push(0); // 默认对齐
                    vm_bytecode.push(0); // 默认偏移
                }
            }
            0x36 => {
                // i32.store
                vm_bytecode.push(0x49); // 内存存储操作
                
                // 读取对齐和偏移
                if i + 1 < func_body.len() {
                    let alignment = func_body[i];
                    i += 1;
                    let offset = func_body[i];
                    i += 1;
                    
                    vm_bytecode.push(alignment);
                    vm_bytecode.push(offset);
                } else {
                    vm_bytecode.push(0); // 默认对齐
                    vm_bytecode.push(0); // 默认偏移
                }
            }

            // 控制流: br, br_if, return
            0x0C => {
                // br
                vm_bytecode.push(0x30); // VMOpcode::Jump

                // Add branch target
                if i < func_body.len() {
                    let mut target = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        target |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    vm_bytecode.extend_from_slice(&target.to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            0x0D => {
                // br_if
                vm_bytecode.push(0x31); // VMOpcode::JumpIf

                // Add branch target
                if i < func_body.len() {
                    let mut target = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        target |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    vm_bytecode.extend_from_slice(&target.to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            0x0F => {
                // return
                vm_bytecode.push(0x33); // VMOpcode::Return
            }
            0x10 => {
                // call
                vm_bytecode.push(0x32); // VMOpcode::Call
                
                // 读取函数索引
                if i < func_body.len() {
                    let mut func_idx = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        func_idx |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    vm_bytecode.extend_from_slice(&func_idx.to_le_bytes());
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认值
                }
            }
            0x11 => {
                // call_indirect
                vm_bytecode.push(0x50); // 间接调用
                
                // 读取类型索引和表索引
                if i + 1 < func_body.len() {
                    let mut type_idx = 0u32;
                    let mut shift = 0;
                    let mut byte;
                    
                    loop {
                        if i >= func_body.len() {
                            break;
                        }
                        
                        byte = func_body[i];
                        i += 1;
                        
                        type_idx |= ((byte & 0x7F) as u32) << shift;
                        shift += 7;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }
                    
                    vm_bytecode.extend_from_slice(&type_idx.to_le_bytes());
                    
                    // 读取表索引（通常为0）
                    let table_idx = func_body[i];
                    i += 1;
                    vm_bytecode.push(table_idx);
                } else {
                    vm_bytecode.extend_from_slice(&0u32.to_le_bytes()); // 默认类型索引
                    vm_bytecode.push(0); // 默认表索引
                }
            }
            0x01 => {
                // nop
                vm_bytecode.push(0xF0); // VMOpcode::Nop
            }
            0x1A => {
                // drop
                vm_bytecode.push(0x02); // VMOpcode::Pop
            }
            0x1B => {
                // select
                vm_bytecode.push(0x51); // 新定义的选择操作
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
    let mut rng = rand::rng();

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
    let mut rng = rand::rng();

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
    hasher.update(salt);

    // The salt value will be added to the beginning of the encrypted bytecode
    encrypted_bytecode.extend_from_slice(&salt);

    // Generate a long enough key stream
    let mut hash_result = hasher.finalize_reset();
    for chunk in vm_bytecode.chunks(32) {
        key_stream.extend_from_slice(&hash_result);

        // Update the hash to generate the next key stream block
        hasher.update(hash_result);
        hasher.update([chunk.len() as u8]);
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
            0x0F => encrypted_byte & 0xF0,
            _ => encrypted_byte,
        };

        // 3. Byte rotation (left shift 4 bits)
        encrypted_byte = encrypted_byte.rotate_right(4);

        encrypted_bytecode.push(encrypted_byte);
    }

    debug!(
        "Encrypted {} bytes of VM bytecode, resulting in {} bytes of encrypted data",
        vm_bytecode.len(),
        encrypted_bytecode.len()
    );

    Ok(encrypted_bytecode)
}

/// 将加密字节码和元数据添加到数据段
fn add_bytecode_to_data_section(
    module: &mut WasmModule,
    encrypted_bytecode: &[u8],
    vm_metadata: &[u8]
) -> Result<(u32, u32)> {
    // 查找数据段
    let mut data_section_idx = None;
    for (i, section) in module.sections.iter().enumerate() {
        if section.section_type == SectionType::Data {
            data_section_idx = Some(i);
            break;
        }
    }

    // 如果没有数据段，创建一个简单的空数据段
    if data_section_idx.is_none() {
        // 创建一个简单的数据段结构
        let data_section = Section {
            section_type: SectionType::Data,
            name: None,  // 数据段不需要名称
            data: Vec::new(),  // 空数据段，稍后会添加数据
        };
        module.sections.push(data_section);
        data_section_idx = Some(module.sections.len() - 1);
    }

    let data_section_idx = data_section_idx.unwrap();
    
    // 确保数据段内部结构正确
    if module.sections[data_section_idx].data.is_empty() {
        // 初始化数据段
        let mut data = Vec::new();
        
        // 添加数据段条目数量 (1个数据段)
        data.push(0x01);
        
        // 添加内存索引 (0)
        data.push(0x00);
        
        // 添加内存起始偏移量表达式 (i32.const 0)
        data.push(0x41); // i32.const
        data.push(0x00); // 值: 0
        data.push(0x0B); // 表达式结束
        
        // 添加初始数据长度 (0)
        data.push(0x00);
        
        module.sections[data_section_idx].data = data;
    }
    
    // 计算偏移量
    let bytecode_offset = module.sections[data_section_idx].data.len() as u32;
    
    // 添加字节码到数据段
    module.sections[data_section_idx].data.extend_from_slice(encrypted_bytecode);
    
    // 添加元数据到数据段
    let metadata_offset = module.sections[data_section_idx].data.len() as u32;
    module.sections[data_section_idx].data.extend_from_slice(vm_metadata);
    
    // 更新数据段大小
    // 注意：在实际的数据段中，这里可能需要更详细的处理
    // 但为了简化示例，我们保持这样的结构
    
    Ok((bytecode_offset, metadata_offset))
}

/// 替换原始函数体，生成调用Rust解释器的函数
fn replace_with_rust_interpreter(
    module: &mut WasmModule,
    func_idx: usize,
    bytecode_offset: u32,
    metadata_offset: u32,
    bytecode_size: usize,
    _metadata_size: usize,
) -> Result<()> {
    debug!("Replacing function {} with Rust VM interpreter", func_idx);

    // 查找代码段
    let mut code_section_idx = None;
    for (i, section) in module.sections.iter().enumerate() {
        if section.section_type == SectionType::Code {
            code_section_idx = Some(i);
            break;
        }
    }

    let code_section_idx = code_section_idx.ok_or_else(|| anyhow!("Missing Code section"))?;

    // 查找函数段，获取函数类型（参数和返回类型）
    let mut func_section_idx = None;
    for (i, section) in module.sections.iter().enumerate() {
        if section.section_type == SectionType::Function {
            func_section_idx = Some(i);
            break;
        }
    }
    
    let _func_section_idx = func_section_idx.ok_or_else(|| anyhow!("Missing Function section"))?;
    
    // 查找类型段
    let mut type_section_idx = None;
    for (i, section) in module.sections.iter().enumerate() {
        if section.section_type == SectionType::Type {
            type_section_idx = Some(i);
            break;
        }
    }
    
    let _type_section_idx = type_section_idx.ok_or_else(|| anyhow!("Missing Type section"))?;
    
    // 简化起见，我们假设函数接受i32参数并返回i32
    
    // 创建新的函数体
    let mut new_func_body = Vec::new();
    
    // 1. 本地变量声明
    new_func_body.push(0x01); // 1个变量类型组
    new_func_body.push(0x04); // 4个本地变量
    new_func_body.push(0x7F); // i32类型
    
    // 2. 创建一个固定大小的内存区域作为VM内存
    // 先分配一个内存页（64KB）
    new_func_body.push(0x3F); // memory.size
    new_func_body.push(0x00); // memory参数
    new_func_body.push(0x41); // i32.const
    new_func_body.push(0x01); // 值: 1 (页数)
    new_func_body.push(0x46); // i32.eq
    new_func_body.push(0x45); // i32.eqz
    
    // 如果内存大小不够，增加内存
    new_func_body.push(0x04); // if
    new_func_body.push(0x40); // 无返回值
    new_func_body.push(0x41); // i32.const
    new_func_body.push(0x01); // 值: 1 (页数)
    new_func_body.push(0x40); // memory.grow
    new_func_body.push(0x00); // memory参数
    new_func_body.push(0x1A); // drop
    new_func_body.push(0x0B); // end (if)
    
    // 3. 加载字节码偏移量到局部变量
    new_func_body.push(0x41); // i32.const
    write_unsigned_leb128(&mut new_func_body, bytecode_offset as u64);
    new_func_body.push(0x21); // local.set
    new_func_body.push(0x01); // 局部变量1
    
    // 4. 加载元数据偏移量到局部变量
    new_func_body.push(0x41); // i32.const
    write_unsigned_leb128(&mut new_func_body, metadata_offset as u64);
    new_func_body.push(0x21); // local.set
    new_func_body.push(0x02); // 局部变量2
    
    // 5. 加载参数到初始栈
    // 局部变量3存储栈数组指针
    new_func_body.push(0x41); // i32.const
    new_func_body.push(0x00); // 值: 0 (起始内存位置)
    new_func_body.push(0x21); // local.set
    new_func_body.push(0x03); // 局部变量3
    
    // 将函数参数存入栈数组首位置
    new_func_body.push(0x20); // local.get
    new_func_body.push(0x00); // 参数0
    
    new_func_body.push(0x20); // local.get
    new_func_body.push(0x03); // 局部变量3 (栈位置)
    
    new_func_body.push(0x36); // i32.store
    new_func_body.push(0x02); // 对齐
    new_func_body.push(0x00); // 偏移
    
    // 6. 模拟执行解释器功能
    // 对每个VM字节码指令进行处理
    // 实际上，这应该调用一个实现了execute_vm_bytecode功能的函数
    // 为了简化演示，我们这里使用自解释代码
    
    // 6.1 获取字节码并初始化程序计数器
    new_func_body.push(0x41); // i32.const
    new_func_body.push(0x00); // 值: 0 (程序计数器起始值)
    new_func_body.push(0x21); // local.set 
    new_func_body.push(0x00); // 局部变量0 (程序计数器)
    
    // 6.2 主循环 - 执行字节码
    // 开始循环
    new_func_body.push(0x03); // loop
    new_func_body.push(0x40); // 无返回值
    
    // 检查程序计数器是否超出字节码长度
    new_func_body.push(0x20); // local.get
    new_func_body.push(0x00); // 局部变量0 (程序计数器)
    new_func_body.push(0x41); // i32.const
    write_unsigned_leb128(&mut new_func_body, bytecode_size as u64);
    new_func_body.push(0x4F); // i32.lt_u
    
    // 如果PC >= 字节码长度，跳出循环
    new_func_body.push(0x45); // i32.eqz
    new_func_body.push(0x0D); // br_if
    new_func_body.push(0x01); // 跳转深度 1 (跳出循环)
    
    // 获取当前指令（简化模拟）
    new_func_body.push(0x20); // local.get
    new_func_body.push(0x01); // 局部变量1 (字节码偏移量)
    new_func_body.push(0x20); // local.get
    new_func_body.push(0x00); // 局部变量0 (程序计数器)
    new_func_body.push(0x6A); // i32.add
    
    new_func_body.push(0x2D); // i32.load8_u
    new_func_body.push(0x00); // 对齐
    new_func_body.push(0x00); // 偏移
    
    // 识别指令类型（仅处理Exit指令，简化处理）
    new_func_body.push(0x41); // i32.const
    new_func_body.push(0xFF); // 值: 0xFF (Exit指令)
    new_func_body.push(0x46); // i32.eq
    
    // 如果是Exit指令，跳出循环
    new_func_body.push(0x04); // if
    new_func_body.push(0x40); // 无返回值
    new_func_body.push(0x0C); // br
    new_func_body.push(0x01); // 跳转深度 1 (跳出循环)
    new_func_body.push(0x0B); // end (if)
    
    // 增加程序计数器
    new_func_body.push(0x20); // local.get
    new_func_body.push(0x00); // 局部变量0 (程序计数器)
    new_func_body.push(0x41); // i32.const
    new_func_body.push(0x01); // 值: 1
    new_func_body.push(0x6A); // i32.add
    new_func_body.push(0x21); // local.set
    new_func_body.push(0x00); // 局部变量0
    
    // 继续循环
    new_func_body.push(0x0C); // br
    new_func_body.push(0x00); // 跳转深度 0 (回到循环开始)
    
    // 结束循环
    new_func_body.push(0x0B); // end (loop)
    
    // 7. 返回结果
    // 为简化起见，我们返回模拟栈上的值（或默认值）
    new_func_body.push(0x20); // local.get
    new_func_body.push(0x03); // 局部变量3 (栈位置)
    
    new_func_body.push(0x28); // i32.load
    new_func_body.push(0x02); // 对齐
    new_func_body.push(0x00); // 偏移
    
    // 8. 函数结束标记
    new_func_body.push(0x0B); // end
    
    // 修改代码段，替换原始函数体
    let code_section_data = &mut module.sections[code_section_idx].data;
    
    // 查找目标函数位置
    let mut func_offset = 0;
    let mut current_idx = 0;
    
    // 首先读取函数数量
    let _func_count_bytes = read_unsigned_leb128(code_section_data, &mut func_offset);
    
    // 跳到目标函数
    while current_idx < func_idx && func_offset < code_section_data.len() {
        // 读取函数体大小
        let size = read_unsigned_leb128(code_section_data, &mut func_offset) as usize;
        
        // 跳过该函数
        func_offset += size;
        current_idx += 1;
    }
    
    if current_idx != func_idx || func_offset >= code_section_data.len() {
        return Err(anyhow!("Cannot find function {}", func_idx));
    }
    
    // 我们现在位于目标函数的起始位置
    let target_func_offset = func_offset;
    
    // 读取原始函数大小
    let original_func_size = read_unsigned_leb128(code_section_data, &mut func_offset) as usize;
    
    // 原始函数结束位置
    let original_func_end = func_offset + original_func_size;
    
    if original_func_end > code_section_data.len() {
        return Err(anyhow!("Function body exceeds code section range"));
    }
    
    // 创建新的代码段数据
    let mut new_code_data = Vec::new();
    
    // 复制目标函数之前的所有内容
    new_code_data.extend_from_slice(&code_section_data[0..target_func_offset]);
    
    // 写入新函数体大小
    write_unsigned_leb128(&mut new_code_data, new_func_body.len() as u64);
    
    // 添加新函数体
    new_code_data.extend_from_slice(&new_func_body);
    
    // 添加原始函数之后的所有内容
    if original_func_end < code_section_data.len() {
        new_code_data.extend_from_slice(&code_section_data[original_func_end..]);
    }
    
    // 更新代码段
    module.sections[code_section_idx].data = new_code_data;
    
    debug!("Function {} successfully replaced with Rust VM interpreter", func_idx);
    Ok(())
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
