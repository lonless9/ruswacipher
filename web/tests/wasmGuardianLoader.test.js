// Unit tests for WasmGuardianLoader

// Import the loader (in a real environment, this would be a proper ES6 import)
// For Jest, we'll load it as a script
const fs = require('fs');
const path = require('path');

// Load the WasmGuardianLoader source
const loaderSource = fs.readFileSync(path.join(__dirname, '../wasmGuardianLoader.js'), 'utf8');
eval(loaderSource);

describe('WasmGuardianLoader', () => {
    let loader;

    beforeEach(() => {
        loader = new WasmGuardianLoader();
    });

    describe('Constructor', () => {
        test('should create instance with default options', () => {
            expect(loader).toBeInstanceOf(WasmGuardianLoader);
            expect(loader.supportedAlgorithms).toEqual(['aes-gcm', 'chacha20poly1305']);
            expect(loader.keyConfig.method).toBe('hardcoded');
        });

        test('should create instance with custom options', () => {
            const customLoader = new WasmGuardianLoader({
                keyMethod: 'server',
                keyServerEndpoint: '/custom/keys',
                authToken: 'test-token',
                timeout: 5000
            });

            expect(customLoader.keyConfig.method).toBe('server');
            expect(customLoader.keyConfig.serverEndpoint).toBe('/custom/keys');
            expect(customLoader.keyConfig.authToken).toBe('test-token');
            expect(customLoader.keyConfig.timeout).toBe(5000);
        });
    });

    describe('Input Validation', () => {
        test('should validate URL parameter', () => {
            expect(() => {
                loader._validateInputs('', 'key', 'aes-gcm');
            }).toThrow('Invalid encrypted WASM URL');

            expect(() => {
                loader._validateInputs(null, 'key', 'aes-gcm');
            }).toThrow('Invalid encrypted WASM URL');
        });

        test('should validate key parameter', () => {
            expect(() => {
                loader._validateInputs('test.wasm', '', 'aes-gcm');
            }).toThrow('Invalid key or key identifier');

            expect(() => {
                loader._validateInputs('test.wasm', null, 'aes-gcm');
            }).toThrow('Invalid key or key identifier');
        });

        test('should validate hex key format for hardcoded method', () => {
            expect(() => {
                loader._validateInputs('test.wasm', 'invalid-hex-key', 'aes-gcm');
            }).toThrow('Invalid key format: must be hexadecimal');

            // Valid hex key should not throw
            expect(() => {
                loader._validateInputs('test.wasm', '0123456789abcdef', 'aes-gcm');
            }).not.toThrow();
        });

        test('should validate algorithm parameter', () => {
            expect(() => {
                loader._validateInputs('test.wasm', 'key', 'invalid-algorithm');
            }).toThrow('Unsupported algorithm');

            // Valid algorithms should not throw
            expect(() => {
                loader._validateInputs('test.wasm', 'key', 'aes-gcm');
            }).not.toThrow();

            expect(() => {
                loader._validateInputs('test.wasm', 'key', 'chacha20poly1305');
            }).not.toThrow();

            expect(() => {
                loader._validateInputs('test.wasm', 'key', 'auto');
            }).not.toThrow();
        });
    });

    describe('Algorithm Detection', () => {
        test('should detect algorithm based on key length', () => {
            const mockEncryptedData = new Uint8Array(100);
            const key32Bytes = '0'.repeat(64); // 32 bytes in hex

            const algorithm = loader._detectAlgorithm(mockEncryptedData, key32Bytes);
            expect(algorithm).toBe('aes-gcm');
        });

        test('should default to aes-gcm for unknown key lengths', () => {
            const mockEncryptedData = new Uint8Array(100);
            const shortKey = '0'.repeat(32); // 16 bytes in hex

            const algorithm = loader._detectAlgorithm(mockEncryptedData, shortKey);
            expect(algorithm).toBe('aes-gcm');
        });
    });

    describe('Utility Functions', () => {
        test('should convert hex to Uint8Array', () => {
            const hex = '48656c6c6f'; // "Hello" in hex
            const result = loader._hexToUint8Array(hex);
            
            expect(result).toBeInstanceOf(Uint8Array);
            expect(Array.from(result)).toEqual([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
        });

        test('should convert hex to ArrayBuffer', () => {
            const hex = '48656c6c6f'; // "Hello" in hex
            const result = loader._hexToArrayBuffer(hex);
            
            expect(result).toBeInstanceOf(ArrayBuffer);
            expect(result.byteLength).toBe(5);
        });
    });

    describe('Key Resolution', () => {
        test('should resolve hardcoded key', () => {
            const hexKey = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
            const result = loader._resolveHardcodedKey(hexKey);
            
            expect(result).toBe(hexKey);
        });

        test('should reject invalid hardcoded key', () => {
            expect(() => {
                loader._resolveHardcodedKey('invalid-hex');
            }).toThrow('Invalid key format');

            expect(() => {
                loader._resolveHardcodedKey(123);
            }).toThrow('Hardcoded key must be a hex string');
        });

        test('should resolve key based on method', async () => {
            const hexKey = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
            
            // Test hardcoded method
            loader.keyConfig.method = 'hardcoded';
            const result = await loader._resolveDecryptionKey(hexKey);
            expect(result).toBe(hexKey);
        });
    });

    describe('Fetch Operations', () => {
        test('should fetch encrypted WASM file', async () => {
            const mockData = createMinimalWasm();
            const mockResponse = {
                ok: true,
                arrayBuffer: jest.fn().mockResolvedValue(mockData.buffer)
            };
            
            fetch.mockResolvedValue(mockResponse);

            const result = await loader._fetchEncryptedWasm('test.wasm');
            
            expect(fetch).toHaveBeenCalledWith('test.wasm');
            expect(result).toBeInstanceOf(Uint8Array);
            expect(result.length).toBe(mockData.length);
        });

        test('should handle fetch errors', async () => {
            const mockResponse = {
                ok: false,
                status: 404,
                statusText: 'Not Found'
            };
            
            fetch.mockResolvedValue(mockResponse);

            await expect(loader._fetchEncryptedWasm('nonexistent.wasm'))
                .rejects.toThrow('Failed to fetch encrypted WASM: 404 Not Found');
        });
    });

    describe('AES-GCM Decryption', () => {
        test('should decrypt AES-GCM encrypted data', async () => {
            const plaintext = createMinimalWasm();
            const encryptedData = createMockEncryptedWasm(plaintext);
            const hexKey = '0'.repeat(64); // 32 bytes in hex

            const result = await loader._decryptAesGcm(encryptedData, hexKey);
            
            expect(result).toBeInstanceOf(Uint8Array);
            expect(window.crypto.subtle.importKey).toHaveBeenCalled();
            expect(window.crypto.subtle.decrypt).toHaveBeenCalled();
        });

        test('should handle AES-GCM decryption errors', async () => {
            const shortData = new Uint8Array(5); // Too short for IV
            const hexKey = '0'.repeat(64);

            await expect(loader._decryptAesGcm(shortData, hexKey))
                .rejects.toThrow('Encrypted data too short to contain IV');
        });

        test('should handle missing SubtleCrypto API', async () => {
            const originalCrypto = window.crypto;
            delete window.crypto;

            const encryptedData = createMockEncryptedWasm(createMinimalWasm());
            const hexKey = '0'.repeat(64);

            await expect(loader._decryptAesGcm(encryptedData, hexKey))
                .rejects.toThrow('SubtleCrypto API not available');

            window.crypto = originalCrypto;
        });
    });

    describe('WASM Instantiation', () => {
        test('should instantiate valid WASM module', async () => {
            const wasmBytes = createMinimalWasm();
            const imports = {};

            const result = await loader._instantiateWasm(wasmBytes, imports);
            
            expect(WebAssembly.instantiate).toHaveBeenCalledWith(wasmBytes, imports);
            expect(result).toBeDefined();
        });

        test('should validate WASM magic number', async () => {
            const invalidWasm = new Uint8Array([0xFF, 0x61, 0x73, 0x6D]); // Invalid magic
            
            await expect(loader._instantiateWasm(invalidWasm, {}))
                .rejects.toThrow('Invalid WASM magic number');
        });

        test('should handle WASM instantiation errors', async () => {
            WebAssembly.instantiate.mockRejectedValue(new Error('Invalid WASM'));
            
            const wasmBytes = createMinimalWasm();
            
            await expect(loader._instantiateWasm(wasmBytes, {}))
                .rejects.toThrow('WASM instantiation failed');
        });
    });

    describe('Client Fingerprinting', () => {
        test('should generate client fingerprint', async () => {
            const fingerprint = await loader._generateClientFingerprint();
            
            expect(typeof fingerprint).toBe('string');
            expect(fingerprint.length).toBeGreaterThan(0);
        });

        test('should handle crypto fingerprint generation', async () => {
            const fingerprint = await loader._generateClientFingerprint();
            
            expect(window.crypto.subtle.digest).toHaveBeenCalled();
            expect(typeof fingerprint).toBe('string');
        });

        test('should fallback to simple hash when crypto fails', async () => {
            window.crypto.subtle.digest.mockRejectedValue(new Error('Crypto error'));
            
            const fingerprint = await loader._generateClientFingerprint();
            
            expect(typeof fingerprint).toBe('string');
            expect(fingerprint.length).toBeGreaterThan(0);
        });
    });

    describe('Delay Utility', () => {
        test('should implement delay function', async () => {
            const start = Date.now();
            await loader._delay(100);
            const end = Date.now();
            
            expect(end - start).toBeGreaterThanOrEqual(90); // Allow some tolerance
        });
    });
});
