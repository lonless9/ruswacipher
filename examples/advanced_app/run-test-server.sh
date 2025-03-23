#!/bin/bash

echo "启动HTTP服务器用于性能测试..."
echo "请访问: http://localhost:8000/performance-test.html"

# 检查python版本并启动相应的HTTP服务器
if command -v python3 &> /dev/null; then
    python3 -m http.server 8000
elif command -v python &> /dev/null; then
    python -m SimpleHTTPServer 8000
else
    echo "错误: 未找到Python。请安装Python或使用其他HTTP服务器。"
    exit 1
fi 