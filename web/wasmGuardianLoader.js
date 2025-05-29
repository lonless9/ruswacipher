/**
 * RusWaCipher Web Runtime Loader
 *
 * A JavaScript loader for decrypting and instantiating encrypted WebAssembly modules
 * in browser environments. Supports both AES-GCM (via SubtleCrypto) and
 * ChaCha20-Poly1305 (via WASM helper module).
 *
 * @version 0.1.0
 * @author RusWaCipher Project
 */

class WasmGuardianLoader {
    constructor(options = {}) {
        this.supportedAlgorithms = ['aes-gcm', 'chacha20poly1305'];
        this.wasmDecryptorHelper = null;
        this.isHelperLoaded = false;

        // Key management configuration
        this.keyConfig = {
            // Key delivery method: 'hardcoded', 'server', 'derived'
            method: options.keyMethod || 'hardcoded',
            // Server endpoint for key fetching
            serverEndpoint: options.keyServerEndpoint || '/api/keys',
            // Authentication token for server requests
            authToken: options.authToken || null,
            // Key derivation parameters
            derivationParams: options.derivationParams || null,
            // Timeout for key requests (ms)
            timeout: options.timeout || 10000,
            // Retry configuration
            retries: options.retries || 3,
            retryDelay: options.retryDelay || 1000
        };
    }

    /**
     * Load and decrypt an encrypted WASM module
     *
     * @param {string} encryptedWasmUrl - URL to the encrypted WASM file (.wasm.enc)
     * @param {string} keyHex - Decryption key in hexadecimal format
     * @param {Object} [wasmImports={}] - Optional WASM import object
     * @param {string} [algorithm='auto'] - Encryption algorithm used ('auto' for detection)
     * @returns {Promise<WebAssembly.Instance>} - Instantiated WASM module
     */
    async loadEncryptedWasm(encryptedWasmUrl, keyOrIdentifier, wasmImports = {}, algorithm = 'auto') {
        try {
            // Validate inputs
            this._validateInputs(encryptedWasmUrl, keyOrIdentifier, algorithm);

            console.log(`[WasmGuardianLoader] Loading encrypted WASM from: ${encryptedWasmUrl}`);
            console.log(`[WasmGuardianLoader] Using algorithm: ${algorithm}`);

            // Step 1: Resolve the decryption key
            const keyHex = await this._resolveDecryptionKey(keyOrIdentifier);

            // Step 2: Fetch encrypted WASM file
            const encryptedData = await this._fetchEncryptedWasm(encryptedWasmUrl);

            // Step 3: Detect algorithm if set to 'auto'
            const detectedAlgorithm = algorithm === 'auto'
                ? this._detectAlgorithm(encryptedData, keyHex)
                : algorithm;

            console.log(`[WasmGuardianLoader] Using algorithm: ${detectedAlgorithm}`);

            // Step 4: Decrypt the WASM data
            const decryptedWasm = await this._decryptWasm(encryptedData, keyHex, detectedAlgorithm);

            // Step 5: Instantiate the WASM module
            const wasmInstance = await this._instantiateWasm(decryptedWasm, wasmImports);

            console.log('[WasmGuardianLoader] WASM module loaded successfully');
            return wasmInstance;

        } catch (error) {
            console.error('[WasmGuardianLoader] Failed to load encrypted WASM:', error);
            throw new Error(`WasmGuardianLoader: ${error.message}`);
        }
    }

    /**
     * Validate input parameters
     * @private
     */
    _validateInputs(url, keyOrIdentifier, algorithm) {
        if (!url || typeof url !== 'string') {
            throw new Error('Invalid encrypted WASM URL');
        }

        if (!keyOrIdentifier) {
            throw new Error('Invalid key or key identifier: must be provided');
        }

        // For hardcoded method, validate hex format
        if (this.keyConfig.method === 'hardcoded' && typeof keyOrIdentifier === 'string') {
            if (!/^[0-9a-fA-F]+$/.test(keyOrIdentifier)) {
                throw new Error('Invalid key format: must be hexadecimal');
            }
        }

        if (algorithm !== 'auto' && !this.supportedAlgorithms.includes(algorithm.toLowerCase())) {
            throw new Error(`Unsupported algorithm: ${algorithm}. Supported: ${this.supportedAlgorithms.join(', ')}, auto`);
        }
    }

