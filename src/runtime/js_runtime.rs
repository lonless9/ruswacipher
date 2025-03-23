use anyhow::{anyhow, Result};

/// Generate JavaScript runtime code for decryption
pub fn generate_runtime(algorithm: &str) -> Result<String> {
    match algorithm.to_lowercase().as_str() {
        "aes-gcm" => Ok(generate_aes_gcm_runtime()),
        "chacha20poly1305" => Ok(generate_chacha20poly1305_runtime()),
        "all" => Ok(generate_combined_runtime()),
        _ => Err(anyhow!("Unsupported algorithm type: {}", algorithm)),
    }
}

/// Generate a combined JavaScript runtime that supports all encryption algorithms
fn generate_combined_runtime() -> String {
    r#"/**
 * RusWaCipher - WebAssembly Decryption Runtime (Combined Algorithms)
 */
(function(global) {
    'use strict';

    // Implementations for specific algorithms
    const AlgorithmImplementations = {
        // AES-GCM WebCrypto Implementation
        'aes-gcm': {
            async decrypt(data, key, header) {
                try {
                    const cryptoKey = await crypto.subtle.importKey(
                        'raw',
                        key,
                        { name: 'AES-GCM' },
                        false,
                        ['decrypt']
                    );
                    
                    const decrypted = await crypto.subtle.decrypt(
                        {
                            name: 'AES-GCM',
                            iv: new Uint8Array(header.nonce)
                        },
                        cryptoKey,
                        data
                    );
                    
                    return new Uint8Array(decrypted);
                } catch (error) {
                    console.error('AES-GCM decryption failed:', error);
                    throw error;
                }
            }
        },
        
        // ChaCha20-Poly1305 Implementation
        'chacha20poly1305': {
            async decrypt(data, key, header) {
                try {
                    // Try to use WebCrypto API if available
                    if (crypto.subtle && this._isChaCha20Poly1305Supported()) {
                        const cryptoKey = await crypto.subtle.importKey(
                            'raw',
                            key,
                            { name: 'CHACHA20-POLY1305' },
                            false,
                            ['decrypt']
                        );
                        
                        const decrypted = await crypto.subtle.decrypt(
                            {
                                name: 'CHACHA20-POLY1305',
                                iv: new Uint8Array(header.nonce)
                            },
                            cryptoKey,
                            data
                        );
                        
                        return new Uint8Array(decrypted);
                    } else {
                        // Fallback to a more robust implementation
                        console.warn('Native ChaCha20-Poly1305 not supported, using fallback implementation');
                        return await this._decryptFallback(data, key, header.nonce);
                    }
                } catch (error) {
                    console.error('ChaCha20-Poly1305 decryption failed:', error);
                    throw error;
                }
            },
            
            // Check if browser supports ChaCha20-Poly1305 in WebCrypto
            _isChaCha20Poly1305Supported() {
                return typeof crypto !== 'undefined' && 
                       typeof crypto.subtle !== 'undefined' &&
                       crypto.subtle.encrypt && 
                       typeof crypto.subtle.encrypt === 'function';
            },
            
            // Fallback implementation (simplified for example)
            async _decryptFallback(data, key, nonce) {
                throw new Error("ChaCha20-Poly1305 fallback implementation not available. Please use a browser with WebCrypto support.");
            }
        }
        
        // Additional algorithms can be added here in the future
    };

    // RusWaCipher namespace
    const RusWaCipher = {
        /**
         * Load module from encrypted WASM file
         * @param {string} url - Encrypted WASM file URL
         * @param {string|Uint8Array} key - Decryption key (Base64 string or Uint8Array)
         * @param {Object} importObject - Import object to pass to WebAssembly
         * @returns {Promise<WebAssembly.Instance>} - WASM module instance
         */
        async load(url, key, importObject = {}) {
            // Load encrypted WASM file
            const response = await fetch(url);
            const encryptedData = new Uint8Array(await response.arrayBuffer());
            
            // Convert key (if it's a base64 string)
            let keyArray = key;
            if (typeof key === 'string') {
                keyArray = this._base64ToUint8Array(key);
            }
            
            // Decrypt WASM data
            const wasmBytes = await this._decrypt(encryptedData, keyArray);
            
            // Callback when decryption is complete (if provided)
            if (typeof this.onDecrypt === 'function') {
                this.onDecrypt();
            }
            
            // Compile and instantiate WASM module
            const result = await WebAssembly.instantiate(wasmBytes, importObject);
            return result.instance;
        },
        
        /**
         * Decrypt WASM data
         * @private
         * @param {Uint8Array} data - Encrypted data
         * @param {Uint8Array} key - Decryption key
         * @returns {Promise<Uint8Array>} - Decrypted WASM data
         */
        async _decrypt(data, key) {
            try {
                // Parse header length
                const headerLenBytes = data.slice(0, 4);
                const headerLen = new DataView(headerLenBytes.buffer).getUint32(0, true);
                
                // Parse header data
                const headerBytes = data.slice(4, 4 + headerLen);
                const headerText = new TextDecoder().decode(headerBytes);
                const header = JSON.parse(headerText);
                
                // Extract encrypted data
                const ciphertext = data.slice(4 + headerLen);
                
                // Check if algorithm is supported
                const algorithm = header.algorithm.toLowerCase();
                if (!AlgorithmImplementations[algorithm]) {
                    throw new Error(`Unsupported algorithm type: ${algorithm}`);
                }
                
                // Use the appropriate algorithm implementation
                return await AlgorithmImplementations[algorithm].decrypt(ciphertext, key, header);
            } catch (error) {
                console.error('Decryption failed:', error);
                throw new Error(`WASM decryption failed: ${error.message}`);
            }
        },
        
        /**
         * Convert Base64 string to Uint8Array
         * @private
         * @param {string} base64 - Base64 encoded string
         * @returns {Uint8Array} - Decoded byte array
         */
        _base64ToUint8Array(base64) {
            const binaryString = atob(base64);
            const bytes = new Uint8Array(binaryString.length);
            for (let i = 0; i < binaryString.length; i++) {
                bytes[i] = binaryString.charCodeAt(i);
            }
            return bytes;
        }
    };
    
    // Expose RusWaCipher to global object
    global.RusWaCipher = RusWaCipher;
    
})(typeof globalThis !== 'undefined' ? globalThis : 
   typeof window !== 'undefined' ? window : 
   typeof global !== 'undefined' ? global : 
   typeof self !== 'undefined' ? self : {});
"#.to_string()
}

