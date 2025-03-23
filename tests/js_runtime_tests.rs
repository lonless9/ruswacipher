use anyhow::Result;
use ruswacipher::crypto::{encrypt_file, generate_key, save_key};
use ruswacipher::runtime::{generate_js_runtime, generate_web_files};
use std::fs;
use std::path::Path;

#[test]
fn test_js_runtime_generation() -> Result<()> {
    // Test generating JavaScript runtime for AES-GCM
    let runtime_path = Path::new("target/test-runtime-aes.js");
    generate_js_runtime(runtime_path, "aes-gcm")?;

    // Verify file was created
    assert!(
        runtime_path.exists(),
        "JavaScript runtime file should be created"
    );

    // Verify file content contains expected AES-GCM decryption code
    let content = fs::read_to_string(runtime_path)?;
    assert!(
        content.contains("RusWaCipher - WebAssembly Decryption Runtime (AES-GCM)"),
        "Runtime should contain AES-GCM comment"
    );
    assert!(
        content.contains("decrypt"),
        "Runtime should contain decryption function"
    );

    // Clean up
    fs::remove_file(runtime_path)?;

    // Test generating JavaScript runtime for ChaCha20Poly1305
    let runtime_path = Path::new("target/test-runtime-chacha.js");
    generate_js_runtime(runtime_path, "chacha20poly1305")?;

    // Verify file was created
    assert!(
        runtime_path.exists(),
        "JavaScript runtime file should be created"
    );

    // Verify file content contains expected ChaCha20Poly1305 decryption code
    let content = fs::read_to_string(runtime_path)?;
    assert!(
        content.contains("RusWaCipher - WebAssembly Decryption Runtime (ChaCha20-Poly1305)"),
        "Runtime should contain ChaCha20-Poly1305 comment"
    );

    // Clean up
    fs::remove_file(runtime_path)?;

    Ok(())
}

#[test]
fn test_web_files_generation() -> Result<()> {
    // Generate Web files for AES-GCM
    let web_dir = Path::new("target/test-web-files");

    // Ensure directory exists
    fs::create_dir_all(web_dir)?;

    // Generate Web files
    generate_web_files(web_dir, "aes-gcm")?;

    // Verify all necessary files are created
    let runtime_path = web_dir.join("ruswacipher-runtime.js");
    let loader_path = web_dir.join("loader.js");
    let html_path = web_dir.join("example.html");

    assert!(runtime_path.exists(), "Runtime file should be created");
    assert!(loader_path.exists(), "Loader file should be created");
    assert!(html_path.exists(), "HTML example file should be created");

    // Verify file content contains expected code
    let runtime_content = fs::read_to_string(&runtime_path)?;
    let loader_content = fs::read_to_string(&loader_path)?;
    let html_content = fs::read_to_string(&html_path)?;

    assert!(
        runtime_content.contains("RusWaCipher"),
        "Runtime should contain RusWaCipher namespace"
    );
    assert!(
        loader_content.contains("WasmLoader"),
        "Loader should contain WasmLoader class"
    );
    assert!(
        loader_content.contains("load(url, key"),
        "Loader should contain load method"
    );
    assert!(
        html_content.contains("<title>RusWaCipher Example</title>"),
        "HTML should contain correct title"
    );

    // Clean up
    fs::remove_dir_all(web_dir)?;

    Ok(())
}

