/**
 * Mock Key Server for RusWaCipher Testing
 * 
 * This is a simple mock server implementation for testing key delivery mechanisms.
 * In production, this would be replaced with a proper backend service with
 * authentication, authorization, and security measures.
 * 
 * @version 0.1.0
 * @author RusWaCipher Project
 */

class MockKeyServer {
    constructor() {
        this.keys = new Map();
        this.accessLog = [];
        this.isRunning = false;
        
        // Default test keys
        this.initializeTestKeys();
        
        // Security settings
        this.settings = {
            requireAuth: false,
            maxRequestsPerMinute: 60,
            allowedOrigins: ['http://localhost', 'http://127.0.0.1'],
            keyExpirationTime: 3600000, // 1 hour in milliseconds
            enableFingerprinting: true
        };
        
        // Rate limiting
        this.requestCounts = new Map();
    }

    /**
     * Initialize test keys for development
     */
    initializeTestKeys() {
        // Test key for AES-GCM (32 bytes = 64 hex chars)
        this.keys.set('test-aes-key', {
            key: '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
            algorithm: 'aes-gcm',
            createdAt: Date.now(),
            expiresAt: Date.now() + 24 * 60 * 60 * 1000, // 24 hours
            accessCount: 0,
            metadata: {
                description: 'Test AES-GCM key for development',
                version: '1.0'
            }
        });

        // Test key for ChaCha20-Poly1305 (32 bytes = 64 hex chars)
        this.keys.set('test-chacha-key', {
            key: 'fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210',
            algorithm: 'chacha20poly1305',
            createdAt: Date.now(),
            expiresAt: Date.now() + 24 * 60 * 60 * 1000, // 24 hours
            accessCount: 0,
            metadata: {
                description: 'Test ChaCha20-Poly1305 key for development',
                version: '1.0'
            }
        });

        // Production-like key with restricted access
        this.keys.set('prod-demo-key', {
            key: 'a1b2c3d4e5f6789012345678901234567890abcdefabcdef1234567890123456',
            algorithm: 'aes-gcm',
            createdAt: Date.now(),
            expiresAt: Date.now() + 60 * 60 * 1000, // 1 hour
            accessCount: 0,
            maxAccess: 100,
            requiresAuth: true,
            metadata: {
                description: 'Production demo key with access restrictions',
                version: '2.0'
            }
        });
    }

    /**
     * Start the mock server (intercept fetch requests)
     */
    start() {
        if (this.isRunning) {
            console.warn('[MockKeyServer] Server is already running');
            return;
        }

        console.log('[MockKeyServer] Starting mock key server...');
        
        // Store original fetch
        this.originalFetch = window.fetch;
        
        // Intercept fetch requests
        window.fetch = async (url, options) => {
            if (this.shouldInterceptRequest(url, options)) {
                return await this.handleKeyRequest(url, options);
            }
            
            // Pass through other requests to original fetch
            return this.originalFetch(url, options);
        };
        
        this.isRunning = true;
        console.log('[MockKeyServer] Mock server started successfully');
        console.log('[MockKeyServer] Available test keys:', Array.from(this.keys.keys()));
    }

    /**
     * Stop the mock server
     */
    stop() {
        if (!this.isRunning) {
            console.warn('[MockKeyServer] Server is not running');
            return;
        }

        console.log('[MockKeyServer] Stopping mock key server...');
        
        // Restore original fetch
        if (this.originalFetch) {
            window.fetch = this.originalFetch;
        }
        
        this.isRunning = false;
        console.log('[MockKeyServer] Mock server stopped');
    }

    /**
     * Check if request should be intercepted
     */
    shouldInterceptRequest(url, options) {
        if (typeof url === 'string') {
            return url.includes('/api/keys') || url.endsWith('/api/keys');
        }
        
        if (url instanceof URL) {
            return url.pathname.includes('/api/keys');
        }
        
        return false;
    }

    /**
     * Handle key request
     */
    async handleKeyRequest(url, options) {
        const startTime = Date.now();
        
        try {
            // Simulate network delay
            await this.delay(50 + Math.random() * 100);
            
            // Parse request
            const requestData = await this.parseRequest(options);
            
            // Log request
            this.logRequest(url, requestData, startTime);
            
            // Validate request
            const validationResult = this.validateRequest(requestData);
            if (!validationResult.valid) {
                return this.createErrorResponse(400, validationResult.error);
            }
            
            // Check rate limiting
            if (!this.checkRateLimit(requestData.clientFingerprint)) {
                return this.createErrorResponse(429, 'Rate limit exceeded');
            }
            
            // Get key
            const keyData = this.getKey(requestData.keyId);
            if (!keyData) {
                return this.createErrorResponse(404, 'Key not found');
            }
            
            // Check key expiration
            if (keyData.expiresAt && Date.now() > keyData.expiresAt) {
                return this.createErrorResponse(410, 'Key expired');
            }
            
            // Check access limits
            if (keyData.maxAccess && keyData.accessCount >= keyData.maxAccess) {
                return this.createErrorResponse(403, 'Key access limit exceeded');
            }
            
            // Update access count
            keyData.accessCount++;
            keyData.lastAccessed = Date.now();
            
            // Create response
            const response = {
                key: keyData.key,
                algorithm: keyData.algorithm,
                expiresAt: keyData.expiresAt,
                metadata: keyData.metadata
            };
            
            return this.createSuccessResponse(response);
            
        } catch (error) {
            console.error('[MockKeyServer] Request handling error:', error);
            return this.createErrorResponse(500, 'Internal server error');
        }
    }

