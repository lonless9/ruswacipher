#!/bin/bash

# Robust WASM Build Script for RusWaCipher
# This script provides multiple methods to build WASM modules without environment variable issues

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."

    local missing_tools=()

    if ! command_exists "rustc"; then
        missing_tools+=("rustc")
    fi

    if ! command_exists "cargo"; then
        missing_tools+=("cargo")
    fi

    if ! command_exists "wasm-bindgen"; then
        missing_tools+=("wasm-bindgen")
    fi

    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        print_status "Install missing tools:"
        for tool in "${missing_tools[@]}"; do
            case $tool in
                "rustc"|"cargo")
                    echo "  - Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    ;;
                "wasm-bindgen")
                    echo "  - Install wasm-bindgen-cli: cargo install wasm-bindgen-cli"
                    ;;
            esac
        done
        exit 1
    fi

    print_success "All prerequisites are installed"
}

# Function to setup Rust WASM target
setup_wasm_target() {
    print_status "Setting up WASM target..."

    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        print_status "Installing wasm32-unknown-unknown target..."
        rustup target add wasm32-unknown-unknown
        print_success "WASM target installed"
    else
        print_success "WASM target already installed"
    fi
}

# Function to clean environment variables that might cause issues
clean_environment() {
    print_status "Cleaning potentially problematic environment variables..."

    # Unset variables that might interfere with WASM compilation
    unset CARGO_TARGET_DIR
    unset RUSTFLAGS
    unset CARGO_BUILD_TARGET
    unset CC
    unset CXX
    unset AR
    unset RANLIB
    unset CARGO_ENCODED_RUSTFLAGS

    # Set clean environment for WASM compilation
    export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=rust-lld

    # Ensure no conflicting flags
    export RUSTFLAGS=""

    print_success "Environment cleaned"
}

# Function to build test WASM module with multiple fallback methods
build_test_wasm() {
    print_status "Building test WASM module..."

    cd test-wasm

    # Method 1: Direct cargo build (most reliable)
    print_status "Attempting Method 1: Direct cargo build..."
    if RUSTFLAGS="" timeout 60 cargo build --target wasm32-unknown-unknown 2>/dev/null; then
        print_success "Method 1 succeeded"
        copy_direct_build_files
        cd ..
        return 0
    else
        print_warning "Method 1 failed or timed out"
    fi

    # Method 2: Optimized cargo build
    print_status "Attempting Method 2: Optimized cargo build..."
    if RUSTFLAGS="-C opt-level=s" timeout 60 cargo build --target wasm32-unknown-unknown --release 2>/dev/null; then
        print_success "Method 2 succeeded"
        copy_direct_build_files
        cd ..
        return 0
    else
        print_warning "Method 2 failed or timed out"
    fi

    # Method 3: wasm-bindgen build (using cargo + wasm-bindgen)
    print_status "Attempting Method 3: wasm-bindgen build..."
    if timeout 60 cargo build --target wasm32-unknown-unknown --release 2>/dev/null; then
        mkdir -p pkg
        if timeout 30 wasm-bindgen target/wasm32-unknown-unknown/release/test_wasm.wasm --out-dir pkg --target web --no-typescript 2>/dev/null; then
            print_success "Method 3 succeeded"
            copy_wasm_files
            cd ..
            return 0
        else
            print_warning "Method 3 wasm-bindgen step failed"
        fi
    else
        print_warning "Method 3 cargo build failed"
    fi

    # Method 4: Last resort - basic cargo build
    print_status "Attempting Method 4: Basic cargo build..."
    if timeout 60 cargo build --target wasm32-unknown-unknown 2>/dev/null; then
        print_success "Method 4 succeeded"
        copy_direct_build_files
        cd ..
        return 0
    else
        print_error "All build methods failed"
        cd ..
        return 1
    fi
}

