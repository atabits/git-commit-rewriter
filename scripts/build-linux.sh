#!/bin/bash

# Script to create Linux AppImage
# Usage: ./build-linux.sh [x86_64|arm64]

set -e

PROJECT_NAME="commit-rewriter"
APP_NAME="Git Commit Rewriter"

# Auto-detect architecture if not specified
if [ -z "$1" ]; then
    ARCH=$(uname -m)
    if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
        ARCH="arm64"
    elif [ "$ARCH" = "x86_64" ]; then
        ARCH="x86_64"
    else
        ARCH="x86_64"  # Default to x86_64
    fi
else
    ARCH=$1
fi

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

# Determine target based on architecture
if [ "$ARCH" = "arm64" ]; then
    TARGET="aarch64-unknown-linux-gnu"
    ARCH_NAME="arm64"
elif [ "$ARCH" = "x86_64" ]; then
    TARGET="x86_64-unknown-linux-gnu"
    ARCH_NAME="x86_64"
else
    error "Unknown architecture: $ARCH"
    error "Use: x86_64 or arm64"
    exit 1
fi

info "Building for Linux ($ARCH_NAME)..."

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

# Check and install cross-compiler for Linux (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    # Check for Linux cross-compiler
    if command -v x86_64-linux-gnu-gcc &> /dev/null; then
        info "✅ Found Linux cross-compiler"
        export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
    elif command -v x86_64-linux-musl-gcc &> /dev/null; then
        info "✅ Found musl cross-compiler (using for gnu target)"
        export CC_x86_64_unknown_linux_gnu=x86_64-linux-musl-gcc
    else
        warn "Cross-compiler not found. Attempting to install..."
        if command -v brew &> /dev/null; then
            # Try installing via crosstool-ng or other methods
            # For now, we'll try to use the system compiler with proper flags
            warn "No cross-compiler found. libz-sys will try to build from source."
            warn "This may require additional setup. Consider installing:"
            warn "  brew install filosottile/musl-cross/musl-cross"
            warn "  or install a proper Linux cross-compilation toolchain"
        else
            warn "Homebrew not found. Please install a cross-compiler manually"
        fi
    fi
fi

# Build project
info "Compiling project..."
# vendored-libgit2 feature will build everything from source, no need for system libs
cargo build --release --target "$TARGET"

BINARY_PATH="target/$TARGET/release/${PROJECT_NAME}"
DIST_DIR="releases"
APPIMAGE_NAME="${PROJECT_NAME}-${VERSION}-${ARCH_NAME}.AppImage"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    error "Binary not found: $BINARY_PATH"
    error "Make sure the build completed successfully"
    exit 1
fi

info "✅ Build completed: $BINARY_PATH"

# Create AppDir structure
APPDIR="releases/AppDir"
info "Creating AppImage structure..."
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

# Copy binary
cp "$BINARY_PATH" "$APPDIR/usr/bin/${PROJECT_NAME}"
chmod +x "$APPDIR/usr/bin/${PROJECT_NAME}"

# Create AppRun script
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/commit-rewriter" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# Create .desktop file
cat > "$APPDIR/usr/share/applications/${PROJECT_NAME}.desktop" << EOF
[Desktop Entry]
Type=Application
Name=${APP_NAME}
Comment=Git commit message rewriter with GUI
Exec=commit-rewriter
Icon=${PROJECT_NAME}
Categories=Development;VersionControl;
Terminal=false
EOF

# Create icon (optional - AppImage works without it)
if [ -f "assets/icon.png" ]; then
    cp "assets/icon.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/${PROJECT_NAME}.png"
    cp "assets/icon.png" "$APPDIR/${PROJECT_NAME}.png"
    info "Icon found and copied"
elif [ -f "icon.png" ]; then
    cp "icon.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/${PROJECT_NAME}.png"
    cp "icon.png" "$APPDIR/${PROJECT_NAME}.png"
    info "Icon found and copied"
else
    warn "No icon found - AppImage will work but without icon"
    warn "To add an icon, place icon.png in project root or assets/icon.png"
fi

# Download appimagetool if not available
APPIMAGETOOL="appimagetool"
if ! command -v "$APPIMAGETOOL" &> /dev/null; then
    info "Downloading appimagetool..."
    APPIMAGETOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-${ARCH_NAME}.AppImage"
    APPIMAGETOOL_TMP="/tmp/appimagetool-${ARCH_NAME}.AppImage"
    
    if ! wget -q "$APPIMAGETOOL_URL" -O "$APPIMAGETOOL_TMP" 2>/dev/null && \
       ! curl -sL "$APPIMAGETOOL_URL" -o "$APPIMAGETOOL_TMP" 2>/dev/null; then
        error "Failed to download appimagetool"
        error "Please install appimagetool manually:"
        error "  wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-${ARCH_NAME}.AppImage"
        error "  chmod +x appimagetool-${ARCH_NAME}.AppImage"
        error "  sudo mv appimagetool-${ARCH_NAME}.AppImage /usr/local/bin/appimagetool"
        exit 1
    fi
    
    chmod +x "$APPIMAGETOOL_TMP"
    APPIMAGETOOL="$APPIMAGETOOL_TMP"
fi

# Create AppImage
info "Creating AppImage..."
mkdir -p "$DIST_DIR"
ORIGINAL_DIR=$(pwd)

# Set environment variables for appimagetool
export ARCH="${ARCH_NAME}"
export VERSION="${VERSION}"

# Create AppImage (appimagetool needs to be run from outside AppDir)
OUTPUT_PATH="$ORIGINAL_DIR/$DIST_DIR/$APPIMAGE_NAME"

if "$APPIMAGETOOL" "$APPDIR" "$OUTPUT_PATH" 2>/dev/null; then
    info "✅ AppImage created successfully"
elif "$APPIMAGETOOL" "$APPDIR" "$OUTPUT_PATH"; then
    info "✅ AppImage created successfully"
else
    error "Failed to create AppImage"
    error "Make sure appimagetool is working correctly"
    rm -rf "$APPDIR"
    exit 1
fi

# Make AppImage executable
chmod +x "$DIST_DIR/$APPIMAGE_NAME"

# Cleanup
rm -rf "$APPDIR"
if [ -f "/tmp/appimagetool-${ARCH_NAME}.AppImage" ]; then
    rm -f "/tmp/appimagetool-${ARCH_NAME}.AppImage"
fi

info "✅ AppImage created: releases/$APPIMAGE_NAME"

# Show file sizes
info "File sizes:"
ls -lh "$BINARY_PATH" | awk '{print $5, $9}'
ls -lh "releases/$APPIMAGE_NAME" | awk '{print $5, $9}'

info ""
info "🎉 Done! Files created:"
info "   - releases/$APPIMAGE_NAME"
info ""
info "Usage:"
info "   chmod +x releases/$APPIMAGE_NAME"
info "   ./releases/$APPIMAGE_NAME"

