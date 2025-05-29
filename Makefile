# Makefile for RusWaCipher
# Provides simple commands to avoid environment variable configuration issues

.PHONY: help build test bench clean wasm wasm-robust web-test install-deps

# Default target
help:
	@echo "RusWaCipher Build Commands"
	@echo "========================="
	@echo ""
	@echo "Main Commands:"
	@echo "  make build        - Build the main project"
	@echo "  make test         - Run all tests"
	@echo "  make bench        - Run benchmarks"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "WASM Commands:"
	@echo "  make wasm         - Build WASM modules (standard method)"
	@echo "  make wasm-robust  - Build WASM modules (robust method with fallbacks)"
	@echo "  make web-test     - Build and test web runtime"
	@echo ""
	@echo "Setup Commands:"
	@echo "  make install-deps - Install required dependencies"
	@echo "  make setup-wasm   - Setup WASM build environment"

# Build the main project
build:
	@echo "🦀 Building RusWaCipher..."
	cargo build --release

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Run benchmarks
bench:
	@echo "📊 Running benchmarks..."
	cargo bench

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf test-wasm/pkg test-wasm/target
	rm -rf wasm-decryptor-helper/pkg wasm-decryptor-helper/target
	rm -f web/test.wasm web/test.wasm.enc web/test*.key
	rm -f web/wasm-decryptor-helper.wasm web/wasm-decryptor-helper.js

# Install required dependencies
install-deps:
	@echo "📦 Installing dependencies..."
	@echo "Checking Rust installation..."
	@rustc --version || (echo "❌ Rust not installed. Install from https://rustup.rs/" && exit 1)
	@echo "Checking wasm-bindgen-cli..."
	@wasm-bindgen --version || (echo "Installing wasm-bindgen-cli..." && cargo install wasm-bindgen-cli)
	@echo "✅ Dependencies installed"

# Setup WASM build environment
setup-wasm:
	@echo "🔧 Setting up WASM environment..."
	rustup target add wasm32-unknown-unknown
	@echo "✅ WASM environment ready"

# Build WASM modules (standard method)
wasm: setup-wasm
	@echo "🌐 Building WASM modules..."
	cd test-wasm && ./build.sh
	@if [ -d "wasm-decryptor-helper" ]; then \
		echo "Building WASM decryption helper..."; \
		cd wasm-decryptor-helper && ./build.sh; \
	fi
	@echo "✅ WASM modules built"

# Build WASM modules (robust method with fallbacks)
wasm-robust: setup-wasm
	@echo "🛡️  Building WASM modules (robust method)..."
	./scripts/build-wasm-robust.sh

# Build and test web runtime
web-test: wasm-robust build
	@echo "🌐 Building and testing web runtime..."
	./scripts/test-web-runtime.sh

# Quick development build
dev:
	@echo "⚡ Quick development build..."
	cargo build

# Run specific benchmark
bench-crypto:
	@echo "🔐 Running crypto benchmarks..."
	cargo bench --bench crypto_benchmarks

bench-wasm:
	@echo "🌐 Running WASM benchmarks..."
	cargo bench --bench wasm_benchmarks

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt

# Check code with clippy
clippy:
	@echo "📎 Running clippy..."
	cargo clippy -- -D warnings

# Full CI pipeline
ci: fmt clippy test bench

# Development workflow
dev-workflow: clean install-deps setup-wasm build test wasm-robust
	@echo "🎉 Development environment ready!"

# Release build
release: clean build wasm-robust
	@echo "🚀 Release build completed"
	@echo "📁 Artifacts:"
	@echo "  - target/release/ruswacipher"
	@echo "  - web/test.wasm"
	@if [ -f "web/wasm-decryptor-helper.wasm" ]; then echo "  - web/wasm-decryptor-helper.wasm"; fi