/// Generate JavaScript decryption code for AES-GCM
fn generate_aes_gcm_runtime() -> String {
    r#"/**
 * RusWaCipher - WebAssembly Decryption Runtime (AES-GCM)
 */
(function(global) {
    'use strict';

    // RusWaCipher namespace
    const RusWaCipher = {
        /**
         * Load module from encrypted WASM file
         * @param {string} url - Encrypted WASM file URL
         * @param {string|Uint8Array} key - Decryption key (Base64 string or Uint8Array)
         * @param {Object} importObject - Import object to pass to WebAssembly
         * @returns {Promise<WebAssembly.Instance>} - WASM module instance
         */
        async load(url, key, importObject = {}) {
            // Load encrypted WASM file
            const response = await fetch(url);
            const encryptedData = new Uint8Array(await response.arrayBuffer());
            
            // Convert key (if it's a base64 string)
            let keyArray = key;
            if (typeof key === 'string') {
                keyArray = this._base64ToUint8Array(key);
            }
            
            // Decrypt WASM data
            const wasmBytes = await this._decrypt(encryptedData, keyArray);
            
            // Callback when decryption is complete (if provided)
            if (typeof this.onDecrypt === 'function') {
                this.onDecrypt();
            }
            
            // Compile and instantiate WASM module
            const result = await WebAssembly.instantiate(wasmBytes, importObject);
            return result.instance;
        },
        
        /**
         * Decrypt WASM data
         * @private
         * @param {Uint8Array} data - Encrypted data
         * @param {Uint8Array} key - Decryption key
         * @returns {Promise<Uint8Array>} - Decrypted WASM data
         */
        async _decrypt(data, key) {
            try {
                // Parse header length
                const headerLenBytes = data.slice(0, 4);
                const headerLen = new DataView(headerLenBytes.buffer).getUint32(0, true);
                
                // Parse header data
                const headerBytes = data.slice(4, 4 + headerLen);
                const headerText = new TextDecoder().decode(headerBytes);
                const header = JSON.parse(headerText);
                
                // Check algorithm type
                if (header.algorithm !== 'aes-gcm') {
                    throw new Error(`Unsupported algorithm type: ${header.algorithm}`);
                }
                
                // Extract encrypted data
                const ciphertext = data.slice(4 + headerLen);
                
                // Decrypt
                const cryptoKey = await crypto.subtle.importKey(
                    'raw',
                    key,
                    { name: 'AES-GCM' },
                    false,
                    ['decrypt']
                );
                
                const decrypted = await crypto.subtle.decrypt(
                    {
                        name: 'AES-GCM',
                        iv: new Uint8Array(header.nonce)
                    },
                    cryptoKey,
                    ciphertext
                );
                
                return new Uint8Array(decrypted);
            } catch (error) {
                console.error('Decryption failed:', error);
                throw new Error(`WASM decryption failed: ${error.message}`);
            }
        },
        
        /**
         * Convert Base64 string to Uint8Array
         * @private
         * @param {string} base64 - Base64 encoded string
         * @returns {Uint8Array} - Decoded byte array
         */
        _base64ToUint8Array(base64) {
            const binaryString = atob(base64);
            const bytes = new Uint8Array(binaryString.length);
            for (let i = 0; i < binaryString.length; i++) {
                bytes[i] = binaryString.charCodeAt(i);
            }
            return bytes;
        }
    };
    
    // Expose RusWaCipher to global object
    global.RusWaCipher = RusWaCipher;
    
})(typeof globalThis !== 'undefined' ? globalThis : 
   typeof window !== 'undefined' ? window : 
   typeof global !== 'undefined' ? global : 
   typeof self !== 'undefined' ? self : {});
"#
    .to_string()
}