    /**
     * Detect encryption algorithm based on encrypted data and key
     * @private
     */
    _detectAlgorithm(encryptedData, keyHex) {
        console.log('[WasmGuardianLoader] Detecting encryption algorithm...');

        // Convert hex key to bytes for analysis
        const keyBytes = this._hexToUint8Array(keyHex);

        // Algorithm detection heuristics:
        // 1. Key length analysis
        if (keyBytes.length === 32) {
            // Both AES-256-GCM and ChaCha20-Poly1305 use 32-byte keys
            // Try to detect based on data structure or other hints

            // For now, default to AES-GCM as it's more commonly supported
            // In a real implementation, you might:
            // - Check for metadata in the encrypted file
            // - Try decryption with both algorithms
            // - Use file naming conventions
            console.log('[WasmGuardianLoader] Key length 32 bytes - defaulting to AES-GCM');
            return 'aes-gcm';
        } else {
            // Unknown key length - try AES-GCM first
            console.log(`[WasmGuardianLoader] Unknown key length ${keyBytes.length} bytes - defaulting to AES-GCM`);
            return 'aes-gcm';
        }
    }

    /**
     * Fetch encrypted WASM file from URL
     * @private
     */
    async _fetchEncryptedWasm(url) {
        console.log('[WasmGuardianLoader] Fetching encrypted WASM file...');

        const response = await fetch(url);
        if (!response.ok) {
            throw new Error(`Failed to fetch encrypted WASM: ${response.status} ${response.statusText}`);
        }

        const arrayBuffer = await response.arrayBuffer();
        console.log(`[WasmGuardianLoader] Fetched ${arrayBuffer.byteLength} bytes`);

        return new Uint8Array(arrayBuffer);
    }

    /**
     * Decrypt WASM data using the specified algorithm
     * @private
     */
    async _decryptWasm(encryptedData, keyHex, algorithm) {
        console.log('[WasmGuardianLoader] Decrypting WASM data...');

        switch (algorithm.toLowerCase()) {
            case 'aes-gcm':
                return await this._decryptAesGcm(encryptedData, keyHex);
            case 'chacha20poly1305':
                return await this._decryptChaCha20Poly1305(encryptedData, keyHex);
            default:
                throw new Error(`Unsupported decryption algorithm: ${algorithm}`);
        }
    }

    /**
     * Decrypt using AES-GCM via SubtleCrypto API
     * @private
     */
    async _decryptAesGcm(encryptedData, keyHex) {
        if (!window.crypto || !window.crypto.subtle) {
            throw new Error('SubtleCrypto API not available in this environment');
        }

        try {
            // Convert hex key to ArrayBuffer
            const keyBuffer = this._hexToArrayBuffer(keyHex);

            // Import the key
            const cryptoKey = await window.crypto.subtle.importKey(
                'raw',
                keyBuffer,
                { name: 'AES-GCM' },
                false,
                ['decrypt']
            );

            // Extract IV and ciphertext (IV is first 12 bytes for AES-GCM)
            const ivLength = 12;
            if (encryptedData.length < ivLength) {
                throw new Error('Encrypted data too short to contain IV');
            }

            const iv = encryptedData.slice(0, ivLength);
            const ciphertext = encryptedData.slice(ivLength);

            console.log(`[WasmGuardianLoader] IV length: ${iv.length}, Ciphertext length: ${ciphertext.length}`);

            // Decrypt
            const decryptedBuffer = await window.crypto.subtle.decrypt(
                {
                    name: 'AES-GCM',
                    iv: iv
                },
                cryptoKey,
                ciphertext
            );

            return new Uint8Array(decryptedBuffer);

        } catch (error) {
            throw new Error(`AES-GCM decryption failed: ${error.message}`);
        }
    }

