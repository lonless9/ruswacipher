#!/bin/bash

# Build script for WASM decryption helper
# This script builds the WASM helper module for ChaCha20-Poly1305 decryption

set -e

echo "🔐 Building WASM Decryption Helper..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is required but not installed."
    echo "Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Build the WASM module
echo "📦 Building WASM module..."
wasm-pack build --target web --out-dir pkg --release

# Copy files to web directory
echo "📁 Copying files to web directory..."
cp pkg/wasm_decryptor_helper.js ../web/wasm-decryptor-helper.js
cp pkg/wasm_decryptor_helper_bg.wasm ../web/wasm-decryptor-helper.wasm

# Create a simple wrapper for easier importing
cat > ../web/wasm-decryptor-helper-wrapper.js << 'EOF'
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
EOF

echo "✅ WASM Decryption Helper built successfully!"
echo "📁 Files created:"
echo "   - ../web/wasm-decryptor-helper.js"
echo "   - ../web/wasm-decryptor-helper.wasm"
echo "   - ../web/wasm-decryptor-helper-wrapper.js"
echo ""
echo "🧪 To test the helper, run the web runtime test script:"
echo "   ./scripts/test-web-runtime.sh"
