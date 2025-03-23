# RusWaCipher 高级示例应用

这个高级示例展示了如何使用 RusWaCipher 加密和保护更复杂的 WebAssembly 模块，并在浏览器中安全地运行它们。

## 项目文件

- `advanced.rs` - 高级 Rust WASM 示例代码，包含多种功能
- `Cargo.toml` - 项目配置
- `index.html` - 交互式 Web 演示页面
- `build.sh` - 构建脚本

编译后，将生成以下文件：
- `advanced.wasm` - 原始 WASM 文件
- `advanced.encrypted.wasm` - 加密的 WASM 文件
- `advanced.wasm.key` - 加密密钥文件
- `advanced.wasm.key.meta` - 密钥元数据文件
- `web/` - 包含运行时和加载器的目录

## 高级功能

这个示例比简单示例包含更多高级功能：

1. **ChaCha20-Poly1305 加密** - 使用比 AES-GCM 更现代的另一种加密算法
2. **高级混淆** - 使用2级混淆保护代码
3. **多功能页面** - 包含多个功能演示的选项卡式界面

## 演示的功能

### 1. 计算器
高级计算器，支持加减乘除四则运算。

### 2. 密码强度检测器
评估密码强度并提供改进建议。评估考虑：
- 密码长度
- 是否包含小写字母
- 是否包含大写字母
- 是否包含数字
- 是否包含特殊字符

### 3. 文本加解密器
使用简单的 XOR 加密算法对文本进行加密和解密，演示了：
- 字符串在 JavaScript 和 WASM 之间的相互转换
- 复杂数据类型的处理
- 内存管理

### 4. JSON 验证器
验证 JSON 结构并提供格式化输出。

## 构建步骤

确保你已安装 wasm32-unknown-unknown 目标：

```bash
rustup target add wasm32-unknown-unknown
```

运行构建脚本：

```bash
chmod +x build.sh
./build.sh
```

## 运行示例

由于浏览器的安全限制，文件需要通过 HTTP 服务器提供。你可以使用任何 HTTP 服务器，例如：

```bash
# 如果你有 Python
python -m http.server

# 或使用 Node.js 的 http-server
npx http-server
```

然后在浏览器中访问：
```
http://localhost:8000/examples/advanced_app/index.html
```

## 技术特点

本示例展示了以下高级技术：

1. **密钥元数据** - 为密钥添加元数据信息，增强密钥管理
2. **内存管理与清理** - 展示了如何在 WebAssembly 中正确分配和释放内存
3. **字符串处理** - 演示 JavaScript 和 Rust/WebAssembly 之间的字符串转换
4. **复杂界面** - 包含选项卡式界面的现代 Web 应用示例
5. **动态组件** - 通过 JavaScript 动态生成界面元素

## 安全注意事项

这个示例使用 ChaCha20-Poly1305 算法进行加密，这是一种现代的高安全性加密算法。它具有以下特点：

- 在嵌入式和低功耗设备上表现良好
- 比 AES 更容易实现高性能的常量时间实现
- 具有较高的随机性和安全性

然而，重要的是要记住，与简单示例一样，浏览器端的解密意味着密钥必须可用于 JavaScript 运行时。对于生产环境，请考虑使用更复杂的密钥管理策略。 