#[test]
fn test_wasm_encryption_with_js_runtime() -> Result<()> {
    // Prepare paths
    let wasm_path = Path::new("tests/samples/simple.wasm");
    let encrypted_path = Path::new("target/test_js_encrypted.wasm");
    let key_path = Path::new("target/test_js_key.bin");
    let runtime_dir = Path::new("target/test_js_runtime");

    // Ensure WASM sample file exists
    assert!(wasm_path.exists(), "Test requires sample WASM file");

    // Ensure runtime directory exists
    fs::create_dir_all(runtime_dir)?;

    // Generate key and save it
    let key = generate_key(32);
    save_key(&key, key_path)?;

    // Encrypt WASM file
    encrypt_file(wasm_path, encrypted_path, Some(key_path), "aes-gcm")?;

    // Verify encrypted file was created
    assert!(encrypted_path.exists(), "Encrypted WASM file should exist");

    // Copy encrypted file to runtime directory, simplifying access
    let runtime_wasm_path = runtime_dir.join("encrypted.wasm");
    fs::copy(encrypted_path, &runtime_wasm_path)?;

    // Generate JavaScript runtime
    generate_web_files(runtime_dir, "aes-gcm")?;

    // Verify all necessary files are created
    let runtime_path = runtime_dir.join("ruswacipher-runtime.js");
    let loader_path = runtime_dir.join("loader.js");
    let html_path = runtime_dir.join("example.html");

    assert!(runtime_path.exists(), "Runtime file should be created");
    assert!(loader_path.exists(), "Loader file should be created");
    assert!(html_path.exists(), "HTML example file should be created");

    // Create a test HTML file, integrating encrypted WASM and key
    let test_html_path = runtime_dir.join("test.html");

    // Read key file content and ensure proper handling
    let key_base64 = fs::read_to_string(key_path)?;
    let key_base64 = key_base64.trim(); // Ensure any extra characters are removed

    // Create a simplified test HTML
    let test_html = r###"<!DOCTYPE html>
<html>
<head>
    <title>RusWaCipher JavaScript Runtime Test</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <script src="ruswacipher-runtime.js"></script>
    <script src="loader.js"></script>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 900px;
            margin: 0 auto;
            padding: 20px;
            line-height: 1.6;
        }
        h1, h2, h3 {
            color: #333;
        }
        pre {
            background-color: #f5f5f5;
            padding: 12px;
            border-radius: 4px;
            overflow: auto;
        }
        code {
            font-family: monospace;
            background-color: #f0f0f0;
            padding: 2px 4px;
            border-radius: 4px;
        }
        .container {
            display: flex;
            flex-wrap: wrap;
            gap: 20px;
        }
        .panel {
            flex: 1;
            min-width: 300px;
            padding: 15px;
            border: 1px solid #ddd;
            border-radius: 8px;
            margin-bottom: 20px;
        }
        .success {
            color: green;
            font-weight: bold;
        }
        .error {
            color: red;
            font-weight: bold;
        }
        button {
            background-color: #4CAF50;
            border: none;
            color: white;
            padding: 8px 16px;
            text-align: center;
            text-decoration: none;
            display: inline-block;
            font-size: 14px;
            margin: 4px 2px;
            cursor: pointer;
            border-radius: 4px;
        }
        button:disabled {
            background-color: #cccccc;
            cursor: not-allowed;
        }
        .function-container {
            margin-bottom: 10px;
            padding: 10px;
            border: 1px solid #eee;
            border-radius: 4px;
        }
        table {
            width: 100%;
            border-collapse: collapse;
        }
        th, td {
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }
        th {
            background-color: #f2f2f2;
        }
        #status {
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            background-color: #4CAF50;
            color: white;
            text-align: center;
            padding: 10px;
            display: none;
        }
    </style>
