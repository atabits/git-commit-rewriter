#!/bin/bash

# Script to build Linux tarball and AppImage
# Usage: ./build-linux.sh
#
# Builds natively for the host architecture (x86_64 or arm64).
# Cross-compilation is not supported: run on a Linux machine or runner.

set -e

PROJECT_NAME="commit-rewriter"
APP_NAME="Git Commit Rewriter"

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
    VERSION=$(grep -m 1 "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/' | tr -d ' ')
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

if [ "$(uname -s)" != "Linux" ]; then
    error "This script must be run on Linux (cross-compilation is not supported)"
    exit 1
fi

# Detect architecture
case "$(uname -m)" in
    x86_64)
        ARCH_NAME="x86_64"
        ;;
    aarch64|arm64)
        ARCH_NAME="arm64"
        ;;
    *)
        error "Unsupported architecture: $(uname -m)"
        exit 1
        ;;
esac

# appimagetool uses its own architecture naming
if [ "$ARCH_NAME" = "arm64" ]; then
    APPIMAGE_ARCH="aarch64"
else
    APPIMAGE_ARCH="x86_64"
fi

info "Building for Linux ($ARCH_NAME), version $VERSION..."

# Check Rust
if ! command -v cargo &> /dev/null; then
    error "Rust is not installed! Install via rustup: https://rustup.rs/"
    exit 1
fi

info "Rust version: $(rustc --version)"

# Build project
info "Compiling project..."
cargo build --release --locked

BINARY_PATH="target/release/${PROJECT_NAME}"
DIST_DIR="releases"
BASE_NAME="${PROJECT_NAME}-${VERSION}-linux-${ARCH_NAME}"
TARBALL_NAME="${BASE_NAME}.tar.gz"
APPIMAGE_NAME="${BASE_NAME}.AppImage"

if [ ! -f "$BINARY_PATH" ]; then
    error "Binary not found: $BINARY_PATH"
    exit 1
fi

info "✅ Build completed: $BINARY_PATH"

mkdir -p "$DIST_DIR"
WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT

# Create tarball
info "Creating tarball..."
TAR_ROOT="$WORK_DIR/$BASE_NAME"
mkdir -p "$TAR_ROOT"
cp "$BINARY_PATH" "$TAR_ROOT/"
cp assets/linux/${PROJECT_NAME}.desktop "$TAR_ROOT/"
cp assets/linux/${PROJECT_NAME}.svg "$TAR_ROOT/"
cp LICENSE README.md "$TAR_ROOT/"
tar -czf "$DIST_DIR/$TARBALL_NAME" -C "$WORK_DIR" "$BASE_NAME"

info "✅ Tarball created: $DIST_DIR/$TARBALL_NAME"

# Create AppDir structure
info "Creating AppImage structure..."
APPDIR="$WORK_DIR/AppDir"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/scalable/apps"

cp "$BINARY_PATH" "$APPDIR/usr/bin/${PROJECT_NAME}"
chmod +x "$APPDIR/usr/bin/${PROJECT_NAME}"

cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/commit-rewriter" "$@"
EOF
chmod +x "$APPDIR/AppRun"

cp assets/linux/${PROJECT_NAME}.desktop "$APPDIR/"
cp assets/linux/${PROJECT_NAME}.desktop "$APPDIR/usr/share/applications/"
cp assets/linux/${PROJECT_NAME}.svg "$APPDIR/"
cp assets/linux/${PROJECT_NAME}.svg "$APPDIR/usr/share/icons/hicolor/scalable/apps/"

# Find or download appimagetool
APPIMAGETOOL="appimagetool"
if ! command -v "$APPIMAGETOOL" &> /dev/null; then
    info "Downloading appimagetool..."
    APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${APPIMAGE_ARCH}.AppImage"
    APPIMAGETOOL="$WORK_DIR/appimagetool.AppImage"

    if ! curl -fsSL "$APPIMAGETOOL_URL" -o "$APPIMAGETOOL"; then
        error "Failed to download appimagetool from $APPIMAGETOOL_URL"
        error "Install appimagetool manually and make sure it is in PATH"
        exit 1
    fi

    chmod +x "$APPIMAGETOOL"
fi

# Allow running without FUSE (e.g. in CI containers)
export APPIMAGE_EXTRACT_AND_RUN=1

info "Creating AppImage..."
ARCH="$APPIMAGE_ARCH" "$APPIMAGETOOL" --no-appstream "$APPDIR" "$DIST_DIR/$APPIMAGE_NAME"
chmod +x "$DIST_DIR/$APPIMAGE_NAME"

info "✅ AppImage created: $DIST_DIR/$APPIMAGE_NAME"

info "File sizes:"
ls -lh "$BINARY_PATH" | awk '{print $5, $9}'
ls -lh "$DIST_DIR/$TARBALL_NAME" | awk '{print $5, $9}'
ls -lh "$DIST_DIR/$APPIMAGE_NAME" | awk '{print $5, $9}'

info ""
info "🎉 Done! Files created:"
info "   - $DIST_DIR/$TARBALL_NAME"
info "   - $DIST_DIR/$APPIMAGE_NAME"
