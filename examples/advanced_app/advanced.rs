use std::ffi::{c_char, CStr, CString};
use std::mem;
use std::os::raw::c_void;

// Simple arithmetic functions
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[no_mangle]
pub extern "C" fn divide(a: i32, b: i32) -> f32 {
    if b == 0 {
        return f32::NAN; // Return NaN for division by zero
    }
    a as f32 / b as f32
}

// Memory management functions
#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut c_void {
    let mut buffer = Vec::with_capacity(size);
    let ptr = buffer.as_mut_ptr();
    mem::forget(buffer);
    ptr as *mut c_void
}

#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut c_void, capacity: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr as *mut u8, 0, capacity);
    }
}

// String handling functions
#[no_mangle]
pub extern "C" fn say_hello(name_ptr: *const c_char) -> *const c_char {
    let name = unsafe {
        if name_ptr.is_null() {
            return CString::new("Name cannot be empty").unwrap().into_raw();
        }
        
        CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
    };
    
    let result = format!("Hello, {}!", name);
    let c_str = CString::new(result).unwrap();
    c_str.into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            let _ = CString::from_raw(ptr);
        }
    }
}

// JSON parsing function (simple)
#[no_mangle]
pub extern "C" fn parse_json(json_ptr: *const c_char) -> *const c_char {
    let json_str = unsafe {
        if json_ptr.is_null() {
            return CString::new("JSON string cannot be empty").unwrap().into_raw();
        }
        
        CStr::from_ptr(json_ptr).to_string_lossy().into_owned()
    };
    
    // Simple JSON validation (just checks for matching braces)
    let mut brace_count = 0;
    for c in json_str.chars() {
        if c == '{' {
            brace_count += 1;
        } else if c == '}' {
            brace_count -= 1;
            if brace_count < 0 {
                return CString::new("Invalid JSON: unmatched closing brace").unwrap().into_raw();
            }
        }
    }
    
    if brace_count != 0 {
        return CString::new("Invalid JSON: missing closing brace").unwrap().into_raw();
    }
    
    // In a real app, we would do actual JSON parsing here
    let result = format!("Valid JSON with {} object(s)", brace_count);
    let c_str = CString::new(result).unwrap();
    c_str.into_raw()
}

// Password strength checker
#[no_mangle]
pub extern "C" fn check_password_strength(password_ptr: *const c_char) -> i32 {
    let password = unsafe {
        if password_ptr.is_null() {
            return 0; // Very weak
        }
        
        CStr::from_ptr(password_ptr).to_string_lossy().into_owned()
    };
    
    if password.len() < 6 {
        return 0; // Very weak
    }
    
    let mut score = 0;
    
    // Length check
    if password.len() >= 8 {
        score += 1;
    }
    if password.len() >= 12 {
        score += 1;
    }
    
    // Complexity checks
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digits = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    
    if has_lowercase { score += 1; }
    if has_uppercase { score += 1; }
    if has_digits { score += 1; }
    if has_special { score += 1; }
    
    score
}

// Simple encryption (XOR with a key)
#[no_mangle]
pub extern "C" fn encrypt_string(
    text_ptr: *const c_char, 
    key_ptr: *const c_char
) -> *const c_char {
    let text = unsafe {
        if text_ptr.is_null() {
            return CString::new("Text cannot be empty").unwrap().into_raw();
        }
        
        CStr::from_ptr(text_ptr).to_string_lossy().into_owned()
    };
    
    let key = unsafe {
        if key_ptr.is_null() {
            return CString::new("Key cannot be empty").unwrap().into_raw();
        }
        
        CStr::from_ptr(key_ptr).to_string_lossy().into_owned()
    };
    
    if key.is_empty() {
        return CString::new("Key must not be empty").unwrap().into_raw();
    }
    
    // Simple XOR encryption
    let key_bytes: Vec<u8> = key.bytes().collect();
    let encrypted: String = text
        .bytes()
        .enumerate()
        .map(|(i, byte)| {
            let key_byte = key_bytes[i % key_bytes.len()];
            (byte ^ key_byte) as char
        })
        .collect();
    
    // Convert to base64 for safer transfer
    let base64_encrypted = base64_encode(&encrypted.as_bytes());
    let c_str = CString::new(base64_encrypted).unwrap();
    c_str.into_raw()
}

#[no_mangle]
pub extern "C" fn decrypt_string(
    encrypted_ptr: *const c_char, 
    key_ptr: *const c_char
) -> *const c_char {
    let encrypted_base64 = unsafe {
        if encrypted_ptr.is_null() {
            return CString::new("Encrypted text cannot be empty").unwrap().into_raw();
        }
        
        CStr::from_ptr(encrypted_ptr).to_string_lossy().into_owned()
    };
    
    let key = unsafe {
        if key_ptr.is_null() {
            return CString::new("Key cannot be empty").unwrap().into_raw();
        }
        
        CStr::from_ptr(key_ptr).to_string_lossy().into_owned()
    };
    
    // Decode base64
    let encrypted_result = base64_decode(&encrypted_base64);
    let encrypted = match encrypted_result {
        Ok(data) => data,
        Err(_) => {
            return CString::new("Invalid base64 data").unwrap().into_raw();
        }
    };
    
    if key.is_empty() {
        return CString::new("Key must not be empty").unwrap().into_raw();
    }
    
    // Simple XOR decryption
    let key_bytes: Vec<u8> = key.bytes().collect();
    let decrypted: String = encrypted
        .iter()
        .enumerate()
        .map(|(i, &byte)| {
            let key_byte = key_bytes[i % key_bytes.len()];
            (byte ^ key_byte) as char
        })
        .collect();
    
    let c_str = CString::new(decrypted).unwrap();
    c_str.into_raw()
}

// Simple base64 encoding and decoding
fn base64_encode(data: &[u8]) -> String {
    static BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = String::new();
    let mut i = 0;
    
    while i + 2 < data.len() {
        let chunk = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        
        result.push(BASE64_CHARS[((chunk >> 18) & 63) as usize] as char);
        result.push(BASE64_CHARS[((chunk >> 12) & 63) as usize] as char);
        result.push(BASE64_CHARS[((chunk >> 6) & 63) as usize] as char);
        result.push(BASE64_CHARS[(chunk & 63) as usize] as char);
        
        i += 3;
    }
    
    if i + 1 == data.len() {
        let chunk = (data[i] as u32) << 16;
        result.push(BASE64_CHARS[((chunk >> 18) & 63) as usize] as char);
        result.push(BASE64_CHARS[((chunk >> 12) & 63) as usize] as char);
        result.push('=');
        result.push('=');
    } else if i + 2 == data.len() {
        let chunk = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        result.push(BASE64_CHARS[((chunk >> 18) & 63) as usize] as char);
        result.push(BASE64_CHARS[((chunk >> 12) & 63) as usize] as char);
        result.push(BASE64_CHARS[((chunk >> 6) & 63) as usize] as char);
        result.push('=');
    }
    
    result
}

fn base64_decode(encoded: &str) -> Result<Vec<u8>, &'static str> {
    if encoded.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;
    
    for c in encoded.bytes() {
        let val = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break, // End of data
            _ => return Err("Invalid base64 character"),
        };
        
        buffer = (buffer << 6) | (val as u32);
        bits += 6;
        
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    
    Ok(result)
} 