    /**
     * Parse request data
     */
    async parseRequest(options) {
        if (!options || !options.body) {
            throw new Error('Invalid request: missing body');
        }
        
        try {
            return JSON.parse(options.body);
        } catch (error) {
            throw new Error('Invalid request: malformed JSON');
        }
    }

    /**
     * Validate request
     */
    validateRequest(requestData) {
        if (!requestData.keyId) {
            return { valid: false, error: 'Missing keyId' };
        }
        
        if (!requestData.timestamp) {
            return { valid: false, error: 'Missing timestamp' };
        }
        
        // Check timestamp (allow 5 minutes skew)
        const timeDiff = Math.abs(Date.now() - requestData.timestamp);
        if (timeDiff > 5 * 60 * 1000) {
            return { valid: false, error: 'Request timestamp too old' };
        }
        
        if (this.settings.enableFingerprinting && !requestData.clientFingerprint) {
            return { valid: false, error: 'Missing client fingerprint' };
        }
        
        return { valid: true };
    }

    /**
     * Check rate limiting
     */
    checkRateLimit(fingerprint) {
        if (!fingerprint) return true;
        
        const now = Date.now();
        const windowStart = now - 60000; // 1 minute window
        
        if (!this.requestCounts.has(fingerprint)) {
            this.requestCounts.set(fingerprint, []);
        }
        
        const requests = this.requestCounts.get(fingerprint);
        
        // Remove old requests
        const recentRequests = requests.filter(time => time > windowStart);
        
        // Check limit
        if (recentRequests.length >= this.settings.maxRequestsPerMinute) {
            return false;
        }
        
        // Add current request
        recentRequests.push(now);
        this.requestCounts.set(fingerprint, recentRequests);
        
        return true;
    }

    /**
     * Get key by ID
     */
    getKey(keyId) {
        return this.keys.get(keyId);
    }

    /**
     * Create success response
     */
    createSuccessResponse(data) {
        return new Response(JSON.stringify(data), {
            status: 200,
            statusText: 'OK',
            headers: {
                'Content-Type': 'application/json',
                'X-Mock-Server': 'RusWaCipher-MockKeyServer'
            }
        });
    }

    /**
     * Create error response
     */
    createErrorResponse(status, message) {
        return new Response(JSON.stringify({ error: message }), {
            status,
            statusText: message,
            headers: {
                'Content-Type': 'application/json',
                'X-Mock-Server': 'RusWaCipher-MockKeyServer'
            }
        });
    }

    /**
     * Log request
     */
    logRequest(url, requestData, startTime) {
        const logEntry = {
            timestamp: new Date().toISOString(),
            url: url.toString(),
            keyId: requestData.keyId,
            clientFingerprint: requestData.clientFingerprint,
            userAgent: requestData.userAgent,
            processingTime: Date.now() - startTime
        };
        
        this.accessLog.push(logEntry);
        
        // Keep only last 1000 entries
        if (this.accessLog.length > 1000) {
            this.accessLog = this.accessLog.slice(-1000);
        }
        
        console.log('[MockKeyServer] Request:', logEntry);
    }

    /**
     * Utility delay function
     */
    delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    /**
     * Get server status and statistics
     */
    getStatus() {
        return {
            isRunning: this.isRunning,
            totalKeys: this.keys.size,
            totalRequests: this.accessLog.length,
            settings: this.settings,
            uptime: this.isRunning ? Date.now() - this.startTime : 0
        };
    }

    /**
     * Add a new key
     */
    addKey(keyId, keyData) {
        this.keys.set(keyId, {
            ...keyData,
            createdAt: Date.now(),
            accessCount: 0
        });
    }

    /**
     * Remove a key
     */
    removeKey(keyId) {
        return this.keys.delete(keyId);
    }

    /**
     * Get access log
     */
    getAccessLog() {
        return [...this.accessLog];
    }
}

// Export for both ES6 modules and CommonJS
if (typeof module !== 'undefined' && module.exports) {
    module.exports = MockKeyServer;
} else if (typeof window !== 'undefined') {
    window.MockKeyServer = MockKeyServer;
}
