#!/usr/bin/env bash
set -euo pipefail

APP_NAME="rusty-chess"
APP_VERSION="0.2.0"
ARCH="amd64"
PKG_DIR_NAME="${APP_NAME}_${APP_VERSION}_${ARCH}"
STAGING_ROOT="build-deb/${PKG_DIR_NAME}"
DEB_OUTPUT="${PKG_DIR_NAME}.deb"

echo "========================================================"
echo " Building ${APP_NAME} v${APP_VERSION} (${ARCH}) .deb Package"
echo "========================================================"

echo "[1/6] Running test suite (cargo test)..."
cargo test

echo "[2/6] Building optimized release binary (cargo build --release)..."
cargo build --release

echo "[3/6] Preparing Debian staging directory: ${STAGING_ROOT}..."
rm -rf "${STAGING_ROOT}"
mkdir -p "${STAGING_ROOT}/DEBIAN"
mkdir -p "${STAGING_ROOT}/usr/bin"
mkdir -p "${STAGING_ROOT}/usr/share/applications"
mkdir -p "${STAGING_ROOT}/usr/share/icons/hicolor/scalable/apps"

echo "[4/6] Copying release binary and setting permissions..."
cp "target/release/${APP_NAME}" "${STAGING_ROOT}/usr/bin/${APP_NAME}"
chmod 755 "${STAGING_ROOT}/usr/bin/${APP_NAME}"

echo "[5/6] Copying DEBIAN metadata, desktop launcher, and SVG icon..."
cp "packaging/debian/DEBIAN/control" "${STAGING_ROOT}/DEBIAN/control"
cp "packaging/debian/DEBIAN/postinst" "${STAGING_ROOT}/DEBIAN/postinst"
chmod 755 "${STAGING_ROOT}/DEBIAN/postinst"

cp "packaging/debian/usr/share/applications/${APP_NAME}.desktop" \
   "${STAGING_ROOT}/usr/share/applications/${APP_NAME}.desktop"
chmod 644 "${STAGING_ROOT}/usr/share/applications/${APP_NAME}.desktop"

cp "packaging/debian/usr/share/icons/hicolor/scalable/apps/${APP_NAME}.svg" \
   "${STAGING_ROOT}/usr/share/icons/hicolor/scalable/apps/${APP_NAME}.svg"
chmod 644 "${STAGING_ROOT}/usr/share/icons/hicolor/scalable/apps/${APP_NAME}.svg"

echo "[6/6] Packaging .deb archive with dpkg-deb..."
dpkg-deb --build --root-owner-group "${STAGING_ROOT}" "${DEB_OUTPUT}"

BIN_SIZE=$(du -h "target/release/${APP_NAME}" | cut -f1)
DEB_SIZE=$(du -h "${DEB_OUTPUT}" | cut -f1)

echo ""
echo "========================================================"
echo " Build Complete!"
echo " - Release Binary Size : ${BIN_SIZE} (target/release/${APP_NAME})"
echo " - Debian Package Size : ${DEB_SIZE} (${DEB_OUTPUT})"
echo ""
echo " Install with:"
echo "   sudo dpkg -i ${DEB_OUTPUT}"
echo "========================================================"
