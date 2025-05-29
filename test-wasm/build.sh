#!/bin/bash

# Build script for test WASM module
# This script builds the test WASM and prepares it for encryption testing

set -e

echo "🦀 Building test WASM module..."

# Build the WASM module
wasm-pack build --target web --out-dir pkg

# Copy the generated WASM file to the web directory
cp pkg/test_wasm_bg.wasm ../web/test.wasm

echo "✅ Test WASM built successfully!"
echo "📁 Output: ../web/test.wasm"
echo ""
echo "Next steps:"
echo "1. Encrypt the WASM file using ruswacipher:"
echo "   cargo run -- encrypt -i web/test.wasm -o web/test.wasm.enc -a aes-gcm --generate-key web/test.key"
echo ""
echo "2. Open web/test.html in a browser and use the generated key to test decryption"
