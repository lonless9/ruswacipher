pub mod js_runtime;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Generate JavaScript runtime code for the specified algorithm and save to file
pub fn generate_js_runtime(output_path: &Path, algorithm: &str) -> Result<()> {
    let runtime_code = js_runtime::generate_runtime(algorithm)?;

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create output directory: {}", parent.display()))?;
    }

    fs::write(output_path, runtime_code)
        .with_context(|| format!("Cannot write runtime file: {}", output_path.display()))?;

    println!(
        "Successfully generated JavaScript runtime: {}",
        output_path.display()
    );
    Ok(())
}

/// Generate loader and runtime to web directory
pub fn generate_web_files(output_dir: &Path, algorithm: &str) -> Result<()> {
    // Generate runtime
    let runtime_path = output_dir.join("ruswacipher-runtime.js");
    generate_js_runtime(&runtime_path, algorithm)?;

    // Generate loader
    let loader_path = output_dir.join("loader.js");
    let loader_code = generate_loader_code();

    fs::write(&loader_path, loader_code)
        .with_context(|| format!("Cannot write loader file: {}", loader_path.display()))?;

    println!("Successfully generated loader: {}", loader_path.display());

    // Generate example HTML
    let html_path = output_dir.join("example.html");
    let html_code = generate_example_html(algorithm);

    fs::write(&html_path, html_code)
        .with_context(|| format!("Cannot write example HTML file: {}", html_path.display()))?;

    println!(
        "Successfully generated example HTML: {}",
        html_path.display()
    );

    Ok(())
}

/// Generate WASM loader code
fn generate_loader_code() -> String {
    r#"/**
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
"#.to_string()
}

/// Generate example HTML page
fn generate_example_html(algorithm: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RusWaCipher Example</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }}
        h1 {{
            color: #2c3e50;
            border-bottom: 1px solid #eee;
            padding-bottom: 10px;
        }}
        pre {{
            background-color: #f8f8f8;
            border-left: 4px solid #4CAF50;
            padding: 15px;
            overflow-x: auto;
        }}
        .note {{
            background-color: #e7f3fe;
            border-left: 6px solid #2196F3;
            padding: 10px;
            margin: 15px 0;
        }}
    </style>
</head>
<body>
    <h1>RusWaCipher Encrypted WASM Example</h1>
    
    <div class="note">
        <p>This page demonstrates how to load and use WebAssembly modules encrypted with RusWaCipher.</p>
        <p>Current encryption algorithm: <strong>{}</strong></p>
    </div>
    
    <h2>Instructions</h2>
    <ol>
        <li>Place your encrypted WASM file on the server</li>
        <li>Use the example code below to load it</li>
        <li>Open the browser console for detailed information</li>
    </ol>
    
    <h2>Example Code</h2>
    <pre><code>// Create loader
const loader = new WasmLoader();

// Load encrypted WASM module
loader.load('your-encrypted.wasm', 'your-key-in-base64')
    .then(instance => {{
        // Use WASM module instance
        console.log('WASM module loaded:', instance);
        
        // Call exported function
        const result = instance.exports.your_function();
        console.log('Function call result:', result);
    }})
    .catch(error => {{
        console.error('Loading failed:', error);
    }});
</code></pre>
    
    <script src="loader.js"></script>
    <script>
        // Demo code after page loads
        console.log('RusWaCipher example page loaded');
        console.log('To test encrypted WASM modules, follow the example code above');
    </script>
</body>
</html>
"#,
        algorithm
    )
}

/// 启动简单的HTTP服务器用于测试
/// 这将阻塞当前线程，直到用户中断（Ctrl+C）
#[cfg(feature = "http-server")]
pub fn start_test_server(directory: &Path, port: u16) -> Result<()> {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    let server_path = directory.to_path_buf();
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).with_context(|| format!("无法绑定到地址: {}", addr))?;

    println!("HTTP服务器已启动于 http://localhost:{}", port);
    println!("提供目录: {}", server_path.display());
    println!("按Ctrl+C停止服务器");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let path = server_path.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, &path) {
                        eprintln!("处理连接时出错: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("连接错误: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(feature = "http-server")]
fn handle_connection(mut stream: TcpStream, root_dir: &Path) -> Result<()> {
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    // 解析请求的路径
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let file_path = if path == "/" {
        root_dir.join("index.html")
    } else {
        root_dir.join(&path[1..]) // 去掉开头的'/'
    };

    let mut response = String::new();

    if file_path.exists() && file_path.is_file() {
        // 获取文件内容
        let content = fs::read(&file_path)?;

        // 确定内容类型
        let content_type = if let Some(ext) = file_path.extension() {
            match ext.to_str().unwrap_or("") {
                "html" => "text/html",
                "js" => "application/javascript",
                "wasm" => "application/wasm",
                "css" => "text/css",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                _ => "application/octet-stream",
            }
        } else {
            "application/octet-stream"
        };

        // 构建HTTP响应
        response.push_str("HTTP/1.1 200 OK\r\n");
        response.push_str(&format!("Content-Type: {}\r\n", content_type));
        response.push_str(&format!("Content-Length: {}\r\n", content.len()));
        response.push_str("Access-Control-Allow-Origin: *\r\n"); // 允许CORS
        response.push_str("\r\n");

        // 发送响应头
        stream.write_all(response.as_bytes())?;

        // 发送文件内容
        stream.write_all(&content)?;
    } else {
        // 文件不存在，返回404
        response.push_str("HTTP/1.1 404 NOT FOUND\r\n");
        response.push_str("Content-Type: text/plain\r\n");
        response.push_str("Content-Length: 9\r\n");
        response.push_str("\r\n");
        response.push_str("Not Found");

        stream.write_all(response.as_bytes())?;
    }

    stream.flush()?;
    Ok(())
}
