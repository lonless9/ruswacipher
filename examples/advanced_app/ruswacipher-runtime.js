/**
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