    /**
     * Decrypt using ChaCha20-Poly1305 via WASM helper module
     * @private
     */
    async _decryptChaCha20Poly1305(encryptedData, keyHex) {
        // Load WASM decryption helper if not already loaded
        if (!this.isHelperLoaded) {
            await this._loadWasmDecryptorHelper();
        }

        try {
            // Convert hex key to bytes
            const keyBytes = this._hexToUint8Array(keyHex);

            // Extract nonce and ciphertext (nonce is first 12 bytes for ChaCha20-Poly1305)
            const nonceLength = 12;
            if (encryptedData.length < nonceLength) {
                throw new Error('Encrypted data too short to contain nonce');
            }

            const nonce = encryptedData.slice(0, nonceLength);
            const ciphertext = encryptedData.slice(nonceLength);

            console.log(`[WasmGuardianLoader] Nonce length: ${nonce.length}, Ciphertext length: ${ciphertext.length}`);

            // Call WASM decryption function
            let decryptedData;
            if (this.wasmDecryptorHelper.decryptChaCha20Poly1305) {
                // Using wrapper module
                decryptedData = await this.wasmDecryptorHelper.decryptChaCha20Poly1305(
                    keyBytes,
                    nonce,
                    ciphertext
                );
            } else if (this.wasmDecryptorHelper.decrypt_chacha20poly1305) {
                // Using direct WASM module
                decryptedData = this.wasmDecryptorHelper.decrypt_chacha20poly1305(
                    keyBytes,
                    nonce,
                    ciphertext
                );
            } else {
                throw new Error('WASM helper does not provide ChaCha20-Poly1305 decryption function');
            }

            return decryptedData;

        } catch (error) {
            throw new Error(`ChaCha20-Poly1305 decryption failed: ${error.message}`);
        }
    }

    /**
     * Load WASM decryption helper module
     * @private
     */
    async _loadWasmDecryptorHelper() {
        console.log('[WasmGuardianLoader] Loading WASM decryption helper...');

        try {
            // Try to use dynamic import if available
            if (typeof window !== 'undefined' && window.import) {
                // ES6 module environment - use dynamic import
                const wasmModule = await window.import('./wasm-decryptor-helper-wrapper.js');
                await wasmModule.initWasmHelper();
                this.wasmDecryptorHelper = wasmModule;
                this.isHelperLoaded = true;
                console.log('[WasmGuardianLoader] WASM helper loaded via ES6 modules');
            } else {
                // Fallback: try to load via script tag approach
                // This requires the helper to be pre-loaded
                if (typeof window !== 'undefined' && window.wasmDecryptorHelper) {
                    this.wasmDecryptorHelper = window.wasmDecryptorHelper;
                    this.isHelperLoaded = true;
                    console.log('[WasmGuardianLoader] WASM helper loaded from global scope');
                } else {
                    // Try to load the helper dynamically
                    await this._loadWasmHelperViaScript();
                }
            }

        } catch (error) {
            throw new Error(`Failed to load WASM decryption helper: ${error.message}`);
        }
    }

    /**
     * Load WASM helper via script tag (fallback method)
     * @private
     */
    async _loadWasmHelperViaScript() {
        return new Promise((resolve, reject) => {
            // For now, we'll indicate that this method needs implementation
            // In a real implementation, this would dynamically load the WASM helper
            reject(new Error('Script-based WASM helper loading not yet implemented. Please use ES6 modules or pre-load the helper.'));
        });
    }

