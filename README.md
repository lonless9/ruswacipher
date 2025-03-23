# RusWaCipher - Rust WebAssembly Encryption Tool

RusWaCipher is a Rust-based tool for encrypting and protecting WebAssembly (WASM) modules, preventing code from being easily analyzed or reverse-engineered.

## Project Goals

- Provide encryption functionality for WASM modules
- Implement browser-side decryption runtime to allow encrypted WASM modules to run securely in browsers
- Protect WebAssembly code intellectual property
- Support multiple encryption algorithms and security levels

## Project Architecture

### Core Components

#### 1. WASM Analyzer
- Function: Read and parse WASM binary file structures
- Modules: `src/wasm/parser.rs`, `src/wasm/structure.rs`
- Technology: Low-level binary parsing to understand WASM module formats

#### 2. Encryption Engine
- Function: Encrypt WASM binary data
- Modules: `src/crypto/engine.rs`, `src/crypto/algorithms.rs`
- Supported Algorithms: AES-GCM, ChaCha20-Poly1305, etc.

#### 3. Decryption Runtime
- Function: Decrypt and execute WASM modules in browser environments
- Modules: `src/runtime/js_runtime.rs`, `web/runtime.js`
- Technology: Generates specialized JavaScript decryption libraries

#### 4. CLI Tools
- Function: Command-line interface for integration into build processes
- Modules: `src/main.rs`, `src/cli/`
- Features: Batch processing, configuration files, multiple options

### File Structure

```
ruswacipher/
├── src/
│   ├── main.rs             // CLI entry point
│   ├── cli/
│   │   ├── mod.rs          // CLI module declaration
│   │   ├── commands.rs     // Command definitions
│   │   └── config.rs       // Configuration processing
│   ├── wasm/
│   │   ├── mod.rs          // WASM module declaration
│   │   ├── parser.rs       // WASM parser
│   │   └── structure.rs    // WASM file structure definitions
│   ├── crypto/
│   │   ├── mod.rs          // Encryption module declaration
│   │   ├── engine.rs       // Encryption engine
│   │   └── algorithms.rs   // Encryption algorithm implementations
│   ├── obfuscation/
│   │   ├── mod.rs          // Obfuscation module declaration
│   │   └── techniques.rs   // Obfuscation techniques implementations
│   └── runtime/
│       ├── mod.rs          // Runtime module declaration
│       └── js_runtime.rs   // JavaScript decryption runtime generator
├── web/
│   ├── ruswacipher-runtime.js // Browser decryption runtime
│   ├── loader.js           // WASM loader
│   └── example.html        // Example HTML page
├── benches/
│   └── encryption_benchmark.rs // Encryption performance benchmarks
├── tests/
│   ├── integration_tests.rs // Integration tests
│   └── samples/            // Test samples
└── examples/
    ├── simple_app/         // Simple application example
    └── advanced_app/       // Advanced application example (planned)
```

## Technology Stack

- **Language**: Rust 2021 Edition
- **WASM Parsing**: Custom parser + `wasmparser`
- **Encryption**: `aes-gcm`, `chacha20poly1305`, `rand`
- **CLI**: `clap`
- **Serialization**: `serde`, `serde_json`
- **Testing**: Rust built-in testing framework
- **Documentation**: Rustdoc

## Project Progress

### Completed Features

✅ **WASM Parsing and Processing**
- Fully implemented WebAssembly binary parser
- Support for processing all standard WASM section types
- Implemented functionality for adding and removing custom sections

✅ **Encryption System**
- Implemented AES-GCM and ChaCha20-Poly1305 encryption algorithms
- Support for random key generation and management
- Encryption performance benchmarks show excellent processing capabilities

✅ **JavaScript Decryption Runtime**
- Generated specialized decryption code for each supported encryption algorithm
- Provided secure browser-side decryption functionality
- Compatible with all major browsers

✅ **Command Line Interface**
- Complete CLI tool supporting encryption, decryption, and generation operations
- Support for flexible parameter configuration
- Provides user-friendly feedback

✅ **Code Obfuscation**
- Implemented multiple code obfuscation techniques
- Support for different obfuscation levels
- Configurable obfuscation options

✅ **Plugin System**
- Custom encryption algorithm plugin support
- Dynamic loading of external encryption modules
- Extensible architecture for third-party contributions

✅ **Example Applications**
- Complete example WASM applications
- Detailed build and usage guides
- Demonstrations of encrypted WASM running in browsers

### Implementation Roadmap

