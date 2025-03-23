# RusWaCipher

WebAssembly混淆和保护工具，用于增强WASM应用程序的安全性。

[![npm version](https://img.shields.io/npm/v/ruswacipher.svg)](https://www.npmjs.com/package/ruswacipher)
[![license](https://img.shields.io/npm/l/ruswacipher.svg)](https://github.com/yourusername/ruswacipher/blob/master/LICENSE)

## 特性

- 强大的WebAssembly代码混淆
- 多种保护技术（变量重命名、死代码插入、控制流混淆、函数分割、虚拟化）
- 可调整的安全级别（低/中/高）
- 命令行工具、Node.js API和Webpack集成

## 安装

```bash
npm install --save-dev ruswacipher
```

## 使用方法

### 命令行工具

```bash
# 基本用法
npx ruswacipher input.wasm

# 指定混淆级别
npx ruswacipher --level high input.wasm

# 保留原始文件
npx ruswacipher --preserve --output output.wasm input.wasm

# 查看所有选项
npx ruswacipher --help
```

### Node.js API

```javascript
const { obfuscateWasm, ObfuscationLevel } = require('ruswacipher');

// 混淆文件
obfuscateWasm('input.wasm', {
  level: ObfuscationLevel.High,
  preserveOriginal: true,
  outputPath: 'output.wasm'
});

// 混淆Buffer
const fs = require('fs');
const wasmBuffer = fs.readFileSync('input.wasm');
const obfuscatedBuffer = obfuscateWasmBuffer(wasmBuffer, ObfuscationLevel.High);
fs.writeFileSync('output.wasm', obfuscatedBuffer);
```

### Webpack集成

在`webpack.config.js`中添加:

```javascript
const { RuswacipherPlugin } = require('ruswacipher');

module.exports = {
  // ... other config
  module: {
    rules: [
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
        use: [
          'wasm-loader',
          {
            loader: 'ruswacipher/dist/webpack-loader',
            options: {
              level: 'high' // 'low', 'medium', 'high' 或 0, 1, 2
            }
          }
        ]
      }
    ]
  },
  plugins: [
    new RuswacipherPlugin({
      level: 'high',
      include: /\.wasm$/,
      exclude: /node_modules/,
      verbose: true
    })
  ],
  experiments: {
    asyncWebAssembly: true
  }
};
```

## 混淆级别

RusWaCipher提供三种混淆级别:

| 级别 | 技术 | 应用场景 |
|------|------|----------|
| 低（Low） | 变量重命名 | 轻度保护，最小的性能影响 |
| 中（Medium） | 变量重命名 + 死代码插入 + 控制流混淆 | 平衡的保护与性能 |
| 高（High） | 所有技术，包括函数分割和虚拟化 | 最大保护，会有一定性能影响 |

## 工作原理

RusWaCipher使用Rust实现的高性能混淆引擎，通过以下步骤保护WebAssembly代码:

1. **变量重命名** - 替换局部变量名称
2. **死代码插入** - 添加不会执行的代码片段
3. **控制流混淆** - 复杂化代码的控制流结构
4. **函数分割** - 将大函数拆分为多个小函数
5. **函数虚拟化** - 通过自定义虚拟机解释器保护关键函数逻辑

## 性能考虑

- **低级别混淆**：几乎没有性能影响，文件大小增加约5-10%
- **中级别混淆**：轻微的性能影响，文件大小增加约20-40%
- **高级别混淆**：可感知的性能影响，文件大小增加约50-100%

## 注意事项

- 虚拟化保护会明显增加文件大小和运行时开销
- 某些复杂的WebAssembly结构可能不兼容高级混淆
- 某些WebAssembly运行时可能与混淆后的代码不兼容

## 贡献

欢迎贡献！请提交问题报告或拉取请求。

## 许可证

MIT 