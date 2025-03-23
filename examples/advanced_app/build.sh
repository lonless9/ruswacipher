#!/bin/bash

# Compile WAT files to WASM using wat2wasm
echo "Compiling WAT files to WASM..."
wat2wasm wat_files/advanced.wat -o advanced.wasm

# Ensure output file exists
if [ ! -f "advanced.wasm" ]; then
    echo "Error: WASM compilation failed!"
    exit 1
fi

# Encrypt WASM file using ruswacipher
echo "Encrypting WASM module..."
cd ../../
cargo run -- encrypt -i examples/advanced_app/advanced.wasm -o examples/advanced_app/advanced.encrypted.wasm -a aes-gcm

# Ensure key file exists and rename for consistency
if [ -f "examples/advanced_app/advanced.encrypted.wasm.key" ]; then
    mv examples/advanced_app/advanced.encrypted.wasm.key examples/advanced_app/advanced.wasm.key
fi

# Generate runtime and loader
echo "Generating Web files..."
cargo run -- generate-web -o examples/advanced_app/web -a aes-gcm

# Create metadata file for the key
echo "{\"keyAlgorithm\":\"aes-gcm\",\"keyType\":\"symmetric\",\"created\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"}" > examples/advanced_app/advanced.wasm.key.meta

echo "Build complete!"
echo "Key saved to examples/advanced_app/advanced.wasm.key"
echo "Encrypted WASM file is at examples/advanced_app/advanced.encrypted.wasm"
echo "Start an HTTP server and access examples/advanced_app/index.html to test the example!" 