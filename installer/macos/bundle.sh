#!/usr/bin/env bash
# bundle.sh — Build FTP Simulator macOS App Bundle + DMG (slim, ~5 MB)
# Run from the installer/macos/ directory.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
VERSION=$(grep '^version' "$REPO_ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/')
APP_NAME="FtpSimulator"
BUNDLE="$SCRIPT_DIR/$APP_NAME.app"
DIST="$REPO_ROOT/dist"

echo "==> Building macOS App Bundle v$VERSION"
mkdir -p "$DIST"

# Clean previous bundle
rm -rf "$BUNDLE"
mkdir -p "$BUNDLE/Contents/MacOS"
mkdir -p "$BUNDLE/Contents/Resources"

# Info.plist
cat > "$BUNDLE/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>             <string>FTP Simulator</string>
  <key>CFBundleIdentifier</key>       <string>com.ftpsimulator.app</string>
  <key>CFBundleVersion</key>          <string>${VERSION}</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleExecutable</key>       <string>launcher</string>
  <key>CFBundlePackageType</key>      <string>APPL</string>
  <key>LSUIElement</key>              <true/>  <!-- menu-bar only app (no dock icon) -->
  <key>NSHighResolutionCapable</key>  <true/>
</dict>
</plist>
PLIST

# Launcher script (starts services then tray)
TARGET="$REPO_ROOT/target/aarch64-apple-darwin/release"
[ -d "$TARGET" ] || TARGET="$REPO_ROOT/target/release"

cat > "$BUNDLE/Contents/MacOS/launcher" << 'SH'
#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
# Start backend
"$DIR/ftp-backend" &
# Launch tray
exec "$DIR/ftp-tray"
SH
chmod +x "$BUNDLE/Contents/MacOS/launcher"

# Copy binaries
cp "$TARGET/ftp-backend"          "$BUNDLE/Contents/MacOS/ftp-backend"
cp "$TARGET/ftp-tray"             "$BUNDLE/Contents/MacOS/ftp-tray"
cp "$TARGET/ftp-installer-helper" "$BUNDLE/Contents/MacOS/ftp-installer-helper"
chmod +x "$BUNDLE/Contents/MacOS/"*

# Create a minimal icon (placeholder)
echo "ICNS placeholder" > "$BUNDLE/Contents/Resources/AppIcon.icns"

# Build DMG
DMG="$DIST/${APP_NAME}-${VERSION}-macos.dmg"
echo "==> Creating DMG $DMG"
hdiutil create -volname "FTP Simulator $VERSION" \
    -srcfolder "$BUNDLE" \
    -ov -format UDZO \
    "$DMG"

echo "==> macOS build done: $DMG"
