#!/bin/bash

# Script to create .app and .dmg for macOS
# Usage: ./build-macos-app.sh [arm64|x86_64]

set -e

PROJECT_NAME="commit-rewriter"
APP_NAME="Git Commit Rewriter"
BUNDLE_ID="com.commitrewriter.app"

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

# Auto-detect architecture if not specified
if [ -z "$1" ]; then
    ARCH=$(uname -m)
    if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
        ARCH="arm64"
    elif [ "$ARCH" = "x86_64" ]; then
        ARCH="x86_64"
    else
        ARCH="arm64"  # Default to arm64
    fi
else
    ARCH=$1
fi

DIST_DIR="releases"

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
    TARGET="aarch64-apple-darwin"
    ARCH_NAME="arm64"
elif [ "$ARCH" = "x86_64" ]; then
    TARGET="x86_64-apple-darwin"
    ARCH_NAME="x86_64"
else
    error "Unknown architecture: $ARCH"
    error "Use: arm64 or x86_64"
    exit 1
fi

info "Building for macOS ($ARCH_NAME)..."

# Build project
info "Compiling project..."
cargo build --release --target "$TARGET"

BINARY_PATH="target/$TARGET/release/${PROJECT_NAME}"
APP_DIR="${DIST_DIR}/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

# Create releases directory
mkdir -p "$DIST_DIR"

# Create .app structure
info "Creating .app structure..."
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy binary
info "Copying binary..."
if [ ! -f "$BINARY_PATH" ]; then
    error "Binary not found: $BINARY_PATH"
    error "Make sure the build completed successfully"
    exit 1
fi
cp "$BINARY_PATH" "$MACOS_DIR/$APP_NAME"
chmod +x "$MACOS_DIR/$APP_NAME"

# Create Info.plist
info "Creating Info.plist..."
cat > "${CONTENTS_DIR}/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 Amin Atabiev. All rights reserved.</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
</dict>
</plist>
EOF

# Sign the application
info "Signing application..."
# Try to sign with ad-hoc signature (works without Apple Developer account)
if codesign --force --deep --sign - "$APP_DIR" 2>/dev/null; then
    info "✅ Application signed successfully"
    # Verify signature
    if codesign --verify --verbose "$APP_DIR" 2>/dev/null; then
        info "✅ Signature verified"
    else
        warn "⚠️  Signature verification failed, but continuing..."
    fi
else
    warn "⚠️  Failed to sign application, but continuing..."
fi

info "✅ .app bundle created: $APP_DIR"

# Create .dmg
info "Creating .dmg file..."
DMG_NAME="${DIST_DIR}/${PROJECT_NAME}-${VERSION}-macos-${ARCH_NAME}.dmg"
DMG_TEMP="${DIST_DIR}/temp_dmg"
DMG_TEMP_MOUNT="${DIST_DIR}/temp_dmg_mount"

# Remove old DMG if exists
rm -f "$DMG_NAME"
rm -rf "$DMG_TEMP"
rm -rf "$DMG_TEMP_MOUNT"

# Create temporary directory for DMG
mkdir -p "$DMG_TEMP"
cp -R "$APP_DIR" "$DMG_TEMP/"

# Create symbolic link to Applications
ln -s /Applications "$DMG_TEMP/Applications"

# Create temporary DMG
info "Creating disk image..."
if ! hdiutil create -volname "$APP_NAME" -srcfolder "$DMG_TEMP" -ov -format UDRW -fs HFS+ "${DMG_NAME}.temp.dmg" 2>/dev/null; then
    # If HFS+ failed, try without filesystem specification
    if ! hdiutil create -volname "$APP_NAME" -srcfolder "$DMG_TEMP" -ov -format UDRW "${DMG_NAME}.temp.dmg"; then
        error "Failed to create temporary DMG"
        rm -rf "$DMG_TEMP"
        exit 1
    fi
fi
if [ ! -f "${DMG_NAME}.temp.dmg" ]; then
    error "Temporary DMG file was not created: ${DMG_NAME}.temp.dmg"
    rm -rf "$DMG_TEMP"
    exit 1
fi

# Mount temporary DMG
info "Configuring disk image..."
if ! hdiutil attach "${DMG_NAME}.temp.dmg" -mountpoint "$DMG_TEMP_MOUNT" -nobrowse -quiet; then
    error "Failed to mount temporary DMG"
    rm -f "${DMG_NAME}.temp.dmg"
    rm -rf "$DMG_TEMP"
    exit 1
fi

# Configure DMG window appearance (optional, requires AppleScript)
if command -v osascript &> /dev/null; then
    info "Configuring DMG appearance..."
    osascript <<EOF 2>/dev/null || true
tell application "Finder"
    tell disk "$APP_NAME"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set bounds of container window to {400, 100, 900, 500}
        set viewOptions to icon view options of container window
        set arrangement of viewOptions to not arranged
        set icon size of viewOptions to 128
        set position of item "$APP_NAME.app" of container window to {200, 200}
        set position of item "Applications" of container window to {500, 200}
        close
        open
        update without registering applications
        delay 2
    end tell
end tell
EOF
fi

# Unmount
hdiutil detach "$DMG_TEMP_MOUNT" -quiet 2>/dev/null || true

# Convert to compressed format
info "Compressing disk image..."
TEMP_DMG="${DMG_NAME}.temp.dmg"
if [ ! -f "$TEMP_DMG" ]; then
    error "Temporary DMG file not found: $TEMP_DMG"
    hdiutil detach "$DMG_TEMP_MOUNT" -quiet 2>/dev/null || true
    rm -rf "$DMG_TEMP"
    rm -rf "$DMG_TEMP_MOUNT"
    exit 1
fi
if ! hdiutil convert "$TEMP_DMG" -format UDZO -o "$DMG_NAME" -quiet; then
    error "Failed to convert DMG to compressed format"
    hdiutil detach "$DMG_TEMP_MOUNT" -quiet 2>/dev/null || true
    rm -f "$TEMP_DMG"
    rm -rf "$DMG_TEMP"
    rm -rf "$DMG_TEMP_MOUNT"
    exit 1
fi

# Cleanup
rm -f "$TEMP_DMG"
rm -rf "$DMG_TEMP"
rm -rf "$DMG_TEMP_MOUNT"

# Remove quarantine attribute from DMG (helps with Gatekeeper)
info "Removing quarantine attribute..."
if xattr -d com.apple.quarantine "$DMG_NAME" 2>/dev/null; then
    info "✅ Quarantine attribute removed"
else
    warn "⚠️  Could not remove quarantine attribute (may not be set)"
fi

info "✅ DMG file created: $DMG_NAME"

# Show file sizes
info "File sizes:"
if [ -d "$APP_DIR" ]; then
    du -sh "$APP_DIR" | awk '{print $1 "\t" $2}'
fi
if [ -f "$DMG_NAME" ]; then
    ls -lh "$DMG_NAME" | awk '{print $5 "\t" $9}'
else
    warn "DMG file not found: $DMG_NAME"
fi

info ""
info "🎉 Done! Files created:"
info "   - ${APP_DIR}"
info "   - ${DMG_NAME}"

