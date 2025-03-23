#!/bin/bash

echo "Starting HTTP server for performance testing..."
echo "Please visit: http://localhost:8000/performance-test.html"

# Check Python version and start appropriate HTTP server
if command -v python3 &> /dev/null; then
    python3 -m http.server 8000
elif command -v python &> /dev/null; then
    python -m SimpleHTTPServer 8000
else
    echo "Error: Python not found. Please install Python or use another HTTP server."
    exit 1
fi 