use crate::wasm::structure::{SectionType, WasmModule};
use anyhow::{anyhow, Result};
use log::{debug, info};
use rand::Rng;
use std::collections::HashSet;

/// WASM instruction type, used for stack analysis
enum WasmInstrType {
    /// Instruction to push a value onto the stack
    Push,
    /// Instruction to pop a value from the stack
    Pop,
    /// End block instruction (such as end, else)
    BlockEnd,
    /// Start block instruction (such as block, loop, if)
    BlockStart,
    /// Instruction to keep the stack unchanged
    Neutral,
    /// Jump instruction (such as br, br_if)
    Branch,
    /// Call instruction
    Call,
    /// Return instruction
    Return,
    /// Other or unknown instruction
    Other,
}

/// Get the number of functions in the module
fn count_funcs(module: &WasmModule) -> usize {
    if let Some(code_section) = module
        .sections
        .iter()
        .find(|s| s.section_type == SectionType::Code)
    {
        if code_section.data.is_empty() {
            return 0;
        }

        // Read the number of functions (the first byte of the code section is the LEB128 encoded number of functions)
        let mut pos = 0;
        let mut num_funcs = 0;
        let mut shift = 0;

        while pos < code_section.data.len() {
            let byte = code_section.data[pos];
            num_funcs |= ((byte & 0x7F) as usize) << shift;
            pos += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        return num_funcs;
    }

    0
}

/// Analyze the impact of instructions on the stack
fn analyze_instr(opcode: u8) -> (WasmInstrType, i32) {
    match opcode {
        // Constant instructions - push onto stack
        0x41 => (WasmInstrType::Push, 1), // i32.const
        0x42 => (WasmInstrType::Push, 1), // i64.const
        0x43 => (WasmInstrType::Push, 1), // f32.const
        0x44 => (WasmInstrType::Push, 1), // f64.const

        // Local variable access
        0x20 => (WasmInstrType::Push, 1),    // local.get
        0x21 => (WasmInstrType::Pop, -1),    // local.set
        0x22 => (WasmInstrType::Neutral, 0), // local.tee

        // Global variable access
        0x23 => (WasmInstrType::Push, 1), // global.get
        0x24 => (WasmInstrType::Pop, -1), // global.set

        // Memory instructions
        0x28..=0x3E => (WasmInstrType::Neutral, 0), // Various memory operations

        // Control instructions
        0x02 => (WasmInstrType::BlockStart, 0),    // block
        0x03 => (WasmInstrType::BlockStart, 0),    // loop
        0x04 => (WasmInstrType::BlockStart, -1),   // if
        0x05 => (WasmInstrType::BlockEnd, 0),      // else
        0x0B => (WasmInstrType::BlockEnd, 0),      // end
        0x0C..=0x0F => (WasmInstrType::Branch, 0), // br, br_if, etc.
        0x10 => (WasmInstrType::Call, 0),          // call
        0x11 => (WasmInstrType::Call, 0),          // call_indirect
        0x0F => (WasmInstrType::Return, 0),        // return

        // Arithmetic instructions - usually pop two values and push one value
        0x6A..=0x7F => (WasmInstrType::Neutral, -1), // Binary operations
        0x45..=0x69 => (WasmInstrType::Neutral, -1), // Unary operations and comparisons

        // Other instructions - default conservative handling
        _ => (WasmInstrType::Other, 0),
    }
}

/// Find safe split points in function body
fn find_safe_split_points(func_body: &[u8]) -> Vec<usize> {
    let mut safe_points = Vec::new();
    let mut stack_depth = 0;
    let mut block_depth = 0;
    let mut pos = 0;

    // Must skip the local variable declaration part
    let mut local_decl_count = 0;
    if !func_body.is_empty() {
        local_decl_count = func_body[0] as usize;
        pos = 1;

        // Skip all local variable declarations
        for _ in 0..local_decl_count {
            // Each local variable declaration has a type and count field
            if pos + 1 < func_body.len() {
                // Skip, do not parse in detail
                pos += 2;
            } else {
                break;
            }
        }
    }

    // Start analyzing the main part
    let mut last_safe_point = pos;

    while pos < func_body.len() {
        let opcode = func_body[pos];
        pos += 1;

        // Analyze the instruction type and stack impact
        let (instr_type, stack_effect) = analyze_instr(opcode);

        // Update the stack depth
        stack_depth += stack_effect;

        // Update the block depth according to the instruction type
        match instr_type {
            WasmInstrType::BlockStart => block_depth += 1,
            WasmInstrType::BlockEnd => {
                if block_depth > 0 {
                    block_depth -= 1
                }
            }
            _ => {}
        }

        // Check if it is a safe split point:
        // 1. Stack depth is 0 (balanced)
        // 2. Not in a control block (such as if, loop, etc.)
        // 3. Not a return instruction
        // 4. Has a certain distance from the last safe point (min_size)
        if stack_depth == 0
            && block_depth == 0
            && !matches!(instr_type, WasmInstrType::Return)
            && pos - last_safe_point >= 10
        {
            safe_points.push(pos);
            last_safe_point = pos;
        }

        // Skip the instruction operand
        match opcode {
            // Process instructions with immediate values
            0x41 => {
                // i32.const - LEB128 encoded
                while pos < func_body.len() && (func_body[pos] & 0x80) != 0 {
                    pos += 1;
                }
                if pos < func_body.len() {
                    pos += 1; // The last byte
                }
            }
            0x42 => {
                // i64.const - LEB128 encoded
                while pos < func_body.len() && (func_body[pos] & 0x80) != 0 {
                    pos += 1;
                }
                if pos < func_body.len() {
                    pos += 1; // The last byte
                }
            }
            0x43 => {
                // f32.const - 4 bytes
                pos += 4.min(func_body.len() - pos);
            }
            0x44 => {
                // f64.const - 8 bytes
                pos += 8.min(func_body.len() - pos);
            }
            0x20..=0x24 => {
                // Local variable and global variable index - LEB128 encoded
                while pos < func_body.len() && (func_body[pos] & 0x80) != 0 {
                    pos += 1;
                }
                if pos < func_body.len() {
                    pos += 1; // The last byte
                }
            }
            0x10 => {
                // call - function index, LEB128 encoded
                while pos < func_body.len() && (func_body[pos] & 0x80) != 0 {
                    pos += 1;
                }
                if pos < func_body.len() {
                    pos += 1; // The last byte
                }
            }
            // Process other instruction operands...
            _ => {}
        }
    }

    safe_points
}

/// Get the exported function indices in the module
fn get_exported_functions(module: &WasmModule) -> HashSet<usize> {
    let mut exported_funcs = HashSet::new();

    // Find all exported functions
    for section in &module.sections {
        if section.section_type == SectionType::Export {
            let export_data = &section.data;
            let mut pos = 0;
            let mut num_exports = 0;
            let mut shift = 0;

            // Read the number of exports
            while pos < export_data.len() {
                let byte = export_data[pos];
                num_exports |= ((byte & 0x7F) as usize) << shift;
                pos += 1;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }

            debug!("Found {} exports", num_exports);

            // Iterate through each export entry
            for _ in 0..num_exports {
                if pos >= export_data.len() {
                    break;
                }

                // Read the export name length
                let mut name_len = 0;
                shift = 0;
                while pos < export_data.len() {
                    let byte = export_data[pos];
                    pos += 1;
                    name_len |= ((byte & 0x7F) as usize) << shift;
                    if byte & 0x80 == 0 {
                        break;
                    }
                    shift += 7;
                }

                // Skip the export name
                pos += name_len;

                // Read the export type
                if pos < export_data.len() {
                    let export_type = export_data[pos];
                    pos += 1;

                    // Check if it is a function export (0x00)
                    if export_type == 0x00 && pos < export_data.len() {
                        // Read the function index
                        let mut func_idx = 0;
                        shift = 0;
                        while pos < export_data.len() {
                            let byte = export_data[pos];
                            pos += 1;
                            func_idx |= ((byte & 0x7F) as usize) << shift;
                            if byte & 0x80 == 0 {
                                break;
                            }
                            shift += 7;
                        }

                        exported_funcs.insert(func_idx);
                        debug!("Added exported function index: {}", func_idx);
                    } else {
                        // Skip non-function exports
                        while pos < export_data.len() && (export_data[pos] & 0x80) != 0 {
                            pos += 1;
                        }
                        if pos < export_data.len() {
                            pos += 1; // Skip the last byte
                        }
                    }
                }
            }
        }
    }

    exported_funcs
}

/// Split large functions
pub fn split_large_functions(module: WasmModule) -> Result<WasmModule> {
    debug!("Starting to split large functions");

    // Get the exported function indices
    let exported_funcs = get_exported_functions(&module);

    // Find the list of large functions
    let large_functions = find_large_functions(&module, &exported_funcs)?;

    // If no large functions are found, return the original module
    if large_functions.is_empty() {
        debug!("No large functions found for splitting");
        return Ok(module);
    }

    debug!("Found {} large functions to split", large_functions.len());

    // Split each large function step by step
    let mut result_module = module;
    for func_idx in large_functions {
        debug!("Processing function {}", func_idx);
        result_module = split_function(result_module, func_idx)?;
    }

    debug!("Function splitting completed");
    Ok(result_module)
}

/// Find functions suitable for splitting
fn find_large_functions(
    module: &WasmModule,
    exported_funcs: &HashSet<usize>,
) -> Result<Vec<usize>> {
    debug!("Total functions in code section: {}", count_funcs(module));

    let mut large_funcs = Vec::new();
    let mut func_bodies: Vec<(usize, usize)> = Vec::new();

    // Find the code section
    if let Some(code_section) = module
        .sections
        .iter()
        .find(|s| s.section_type == SectionType::Code)
    {
        let mut pos = 1; // Skip the function count field
        while pos < code_section.data.len() {
            let body_start = pos;
            let mut size_bytes = 0;
            let mut shift = 0;

            // Read the function body size
            while pos < code_section.data.len() {
                let byte = code_section.data[pos];
                size_bytes |= ((byte & 0x7F) as u32) << shift;
                pos += 1;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }

            let body_size = size_bytes as usize;
            let body_end = pos + body_size;

            // Save the function body position information
            func_bodies.push((pos, body_size));

            // Move to the next function body
            pos = body_end;
        }
    }

    // Iterate through all functions to find large functions
    for (func_idx, (func_pos, func_size)) in func_bodies.iter().enumerate() {
        if *func_size > 100 {
            // The threshold for large functions
            // Do not split exported functions
            if exported_funcs.contains(&func_idx) {
                debug!("Function {} is exported, skipping", func_idx);
                continue;
            }

            // Analyze the function body to find safe split points
            if let Some(code_section) = module
                .sections
                .iter()
                .find(|s| s.section_type == SectionType::Code)
            {
                let func_body = &code_section.data[*func_pos..*func_pos + *func_size];
                let safe_points = find_safe_split_points(func_body);

                // Modify: As long as there is at least 1 safe split point, it can be split
                if safe_points.len() >= 1 {
                    debug!(
                        "Function {} with size {} has {} safe split points, adding to list",
                        func_idx,
                        func_size,
                        safe_points.len()
                    );
                    large_funcs.push(func_idx);
                } else {
                    debug!(
                        "Function {} has size {} but only {} safe split points, skipping",
                        func_idx,
                        func_size,
                        safe_points.len()
                    );
                }
            }
        }
    }

    debug!(
        "Found {} large functions suitable for splitting",
        large_funcs.len()
    );
    Ok(large_funcs)
}

/// Split a single function
fn split_function(module: WasmModule, func_idx: usize) -> Result<WasmModule> {
    debug!("Splitting function {}", func_idx);

    let mut modified_module = module;

    // 1. Find Function and Code sections
    let mut function_section_idx = None;
    let mut code_section_idx = None;

    for (i, section) in modified_module.sections.iter().enumerate() {
        match section.section_type {
            SectionType::Function => function_section_idx = Some(i),
            SectionType::Code => code_section_idx = Some(i),
            _ => {}
        }
    }

    // Ensure we found the necessary sections
    let function_section_idx =
        function_section_idx.ok_or_else(|| anyhow!("Missing Function section"))?;
    let code_section_idx = code_section_idx.ok_or_else(|| anyhow!("Missing Code section"))?;

    // 2. Extract target function body from code section
    // Default initialization, avoid "possibly-uninitialized" error
    let mut function_type_index = 0;
    let mut function_body_data = Vec::new();
    let mut func_body_start = 0;
    let mut func_body_end = 0;
    let mut func_start = 0;
    let mut num_funcs = 0;

    {
        // Get the function type index - correctly parse the function section
        let function_section = &modified_module.sections[function_section_idx];
        let function_data = &function_section.data;

        // The first byte of the function section is the function count (LEB128 encoded)
        let mut pos = 0;
        let mut num_func_types = 0;
        let mut shift = 0;

        while pos < function_data.len() {
            let byte = function_data[pos];
            num_func_types |= ((byte & 0x7F) as usize) << shift;
            pos += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        if func_idx >= num_func_types {
            return Err(anyhow!(
                "Function index {} out of range (only {} functions)",
                func_idx,
                num_func_types
            ));
        }

        // Skip the function type index in the function section until we reach the target function
        let mut current_func_idx = 0;
        while current_func_idx < func_idx && pos < function_data.len() {
            // Read the LEB128 encoded type index (usually a single byte, but theoretically it can be multi-byte)
            let mut byte;
            loop {
                if pos >= function_data.len() {
                    return Err(anyhow!("Unexpected end of function section"));
                }
                byte = function_data[pos];
                pos += 1;
                if byte & 0x80 == 0 {
                    break;
                }
            }
            current_func_idx += 1;
        }

        // Read the type index of the target function
        if pos < function_data.len() {
            function_type_index = function_data[pos] as usize;
        } else {
            return Err(anyhow!(
                "Function type index not found for function {}",
                func_idx
            ));
        }

        // Extract function body
        let code_section = &modified_module.sections[code_section_idx];
        let code_data = &code_section.data;

        // First read the function count
        let mut i = 0;
        shift = 0;

        while i < code_data.len() {
            let byte = code_data[i];
            num_funcs |= ((byte & 0x7F) as usize) << shift;
            i += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        if num_funcs <= func_idx {
            return Err(anyhow!(
                "Function index {} out of range (only {} functions)",
                func_idx,
                num_funcs
            ));
        }

        // Find the target function body
        let mut current_func_idx = 0;
        while current_func_idx < num_funcs && i < code_data.len() {
            func_start = i;

            // Read the function body size
            let mut size = 0;
            shift = 0;

            while i < code_data.len() {
                let byte = code_data[i];
                size |= ((byte & 0x7F) as usize) << shift;
                i += 1;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }

            if current_func_idx == func_idx {
                // Find the target function
                func_body_start = i;
                func_body_end = i + size;

                if func_body_end > code_data.len() {
                    return Err(anyhow!("Function body exceeds code section range"));
                }

                function_body_data = code_data[func_body_start..func_body_end].to_vec();
                break;
            }

            // Move to the next function
            i += size;
            current_func_idx += 1;
        }

        if current_func_idx != func_idx {
            return Err(anyhow!("Cannot find function {}", func_idx));
        }
    }

    // 3. Find safe function split points
    // Use instruction analysis to find stack balance safe points
    let min_split_size = 10; // Minimum size of sub-functions
    let max_split_points = 4; // Maximum split into 5 functions (4 split points)

    let split_points = find_safe_split_points(&function_body_data);

    // If no safe split points are found, return the original module
    if split_points.is_empty() {
        debug!(
            "Not enough safe split points found for function {}. Skipping split.",
            func_idx
        );
        return Ok(modified_module);
    }

    let num_subfuncs = split_points.len() + 1; // The number of split points + 1 = the number of sub-functions

    debug!(
        "Splitting function {} into {} sub-functions at safe points",
        func_idx, num_subfuncs
    );

    // 4. Create sub-function bodies
    let mut sub_func_bodies = Vec::with_capacity(num_subfuncs);

    // The first sub-function starts from the original function body
    let mut start_pos = 0;

    // Analyze the function signature before splitting to ensure correct handling of parameters and return values
    // For simplicity, we assume all sub-functions have the same signature and preserve the original function's parameters and return values

    for (i, &split_point) in split_points.iter().enumerate() {
        // Create a sub-function body from start_pos to split_point
        let mut sub_body = function_body_data[start_pos..split_point].to_vec();

        // Add a call to the next sub-function
        sub_body.push(0x10); // call instruction

        // Get the index of the next sub-function
        let next_func_idx = num_funcs + i + 1;

        // Write the function index (LEB128 encoded)
        let mut func_idx_encoded = Vec::new();
        write_leb128(&mut func_idx_encoded, next_func_idx as u32);
        sub_body.extend_from_slice(&func_idx_encoded);

        // Ensure the sub-function ends with an end instruction
        if sub_body.last() != Some(&0x0B) {
            sub_body.push(0x0B); // end opcode
        }

        // Create a complete function body (with local variable declaration)
        let mut full_body = Vec::new();

        // Add a local variable declaration (for simplicity, no local variables are declared)
        full_body.push(0x00); // 0 local variable groups

        // Add the function body
        full_body.extend_from_slice(&sub_body);

        // Calculate size and encode as LEB128
        let mut size_encoded = Vec::new();
        write_leb128(&mut size_encoded, full_body.len() as u32);

        // Combine size and function body
        let mut complete_body = size_encoded;
        complete_body.extend_from_slice(&full_body);

        sub_func_bodies.push(complete_body);

        start_pos = split_point;
    }

    // Process the last sub-function, ensuring the original function's return value is preserved
    let mut last_body = function_body_data[start_pos..].to_vec();

    // Ensure the last sub-function ends with an end instruction
    if last_body.last() != Some(&0x0B) {
        last_body.push(0x0B); // end opcode
    }

    // Create a complete function body
    let mut full_last_body = Vec::new();

    // Add a local variable declaration
    full_last_body.push(0x00); // 0 local variable groups

    // Add the function body
    full_last_body.extend_from_slice(&last_body);

    // Calculate size and encode
    let mut size_encoded = Vec::new();
    write_leb128(&mut size_encoded, full_last_body.len() as u32);

    // Combine size and function body
    let mut complete_last_body = size_encoded;
    complete_last_body.extend_from_slice(&full_last_body);

    sub_func_bodies.push(complete_last_body);

    // 5. Modify the original function body to call the first sub-function and preserve the original function signature
    let mut new_main_body = Vec::new();

    // Add a local variable declaration
    new_main_body.push(0x00); // 0 local variable groups

    // Add a call to the first sub-function
    new_main_body.push(0x10); // call instruction

    // Get the index of the first sub-function
    let first_subfunc_idx = num_funcs;
    let mut func_idx_encoded = Vec::new();
    write_leb128(&mut func_idx_encoded, first_subfunc_idx as u32);
    new_main_body.extend_from_slice(&func_idx_encoded);

    // Add an end instruction
    new_main_body.push(0x0B); // end opcode

    // Calculate size and encode
    let mut main_size_encoded = Vec::new();
    write_leb128(&mut main_size_encoded, new_main_body.len() as u32);

    // Combine size and function body
    let mut complete_main_body = main_size_encoded;
    complete_main_body.extend_from_slice(&new_main_body);

    // 6. Update the code section and function section

    // Update the function section, add sub-function type
    {
        let function_section = &mut modified_module.sections[function_section_idx];
        let function_data = &mut function_section.data;

        // The function section data starts with a LEB128 encoded value representing the number of function types, followed by the type index of each function
        // We need to find the position of the current function count, then update it
        let mut pos = 0;
        let mut num_funcs = 0;
        let mut shift = 0;

        // Read the current function count
        while pos < function_data.len() {
            let byte = function_data[pos];
            num_funcs |= ((byte & 0x7F) as usize) << shift;
            pos += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        // Calculate the new function count
        let new_num_funcs = num_funcs + num_subfuncs;

        // Add type information for each sub-function (using the same type as the original function)
        for _ in 0..num_subfuncs {
            // Add type index (LEB128 encoded)
            let mut type_idx_encoded = Vec::new();
            write_leb128(&mut type_idx_encoded, function_type_index as u32);
            function_data.extend_from_slice(&type_idx_encoded);
        }

        // Update the function count in the function section (replace the leading LEB128 encoded value)
        let mut updated_function_data = Vec::new();
        write_leb128(&mut updated_function_data, new_num_funcs as u32);

        // Copy the remaining data except the count
        updated_function_data.extend_from_slice(&function_data[pos..]);

        // Replace the function section data
        *function_data = updated_function_data;
    }

    // Update the code section, replace the original function and add sub-functions
    {
        let code_section = &mut modified_module.sections[code_section_idx];

        // Replace the original function body
        code_section.data.splice(
            func_start..func_body_end,
            complete_main_body.iter().cloned(),
        );

        // Determine the position of the sub-functions (after all existing functions)
        let subfuncs_start = code_section.data.len();

        // Add sub-functions
        for sub_body in sub_func_bodies {
            code_section.data.extend_from_slice(&sub_body);
        }

        // Update the function count in the code section
        let mut updated_code_data = Vec::new();

        // Read the original function count
        let mut i = 0;
        let mut num_funcs = 0;
        let mut shift = 0;

        while i < code_section.data.len() {
            let byte = code_section.data[i];
            num_funcs |= ((byte & 0x7F) as usize) << shift;
            i += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        // Calculate the new function count
        let new_num_funcs = num_funcs + num_subfuncs;

        // Encode the new function count
        write_leb128(&mut updated_code_data, new_num_funcs as u32);

        // Add the remaining code section data
        updated_code_data.extend_from_slice(&code_section.data[i..]);

        // Update the code section
        code_section.data = updated_code_data;
    }

    debug!("Function {} splitting completed", func_idx);

    Ok(modified_module)
}

/// Encode an unsigned integer as LEB128 format
fn write_leb128(output: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            // If there are more bytes to write, set the highest bit
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}
