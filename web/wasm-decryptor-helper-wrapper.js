/**
 * WASM Decryption Helper Wrapper
 * Provides a simplified interface for loading and using the WASM decryption helper
 */

import init, {
    decrypt_chacha20poly1305,
    encrypt_chacha20poly1305,
    get_helper_info,
    test_helper
} from './wasm-decryptor-helper.js';

let isInitialized = false;

/**
 * Initialize the WASM helper module
 */
export async function initWasmHelper() {
    if (!isInitialized) {
        await init();
        isInitialized = true;
        console.log('🔐 WASM Decryption Helper loaded successfully');

        // Run self-test
        const testResult = test_helper();
        if (!testResult) {
            throw new Error('WASM helper self-test failed');
        }
    }
}

/**
 * Decrypt data using ChaCha20-Poly1305
 */
export async function decryptChaCha20Poly1305(key, nonce, ciphertext) {
    if (!isInitialized) {
        await initWasmHelper();
    }

    return decrypt_chacha20poly1305(key, nonce, ciphertext);
}

/**
 * Encrypt data using ChaCha20-Poly1305 (for testing)
 */
export async function encryptChaCha20Poly1305(key, nonce, plaintext) {
    if (!isInitialized) {
        await initWasmHelper();
    }

    return encrypt_chacha20poly1305(key, nonce, plaintext);
}

/**
 * Get helper information
 */
export async function getHelperInfo() {
    if (!isInitialized) {
        await initWasmHelper();
    }

    return JSON.parse(get_helper_info());
}

/**
 * Check if helper is initialized
 */
export function isHelperInitialized() {
    return isInitialized;
}
