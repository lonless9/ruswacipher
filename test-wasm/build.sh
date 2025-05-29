#!/bin/bash

# Build script for test WASM module
# This script builds the test WASM and prepares it for encryption testing

set -e

echo "🦀 Building test WASM module..."

# Check if wasm-bindgen is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "❌ wasm-bindgen-cli is required but not installed."
    echo "Install with: cargo install wasm-bindgen-cli"
    exit 1
fi

# Build the WASM module using cargo
cargo build --target wasm32-unknown-unknown --release

# Create output directory
mkdir -p pkg

# Generate bindings using wasm-bindgen
wasm-bindgen target/wasm32-unknown-unknown/release/test_wasm.wasm \
    --out-dir pkg \
    --target web \
    --no-typescript

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
