#!/bin/bash

# Quick Start Script for RusWaCipher
# Solves environment variable issues and gets you up and running quickly

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 RusWaCipher Quick Start${NC}"
echo "=========================="
echo ""

# Step 1: Clean environment
echo -e "${BLUE}Step 1: Cleaning environment variables...${NC}"
unset CARGO_TARGET_DIR RUSTFLAGS CARGO_BUILD_TARGET CC CXX AR RANLIB CARGO_ENCODED_RUSTFLAGS
export RUSTFLAGS=""
echo -e "${GREEN}✅ Environment cleaned${NC}"
echo ""

# Step 2: Setup WASM target
echo -e "${BLUE}Step 2: Setting up WASM target...${NC}"
rustup target add wasm32-unknown-unknown
echo -e "${GREEN}✅ WASM target ready${NC}"
echo ""

# Step 3: Build main project
echo -e "${BLUE}Step 3: Building RusWaCipher...${NC}"
cargo build --release
echo -e "${GREEN}✅ RusWaCipher built${NC}"
echo ""

# Step 4: Build WASM modules
echo -e "${BLUE}Step 4: Building WASM modules...${NC}"
./scripts/build-wasm-robust.sh
echo -e "${GREEN}✅ WASM modules built${NC}"
echo ""

# Step 5: Test encryption
echo -e "${BLUE}Step 5: Testing encryption...${NC}"
./target/release/ruswacipher encrypt -i web/test.wasm -o web/test.wasm.enc -a aes-gcm --generate-key web/test.key
echo -e "${GREEN}✅ Encryption test completed${NC}"
echo ""

# Step 6: Show results
echo -e "${BLUE}🎉 Quick start completed successfully!${NC}"
echo ""
echo "Files created:"
echo "  📁 target/release/ruswacipher (main executable)"
echo "  🌐 web/test.wasm (test WASM module)"
echo "  🔒 web/test.wasm.enc (encrypted WASM)"
echo "  🔑 web/test.key (encryption key)"
echo ""
echo "Next steps:"
echo "  1. Run benchmarks: cargo bench"
echo "  2. Run tests: cargo test"
echo "  3. Test web runtime: ./scripts/test-web-runtime.sh"
echo ""
echo -e "${YELLOW}💡 Tip: Use 'make help' to see all available commands${NC}"