#### Phase 1: Basic Infrastructure (Completed ✓)
- [x] Project initialization
- [x] Basic WASM parser implementation
- [x] Simple encryption/decryption functionality
- [x] Basic CLI framework

#### Phase 2: Core Functionality (Completed ✓)
- [x] Complete WASM parsing and modification
- [x] Multiple encryption algorithm support
- [x] JavaScript decryption runtime
- [x] Basic example applications

#### Phase 3: Advanced Features (In Progress 🔄)
- [x] Code obfuscation options
- [x] Performance optimization
- [x] Custom encryption plugin system
- [ ] Integration with build tools (such as webpack)

#### Phase 4: Testing and Refinement (In Progress 🔄)
- [x] Unit tests and integration tests
- [x] Performance benchmarks
- [ ] Documentation refinement
- [ ] Error handling and logging system

### Next Steps

1. **Build Tool Integration** - Develop webpack plugin to simplify usage in frontend projects
2. **Documentation Refinement** - Write detailed API documentation and user guides
3. **Advanced Example Applications** - Create more complex example applications demonstrating real-world use cases
4. **Error Handling Improvements** - Enhance error reporting and recovery mechanisms

## Installation

```bash
cargo install --path .
```

## Usage

### Encrypting WASM Modules

```bash
ruswacipher encrypt -i input.wasm -o encrypted.wasm -a aes-gcm -b --obfuscation-level 1
```

Parameter Explanation:
- `-i, --input`: Input WASM file path
- `-o, --output`: Output encrypted file path
- `-k, --key-file`: (Optional) Key file path, generates a new key if not provided
- `-a, --algorithm`: (Optional) Encryption algorithm, supports "aes-gcm" or "chacha20poly1305", default "aes-gcm"
- `-b, --obfuscate`: (Optional) Enable code obfuscation
- `--obfuscation-level`: (Optional) Obfuscation level (1-3, default is 1)

### Decrypting WASM Modules

```bash
ruswacipher decrypt -i encrypted.wasm -o decrypted.wasm -k key_file.key
```

### Generating JavaScript Runtime

```bash
ruswacipher generate-runtime -o runtime.js -a aes-gcm
```

### Generating Web Files (Runtime, Loader, and Example HTML)

```bash
ruswacipher generate-web -o web_dir -a aes-gcm
```

### Using With Plugins

```bash
ruswacipher encrypt -i input.wasm -o encrypted.wasm -a custom-algorithm --plugin-path /path/to/plugins
```

### Using in Web Projects

Include the loader and runtime in your HTML:

```html
<script src="loader.js"></script>
<script>
  // Create loader
  const loader = new WasmLoader();
  
  // Load encrypted WASM module
  loader.load('encrypted.wasm', 'your-key-in-base64')
    .then(instance => {
      // Use the decrypted WASM module
      const result = instance.exports.your_function();
      console.log('Result:', result);
    })
    .catch(error => {
      console.error('Loading failed:', error);
    });
</script>
```

## Example Applications

See the `examples/simple_app` directory for a complete example, including:
- Simple WASM module code
- Complete example of encrypting and using encrypted modules
- Build and run instructions

## Benchmarks

Run encryption performance tests with:

```bash
cargo bench --bench encryption_benchmark
```

## Plugin Development

For information on developing custom encryption plugins, see [Plugin Development Guide](docs/plugin_guide.md).

## Security Considerations

RusWaCipher provides protection for WebAssembly modules primarily through obfuscation and encryption. When using this tool, consider the following security aspects:

### Browser-Side Decryption Limitations

- The decryption happens in the browser, which means:
  - The encryption key must be available to the JavaScript runtime
  - Once decrypted, the WebAssembly module can be inspected in browser developer tools
  - Encryption provides a significant barrier to casual inspection but is not unbreakable

### Key Management

- For demonstration purposes, examples embed the key directly in JavaScript
- For production environments, consider:
  - Server-side key management using HSMs or key management services
  - API-based temporary key distribution with proper authentication
  - Separating critical logic between server and client components

### Recommended Security Architecture

For applications requiring higher security:
1. Use RusWaCipher as one layer in a defense-in-depth strategy
2. Keep highly sensitive algorithms on secured servers
3. Use the encrypted WASM for less sensitive operations
4. Implement proper authentication and authorization for your application
5. Consider using secure channels (HTTPS) for all communications

For more detailed security guidance, see the sections on key management in our [Plugin Development Guide](docs/plugin_guide.md) and the [Simple App Example](examples/simple_app/README.md).

## Contribution Guidelines

Pull Requests and Issues are welcome! Please ensure you follow these steps:

1. Fork the project
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details 