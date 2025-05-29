# RusWaCipher API Reference

## Table of Contents

1. [Rust API](#rust-api)
2. [JavaScript API](#javascript-api)
3. [CLI Interface](#cli-interface)
4. [Error Handling](#error-handling)

## Rust API

### Core Modules

#### `ruswacipher::crypto`

##### Traits

```rust
pub trait Cipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult>;
    fn decrypt(&self, iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>>;
    fn iv_length(&self) -> usize;
    fn key_length(&self) -> usize;
}
```

##### Structs

```rust
pub struct EncryptionResult {
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

impl EncryptionResult {
    pub fn serialize(&self) -> Vec<u8>;
    pub fn deserialize(data: &[u8], iv_length: usize) -> Result<Self>;
}

pub struct AesGcmCipher { /* private fields */ }

impl AesGcmCipher {
    pub fn new(key: &[u8]) -> Result<Self>;
}

pub struct ChaCha20Poly1305Cipher { /* private fields */ }

impl ChaCha20Poly1305Cipher {
    pub fn new(key: &[u8]) -> Result<Self>;
}
```

##### Functions

```rust
pub fn generate_key(algorithm: &EncryptionAlgorithm) -> Result<Vec<u8>>;
```

#### `ruswacipher::wasm`

```rust
pub struct WasmParser;

impl WasmParser {
    pub fn validate_wasm(data: &[u8]) -> Result<()>;
    pub fn get_module_info(data: &[u8]) -> Result<WasmModuleInfo>;
}

pub struct WasmModuleInfo {
    pub version: u32,
    pub type_count: u32,
    pub import_count: u32,
    pub function_count: u32,
    pub export_count: u32,
}

pub struct WasmWriter;

impl WasmWriter {
    pub fn write_wasm_file<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<()>;
    pub fn write_validated_wasm_file<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<()>;
}
```

#### `ruswacipher::config`

```rust
#[derive(Debug, Clone)]
pub enum EncryptionAlgorithm {
    AesGcm,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub key_file: Option<PathBuf>,
    pub key_hex: Option<String>,
    pub key_base64: Option<String>,
    pub generate_key: bool,
    pub key_output_file: Option<PathBuf>,
    pub key_format: KeyFormat,
}

#[derive(Debug, Clone)]
pub struct DecryptionConfig {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub key_file: PathBuf,
}
```

#### `ruswacipher::io`

```rust
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>>;
pub fn write_file<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<()>;
pub fn read_key_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>>;
pub fn write_key_file<P: AsRef<Path>>(path: P, key: &[u8]) -> Result<()>;
pub fn write_key_file_with_format<P: AsRef<Path>>(
    path: P,
    key: &[u8],
    format: &KeyFormat,
) -> Result<()>;
```

### Example Usage

```rust
use ruswacipher::{
    crypto::{AesGcmCipher, Cipher, generate_key},
    config::EncryptionAlgorithm,
    io::{read_file, write_file},
};

// Generate a key
let key = generate_key(&EncryptionAlgorithm::AesGcm)?;

// Create cipher
let cipher = AesGcmCipher::new(&key)?;

// Read WASM file
let wasm_data = read_file("input.wasm")?;

// Encrypt
let encrypted = cipher.encrypt(&wasm_data)?;
let serialized = encrypted.serialize();

// Write encrypted file
write_file("output.wasm.enc", &serialized)?;

// Decrypt
let decrypted = cipher.decrypt(&encrypted.iv, &encrypted.ciphertext)?;
```

## JavaScript API

### WasmGuardianLoader Class

#### Constructor

```javascript
new WasmGuardianLoader(options = {})
```

**Parameters:**
- `options` (Object): Configuration options
  - `keyMethod` (string): Key delivery method ('hardcoded', 'server', 'derived')
  - `keyServerEndpoint` (string): Server endpoint for key fetching
  - `authToken` (string): Authentication token for server requests
  - `timeout` (number): Request timeout in milliseconds
  - `retries` (number): Number of retry attempts
  - `retryDelay` (number): Delay between retries in milliseconds

#### Methods

##### `loadEncryptedWasm(url, keyOrIdentifier, imports, algorithm)`

Load and decrypt an encrypted WASM module.

**Parameters:**
- `url` (string): URL to the encrypted WASM file
- `keyOrIdentifier` (string): Decryption key (hex) or key identifier
- `imports` (Object): WASM import object (optional)
- `algorithm` (string): Encryption algorithm ('auto', 'aes-gcm', 'chacha20poly1305')

**Returns:** `Promise<WebAssembly.Instance>`

**Example:**
```javascript
const loader = new WasmGuardianLoader();
const instance = await loader.loadEncryptedWasm(
    'module.wasm.enc',
    '0123456789abcdef...',
    { env: { console_log: console.log } },
    'aes-gcm'
);
```

#### Private Methods (Internal API)

##### `_validateInputs(url, keyOrIdentifier, algorithm)`
##### `_detectAlgorithm(encryptedData, keyHex)`
##### `_fetchEncryptedWasm(url)`
##### `_decryptWasm(encryptedData, keyHex, algorithm)`
##### `_decryptAesGcm(encryptedData, keyHex)`
##### `_decryptChaCha20Poly1305(encryptedData, keyHex)`
##### `_instantiateWasm(wasmBytes, imports)`
##### `_resolveDecryptionKey(keyOrIdentifier)`
##### `_generateClientFingerprint()`

### WASM Decryption Helper

#### Functions

```javascript
// Available in WASM helper module
decrypt_chacha20poly1305(key, nonce, ciphertext) -> Uint8Array
encrypt_chacha20poly1305(key, nonce, plaintext) -> Uint8Array
get_helper_info() -> Object
test_helper() -> boolean
```

### Error Types

```javascript
// Common error messages
'Invalid encrypted WASM URL'
'Invalid key or key identifier'
'Invalid key format: must be hexadecimal'
'Unsupported algorithm: {algorithm}'
'SubtleCrypto API not available in this environment'
'Failed to fetch encrypted WASM: {status} {statusText}'
'Encrypted data too short to contain IV'
'Invalid WASM magic number - decryption may have failed'
'WASM instantiation failed: {error}'
```

## CLI Interface

### Commands

#### `encrypt`

Encrypt a WASM file.

```bash
ruswacipher encrypt [OPTIONS] -i <INPUT> -o <OUTPUT>
```

**Required Arguments:**
- `-i, --input <INPUT>`: Input WASM file path
- `-o, --output <OUTPUT>`: Output encrypted file path

**Optional Arguments:**
- `-a, --algorithm <ALGORITHM>`: Encryption algorithm [default: aes-gcm]
- `-k, --key <KEY>`: Key file path
- `--key-hex <KEY_HEX>`: Key in hexadecimal format
- `--key-base64 <KEY_BASE64>`: Key in Base64 format
- `--generate-key <GENERATE_KEY>`: Generate new key and save to file
- `--key-format <KEY_FORMAT>`: Key output format [default: hex]

#### `decrypt`

Decrypt a WASM file.

```bash
ruswacipher decrypt [OPTIONS] -i <INPUT> -o <OUTPUT> -k <KEY>
```

**Required Arguments:**
- `-i, --input <INPUT>`: Input encrypted file path
- `-o, --output <OUTPUT>`: Output decrypted WASM file path
- `-k, --key <KEY>`: Key file path

### Global Options

- `-v, --verbose`: Enable verbose logging
- `-h, --help`: Print help information
- `-V, --version`: Print version information

### Exit Codes

- `0`: Success
- `1`: General error
- `2`: Invalid arguments
- `3`: File I/O error
- `4`: Encryption/decryption error
- `5`: WASM parsing error

## Error Handling

### Rust Error Types

```rust
#[derive(Error, Debug)]
pub enum RusWaCipherError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("WASM parsing error: {0}")]
    WasmParser(#[from] wasmparser::BinaryReaderError),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    #[error("Key management error: {0}")]
    KeyManagement(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Hex decoding error: {0}")]
    HexDecode(#[from] hex::FromHexError),
    
    #[error("Base64 decoding error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
}
```

### JavaScript Error Handling

```javascript
try {
    const instance = await loader.loadEncryptedWasm(url, key);
    // Use instance
} catch (error) {
    if (error.message.includes('Invalid WASM magic number')) {
        console.error('Decryption failed - wrong key or corrupted data');
    } else if (error.message.includes('SubtleCrypto API not available')) {
        console.error('Browser does not support WebCrypto API');
    } else if (error.message.includes('Failed to fetch')) {
        console.error('Network error loading encrypted WASM');
    } else {
        console.error('Unknown error:', error);
    }
}
```

### Best Practices

1. **Always handle errors**: Never ignore error returns or promise rejections
2. **Provide user feedback**: Show meaningful error messages to users
3. **Log for debugging**: Log detailed error information for debugging
4. **Graceful degradation**: Provide fallbacks when possible
5. **Validate inputs**: Check inputs before processing to prevent errors
