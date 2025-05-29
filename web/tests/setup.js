// Jest setup file for web runtime tests

// Mock WebAssembly if not available in test environment
if (typeof WebAssembly === 'undefined') {
    global.WebAssembly = {
        instantiate: jest.fn().mockResolvedValue({
            instance: {
                exports: {
                    add: jest.fn((a, b) => a + b),
                    multiply: jest.fn((a, b) => a * b),
                    greet: jest.fn((name) => `Hello, ${name}!`),
                    sum_array: jest.fn((arr) => arr.reduce((a, b) => a + b, 0))
                }
            }
        }),
        Module: jest.fn(),
        Instance: jest.fn()
    };
}

// Mock SubtleCrypto API
if (typeof window !== 'undefined' && !window.crypto) {
    window.crypto = {
        subtle: {
            importKey: jest.fn().mockResolvedValue({}),
            decrypt: jest.fn().mockImplementation((algorithm, key, data) => {
                // Mock AES-GCM decryption - return the data without the first 12 bytes (IV)
                const ciphertext = new Uint8Array(data);
                if (ciphertext.length > 12) {
                    return Promise.resolve(ciphertext.slice(12).buffer);
                }
                return Promise.resolve(new ArrayBuffer(0));
            }),
            digest: jest.fn().mockImplementation((algorithm, data) => {
                // Mock SHA-256 digest
                const hash = new Uint8Array(32);
                for (let i = 0; i < 32; i++) {
                    hash[i] = i;
                }
                return Promise.resolve(hash.buffer);
            })
        },
        getRandomValues: jest.fn().mockImplementation((array) => {
            for (let i = 0; i < array.length; i++) {
                array[i] = Math.floor(Math.random() * 256);
            }
            return array;
        })
    };
}

// Mock fetch API
global.fetch = jest.fn();

// Mock console methods to avoid noise in tests
global.console = {
    ...console,
    log: jest.fn(),
    warn: jest.fn(),
    error: jest.fn(),
    info: jest.fn(),
    debug: jest.fn()
};

// Helper function to create mock encrypted WASM data
global.createMockEncryptedWasm = function(plaintextWasm) {
    // Create mock encrypted data: 12-byte IV + ciphertext
    const iv = new Uint8Array(12);
    crypto.getRandomValues(iv);
    
    const encrypted = new Uint8Array(iv.length + plaintextWasm.length);
    encrypted.set(iv, 0);
    encrypted.set(plaintextWasm, iv.length);
    
    return encrypted;
};

// Helper function to create minimal valid WASM module
global.createMinimalWasm = function() {
    return new Uint8Array([
        0x00, 0x61, 0x73, 0x6D, // WASM magic number
        0x01, 0x00, 0x00, 0x00, // Version
        // Type section
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
        // Function section
        0x03, 0x02, 0x01, 0x00,
        // Code section
        0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B,
    ]);
};

// Mock navigator object
if (typeof navigator === 'undefined') {
    global.navigator = {
        userAgent: 'Mozilla/5.0 (Test Environment)',
        language: 'en-US',
        hardwareConcurrency: 4
    };
}

// Mock screen object
if (typeof screen === 'undefined') {
    global.screen = {
        width: 1920,
        height: 1080
    };
}

// Mock Date for consistent fingerprinting
const OriginalDate = Date;
global.Date = class extends OriginalDate {
    getTimezoneOffset() {
        return -480; // UTC+8
    }
};

// Reset all mocks before each test
beforeEach(() => {
    jest.clearAllMocks();
    
    // Reset fetch mock
    fetch.mockClear();
    
    // Reset WebAssembly mock
    if (WebAssembly.instantiate) {
        WebAssembly.instantiate.mockClear();
    }
});

// Clean up after each test
afterEach(() => {
    // Clean up any global state
    if (typeof window !== 'undefined') {
        delete window.wasmDecryptorHelper;
    }
});