</head>
<body>
    <div id="status"></div>
    <h1>RusWaCipher JavaScript Runtime Test</h1>
    <p>This page demonstrates loading and executing encrypted WebAssembly modules.</p>
    
    <div class="container">
        <div class="panel">
            <h2>Module Information</h2>
            <div id="loading">
                <button id="loadButton">Load Encrypted WASM Module</button>
                <div id="loadStatus"></div>
            </div>
            <div id="moduleInfo" style="display: none;">
                <h3>Module Details</h3>
                <table>
                    <tr>
                        <th>Property</th>
                        <th>Value</th>
                    </tr>
                    <tr>
                        <td>File Name</td>
                        <td>encrypted.wasm</td>
                    </tr>
                    <tr>
                        <td>Decryption Time</td>
                        <td id="decryptionTime">-</td>
                    </tr>
                    <tr>
                        <td>Instantiation Time</td>
                        <td id="instantiationTime">-</td>
                    </tr>
                    <tr>
                        <td>Total Time</td>
                        <td id="totalTime">-</td>
                    </tr>
                </table>
                
                <h3>Exports</h3>
                <div id="exports"></div>
                
                <h3>Memory</h3>
                <div id="memory"></div>
            </div>
        </div>
        
        <div class="panel">
            <h2>Function Testing</h2>
            <div id="functionPanel" style="display: none;">
                <div id="functionList"></div>
            </div>
            <div id="functionNotAvailable">
                <p>Load a module to test its exported functions.</p>
            </div>
        </div>
    </div>
    
    <div class="panel">
        <h2>Console Output</h2>
        <pre id="console"></pre>
    </div>
    
    <script>
        // Encrypted WASM file path - using file in current directory
        const wasmPath = 'encrypted.wasm';
        
        // Decryption key
        const key = 'KEY_PLACEHOLDER';
        
        // Global variables
        let wasmInstance = null;
        let wasmMemory = null;
        
        // Console logging
        function log(message, type = 'info') {
            const console = document.getElementById('console');
            const date = new Date();
            const timestamp = `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}:${date.getSeconds().toString().padStart(2, '0')}.${date.getMilliseconds().toString().padStart(3, '0')}`;
            
            let className = '';
            if (type === 'success') className = 'success';
            if (type === 'error') className = 'error';
            
            console.innerHTML += `<div class="${className}">[${timestamp}] ${message}</div>`;
            console.scrollTop = console.scrollHeight;
        }
        
        // Show status message
        function showStatus(message, duration = 3000) {
            const status = document.getElementById('status');
            status.textContent = message;
            status.style.display = 'block';
            
            setTimeout(() => {
                status.style.display = 'none';
            }, duration);
        }
        
        // Display module information
        function displayModuleInfo(instance, metrics) {
            const moduleInfo = document.getElementById('moduleInfo');
            moduleInfo.style.display = 'block';
            
            document.getElementById('decryptionTime').textContent = `${metrics.decryptionTime} ms`;
            document.getElementById('instantiationTime').textContent = `${metrics.instantiationTime} ms`;
            document.getElementById('totalTime').textContent = `${metrics.totalTime} ms`;
            
            // Display exports
            const exportsDiv = document.getElementById('exports');
            exportsDiv.innerHTML = '';
            let exportTable = '<table><tr><th>Name</th><th>Type</th></tr>';
            
            // Function to get type of export
            function getExportType(exp) {
                if (typeof exp === 'function') return 'Function';
                if (exp instanceof WebAssembly.Memory) return 'Memory';
                if (exp instanceof WebAssembly.Table) return 'Table';
                if (exp instanceof WebAssembly.Global) return 'Global';
                return typeof exp;
            }
            
            for (const name in instance.exports) {
                const exp = instance.exports[name];
                const type = getExportType(exp);
                exportTable += `<tr><td>${name}</td><td>${type}</td></tr>`;
                
                // Save memory reference if available
                if (type === 'Memory') {
                    wasmMemory = exp;
                }
            }
            
            exportTable += '</table>';
            exportsDiv.innerHTML = exportTable;
            
            // Display memory info if available
            const memoryDiv = document.getElementById('memory');
            if (wasmMemory) {
                const pages = wasmMemory.buffer.byteLength / 65536;
                memoryDiv.innerHTML = `
                    <table>
                        <tr><th>Property</th><th>Value</th></tr>
                        <tr><td>Pages</td><td>${pages}</td></tr>
                        <tr><td>Size</td><td>${wasmMemory.buffer.byteLength.toLocaleString()} bytes</td></tr>
                    </table>
                `;
            } else {
                memoryDiv.innerHTML = '<p>No memory exports found.</p>';
            }
            
            // Display function panel
            displayFunctionPanel(instance);
        }
        
        // Display function testing panel
        function displayFunctionPanel(instance) {
            const functionPanel = document.getElementById('functionPanel');
            const functionNotAvailable = document.getElementById('functionNotAvailable');
            
            // Check if there are any exported functions
            let hasFunctions = false;
            for (const name in instance.exports) {
                if (typeof instance.exports[name] === 'function') {
                    hasFunctions = true;
                    break;
                }
            }
            
            if (hasFunctions) {
                functionPanel.style.display = 'block';
                functionNotAvailable.style.display = 'none';
                
                // Build function interfaces
                const functionList = document.getElementById('functionList');
                functionList.innerHTML = '';
                
                for (const name in instance.exports) {
                    const exp = instance.exports[name];
                    if (typeof exp === 'function') {
                        const functionContainer = document.createElement('div');
                        functionContainer.className = 'function-container';
                        
                        const functionHeader = document.createElement('h3');
                        functionHeader.textContent = name;
                        
                        const paramForm = document.createElement('div');
                        
                        // Create parameter inputs (we don't know the signature, so let's provide 3 inputs)
                        const paramCount = exp.length || 3; // Some WASM functions don't properly report length
                        
                        for (let i = 0; i < paramCount; i++) {
                            const inputGroup = document.createElement('div');
                            inputGroup.style.marginBottom = '10px';
                            
                            const label = document.createElement('label');
                            label.textContent = `Parameter ${i+1}: `;
                            label.htmlFor = `${name}_param_${i}`;
                            
                            const input = document.createElement('input');
                            input.type = 'text';
                            input.id = `${name}_param_${i}`;
                            input.placeholder = 'Enter value (number)';
                            
                            inputGroup.appendChild(label);
                            inputGroup.appendChild(input);
                            paramForm.appendChild(inputGroup);
                        }
                        
                        const resultDiv = document.createElement('div');
                        resultDiv.id = `${name}_result`;
                        resultDiv.style.marginTop = '10px';
                        
                        const callButton = document.createElement('button');
                        callButton.textContent = `Call ${name}()`;
                        callButton.onclick = function() {
                            try {
                                // Collect parameters
                                const params = [];
                                for (let i = 0; i < paramCount; i++) {
                                    const inputValue = document.getElementById(`${name}_param_${i}`).value;
                                    // Convert to number if possible
                                    params.push(inputValue === '' ? 0 : Number(inputValue));
                                }
                                
                                // Call the function
                                const startTime = performance.now();
                                const result = exp(...params);
                                const endTime = performance.now();
                                
                                // Display result
                                resultDiv.innerHTML = `
                                    <p><strong>Result:</strong> ${result}</p>
                                    <p><small>Execution time: ${(endTime - startTime).toFixed(3)} ms</small></p>
                                `;
                                
                                log(`Called ${name}(${params.join(', ')}) => ${result}`, 'success');
                            } catch (error) {
                                resultDiv.innerHTML = `<p class="error">Error: ${error.message}</p>`;
                                log(`Error calling ${name}: ${error.message}`, 'error');
                            }
                        };
                        
                        functionContainer.appendChild(functionHeader);
                        functionContainer.appendChild(paramForm);
                        functionContainer.appendChild(callButton);
                        functionContainer.appendChild(resultDiv);
                        
                        functionList.appendChild(functionContainer);
                    }
                }
            } else {
                functionPanel.style.display = 'none';
                functionNotAvailable.style.display = 'block';
                functionNotAvailable.innerHTML = '<p>No callable functions were exported from this module.</p>';
            }
        }
        
        // Load encrypted WASM module
        document.getElementById('loadButton').addEventListener('click', async () => {
            const loadButton = document.getElementById('loadButton');
            const loadStatus = document.getElementById('loadStatus');
            
            loadButton.disabled = true;
            loadStatus.innerHTML = '<p>Loading and decrypting module...</p>';
            
            try {
                log(`Starting to load encrypted WASM from ${wasmPath}`);
                log(`Using loader class: WasmLoader`);
                
                // Track timing
                const startTime = performance.now();
                
                // Create loader instance
                const loader = new WasmLoader();
                
                // Try to load encrypted WASM
                let decryptionEndTime;
                
                loader.onDecrypt = () => {
                    decryptionEndTime = performance.now();
                    log(`Module successfully decrypted in ${(decryptionEndTime - startTime).toFixed(2)} ms`);
                };
                
                // Call load with timing callbacks
                const instance = await loader.load(wasmPath, key);
                const endTime = performance.now();
                
                // Save instance for later use
                wasmInstance = instance;
                
                // Calculate metrics
                const metrics = {
                    decryptionTime: (decryptionEndTime - startTime).toFixed(2),
                    instantiationTime: (endTime - decryptionEndTime).toFixed(2),
                    totalTime: (endTime - startTime).toFixed(2)
                };
                
                // Update UI
                loadStatus.innerHTML = '<p class="success">Successfully loaded and decrypted WASM module!</p>';
                showStatus('Module loaded successfully!');
                log(`Module instantiated in ${metrics.instantiationTime} ms`, 'success');
                log(`Total loading time: ${metrics.totalTime} ms`, 'success');
                
                // Display module information
                displayModuleInfo(instance, metrics);
                
            } catch (error) {
                loadStatus.innerHTML = `<p class="error">Error: ${error.message}</p>`;
                log(`Failed to load module: ${error.message}`, 'error');
                loadButton.disabled = false;
            }
        });
        
        // Initialize
        document.addEventListener('DOMContentLoaded', () => {
            log('Test page loaded. Click "Load Encrypted WASM Module" to begin.');
        });
    </script>
</body>
</html>"###;

    // Replace key placeholder
    let test_html = test_html.replace("KEY_PLACEHOLDER", &key_base64);

    fs::write(&test_html_path, test_html)?;

    // Since we cannot run browsers in automated tests, we cannot verify the actual JavaScript decryption runtime
    // Here we only verify the generated files are correct, and actual testing needs to be done manually in a browser
    println!(
        "To test JavaScript decryption runtime in browser, open file: {}",
        test_html_path.display()
    );
    println!("Note: Opening file directly in browser may cause CORS errors. It's recommended to use a local web server:");
    println!("  1. cd {}", std::env::current_dir()?.display());
    println!("  2. python3 -m http.server 8000");
    println!(
        "  3. Access in browser: http://localhost:8000/{}",
        runtime_dir.display().to_string().replace("\\", "/")
    );

    // Clean up (commented out for manual testing)
    // fs::remove_file(encrypted_path)?;
    // fs::remove_file(key_path)?;
    // fs::remove_dir_all(runtime_dir)?;

    Ok(())
}