# Function to copy WASM files from wasm-pack build
copy_wasm_files() {
    if [ -f "pkg/test_wasm_bg.wasm" ]; then
        cp pkg/test_wasm_bg.wasm ../web/test.wasm
        print_success "Copied test.wasm to web directory"
    else
        print_error "WASM file not found in pkg directory"
        return 1
    fi
}

# Function to copy WASM files from direct cargo build
copy_direct_build_files() {
    local wasm_file="target/wasm32-unknown-unknown/release/test_wasm.wasm"
    if [ ! -f "$wasm_file" ]; then
        wasm_file="target/wasm32-unknown-unknown/debug/test_wasm.wasm"
    fi

    if [ -f "$wasm_file" ]; then
        cp "$wasm_file" ../web/test.wasm
        print_success "Copied test.wasm to web directory"
    else
        print_error "WASM file not found in target directory"
        return 1
    fi
}

# Function to build WASM decryption helper
build_wasm_helper() {
    print_status "Building WASM decryption helper..."

    if [ ! -d "wasm-decryptor-helper" ]; then
        print_warning "WASM decryption helper directory not found, skipping..."
        return 0
    fi

    cd wasm-decryptor-helper

    # Try the existing build script first
    if [ -f "build.sh" ] && timeout 60 ./build.sh 2>/dev/null; then
        print_success "WASM helper built using existing script"
        cd ..
        return 0
    fi

    # Fallback to direct cargo + wasm-bindgen build
    print_status "Fallback: Direct cargo + wasm-bindgen build for helper..."
    if timeout 60 cargo build --target wasm32-unknown-unknown --release 2>/dev/null; then
        mkdir -p pkg
        if timeout 30 wasm-bindgen target/wasm32-unknown-unknown/release/wasm_decryptor_helper.wasm --out-dir pkg --target web --no-typescript 2>/dev/null; then
            # Copy files manually
            cp pkg/wasm_decryptor_helper.js ../web/wasm-decryptor-helper.js 2>/dev/null || true
            cp pkg/wasm_decryptor_helper_bg.wasm ../web/wasm-decryptor-helper.wasm 2>/dev/null || true
            print_success "WASM helper built successfully"
        else
            print_warning "WASM helper wasm-bindgen step failed, continuing without it..."
        fi
    else
        print_warning "WASM helper cargo build failed, continuing without it..."
    fi

    cd ..
}

# Function to verify build results
verify_build() {
    print_status "Verifying build results..."

    if [ -f "web/test.wasm" ]; then
        local size=$(stat -c%s "web/test.wasm" 2>/dev/null || stat -f%z "web/test.wasm" 2>/dev/null || echo "unknown")
        print_success "test.wasm created successfully (size: $size bytes)"

        # Basic WASM validation
        if command_exists "file" && file "web/test.wasm" | grep -q "WebAssembly"; then
            print_success "WASM file format validated"
        elif [ "${size}" != "unknown" ] && [ "$size" -gt 8 ]; then
            print_success "WASM file appears valid (size check passed)"
        else
            print_warning "Could not validate WASM file format"
        fi
    else
        print_error "test.wasm was not created"
        return 1
    fi

    if [ -f "web/wasm-decryptor-helper.wasm" ]; then
        print_success "WASM decryption helper created successfully"
    else
        print_warning "WASM decryption helper not created (optional)"
    fi
}

# Main execution
main() {
    echo "🦀 Robust WASM Build Script for RusWaCipher"
    echo "==========================================="

    check_prerequisites
    setup_wasm_target
    clean_environment

    # Build test WASM
    if build_test_wasm; then
        print_success "Test WASM build completed"
    else
        print_error "Test WASM build failed"
        exit 1
    fi

    # Build WASM helper (optional)
    build_wasm_helper

    # Verify results
    verify_build

    print_success "🎉 WASM build process completed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Test the WASM file: cargo run -- encrypt -i web/test.wasm -o web/test.wasm.enc -a aes-gcm --generate-key web/test.key"
    echo "2. Run web tests: ./scripts/test-web-runtime.sh"
    echo "3. Open web/test.html in a browser to test decryption"
}

# Run main function
main "$@"
