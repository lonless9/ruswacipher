# RusWaCipher Advanced Example Application

> **IMPORTANT NOTICE**: This example is part of RusWaCipher, which is currently in early development stage. The functionality demonstrated in this example may be limited or subject to change.

This advanced example demonstrates how to use RusWaCipher to encrypt and protect more complex WebAssembly modules and run them securely in browsers.

## Project Files

- `advanced.rs` - Advanced Rust WASM example code with multiple functionalities
- `Cargo.toml` - Project configuration
- `index.html` - Interactive web demonstration page
- `build.sh` - Build script

After compilation, the following files will be generated:
- `advanced.wasm` - Original WASM file
- `advanced.encrypted.wasm` - Encrypted WASM file
- `advanced.wasm.key` - Encryption key file
- `advanced.wasm.key.meta` - Key metadata file
- `web/` - Directory containing runtime and loader

## Advanced Features

This example includes more advanced features compared to the simple example:

1. **ChaCha20-Poly1305 Encryption** - Uses a more modern encryption algorithm than AES-GCM
2. **Advanced Obfuscation** - Protects code with level 2 obfuscation
3. **Multi-function Page** - Tabbed interface containing multiple functionality demos

## Demonstrated Functions

### 1. Calculator
Advanced calculator supporting addition, subtraction, multiplication, and division.

### 2. Password Strength Detector
Evaluates password strength and provides improvement suggestions. The evaluation considers:
- Password length
- Presence of lowercase letters
- Presence of uppercase letters
- Presence of numbers
- Presence of special characters

### 3. Text Encryptor/Decryptor
Encrypts and decrypts text using a simple XOR encryption algorithm, demonstrating:
- String conversion between JavaScript and WASM
- Handling of complex data types
- Memory management

### 4. JSON Validator
Validates JSON structures and provides formatted output.

## Build Steps

Ensure you have the wasm32-unknown-unknown target installed:

```bash
rustup target add wasm32-unknown-unknown
```

Run the build script:

```bash
chmod +x build.sh
./build.sh
```

## Running the Example

Due to browser security restrictions, files need to be served through an HTTP server. You can use any HTTP server, for example:

```bash
# If you have Python
python -m http.server

# Or using Node.js http-server
npx http-server
```

Then access in your browser:
```
http://localhost:8000/examples/advanced_app/index.html
```

## Technical Features

This example showcases the following advanced techniques:

1. **Key Metadata** - Adds metadata information to keys, enhancing key management
2. **Memory Management and Cleanup** - Shows how to properly allocate and free memory in WebAssembly
3. **String Handling** - Demonstrates string conversion between JavaScript and Rust/WebAssembly
4. **Complex Interface** - Modern web application example with tabbed interface
5. **Dynamic Components** - Dynamically generates interface elements via JavaScript

## Security Considerations

This example uses the ChaCha20-Poly1305 algorithm for encryption, which is a modern, highly secure encryption algorithm. It has the following characteristics:

- Performs well on embedded and low-power devices
- Easier to implement with high-performance constant-time implementations compared to AES
- Provides high randomness and security

However, it's important to remember that, as with the simple example, browser-side decryption means the key must be available to the JavaScript runtime. For production environments, consider using more sophisticated key management strategies. 