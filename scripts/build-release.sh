#!/bin/bash

# RusWaCipher Release Build Script
# Builds cross-platform binaries and packages for distribution

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="ruswacipher"
VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
BUILD_DIR="target/release-builds"
DIST_DIR="dist"

# Target platforms
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "x86_64-pc-windows-gnu"
    "x86_64-apple-darwin"
    "aarch64-unknown-linux-gnu"
    "aarch64-apple-darwin"
)

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

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if cargo is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust."
        exit 1
    fi
    
    # Check if cross is installed for cross-compilation
    if ! command -v cross &> /dev/null; then
        print_warning "Cross is not installed. Installing..."
        cargo install cross
    fi
    
    # Check if zip is available
    if ! command -v zip &> /dev/null; then
        print_error "zip command is not available. Please install zip."
        exit 1
    fi
    
    print_success "Prerequisites check completed"
}

# Clean previous builds
clean_builds() {
    print_status "Cleaning previous builds..."
    rm -rf "$BUILD_DIR"
    rm -rf "$DIST_DIR"
    mkdir -p "$BUILD_DIR"
    mkdir -p "$DIST_DIR"
    print_success "Cleaned previous builds"
}

# Build for a specific target
build_target() {
    local target=$1
    print_status "Building for target: $target"
    
    # Determine if we should use cross or cargo
    local build_cmd="cargo"
    if [[ "$target" != "$(rustc -vV | grep host | cut -d' ' -f2)" ]]; then
        build_cmd="cross"
    fi
    
    # Build the binary
    if $build_cmd build --release --target "$target"; then
        print_success "Built successfully for $target"
        
        # Copy binary to build directory
        local binary_name="$PROJECT_NAME"
        local binary_ext=""
        
        if [[ "$target" == *"windows"* ]]; then
            binary_ext=".exe"
        fi
        
        local source_path="target/$target/release/$binary_name$binary_ext"
        local dest_dir="$BUILD_DIR/$target"
        
        mkdir -p "$dest_dir"
        cp "$source_path" "$dest_dir/$binary_name$binary_ext"
        
        # Copy additional files
        cp README.md "$dest_dir/"
        cp LICENSE "$dest_dir/"
        
        # Create target-specific documentation
        cat > "$dest_dir/INSTALL.md" << EOF
# $PROJECT_NAME v$VERSION

## Installation

### Linux/macOS
1. Extract the archive
2. Copy the binary to a directory in your PATH:
   \`\`\`bash
   sudo cp $binary_name /usr/local/bin/
   \`\`\`

### Windows
1. Extract the archive
2. Add the directory containing $binary_name.exe to your PATH environment variable

## Usage

Run \`$binary_name --help\` for usage information.

## Documentation

- User Guide: https://github.com/lonless9/ruswacipher/blob/main/docs/user-guide.md
- API Reference: https://github.com/lonless9/ruswacipher/blob/main/docs/api-reference.md
- Security Guide: https://github.com/lonless9/ruswacipher/blob/main/docs/security-guide.md

## Support

- GitHub Issues: https://github.com/lonless9/ruswacipher/issues
- Documentation: https://github.com/lonless9/ruswacipher/docs
EOF
        
        return 0
    else
        print_error "Failed to build for $target"
        return 1
    fi
}

# Package binaries
package_binaries() {
    print_status "Packaging binaries..."
    
    for target in "${TARGETS[@]}"; do
        local build_path="$BUILD_DIR/$target"
        
        if [[ -d "$build_path" ]]; then
            print_status "Packaging $target..."
            
            local archive_name="$PROJECT_NAME-v$VERSION-$target"
            
            # Create archive
            cd "$BUILD_DIR"
            if [[ "$target" == *"windows"* ]]; then
                zip -r "../$DIST_DIR/$archive_name.zip" "$target/"
            else
                tar -czf "../$DIST_DIR/$archive_name.tar.gz" "$target/"
            fi
            cd - > /dev/null
            
            print_success "Packaged $target"
        else
            print_warning "No build found for $target, skipping packaging"
        fi
    done
}

# Build web runtime
build_web_runtime() {
    print_status "Building web runtime..."
    
    cd web
    
    # Install dependencies if needed
    if [[ ! -d "node_modules" ]]; then
        print_status "Installing Node.js dependencies..."
        npm install
    fi
    
    # Build production bundle
    npm run build
    
    # Copy built files to dist
    mkdir -p "../$DIST_DIR/web-runtime"
    cp dist/* "../$DIST_DIR/web-runtime/"
    cp wasmGuardianLoader.js "../$DIST_DIR/web-runtime/"
    cp README.md "../$DIST_DIR/web-runtime/"
    
    # Create web runtime package
    cd "../$DIST_DIR"
    zip -r "ruswacipher-web-runtime-v$VERSION.zip" web-runtime/
    cd - > /dev/null
    
    cd ..
    print_success "Built web runtime"
}

# Generate checksums
generate_checksums() {
    print_status "Generating checksums..."
    
    cd "$DIST_DIR"
    
    # Generate SHA256 checksums
    if command -v sha256sum &> /dev/null; then
        sha256sum *.tar.gz *.zip > "checksums-sha256.txt" 2>/dev/null || true
    elif command -v shasum &> /dev/null; then
        shasum -a 256 *.tar.gz *.zip > "checksums-sha256.txt" 2>/dev/null || true
    fi
    
    # Generate MD5 checksums
    if command -v md5sum &> /dev/null; then
        md5sum *.tar.gz *.zip > "checksums-md5.txt" 2>/dev/null || true
    elif command -v md5 &> /dev/null; then
        md5 *.tar.gz *.zip > "checksums-md5.txt" 2>/dev/null || true
    fi
    
    cd - > /dev/null
    print_success "Generated checksums"
}

# Create release notes
create_release_notes() {
    print_status "Creating release notes..."
    
    cat > "$DIST_DIR/RELEASE_NOTES.md" << EOF
# RusWaCipher v$VERSION Release

## Downloads

### CLI Tool

| Platform | Architecture | Download |
|----------|-------------|----------|
| Linux | x86_64 | [ruswacipher-v$VERSION-x86_64-unknown-linux-gnu.tar.gz](ruswacipher-v$VERSION-x86_64-unknown-linux-gnu.tar.gz) |
| Linux (musl) | x86_64 | [ruswacipher-v$VERSION-x86_64-unknown-linux-musl.tar.gz](ruswacipher-v$VERSION-x86_64-unknown-linux-musl.tar.gz) |
| Linux | ARM64 | [ruswacipher-v$VERSION-aarch64-unknown-linux-gnu.tar.gz](ruswacipher-v$VERSION-aarch64-unknown-linux-gnu.tar.gz) |
| Windows | x86_64 | [ruswacipher-v$VERSION-x86_64-pc-windows-gnu.zip](ruswacipher-v$VERSION-x86_64-pc-windows-gnu.zip) |
| macOS | x86_64 | [ruswacipher-v$VERSION-x86_64-apple-darwin.tar.gz](ruswacipher-v$VERSION-x86_64-apple-darwin.tar.gz) |
| macOS | ARM64 | [ruswacipher-v$VERSION-aarch64-apple-darwin.tar.gz](ruswacipher-v$VERSION-aarch64-apple-darwin.tar.gz) |

### Web Runtime

| Component | Download |
|-----------|----------|
| JavaScript Runtime | [ruswacipher-web-runtime-v$VERSION.zip](ruswacipher-web-runtime-v$VERSION.zip) |

## Verification

Verify downloads using the provided checksums:
- [SHA256 Checksums](checksums-sha256.txt)
- [MD5 Checksums](checksums-md5.txt)

## Installation

See the INSTALL.md file in each archive for platform-specific installation instructions.

## Documentation

- [User Guide](https://github.com/lonless9/ruswacipher/blob/main/docs/user-guide.md)
- [API Reference](https://github.com/lonless9/ruswacipher/blob/main/docs/api-reference.md)
- [Security Guide](https://github.com/lonless9/ruswacipher/blob/main/docs/security-guide.md)

## Changes

See [CHANGELOG.md](https://github.com/lonless9/ruswacipher/blob/main/CHANGELOG.md) for detailed changes.

## Support

- Report issues: [GitHub Issues](https://github.com/lonless9/ruswacipher/issues)
- Documentation: [GitHub Wiki](https://github.com/lonless9/ruswacipher/wiki)
EOF
    
    print_success "Created release notes"
}

# Main build process
main() {
    print_status "Starting release build for RusWaCipher v$VERSION"
    
    check_prerequisites
    clean_builds
    
    # Build for all targets
    local failed_targets=()
    for target in "${TARGETS[@]}"; do
        if ! build_target "$target"; then
            failed_targets+=("$target")
        fi
    done
    
    # Report failed builds
    if [[ ${#failed_targets[@]} -gt 0 ]]; then
        print_warning "Failed to build for targets: ${failed_targets[*]}"
    fi
    
    package_binaries
    build_web_runtime
    generate_checksums
    create_release_notes
    
    print_success "Release build completed!"
    print_status "Build artifacts are in the '$DIST_DIR' directory"
    
    # List generated files
    print_status "Generated files:"
    ls -la "$DIST_DIR"
}

# Run main function
main "$@"
