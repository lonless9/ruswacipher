use crate::wasm::structure::{Section, SectionType, WasmModule};
use anyhow::Result;
use log::{debug, info};
use rand::Rng;

/// Create and return an exact copy of the module
fn clone_module(module: &WasmModule) -> WasmModule {
    let mut new_module = WasmModule {
        version: module.version,
        sections: Vec::new(),
    };

    // Copy all sections
    for section in &module.sections {
        let new_section = Section {
            section_type: section.section_type,
            name: section.name.clone(),
            data: section.data.clone(),
        };
        new_module.sections.push(new_section);
    }

    new_module
}

/// Find the start and end positions of function bodies in WebAssembly binary code
fn find_function_bodies(code_data: &[u8]) -> Vec<(usize, usize)> {
    let mut functions = Vec::new();
    let mut i = 0;

    // The code section format: a variable-length integer representing the number of functions, followed by a list of function bodies
    // First read the number of functions
    let mut count = 0;
    let mut shift = 0;
    while i < code_data.len() {
        let byte = code_data[i];
        count |= ((byte & 0x7F) as usize) << shift;
        i += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    debug!("Found {} function bodies", count);

    // Parse each function body
    for _ in 0..count {
        if i >= code_data.len() {
            break;
        }

        // Read the function body size
        let mut size = 0;
        let mut shift = 0;
        let start_pos = i;

        while i < code_data.len() {
            let byte = code_data[i];
            size |= ((byte & 0x7F) as usize) << shift;
            i += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        // Add the function body range (including the size field)
        let func_start = start_pos;
        let func_end = i + size;

        if func_end <= code_data.len() {
            functions.push((func_start, func_end));
            i = func_end; // Move to the next function body
        } else {
            break; // Prevent out of bounds
        }
    }

    functions
}

#[allow(dead_code)]
/// Generate a sequence of dead code instructions - used for adding dead code
fn generate_dead_code(_rng: &mut impl Rng) -> Vec<u8> {
    // Create a code block that will never be executed
    // 1. Create an if block with a condition that is always false
    // 2. Place some instructions inside the block
    // 3. End the block

    // Create an if block with a condition that is always false
    // i32.const 0 (push 0 onto stack), if block start (type: void),
    // Simple nop instructions that do not affect the stack, end the if block
    let dead_code = vec![
        0x41, 0x00, // i32.const 0 (push 0 onto stack)
        0x04, 0x40, // if block start (type: void)
        0x01, 0x01, 0x01, // nop instructions
        0x0B, // End the if block
    ];

    dead_code
}

/// Modify the function body to make its control flow more complex
fn obfuscate_function_body(function_data: &[u8], _rng: &mut impl Rng) -> Vec<u8> {
    // Initialize the result
    let mut result = Vec::new();
    let mut i = 0;

    // Read the function body size
    let mut _size = 0;
    let mut shift = 0;

    while i < function_data.len() {
        let byte = function_data[i];
        _size |= ((byte & 0x7F) as usize) << shift;
        i += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    // Copy the local variable declaration part of the original function body
    let header_start = i;
    let mut header_end = i;

    // Just skip the local variable part, no need to parse it in detail
    if i < function_data.len() {
        // Read the number of local variable groups
        let mut local_count = 0;
        let mut shift = 0;

        while header_end < function_data.len() {
            let byte = function_data[header_end];
            local_count |= ((byte & 0x7F) as usize) << shift;
            header_end += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        // Skip the local variable groups
        for _ in 0..local_count {
            if header_end >= function_data.len() {
                break;
            }

            // Read the number of variables in the group
            let mut _count = 0;
            let mut shift = 0;

            while header_end < function_data.len() {
                let byte = function_data[header_end];
                _count |= ((byte & 0x7F) as usize) << shift;
                header_end += 1;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }

            // Skip the type byte
            if header_end < function_data.len() {
                header_end += 1;
            }
        }
    }

    // Create a new function body (excluding the size prefix)
    let mut new_body = Vec::new();

    // Copy the local variable declaration part
    new_body.extend_from_slice(&function_data[header_start..header_end]);

    // Add the original instruction sequence (skipping the variable declaration part)
    new_body.extend_from_slice(&function_data[header_end..]);

    // Calculate the new function body size
    let new_size = new_body.len();

    // Encode the new size
    let mut encoded_size = Vec::new();
    let mut value = new_size;

    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        encoded_size.push(byte);
        if value == 0 {
            break;
        }
    }

    // Build the final result: size + function body
    result.extend_from_slice(&encoded_size);
    result.extend_from_slice(&new_body);

    result
}

/// Add dead code to the code
pub fn add_dead_code(module: WasmModule) -> Result<WasmModule> {
    debug!("添加死代码混淆");

    // Create a module copy
    let mut modified_module = clone_module(&module);
    let _rng = rand::rng();

    // Find the code section
    for section in &mut modified_module.sections {
        if section.section_type == SectionType::Code {
            debug!("Found code section, size: {} bytes", section.data.len());

            // Parse the function bodies
            let function_bodies = find_function_bodies(&section.data);
            debug!("Found {} function bodies", function_bodies.len());

            // If there are no function bodies, skip
            if function_bodies.is_empty() {
                continue;
            }

            // Create a new code section
            let mut new_code_data = Vec::new();

            // Keep the number of functions unchanged
            new_code_data.extend_from_slice(&section.data[0..function_bodies[0].0]);

            // Process each function body
            for (start, end) in function_bodies {
                let function_data = &section.data[start..end];

                // Create a modified function body
                let mut modified_function = Vec::new();

                // Read the function body size
                let mut i = 0;
                let mut _size = 0;
                let mut shift = 0;

                while i < function_data.len() {
                    let byte = function_data[i];
                    _size |= ((byte & 0x7F) as usize) << shift;
                    i += 1;
                    if byte & 0x80 == 0 {
                        break;
                    }
                    shift += 7;
                }

                // Copy the original function body, but insert dead code at the beginning of the function body
                let header_start = i;
                let mut header_end = i;

                // Simple skip the local variable part, no need to parse it in detail
                if i < function_data.len() {
                    let mut local_count = 0;
                    let mut shift = 0;

                    // Read the number of local variable groups
                    while header_end < function_data.len() {
                        let byte = function_data[header_end];
                        local_count |= ((byte & 0x7F) as usize) << shift;
                        header_end += 1;
                        if byte & 0x80 == 0 {
                            break;
                        }
                        shift += 7;
                    }

                    // Skip the local variable groups
                    for _ in 0..local_count {
                        if header_end >= function_data.len() {
                            break;
                        }

                        // Read the number of variables in the group
                        let mut _count = 0;
                        let mut shift = 0;

                        while header_end < function_data.len() {
                            let byte = function_data[header_end];
                            _count |= ((byte & 0x7F) as usize) << shift;
                            header_end += 1;
                            if byte & 0x80 == 0 {
                                break;
                            }
                            shift += 7;
                        }

                        // Skip the type byte
                        if header_end < function_data.len() {
                            header_end += 1;
                        }
                    }
                }

                // Create a new function body (excluding the size prefix)
                let mut new_function_body = Vec::new();

                // Copy the local variable declaration part
                new_function_body.extend_from_slice(&function_data[header_start..header_end]);

                // Add a nop sequence at the beginning of the function body as dead code
                // A simple nop sequence does not affect program execution, just increases code size
                new_function_body.extend_from_slice(&[0x01; 5]); // 5 nop instructions

                // Add the original instruction sequence
                new_function_body.extend_from_slice(&function_data[header_end..]);

                // Calculate the new function body size
                let new_size = new_function_body.len();

                // Encode the new size
                let mut encoded_size = Vec::new();
                let mut value = new_size;

                loop {
                    let mut byte = (value & 0x7F) as u8;
                    value >>= 7;
                    if value != 0 {
                        byte |= 0x80;
                    }
                    encoded_size.push(byte);
                    if value == 0 {
                        break;
                    }
                }

                // Assemble the new function: size + function body
                modified_function.extend_from_slice(&encoded_size);
                modified_function.extend_from_slice(&new_function_body);

                // Add the modified function body to the new code section
                new_code_data.extend_from_slice(&modified_function);
            }

            // Update the code section data
            section.data = new_code_data;

            break;
        }
    }

    info!("Dead code addition completed");
    Ok(modified_module)
}

/// Obfuscate the control flow
pub fn obfuscate_control_flow(module: WasmModule) -> Result<WasmModule> {
    debug!("Obfuscating control flow");

    // Create a module copy
    let mut modified_module = clone_module(&module);
    let mut rng = rand::rng();

    // Find the code section
    for section in &mut modified_module.sections {
        if section.section_type == SectionType::Code {
            debug!("Found code section, size: {} bytes", section.data.len());

            // Parse the function bodies
            let function_bodies = find_function_bodies(&section.data);
            debug!("Found {} function bodies", function_bodies.len());

            // If there are no function bodies, skip
            if function_bodies.is_empty() {
                continue;
            }

            // Create a new code section
            let mut new_code_data = Vec::new();

            // Keep the number of functions unchanged
            new_code_data.extend_from_slice(&section.data[0..function_bodies[0].0]);

            // Process each function body
            for (start, end) in function_bodies {
                let function_data = &section.data[start..end];

                // Obfuscate the control flow of the function body
                let new_function_data = obfuscate_function_body(function_data, &mut rng);

                // Add the modified function body to the new code section
                new_code_data.extend_from_slice(&new_function_data);
            }

            // Update the code section data
            section.data = new_code_data;

            break;
        }
    }

    info!("Function control flow obfuscation completed");
    Ok(modified_module)
}
