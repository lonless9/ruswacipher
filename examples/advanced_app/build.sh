#!/bin/bash

# 使用wat2wasm编译WAT文件到WASM
echo "编译WAT文件到WASM..."
wat2wasm wat_files/advanced.wat -o advanced.wasm

# 确保输出文件存在
if [ ! -f "advanced.wasm" ]; then
    echo "错误: WASM编译失败!"
    exit 1
fi

# 使用ruswacipher加密WASM文件
echo "加密WASM模块..."
cd ../../
cargo run -- encrypt -i examples/advanced_app/advanced.wasm -o examples/advanced_app/advanced.encrypted.wasm -a aes-gcm

# 确保密钥文件存在，并重命名为一致的名称
if [ -f "examples/advanced_app/advanced.encrypted.wasm.key" ]; then
    mv examples/advanced_app/advanced.encrypted.wasm.key examples/advanced_app/advanced.wasm.key
fi

# 生成运行时和加载器
echo "生成Web文件..."
cargo run -- generate-web -o examples/advanced_app/web -a aes-gcm

# 创建密钥的清单文件
echo "{\"keyAlgorithm\":\"aes-gcm\",\"keyType\":\"symmetric\",\"created\":\"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"}" > examples/advanced_app/advanced.wasm.key.meta

echo "构建完成!"
echo "密钥已保存至 examples/advanced_app/advanced.wasm.key"
echo "加密的WASM文件位于 examples/advanced_app/advanced.encrypted.wasm"
echo "启动HTTP服务器并访问 examples/advanced_app/index.html 测试示例!" 