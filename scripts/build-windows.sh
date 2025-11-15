#!/bin/bash

# Script to create .exe and archive for Windows (cross-compilation from macOS/Linux)
# Usage: ./build-windows.sh

set -e

PROJECT_NAME="commit-rewriter"
APP_NAME="Git Commit Rewriter"
TARGET="x86_64-pc-windows-gnu"

# Determine project root
if [ -f "Cargo.toml" ]; then
    PROJECT_ROOT="."
elif [ -f "../Cargo.toml" ]; then
    PROJECT_ROOT=".."
    cd "$PROJECT_ROOT"
else
    echo "Error: Cannot find Cargo.toml"
    exit 1
fi

# Get version from environment variable or Cargo.toml
if [ -z "$VERSION" ]; then
    if [ -f "Cargo.toml" ]; then
        VERSION=$(grep -m 1 "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/' | tr -d ' ')
    elif [ -f "../Cargo.toml" ]; then
        VERSION=$(grep -m 1 "^version" ../Cargo.toml | sed 's/version = "\(.*\)"/\1/' | tr -d ' ')
    else
        VERSION="2.0.0"
    fi
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

info "Building for Windows (cross-compilation)..."

# Check Rust
if ! command -v rustc &> /dev/null; then
    error "Rust is not installed! Install via rustup: https://rustup.rs/"
    exit 1
fi

info "Rust version: $(rustc --version)"

# Install target platform
if rustup target list --installed | grep -q "$TARGET"; then
    info "Target platform $TARGET already installed"
else
    info "Installing target platform $TARGET..."
    rustup target add "$TARGET"
fi

# Check MinGW (for macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        warn "MinGW not found. Install: brew install mingw-w64"
        warn "Creating cross-compilation configuration..."
        mkdir -p .cargo
        cat > .cargo/config.toml << EOF
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
EOF
    fi
fi

# Build project (Windows subsystem without console)
info "Compiling project (Windows GUI, no console)..."
RUSTFLAGS="-C link-arg=-Wl,--subsystem,windows" cargo build --release --target "$TARGET"

BINARY_PATH="target/$TARGET/release/${PROJECT_NAME}.exe"
EXE_NAME="${APP_NAME}.exe"
DIST_DIR="releases"
ZIP_NAME="${PROJECT_NAME}-${VERSION}-windows-portable.zip"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    error "Binary not found: $BINARY_PATH"
    exit 1
fi

info "✅ Build completed: $BINARY_PATH"

# Create temporary directory for packaging
TEMP_DIR="releases/temp-windows"
info "Creating distribution..."
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"

# Copy binary
cp "$BINARY_PATH" "$TEMP_DIR/$EXE_NAME"
chmod +x "$TEMP_DIR/$EXE_NAME"

# Create README for Windows
cat > "$TEMP_DIR/README.txt" << EOF
Git Commit Rewriter - Windows Edition
=====================================

Version: $VERSION

Installation:
-------------
1. Extract the archive
2. Run ${EXE_NAME}
3. (Optional) Add to PATH to run from command line

Requirements:
-------------
- Windows 10 or newer
- Visual C++ Redistributable (usually already installed)

Usage:
------
Run ${EXE_NAME} by double-clicking or from command line.

License:
--------
MIT License

Copyright (c) 2024 Amin Atabiev
EOF

# Create ZIP archive
info "Creating ZIP archive..."
mkdir -p "$DIST_DIR"
ORIGINAL_DIR=$(pwd)
cd "$TEMP_DIR"
zip -r "$ORIGINAL_DIR/$DIST_DIR/$ZIP_NAME" . > /dev/null
cd "$ORIGINAL_DIR"

# Cleanup temporary directory
rm -rf "$TEMP_DIR"

info "✅ ZIP archive created: releases/$ZIP_NAME"

# Show file sizes
info "File sizes:"
ls -lh "$BINARY_PATH" | awk '{print $5, $9}'
ls -lh "releases/$ZIP_NAME" | awk '{print $5, $9}'

info ""
info "🎉 Done! Files created:"
info "   - releases/$ZIP_NAME"