    /**
     * Instantiate decrypted WASM module
     * @private
     */
    async _instantiateWasm(wasmBytes, imports) {
        console.log('[WasmGuardianLoader] Instantiating WASM module...');

        try {
            // Validate WASM magic number
            if (wasmBytes.length < 4 ||
                wasmBytes[0] !== 0x00 || wasmBytes[1] !== 0x61 ||
                wasmBytes[2] !== 0x73 || wasmBytes[3] !== 0x6D) {
                throw new Error('Invalid WASM magic number - decryption may have failed');
            }

            const wasmModule = await WebAssembly.instantiate(wasmBytes, imports);
            console.log('[WasmGuardianLoader] WASM instantiation successful');

            return wasmModule.instance;

        } catch (error) {
            throw new Error(`WASM instantiation failed: ${error.message}`);
        }
    }

    /**
     * Resolve decryption key based on configuration
     * @private
     */
    async _resolveDecryptionKey(keyOrIdentifier) {
        console.log(`[WasmGuardianLoader] Resolving key using method: ${this.keyConfig.method}`);

        switch (this.keyConfig.method) {
            case 'hardcoded':
                return this._resolveHardcodedKey(keyOrIdentifier);
            case 'server':
                return await this._fetchKeyFromServer(keyOrIdentifier);
            case 'derived':
                return await this._deriveKey(keyOrIdentifier);
            default:
                throw new Error(`Unsupported key method: ${this.keyConfig.method}`);
        }
    }

    /**
     * Resolve hardcoded key (existing behavior)
     * @private
     */
    _resolveHardcodedKey(keyHex) {
        if (typeof keyHex !== 'string') {
            throw new Error('Hardcoded key must be a hex string');
        }

        // Validate hex format
        if (!/^[0-9a-fA-F]+$/.test(keyHex)) {
            throw new Error('Invalid key format: must be hexadecimal');
        }

        console.log('[WasmGuardianLoader] Using hardcoded key');
        return keyHex;
    }

    /**
     * Fetch decryption key from server
     * @private
     */
    async _fetchKeyFromServer(keyIdentifier) {
        console.log(`[WasmGuardianLoader] Fetching key from server: ${keyIdentifier}`);

        const requestPayload = {
            keyId: keyIdentifier,
            timestamp: Date.now(),
            userAgent: navigator.userAgent,
            // Add additional client verification data
            clientFingerprint: await this._generateClientFingerprint()
        };

        let lastError;
        for (let attempt = 0; attempt < this.keyConfig.retries; attempt++) {
            try {
                const response = await this._makeKeyRequest(requestPayload);

                if (!response.ok) {
                    throw new Error(`Server responded with ${response.status}: ${response.statusText}`);
                }

                const keyData = await response.json();

                if (!keyData.key) {
                    throw new Error('Server response missing key data');
                }

                // Validate key format
                if (!/^[0-9a-fA-F]+$/.test(keyData.key)) {
                    throw new Error('Server returned invalid key format');
                }

                console.log('[WasmGuardianLoader] Key fetched successfully from server');
                return keyData.key;

            } catch (error) {
                lastError = error;
                console.warn(`[WasmGuardianLoader] Key fetch attempt ${attempt + 1} failed:`, error.message);

                if (attempt < this.keyConfig.retries - 1) {
                    await this._delay(this.keyConfig.retryDelay * (attempt + 1));
                }
            }
        }

        throw new Error(`Failed to fetch key after ${this.keyConfig.retries} attempts: ${lastError.message}`);
    }

    /**
     * Make HTTP request to key server
     * @private
     */
    async _makeKeyRequest(payload) {
        const headers = {
            'Content-Type': 'application/json',
            'X-Requested-With': 'WasmGuardianLoader'
        };

        if (this.keyConfig.authToken) {
            headers['Authorization'] = `Bearer ${this.keyConfig.authToken}`;
        }

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.keyConfig.timeout);

