/**
 * RusWaCipher - WASM Loader
 * Simplifies loading of encrypted WASM modules
 */
class WasmLoader {
    /**
     * Create a loader
     * @param {Object} options - Loading options
     * @param {string} options.runtimeUrl - Runtime JS file URL (optional, defaults to "ruswacipher-runtime.js")
     */
    constructor(options = {}) {
        this.options = Object.assign({
            runtimeUrl: 'ruswacipher-runtime.js'
        }, options);
        
        this.runtimeLoaded = false;
    }
    
    /**
     * Ensure runtime is loaded
     * @private
     * @returns {Promise<void>}
     */
    async _ensureRuntime() {
        if (this.runtimeLoaded) {
            return;
        }
        
        return new Promise((resolve, reject) => {
            const script = document.createElement('script');
            script.src = this.options.runtimeUrl;
            script.onload = () => {
                this.runtimeLoaded = true;
                resolve();
            };
            script.onerror = () => {
                reject(new Error(`Unable to load RusWaCipher runtime: ${this.options.runtimeUrl}`));
            };
            document.head.appendChild(script);
        });
    }
    
    /**
     * Load encrypted WASM module
     * @param {string} url - Encrypted WASM file URL
     * @param {string|Uint8Array} key - Decryption key (Base64 string or Uint8Array)
     * @param {Object} importObject - Import object to pass to WebAssembly
     * @returns {Promise<WebAssembly.Instance>} - WASM module instance
     */
    async load(url, key, importObject = {}) {
        await this._ensureRuntime();
        
        if (!window.RusWaCipher) {
            throw new Error('RusWaCipher runtime not loaded correctly');
        }
        
        return window.RusWaCipher.load(url, key, importObject);
    }
}

// Export WasmLoader
if (typeof module !== 'undefined' && typeof module.exports !== 'undefined') {
    module.exports = { WasmLoader };
} else {
    window.WasmLoader = WasmLoader;
}
