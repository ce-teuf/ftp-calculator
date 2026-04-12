#!/usr/bin/env bash
# build-deb.sh — Build the ftp-simulator Debian package.
#
# Usage: bash installer/build-deb.sh [--version X.Y.Z]
#
# Requires: cargo, npm, dpkg-deb
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
DEB_STAGING="$SCRIPT_DIR/debian"
OUT_DIR="$REPO_ROOT/dist"

VERSION="${1:-$(grep '^version' "$REPO_ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/')}"
PACKAGE="ftp-simulator_${VERSION}_amd64"

echo "==> Building FTP Simulator v${VERSION}"

# ── 1. Build frontend ────────────────────────────────────────────────────────
echo "--- Building Svelte frontend"
cd "$REPO_ROOT/app/dashboard"
npm ci --silent
npm run build

# ── 2. Build backend (release, with embedded frontend) ────────────────────────
echo "--- Building Rust backend"
cd "$REPO_ROOT"
SQLX_OFFLINE=true cargo build --release -p ftp-backend

# ── 3. Assemble .deb staging tree ────────────────────────────────────────────
echo "--- Assembling package"

# Update version in control file
sed -i "s/^Version:.*/Version: ${VERSION}/" "$DEB_STAGING/DEBIAN/control"

# Place binaries
install -m 0755 "$REPO_ROOT/target/release/ftp-backend" "$DEB_STAGING/usr/bin/ftp-backend"

# Bundler le docker-compose.prod.yml (géré par systemd en prod)
install -m 0644 "$SCRIPT_DIR/docker-compose.prod.yml" \
                "$DEB_STAGING/usr/lib/ftp-simulator/docker-compose.prod.yml"

# Fix script permissions
chmod 0755 "$DEB_STAGING/DEBIAN/postinst"
chmod 0755 "$DEB_STAGING/DEBIAN/prerm"

# ── 5. Build .deb ────────────────────────────────────────────────────────────
mkdir -p "$OUT_DIR"
dpkg-deb --build --root-owner-group "$DEB_STAGING" "$OUT_DIR/${PACKAGE}.deb"

echo ""
echo "==> Package built: $OUT_DIR/${PACKAGE}.deb"
echo "    Install with:  sudo dpkg -i $OUT_DIR/${PACKAGE}.deb"