        try {
            const response = await fetch(this.keyConfig.serverEndpoint, {
                method: 'POST',
                headers,
                body: JSON.stringify(payload),
                signal: controller.signal,
                credentials: 'same-origin' // Include cookies for authentication
            });

            clearTimeout(timeoutId);
            return response;
        } catch (error) {
            clearTimeout(timeoutId);
            if (error.name === 'AbortError') {
                throw new Error(`Key request timeout after ${this.keyConfig.timeout}ms`);
            }
            throw error;
        }
    }

    /**
     * Generate client fingerprint for server verification
     * @private
     */
    async _generateClientFingerprint() {
        const components = [
            navigator.userAgent,
            navigator.language,
            screen.width + 'x' + screen.height,
            new Date().getTimezoneOffset(),
            navigator.hardwareConcurrency || 'unknown'
        ];

        const fingerprint = components.join('|');

        // Create a simple hash of the fingerprint
        if (window.crypto && window.crypto.subtle) {
            try {
                const encoder = new TextEncoder();
                const data = encoder.encode(fingerprint);
                const hashBuffer = await window.crypto.subtle.digest('SHA-256', data);
                const hashArray = Array.from(new Uint8Array(hashBuffer));
                return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
            } catch (error) {
                console.warn('[WasmGuardianLoader] Failed to generate crypto fingerprint, using simple hash');
            }
        }

        // Fallback: simple string hash
        let hash = 0;
        for (let i = 0; i < fingerprint.length; i++) {
            const char = fingerprint.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32-bit integer
        }
        return Math.abs(hash).toString(16);
    }

    /**
     * Derive key using client-side computation
     * @private
     */
    async _deriveKey(derivationData) {
        console.log('[WasmGuardianLoader] Deriving key using client-side computation');

        if (!this.keyConfig.derivationParams) {
            throw new Error('Key derivation parameters not configured');
        }

        const { algorithm, iterations, salt } = this.keyConfig.derivationParams;

        if (!window.crypto || !window.crypto.subtle) {
            throw new Error('SubtleCrypto API required for key derivation');
        }

        try {
            // Convert inputs to appropriate formats
            const encoder = new TextEncoder();
            const keyMaterial = encoder.encode(derivationData);
            const saltBytes = typeof salt === 'string' ? encoder.encode(salt) : salt;

            // Import the key material
            const importedKey = await window.crypto.subtle.importKey(
                'raw',
                keyMaterial,
                { name: 'PBKDF2' },
                false,
                ['deriveBits']
            );

            // Derive the key
            const derivedBits = await window.crypto.subtle.deriveBits(
                {
                    name: 'PBKDF2',
                    salt: saltBytes,
                    iterations: iterations || 100000,
                    hash: algorithm || 'SHA-256'
                },
                importedKey,
                256 // 32 bytes for AES-256 or ChaCha20
            );

            // Convert to hex string
            const keyArray = Array.from(new Uint8Array(derivedBits));
            const keyHex = keyArray.map(b => b.toString(16).padStart(2, '0')).join('');

            console.log('[WasmGuardianLoader] Key derived successfully');
            return keyHex;

        } catch (error) {
            throw new Error(`Key derivation failed: ${error.message}`);
        }
    }

    /**
     * Utility function for delays
     * @private
     */
    _delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    /**
     * Convert hex string to ArrayBuffer
     * @private
     */
    _hexToArrayBuffer(hex) {
        const bytes = new Uint8Array(hex.length / 2);
        for (let i = 0; i < hex.length; i += 2) {
            bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
        }
        return bytes.buffer;
    }

    /**
     * Convert hex string to Uint8Array
     * @private
     */
    _hexToUint8Array(hex) {
        const bytes = new Uint8Array(hex.length / 2);
        for (let i = 0; i < hex.length; i += 2) {
            bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
        }
        return bytes;
    }

    /**
     * Get loader version and supported algorithms
     */
    getInfo() {
        return {
            version: '0.1.0',
            supportedAlgorithms: this.supportedAlgorithms,
            hasSubtleCrypto: !!(window.crypto && window.crypto.subtle),
            wasmHelperLoaded: this.isHelperLoaded
        };
    }
}

// Export for both ES6 modules and CommonJS
if (typeof module !== 'undefined' && module.exports) {
    module.exports = WasmGuardianLoader;
} else if (typeof window !== 'undefined') {
    window.WasmGuardianLoader = WasmGuardianLoader;
}
