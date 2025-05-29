#!/usr/bin/env node

/**
 * RusWaCipher Loader Obfuscation Script
 * 
 * This script applies various obfuscation techniques to the WasmGuardianLoader
 * to make reverse engineering more difficult.
 * 
 * Usage: node scripts/obfuscate-loader.js [options]
 * 
 * @version 0.1.0
 * @author RusWaCipher Project
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

class LoaderObfuscator {
    constructor(options = {}) {
        this.options = {
            inputFile: options.inputFile || 'web/wasmGuardianLoader.js',
            outputFile: options.outputFile || 'web/wasmGuardianLoader.obfuscated.js',
            level: options.level || 'basic', // 'basic', 'medium', 'advanced'
            addAntiDebug: options.addAntiDebug || false,
            addDomainLock: options.addDomainLock || false,
            allowedDomains: options.allowedDomains || ['localhost', '127.0.0.1'],
            addExpiration: options.addExpiration || false,
            expirationDate: options.expirationDate || null,
            ...options
        };
        
        this.stringMap = new Map();
        this.functionMap = new Map();
        this.variableMap = new Map();
    }

    /**
     * Main obfuscation method
     */
    async obfuscate() {
        console.log('🔒 Starting RusWaCipher loader obfuscation...');
        console.log(`📁 Input: ${this.options.inputFile}`);
        console.log(`📁 Output: ${this.options.outputFile}`);
        console.log(`🔧 Level: ${this.options.level}`);

        try {
            // Read source file
            const sourceCode = fs.readFileSync(this.options.inputFile, 'utf8');
            
            // Apply obfuscation techniques
            let obfuscatedCode = sourceCode;
            
            // Basic obfuscation
            if (this.options.level === 'basic' || this.options.level === 'medium' || this.options.level === 'advanced') {
                obfuscatedCode = this.basicObfuscation(obfuscatedCode);
            }
            
            // Medium obfuscation
            if (this.options.level === 'medium' || this.options.level === 'advanced') {
                obfuscatedCode = this.mediumObfuscation(obfuscatedCode);
            }
            
            // Advanced obfuscation
            if (this.options.level === 'advanced') {
                obfuscatedCode = this.advancedObfuscation(obfuscatedCode);
            }
            
            // Add protection measures
            obfuscatedCode = this.addProtectionMeasures(obfuscatedCode);
            
            // Write obfuscated file
            fs.writeFileSync(this.options.outputFile, obfuscatedCode);
            
            console.log('✅ Obfuscation completed successfully!');
            console.log(`📊 Original size: ${sourceCode.length} bytes`);
            console.log(`📊 Obfuscated size: ${obfuscatedCode.length} bytes`);
            console.log(`📊 Size increase: ${((obfuscatedCode.length / sourceCode.length - 1) * 100).toFixed(1)}%`);
            
        } catch (error) {
            console.error('❌ Obfuscation failed:', error.message);
            process.exit(1);
        }
    }

    /**
     * Basic obfuscation techniques
     */
    basicObfuscation(code) {
        console.log('🔧 Applying basic obfuscation...');
        
        // Remove comments and extra whitespace
        code = this.removeComments(code);
        code = this.minifyWhitespace(code);
        
        // Encode obvious strings
        code = this.encodeStrings(code);
        
        return code;
    }

    /**
     * Medium obfuscation techniques
     */
    mediumObfuscation(code) {
        console.log('🔧 Applying medium obfuscation...');
        
        // Rename variables and functions
        code = this.renameIdentifiers(code);
        
        // Add fake code paths
        code = this.addFakeCodePaths(code);
        
        return code;
    }

    /**
     * Advanced obfuscation techniques
     */
    advancedObfuscation(code) {
        console.log('🔧 Applying advanced obfuscation...');
        
        // Control flow flattening (simplified)
        code = this.flattenControlFlow(code);
        
        // Add opaque predicates
        code = this.addOpaquePredicates(code);
        
        return code;
    }

    /**
     * Remove comments from code
     */
    removeComments(code) {
        // Remove single-line comments
        code = code.replace(/\/\/.*$/gm, '');
        
        // Remove multi-line comments
        code = code.replace(/\/\*[\s\S]*?\*\//g, '');
        
        return code;
    }

    /**
     * Minify whitespace
     */
    minifyWhitespace(code) {
        // Replace multiple spaces with single space
        code = code.replace(/\s+/g, ' ');
        
        // Remove spaces around operators and punctuation
        code = code.replace(/\s*([{}();,=+\-*/<>!&|])\s*/g, '$1');
        
        // Remove leading/trailing whitespace
        code = code.trim();
        
        return code;
    }

    /**
     * Encode strings to make them less obvious
     */
    encodeStrings(code) {
        const stringRegex = /(['"`])((?:(?!\1)[^\\]|\\.)*)(\1)/g;
        const encodedStrings = [];
        
        code = code.replace(stringRegex, (match, quote, content, endQuote) => {
            if (content.length > 5) { // Only encode longer strings
                const encoded = Buffer.from(content).toString('base64');
                const index = encodedStrings.length;
                encodedStrings.push(encoded);
                return `_decode(${index})`;
            }
            return match;
        });
        
        // Add decoder function and string array at the beginning
        if (encodedStrings.length > 0) {
            const stringArray = `const _strings=['${encodedStrings.join("','")}'];`;
            const decoder = `const _decode=i=>atob(_strings[i]);`;
            code = stringArray + decoder + code;
        }
        
        return code;
    }

    /**
     * Rename identifiers to meaningless names
     */
    renameIdentifiers(code) {
        // Simple identifier renaming (this is a basic implementation)
        const identifiers = [
            '_resolveDecryptionKey', '_fetchKeyFromServer', '_deriveKey',
            '_generateClientFingerprint', '_makeKeyRequest', '_delay',
            'keyConfig', 'supportedAlgorithms', 'wasmDecryptorHelper'
        ];
        
        identifiers.forEach((identifier, index) => {
            const newName = `_${this.generateRandomName(3)}`;
            const regex = new RegExp(`\\b${identifier}\\b`, 'g');
            code = code.replace(regex, newName);
        });
        
        return code;
    }

    /**
     * Add fake code paths to confuse analysis
     */
    addFakeCodePaths(code) {
        const fakeCode = `
        if(Math.random()<0.001){
            const _fake=()=>{console.log('fake');return false;};
            if(_fake()){throw new Error('fake error');}
        }`;
        
        // Insert fake code at random positions
        const lines = code.split('\n');
        const insertPositions = Math.floor(lines.length / 10);
        
        for (let i = 0; i < insertPositions; i++) {
            const randomLine = Math.floor(Math.random() * lines.length);
            lines.splice(randomLine, 0, fakeCode);
        }
        
        return lines.join('\n');
    }

    /**
     * Simplified control flow flattening
     */
    flattenControlFlow(code) {
        // This is a very basic implementation
        // In practice, this would be much more sophisticated
        return code.replace(/if\s*\(([^)]+)\)\s*{([^}]+)}/g, (match, condition, body) => {
            return `(${condition})&&(()=>{${body}})();`;
        });
    }

    /**
     * Add opaque predicates (always true/false conditions)
     */
    addOpaquePredicates(code) {
        const predicates = [
            '(Math.floor(Math.random()*2)===0||true)',
            '(new Date().getTime()>0)',
            '(typeof window!=="undefined")'
        ];
        
        // Add random predicates before important operations
        code = code.replace(/throw new Error/g, (match) => {
            const predicate = predicates[Math.floor(Math.random() * predicates.length)];
            return `if(${predicate})${match}`;
        });
        
        return code;
    }

    /**
     * Add protection measures
     */
    addProtectionMeasures(code) {
        let protectionCode = '';
        
        // Anti-debugging
        if (this.options.addAntiDebug) {
            protectionCode += this.generateAntiDebugCode();
        }
        
        // Domain lock
        if (this.options.addDomainLock) {
            protectionCode += this.generateDomainLockCode();
        }
        
        // Expiration check
        if (this.options.addExpiration && this.options.expirationDate) {
            protectionCode += this.generateExpirationCode();
        }
        
        // Wrap original code in protection
        if (protectionCode) {
            code = `(function(){${protectionCode}${code}})();`;
        }
        
        return code;
    }

    /**
     * Generate anti-debugging code
     */
    generateAntiDebugCode() {
        return `
        setInterval(()=>{
            const start=performance.now();
            debugger;
            if(performance.now()-start>100){
                throw new Error('Debug detected');
            }
        },1000);
        `;
    }

    /**
     * Generate domain lock code
     */
    generateDomainLockCode() {
        const domains = this.options.allowedDomains.map(d => `'${d}'`).join(',');
        return `
        if(![${domains}].includes(location.hostname)){
            throw new Error('Unauthorized domain');
        }
        `;
    }

    /**
     * Generate expiration code
     */
    generateExpirationCode() {
        const timestamp = new Date(this.options.expirationDate).getTime();
        return `
        if(Date.now()>${timestamp}){
            throw new Error('Code expired');
        }
        `;
    }

    /**
     * Generate random identifier name
     */
    generateRandomName(length) {
        const chars = 'abcdefghijklmnopqrstuvwxyz';
        let result = '';
        for (let i = 0; i < length; i++) {
            result += chars.charAt(Math.floor(Math.random() * chars.length));
        }
        return result;
    }
}

// CLI interface
if (require.main === module) {
    const args = process.argv.slice(2);
    const options = {};
    
    // Parse command line arguments
    for (let i = 0; i < args.length; i += 2) {
        const key = args[i].replace(/^--/, '');
        const value = args[i + 1];
        
        switch (key) {
            case 'level':
                options.level = value;
                break;
            case 'input':
                options.inputFile = value;
                break;
            case 'output':
                options.outputFile = value;
                break;
            case 'anti-debug':
                options.addAntiDebug = value === 'true';
                break;
            case 'domain-lock':
                options.addDomainLock = value === 'true';
                break;
            case 'domains':
                options.allowedDomains = value.split(',');
                break;
            case 'expiration':
                options.addExpiration = true;
                options.expirationDate = value;
                break;
        }
    }
    
    const obfuscator = new LoaderObfuscator(options);
    obfuscator.obfuscate();
}

module.exports = LoaderObfuscator;
