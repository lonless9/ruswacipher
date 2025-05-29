use wasm_bindgen::prelude::*;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce, Key,
};

// Import console.log for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Set up panic hook for better error messages
#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

// Use wee_alloc as the global allocator for smaller WASM size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();
    
    console_log!("🔐 WASM Decryption Helper initialized");
}

/// Decrypt data using ChaCha20-Poly1305
/// 
/// # Arguments
/// * `key` - 32-byte encryption key
/// * `nonce` - 12-byte nonce
/// * `ciphertext` - Encrypted data
/// 
/// # Returns
/// Decrypted data as Uint8Array
#[wasm_bindgen]
pub fn decrypt_chacha20poly1305(
    key: &[u8],
    nonce: &[u8], 
    ciphertext: &[u8]
) -> Result<Vec<u8>, JsValue> {
    console_log!(
        "ChaCha20-Poly1305 decrypt: key_len={}, nonce_len={}, ciphertext_len={}", 
        key.len(), 
        nonce.len(), 
        ciphertext.len()
    );

    // Validate key length
    if key.len() != 32 {
        return Err(JsValue::from_str(&format!(
            "Invalid key length: expected 32 bytes, got {}", 
            key.len()
        )));
    }

    // Validate nonce length
    if nonce.len() != 12 {
        return Err(JsValue::from_str(&format!(
            "Invalid nonce length: expected 12 bytes, got {}", 
            nonce.len()
        )));
    }

    // Create cipher instance
    let key_array: &[u8; 32] = key.try_into()
        .map_err(|_| JsValue::from_str("Failed to convert key to array"))?;
    let cipher_key = Key::from_slice(key_array);
    let cipher = ChaCha20Poly1305::new(cipher_key);

    // Create nonce
    let nonce_array: &[u8; 12] = nonce.try_into()
        .map_err(|_| JsValue::from_str("Failed to convert nonce to array"))?;
    let cipher_nonce = Nonce::from_slice(nonce_array);

    // Decrypt
    match cipher.decrypt(cipher_nonce, ciphertext) {
        Ok(plaintext) => {
            console_log!("ChaCha20-Poly1305 decryption successful: {} bytes", plaintext.len());
            Ok(plaintext)
        }
        Err(e) => {
            let error_msg = format!("ChaCha20-Poly1305 decryption failed: {:?}", e);
            console_log!("{}", error_msg);
            Err(JsValue::from_str(&error_msg))
        }
    }
}

/// Encrypt data using ChaCha20-Poly1305 (for testing purposes)
/// 
/// # Arguments
/// * `key` - 32-byte encryption key
/// * `nonce` - 12-byte nonce
/// * `plaintext` - Data to encrypt
/// 
/// # Returns
/// Encrypted data as Uint8Array
#[wasm_bindgen]
pub fn encrypt_chacha20poly1305(
    key: &[u8],
    nonce: &[u8],
    plaintext: &[u8]
) -> Result<Vec<u8>, JsValue> {
    console_log!(
        "ChaCha20-Poly1305 encrypt: key_len={}, nonce_len={}, plaintext_len={}", 
        key.len(), 
        nonce.len(), 
        plaintext.len()
    );

    // Validate key length
    if key.len() != 32 {
        return Err(JsValue::from_str(&format!(
            "Invalid key length: expected 32 bytes, got {}", 
            key.len()
        )));
    }

    // Validate nonce length
    if nonce.len() != 12 {
        return Err(JsValue::from_str(&format!(
            "Invalid nonce length: expected 12 bytes, got {}", 
            nonce.len()
        )));
    }

    // Create cipher instance
    let key_array: &[u8; 32] = key.try_into()
        .map_err(|_| JsValue::from_str("Failed to convert key to array"))?;
    let cipher_key = Key::from_slice(key_array);
    let cipher = ChaCha20Poly1305::new(cipher_key);

    // Create nonce
    let nonce_array: &[u8; 12] = nonce.try_into()
        .map_err(|_| JsValue::from_str("Failed to convert nonce to array"))?;
    let cipher_nonce = Nonce::from_slice(nonce_array);

    // Encrypt
    match cipher.encrypt(cipher_nonce, plaintext) {
        Ok(ciphertext) => {
            console_log!("ChaCha20-Poly1305 encryption successful: {} bytes", ciphertext.len());
            Ok(ciphertext)
        }
        Err(e) => {
            let error_msg = format!("ChaCha20-Poly1305 encryption failed: {:?}", e);
            console_log!("{}", error_msg);
            Err(JsValue::from_str(&error_msg))
        }
    }
}

/// Get information about the WASM decryption helper
#[wasm_bindgen]
pub fn get_helper_info() -> JsValue {
    let info = serde_json::json!({
        "name": "wasm-decryptor-helper",
        "version": "0.1.0",
        "supported_algorithms": ["chacha20poly1305"],
        "key_length": 32,
        "nonce_length": 12
    });
    
    JsValue::from_str(&info.to_string())
}

/// Test function to verify the helper is working
#[wasm_bindgen]
pub fn test_helper() -> bool {
    console_log!("Testing WASM decryption helper...");
    
    // Test data
    let key = [0u8; 32];
    let nonce = [0u8; 12];
    let plaintext = b"Hello, WASM decryption helper!";
    
    // Encrypt then decrypt
    match encrypt_chacha20poly1305(&key, &nonce, plaintext) {
        Ok(ciphertext) => {
            match decrypt_chacha20poly1305(&key, &nonce, &ciphertext) {
                Ok(decrypted) => {
                    let success = decrypted == plaintext;
                    console_log!("Helper test result: {}", if success { "PASS" } else { "FAIL" });
                    success
                }
                Err(_) => {
                    console_log!("Helper test result: FAIL (decryption error)");
                    false
                }
            }
        }
        Err(_) => {
            console_log!("Helper test result: FAIL (encryption error)");
            false
        }
    }
}
