# RusWaCipher Security Guide

## Table of Contents

1. [Security Overview](#security-overview)
2. [Threat Model](#threat-model)
3. [Cryptographic Security](#cryptographic-security)
4. [Key Management](#key-management)
5. [Deployment Security](#deployment-security)
6. [Browser Security](#browser-security)
7. [Attack Vectors and Mitigations](#attack-vectors-and-mitigations)
8. [Security Checklist](#security-checklist)

## Security Overview

RusWaCipher provides encryption for WebAssembly modules to protect intellectual property and sensitive code. However, security depends heavily on proper implementation and deployment practices.

### Security Goals

1. **Confidentiality**: Protect WASM code from unauthorized access
2. **Integrity**: Ensure WASM code hasn't been tampered with
3. **Authentication**: Verify the source of encrypted WASM modules
4. **Access Control**: Restrict access to decryption keys

### Security Limitations

⚠️ **Important**: Client-side decryption has inherent limitations:

- Keys must be available to the client for decryption
- Determined attackers with sufficient resources can extract keys
- Browser security model provides limited protection
- This solution provides **obfuscation and deterrence**, not absolute security

## Threat Model

### Assets to Protect

1. **WASM Source Code**: The original unencrypted WebAssembly modules
2. **Decryption Keys**: Keys used to decrypt WASM modules
3. **Business Logic**: Algorithms and intellectual property in WASM
4. **User Data**: Data processed by WASM modules

### Threat Actors

1. **Casual Attackers**: Users trying to reverse-engineer for curiosity
2. **Competitors**: Organizations seeking to steal intellectual property
3. **Malicious Users**: Attackers trying to modify or inject code
4. **Automated Tools**: Bots and scrapers attempting mass extraction

### Attack Scenarios

1. **Static Analysis**: Analyzing encrypted WASM files offline
2. **Dynamic Analysis**: Intercepting decryption at runtime
3. **Key Extraction**: Extracting keys from browser memory or network
4. **Man-in-the-Middle**: Intercepting key delivery
5. **Social Engineering**: Tricking users into revealing keys

## Cryptographic Security

### Supported Algorithms

#### AES-GCM (Recommended)

- **Key Size**: 256 bits (32 bytes)
- **IV Size**: 96 bits (12 bytes)
- **Security Level**: High
- **Performance**: Excellent (hardware acceleration available)
- **Browser Support**: Universal via WebCrypto API

```javascript
// AES-GCM provides:
// - Confidentiality (encryption)
// - Integrity (authentication tag)
// - Resistance to chosen-plaintext attacks
```

#### ChaCha20-Poly1305 (Alternative)

- **Key Size**: 256 bits (32 bytes)
- **Nonce Size**: 96 bits (12 bytes)
- **Security Level**: High
- **Performance**: Good (software implementation)
- **Browser Support**: Via WASM helper module

### Cryptographic Best Practices

1. **Use Strong Keys**: Always generate keys using cryptographically secure random number generators
2. **Unique IVs/Nonces**: Never reuse IVs with the same key
3. **Key Rotation**: Implement regular key rotation for long-term deployments
4. **Algorithm Selection**: Prefer AES-GCM for most use cases

```rust
// Good: Cryptographically secure key generation
let key = ruswacipher::crypto::generate_key(&EncryptionAlgorithm::AesGcm)?;

// Bad: Predictable or weak keys
let weak_key = vec![0u8; 32]; // All zeros - never do this!
```

## Key Management

### Key Generation

```bash
# Generate strong keys using the CLI
ruswacipher encrypt -i input.wasm -o output.wasm --generate-key secure.key

# Verify key strength
hexdump -C secure.key | head -5
```

### Key Storage

#### Server-Side (Recommended)

```javascript
// Store keys on server, deliver on-demand
const loader = new WasmGuardianLoader({
    keyMethod: 'server',
    keyServerEndpoint: '/api/wasm-keys',
    authToken: 'bearer-token'
});
```

**Benefits:**
- Keys not exposed in client-side code
- Access control and logging possible
- Key rotation without client updates

**Implementation:**
```javascript
// Server endpoint example (Node.js/Express)
app.post('/api/wasm-keys', authenticateUser, (req, res) => {
    const { keyId, clientFingerprint } = req.body;
    
    // Validate request
    if (!isValidKeyRequest(req.user, keyId, clientFingerprint)) {
        return res.status(403).json({ error: 'Access denied' });
    }
    
    // Log access
    logKeyAccess(req.user.id, keyId, req.ip);
    
    // Return key
    const key = getKeyForModule(keyId);
    res.json({ key });
});
```

#### Client-Side (Development Only)

```javascript
// Only for development/testing
const loader = new WasmGuardianLoader({
    keyMethod: 'hardcoded'
});

await loader.loadEncryptedWasm('module.wasm.enc', 'hex-key-here');
```

⚠️ **Warning**: Never use hardcoded keys in production!

### Key Derivation (Future Feature)

```javascript
// Derive keys from user passwords
const loader = new WasmGuardianLoader({
    keyMethod: 'derived',
    derivationParams: {
        algorithm: 'PBKDF2',
        iterations: 100000,
        salt: 'unique-salt-per-user'
    }
});
```

## Deployment Security

### HTTPS Requirements

**Mandatory**: Always serve encrypted WASM over HTTPS.

```nginx
# Nginx configuration
server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    
    location /encrypted-wasm/ {
        # Serve encrypted WASM files
        add_header Cache-Control "private, no-cache";
    }
    
    location /api/wasm-keys {
        # Key delivery endpoint
        proxy_pass http://backend;
        add_header Cache-Control "no-store";
    }
}
```

### Content Security Policy

```html
<meta http-equiv="Content-Security-Policy" content="
    default-src 'self';
    script-src 'self' 'unsafe-eval';
    connect-src 'self' https://api.yourdomain.com;
    worker-src 'self' blob:;
">
```

### Access Control

```javascript
// Implement proper authentication
async function authenticateUser(req, res, next) {
    const token = req.headers.authorization?.replace('Bearer ', '');
    
    if (!token) {
        return res.status(401).json({ error: 'No token provided' });
    }
    
    try {
        const user = await verifyJWT(token);
        req.user = user;
        next();
    } catch (error) {
        return res.status(401).json({ error: 'Invalid token' });
    }
}
```

## Browser Security

### WebCrypto API Security

```javascript
// Verify WebCrypto availability
if (!window.crypto || !window.crypto.subtle) {
    throw new Error('WebCrypto API not available - ensure HTTPS');
}

// Use secure contexts only
if (!window.isSecureContext) {
    console.warn('Not in secure context - WebCrypto may be limited');
}
```

### Memory Protection

```javascript
// Clear sensitive data when possible
function clearSensitiveData(array) {
    if (array && array.fill) {
        array.fill(0);
    }
}

// Example usage
const key = await getDecryptionKey();
try {
    const decrypted = await decrypt(data, key);
    return decrypted;
} finally {
    clearSensitiveData(key);
}
```

### Subresource Integrity

```html
<!-- Verify loader integrity -->
<script src="wasmGuardianLoader.js" 
        integrity="sha384-hash-here"
        crossorigin="anonymous"></script>
```

## Attack Vectors and Mitigations

### 1. Network Interception

**Attack**: Intercepting encrypted WASM or keys during transmission.

**Mitigations**:
- Use HTTPS with strong TLS configuration
- Implement certificate pinning
- Use HSTS headers

### 2. Browser DevTools Analysis

**Attack**: Using browser developer tools to inspect network requests and memory.

**Mitigations**:
- Obfuscate key delivery mechanisms
- Implement anti-debugging techniques
- Use short-lived keys

### 3. Automated Scraping

**Attack**: Automated tools extracting keys or encrypted WASM.

**Mitigations**:
- Rate limiting on key endpoints
- CAPTCHA for suspicious requests
- User agent validation

### 4. Memory Dumps

**Attack**: Extracting keys from browser memory.

**Mitigations**:
- Clear keys after use
- Use short-lived keys
- Implement key rotation

### 5. Social Engineering

**Attack**: Tricking users into revealing keys or access credentials.

**Mitigations**:
- User education
- Multi-factor authentication
- Access logging and monitoring

## Security Checklist

### Development Phase

- [ ] Use cryptographically secure key generation
- [ ] Never hardcode keys in source code
- [ ] Implement proper error handling without information leakage
- [ ] Use secure coding practices
- [ ] Conduct security code review

### Deployment Phase

- [ ] Enable HTTPS with strong TLS configuration
- [ ] Implement proper authentication and authorization
- [ ] Set up security headers (CSP, HSTS, etc.)
- [ ] Configure rate limiting
- [ ] Set up access logging and monitoring

### Operational Phase

- [ ] Monitor for suspicious access patterns
- [ ] Implement key rotation procedures
- [ ] Keep dependencies updated
- [ ] Conduct regular security assessments
- [ ] Have incident response procedures

### Key Management

- [ ] Store keys securely on server-side
- [ ] Implement access controls for key retrieval
- [ ] Log all key access attempts
- [ ] Use unique keys per module/user when possible
- [ ] Implement key expiration and rotation

### Browser Security

- [ ] Verify WebCrypto API availability
- [ ] Implement proper error handling
- [ ] Clear sensitive data from memory
- [ ] Use Subresource Integrity for critical scripts
- [ ] Implement Content Security Policy

## Conclusion

RusWaCipher provides a reasonable level of protection against casual reverse engineering and automated extraction tools. However, it's important to understand that client-side decryption has fundamental limitations.

For maximum security:

1. **Defense in Depth**: Use multiple security layers
2. **Proper Key Management**: Implement server-side key delivery
3. **Regular Updates**: Keep all components updated
4. **Monitoring**: Implement comprehensive logging and monitoring
5. **Incident Response**: Have procedures for security incidents

Remember: Security is a process, not a product. Regular assessment and improvement of your security posture is essential.
