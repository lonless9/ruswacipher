# WASM Build Guide - Avoiding Environment Variable Issues

This guide provides robust methods to build WebAssembly modules for RusWaCipher without encountering environment variable configuration issues or hanging builds.

## 🎯 Problem Solved

**Original Issue:** WASM compilation hanging at "Compiling to Wasm..." step due to:
- Conflicting RUSTFLAGS (`-C embed-bitcode=no` vs `-C lto`)
- Environment variable conflicts
- wasm-pack hanging during wasm-bindgen installation

**Solution:** Multiple fallback build methods with proper environment cleanup.

## 🚨 Common Issues and Solutions

### Issue: WASM Compilation Hanging

**Symptoms:**
- Build process hangs at "Compiling to Wasm..."
- `wasm-pack build` never completes
- Process appears frozen without error messages

**Root Causes:**
1. Conflicting environment variables
2. Network timeouts during dependency resolution
3. Incompatible linker configurations
4. Resource exhaustion during compilation

### Issue: Environment Variable Conflicts

**Symptoms:**
- Inconsistent build results
- "linker not found" errors
- Compilation errors related to target configuration

**Root Causes:**
1. `CARGO_TARGET_DIR` pointing to incompatible locations
2. `RUSTFLAGS` containing conflicting options
3. Cross-compilation variables interfering with WASM builds

## ⚡ Quick Start (Recommended)

```bash
# One command to solve everything
./scripts/quick-start.sh
```

This script automatically:
- ✅ Cleans conflicting environment variables
- ✅ Sets up WASM target
- ✅ Builds main project and WASM modules
- ✅ Tests encryption functionality
- ✅ Provides clear next steps

## 🛠️ Robust Build Solutions

### Method 1: Use the Robust Build Script (Recommended)

```bash
# Run the robust build script with multiple fallback methods
./scripts/build-wasm-robust.sh
```

This script:
- ✅ Cleans problematic environment variables
- ✅ Provides multiple build methods with timeouts
- ✅ Automatically falls back if one method fails
- ✅ Validates build results
- ✅ Handles both test WASM and helper modules

### Method 2: Use Makefile Commands

```bash
# Setup and build everything
make dev-workflow

# Or just build WASM robustly
make wasm-robust

# Or use standard method
make wasm
```

### Method 3: Manual Step-by-Step Build

```bash
# 1. Clean environment
unset CARGO_TARGET_DIR RUSTFLAGS CARGO_BUILD_TARGET CC CXX AR RANLIB

# 2. Setup WASM target
rustup target add wasm32-unknown-unknown

# 3. Set clean linker
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=rust-lld

# 4. Build with timeout
cd test-wasm
timeout 60 wasm-pack build --target web --out-dir pkg --release --no-typescript

# 5. Copy result
cp pkg/test_wasm_bg.wasm ../web/test.wasm
```

## 📁 Configuration Files

### `.cargo/config.toml`

This file provides consistent build configuration:

```toml
[target.wasm32-unknown-unknown]
linker = "rust-lld"
rustflags = ["-C", "opt-level=s", "-C", "lto=fat"]

[profile.release-wasm]
inherits = "release"
opt-level = "s"
lto = true
panic = "abort"
```

### Environment Variables to Avoid

**Never set these when building WASM:**
- `CARGO_TARGET_DIR` (can cause path conflicts)
- `RUSTFLAGS` (may contain incompatible flags)
- `CC`, `CXX`, `AR`, `RANLIB` (cross-compilation tools)
- `CARGO_BUILD_TARGET` (forces wrong target)

**Safe to set:**
- `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=rust-lld`
- `WASM_PACK_CACHE_DIR` (for custom cache location)

## 🔧 Troubleshooting

### Build Hangs at "Compiling to Wasm"

1. **Kill the process:** `Ctrl+C` or `pkill wasm-pack`
2. **Clean everything:** `make clean`
3. **Use robust script:** `./scripts/build-wasm-robust.sh`

### "Linker not found" Errors

1. **Install rust-lld:** `rustup component add llvm-tools-preview`
2. **Check target:** `rustup target list --installed | grep wasm32`
3. **Use config file:** Ensure `.cargo/config.toml` is present

### Network Timeout Issues

1. **Use offline mode:** `cargo build --offline` (after initial fetch)
2. **Increase timeout:** Edit `.cargo/config.toml` and set `timeout = 120`
3. **Use different registry:** Switch to sparse protocol in config

### Memory/Resource Issues

1. **Reduce parallel jobs:** `export CARGO_BUILD_JOBS=1`
2. **Use release profile:** `--release` flag for smaller memory usage
3. **Close other applications:** Free up system resources

## 🚀 Quick Start Commands

```bash
# Complete setup from scratch
make install-deps
make dev-workflow

# Just build WASM modules
make wasm-robust

# Test everything
make web-test

# Clean and rebuild
make clean
make wasm-robust
```

## 📊 Verification

After building, verify your WASM files:

```bash
# Check file exists and size
ls -la web/test.wasm

# Validate WASM format
file web/test.wasm

# Test with ruswacipher
cargo run -- encrypt -i web/test.wasm -o web/test.wasm.enc -a aes-gcm --generate-key web/test.key
```

## 🔍 Advanced Debugging

### Enable Verbose Output

```bash
# For wasm-pack
RUST_LOG=debug wasm-pack build --target web --out-dir pkg

# For cargo
cargo build --target wasm32-unknown-unknown --verbose
```

### Check Build Environment

```bash
# Show current environment
env | grep -E "(CARGO|RUST|CC|CXX)"

# Show Rust configuration
rustc --print cfg
rustup show
```

### Manual Dependency Resolution

```bash
# Pre-fetch dependencies
cargo fetch --target wasm32-unknown-unknown

# Build offline
cargo build --target wasm32-unknown-unknown --offline
```

## 📚 Additional Resources

- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [Cargo Configuration](https://doc.rust-lang.org/cargo/reference/config.html)

## 🆘 Getting Help

If you continue to experience issues:

1. **Check the logs:** Look for specific error messages
2. **Try different methods:** Use the robust script's fallback methods
3. **Update tools:** Ensure latest versions of Rust and wasm-pack
4. **Clean environment:** Start with a fresh shell session
