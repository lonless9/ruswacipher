# 🔐 RusWaCipher - Rust WebAssembly Encryption Tool

> **⚠️ IMPORTANT NOTICE**: RusWaCipher is currently in early development stage. The API and functionality are subject to change.

[![CI](https://github.com/lonless9/ruswacipher/actions/workflows/ci.yml/badge.svg)](https://github.com/lonless9/ruswacipher/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

A Rust-based tool for encrypting and protecting WebAssembly (WASM) modules, preventing code from being easily analyzed or reverse-engineered.

## 🎯 Project Goals

- 🔒 Provide encryption functionality for WASM modules
- 🌐 Implement browser-side decryption runtime
- 🛡️ Protect WebAssembly code intellectual property
- 🔑 Support multiple encryption algorithms

## 🧩 Core Components

- 📊 **WASM Analyzer**: Parse and modify WASM binary structures
- 🔐 **Encryption Engine**: Support for AES-GCM and ChaCha20-Poly1305
- 🔄 **Decryption Runtime**: Browser-side JavaScript decryption library
- 💻 **CLI Tools**: Command-line interface for encryption operations
- 🧰 **Plugin System**: Support for custom encryption algorithms

## 🛠️ Technology Stack

- 🦀 **Language**: Rust 2021 Edition
- 📦 **WASM Parsing**: `wasmparser`
- 🔒 **Encryption**: `aes-gcm`, `chacha20poly1305`
- ⌨️ **CLI**: `clap`

## 📊 Project Status

RusWaCipher is an early-stage project with many features under active development:

- 🟢 **Core Encryption**: Basic encryption/decryption functionality
- 🟡 **WASM Processing**: Binary parsing and modification
- 🟡 **Code Obfuscation**: Control flow obfuscation, dead code insertion
- 🟡 **JavaScript Runtime**: Browser-side decryption
- 🟡 **Plugin System**: Custom algorithm integration

## 📝 Basic Usage

### 📥 Installation

```bash
cargo install --path .
```

### 🔒 Encrypting WASM Modules

```bash
ruswacipher encrypt -i input.wasm -o encrypted.wasm -a aes-gcm -b
```

### 🔓 Decrypting WASM Modules

```bash
ruswacipher decrypt -i encrypted.wasm -o decrypted.wasm -k key_file.key
```

### 🌐 Generating JavaScript Runtime

```bash
ruswacipher generate-web -o web_dir -a aes-gcm
```

## 🔐 Security Considerations

- ⚠️ Browser-side decryption requires the key to be available in JavaScript
- 🛡️ For production, consider server-side key management
- 🔒 Use this tool as one layer in a defense-in-depth strategy

## 📚 Documentation

Plugin development documentation and advanced features will be available soon.

## 🔄 Continuous Integration

This project uses GitHub Actions for continuous integration and delivery:

- 🧪 **CI Workflow**: Automatically runs tests, linting, and builds for every push and pull request
- 🛡️ **Security Audit**: Regularly scans dependencies for security vulnerabilities
- 📦 **Release Workflow**: Automates the creation of cross-platform releases when a new version is tagged

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 