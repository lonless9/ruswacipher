# RusWaCipher User Guide

## Table of Contents

1. [Installation](#installation)
2. [CLI Tool Usage](#cli-tool-usage)
3. [JavaScript Runtime Integration](#javascript-runtime-integration)
4. [Security Best Practices](#security-best-practices)
5. [Troubleshooting](#troubleshooting)

## Installation

### Prerequisites

- Rust 1.70+ (for building from source)
- Node.js 16+ (for web runtime development)
- Modern web browser with WebAssembly support

### Installing the CLI Tool

#### From Source

```bash
git clone https://github.com/lonless9/ruswacipher.git
cd ruswacipher
cargo install --path .
```

#### Pre-built Binaries

Download the latest release from [GitHub Releases](https://github.com/lonless9/ruswacipher/releases).

### Installing the Web Runtime

#### Via NPM (Coming Soon)

```bash
npm install ruswacipher-web-runtime
```

#### Manual Installation

1. Download `wasmGuardianLoader.js` from the repository
2. Include it in your web project
3. Optionally include the WASM decryption helper for ChaCha20-Poly1305 support

## CLI Tool Usage

### Basic Commands

#### Encrypting WASM Files

```bash
# Encrypt with AES-GCM (default algorithm)
ruswacipher encrypt -i input.wasm -o encrypted.wasm --generate-key key.txt

# Encrypt with ChaCha20-Poly1305
ruswacipher encrypt -i input.wasm -o encrypted.wasm -a chacha20poly1305 --generate-key key.txt

# Encrypt with existing key
ruswacipher encrypt -i input.wasm -o encrypted.wasm -k existing-key.txt

# Encrypt with hex key directly
ruswacipher encrypt -i input.wasm -o encrypted.wasm --key-hex 0123456789abcdef...
```

#### Decrypting WASM Files

```bash
# Decrypt encrypted WASM file
ruswacipher decrypt -i encrypted.wasm -o decrypted.wasm -k key.txt
```

### Command Line Options

#### Encrypt Command

- `-i, --input <FILE>`: Input WASM file path
- `-o, --output <FILE>`: Output encrypted file path
- `-a, --algorithm <ALGORITHM>`: Encryption algorithm (`aes-gcm` or `chacha20poly1305`)
- `-k, --key <FILE>`: Key file path
- `--key-hex <HEX>`: Key in hexadecimal format
- `--key-base64 <BASE64>`: Key in Base64 format
- `--generate-key <FILE>`: Generate new key and save to file
- `--key-format <FORMAT>`: Key output format (`hex`, `base64`, `raw`)

#### Decrypt Command

- `-i, --input <FILE>`: Input encrypted file path
- `-o, --output <FILE>`: Output decrypted WASM file path
- `-k, --key <FILE>`: Key file path

#### Global Options

- `-v, --verbose`: Enable verbose logging
- `-h, --help`: Show help information
- `-V, --version`: Show version information

### Examples

#### Complete Workflow

```bash
# 1. Encrypt a WASM module
ruswacipher encrypt \
    -i my-module.wasm \
    -o my-module.wasm.enc \
    -a aes-gcm \
    --generate-key my-module.key \
    --key-format hex

# 2. The encrypted file and key are now ready for distribution
# 3. To decrypt (for testing):
ruswacipher decrypt \
    -i my-module.wasm.enc \
    -o decrypted.wasm \
    -k my-module.key
```

#### Using Different Key Formats

```bash
# Generate key in Base64 format
ruswacipher encrypt -i input.wasm -o output.wasm --generate-key key.b64 --key-format base64

# Generate key in raw binary format
ruswacipher encrypt -i input.wasm -o output.wasm --generate-key key.bin --key-format raw

# Use hex key directly
ruswacipher encrypt -i input.wasm -o output.wasm --key-hex 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```

## JavaScript Runtime Integration

### Basic Usage

```html
<!DOCTYPE html>
<html>
<head>
    <title>Encrypted WASM Example</title>
</head>
<body>
    <script src="wasmGuardianLoader.js"></script>
    <script>
        async function loadEncryptedWasm() {
            const loader = new WasmGuardianLoader();
            
            try {
                const wasmInstance = await loader.loadEncryptedWasm(
                    'my-module.wasm.enc',  // Encrypted WASM file URL
                    'your-hex-key-here',   // Decryption key
                    {},                    // WASM imports (optional)
                    'auto'                 // Algorithm detection (optional)
                );
                
                // Use the WASM instance
                const result = wasmInstance.exports.myFunction(42);
                console.log('Result:', result);
                
            } catch (error) {
                console.error('Failed to load encrypted WASM:', error);
            }
        }
        
        loadEncryptedWasm();
    </script>
</body>
</html>
```

### Advanced Configuration

```javascript
// Configure key delivery method
const loader = new WasmGuardianLoader({
    keyMethod: 'server',
    keyServerEndpoint: '/api/wasm-keys',
    authToken: 'your-auth-token',
    timeout: 10000,
    retries: 3
});

// Load with server-side key delivery
const wasmInstance = await loader.loadEncryptedWasm(
    'my-module.wasm.enc',
    'module-identifier',  // Key identifier instead of actual key
    {
        env: {
            console_log: (ptr, len) => {
                // Custom import function
                console.log('WASM log:', readString(ptr, len));
            }
        }
    },
    'aes-gcm'
);
```

### Key Delivery Methods

#### 1. Hardcoded Keys (Default)

```javascript
const loader = new WasmGuardianLoader({
    keyMethod: 'hardcoded'
});

await loader.loadEncryptedWasm('module.wasm.enc', 'hex-key-here');
```

#### 2. Server-Side Key Delivery

```javascript
const loader = new WasmGuardianLoader({
    keyMethod: 'server',
    keyServerEndpoint: '/api/keys',
    authToken: 'bearer-token'
});

await loader.loadEncryptedWasm('module.wasm.enc', 'key-identifier');
```

#### 3. Key Derivation (Future Feature)

```javascript
const loader = new WasmGuardianLoader({
    keyMethod: 'derived',
    derivationParams: {
        algorithm: 'PBKDF2',
        iterations: 100000,
        salt: 'your-salt'
    }
});

await loader.loadEncryptedWasm('module.wasm.enc', 'password');
```

## Security Best Practices

### Key Management

1. **Never hardcode keys in production**: Use server-side key delivery or key derivation
2. **Rotate keys regularly**: Implement key rotation for long-term deployments
3. **Use strong keys**: Always use cryptographically secure random keys
4. **Protect key transmission**: Use HTTPS for all key delivery
5. **Implement access controls**: Restrict key access based on user authentication

### Deployment Security

1. **Use HTTPS**: Always serve encrypted WASM over HTTPS
2. **Implement CSP**: Use Content Security Policy to prevent XSS attacks
3. **Validate origins**: Implement server-side origin validation for key requests
4. **Monitor access**: Log and monitor key access patterns
5. **Rate limiting**: Implement rate limiting for key requests

### Algorithm Selection

- **AES-GCM**: Recommended for most use cases, widely supported
- **ChaCha20-Poly1305**: Alternative for environments where AES hardware acceleration is not available

### Example Security Headers

```http
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-eval'
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
```

## Troubleshooting

### Common Issues

#### "Invalid WASM magic number"

- **Cause**: Decryption failed or wrong key used
- **Solution**: Verify the key is correct and matches the encryption algorithm

#### "SubtleCrypto API not available"

- **Cause**: Browser doesn't support WebCrypto or page not served over HTTPS
- **Solution**: Use HTTPS or test in a supported browser

#### "Failed to fetch encrypted WASM"

- **Cause**: Network error or file not found
- **Solution**: Check file path and network connectivity

#### "Key request timeout"

- **Cause**: Server-side key delivery is slow or unavailable
- **Solution**: Check server status and increase timeout if needed

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
# CLI tool
ruswacipher -v encrypt -i input.wasm -o output.wasm --generate-key key.txt

# JavaScript (check browser console)
const loader = new WasmGuardianLoader({ debug: true });
```

### Performance Optimization

1. **Preload keys**: Cache keys when possible
2. **Use compression**: Compress WASM before encryption
3. **Optimize file size**: Remove debug information from WASM
4. **Use CDN**: Serve encrypted WASM files from a CDN

### Getting Help

- **GitHub Issues**: [Report bugs and request features](https://github.com/lonless9/ruswacipher/issues)
- **Documentation**: [Full documentation](https://github.com/lonless9/ruswacipher/docs)
- **Examples**: [Example projects](https://github.com/lonless9/ruswacipher/examples)