/// Generate JavaScript decryption code for ChaCha20-Poly1305
fn generate_chacha20poly1305_runtime() -> String {
    r#"/**
 * RusWaCipher - WebAssembly Decryption Runtime (ChaCha20-Poly1305)
 */
(function(global) {
    'use strict';

    /**
     * Simple JavaScript implementation of ChaCha20-Poly1305
     * Note: This is just an example, in practice you would use WebCrypto API or more optimized implementations
     */
    class ChaCha20Poly1305 {
        constructor(key) {
            this.key = key;
            // Warning to use native WebCrypto API
            console.warn('JavaScript implementation of ChaCha20-Poly1305 is for demonstration only, use WebCrypto API in production');
        }

        async decrypt(nonce, ciphertext) {
            // In a real implementation, this would be complete ChaCha20-Poly1305 decryption logic
            // As an example, we just return a pseudo-decryption result
            console.error('Unimplemented ChaCha20-Poly1305 decryption function, please use WebCrypto API or other encryption libraries');
            throw new Error('ChaCha20-Poly1305 decryption not implemented');
        }
    }

    // RusWaCipher namespace
    const RusWaCipher = {
        /**
         * Load module from encrypted WASM file
         * @param {string} url - Encrypted WASM file URL
         * @param {string|Uint8Array} key - Decryption key (Base64 string or Uint8Array)
         * @param {Object} importObject - Import object to pass to WebAssembly
         * @returns {Promise<WebAssembly.Instance>} - WASM module instance
         */
        async load(url, key, importObject = {}) {
            // Load encrypted WASM file
            const response = await fetch(url);
            const encryptedData = new Uint8Array(await response.arrayBuffer());
            
            // Convert key (if it's a base64 string)
            let keyArray = key;
            if (typeof key === 'string') {
                keyArray = this._base64ToUint8Array(key);
            }
            
            // Decrypt WASM data
            const wasmBytes = await this._decrypt(encryptedData, keyArray);
            
            // Callback when decryption is complete (if provided)
            if (typeof this.onDecrypt === 'function') {
                this.onDecrypt();
            }
            
            // Compile and instantiate WASM module
            const result = await WebAssembly.instantiate(wasmBytes, importObject);
            return result.instance;
        },
        
        /**
         * Decrypt WASM data
         * @private
         * @param {Uint8Array} data - Encrypted data
         * @param {Uint8Array} key - Decryption key
         * @returns {Promise<Uint8Array>} - Decrypted WASM data
         */
        async _decrypt(data, key) {
            try {
                // Parse header length
                const headerLenBytes = data.slice(0, 4);
                const headerLen = new DataView(headerLenBytes.buffer).getUint32(0, true);
                
                // Parse header data
                const headerBytes = data.slice(4, 4 + headerLen);
                const headerText = new TextDecoder().decode(headerBytes);
                const header = JSON.parse(headerText);
                
                // Check algorithm type
                if (header.algorithm !== 'chacha20poly1305') {
                    throw new Error(`Unsupported algorithm type: ${header.algorithm}`);
                }
                
                // Extract encrypted data
                const ciphertext = data.slice(4 + headerLen);
                
                // Try to use WebCrypto API (if browser supports ChaCha20-Poly1305)
                if (crypto.subtle && this._isChaCha20Poly1305Supported()) {
                    const cryptoKey = await crypto.subtle.importKey(
                        'raw',
                        key,
                        { name: 'CHACHA20-POLY1305' },
                        false,
                        ['decrypt']
                    );
                    
                    const decrypted = await crypto.subtle.decrypt(
                        {
                            name: 'CHACHA20-POLY1305',
                            iv: new Uint8Array(header.nonce)
                        },
                        cryptoKey,
                        ciphertext
                    );
                    
                    return new Uint8Array(decrypted);
                } else {
                    // Fallback to JavaScript implementation
                    const cipher = new ChaCha20Poly1305(key);
                    const decrypted = await cipher.decrypt(new Uint8Array(header.nonce), ciphertext);
                    return new Uint8Array(decrypted);
                }
            } catch (error) {
                console.error('Decryption failed:', error);
                throw new Error(`WASM decryption failed: ${error.message}`);
            }
        },
        
        /**
         * Check if browser supports ChaCha20-Poly1305
         * @private
         * @returns {boolean} - Whether supported
         */
        _isChaCha20Poly1305Supported() {
            try {
                return crypto.subtle && 
                       typeof crypto.subtle.importKey === 'function' &&
                       typeof crypto.subtle.decrypt === 'function';
                // Note: This is not an accurate detection as we cannot determine if specific algorithms are supported
            } catch (e) {
                return false;
            }
        },
        
        /**
         * Convert Base64 string to Uint8Array
         * @private
         * @param {string} base64 - Base64 encoded string
         * @returns {Uint8Array} - Decoded byte array
         */
        _base64ToUint8Array(base64) {
            const binaryString = atob(base64);
            const bytes = new Uint8Array(binaryString.length);
            for (let i = 0; i < binaryString.length; i++) {
                bytes[i] = binaryString.charCodeAt(i);
            }
            return bytes;
        }
    };
    
    // Expose RusWaCipher to global object
    global.RusWaCipher = RusWaCipher;
    
})(typeof globalThis !== 'undefined' ? globalThis : 
   typeof window !== 'undefined' ? window : 
   typeof global !== 'undefined' ? global : 
   typeof self !== 'undefined' ? self : {});
"#.to_string()
}
