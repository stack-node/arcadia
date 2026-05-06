#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_NAME="Launcher"
BUNDLE_NAME="Arcadia Launcher"
BUNDLE_ID="com.stacknode.arcadia.development-launcher"
EXECUTABLE_NAME="ArcadiaDevelopmentLauncher"
APP_DIR="${SCRIPT_DIR}/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"
ICON_SOURCE="${SCRIPT_DIR}/../../../Resources/Icons/Production/Final-1-appicon-1024.png"
ICONSET_DIR="${SCRIPT_DIR}/.build/Launcher.iconset"
ICON_FILE="Launcher.icns"
STATUS_ICON_FILE="StatusIcon.png"
BIN_DIR="$(swift build -c release --package-path "${SCRIPT_DIR}" --show-bin-path)"

swift build -c release --package-path "${SCRIPT_DIR}"

rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

cp "${BIN_DIR}/${EXECUTABLE_NAME}" "${MACOS_DIR}/${EXECUTABLE_NAME}"
cp "${ICON_SOURCE}" "${RESOURCES_DIR}/${STATUS_ICON_FILE}"

rm -rf "${ICONSET_DIR}"
mkdir -p "${ICONSET_DIR}"
sips -z 16 16 "${ICON_SOURCE}" --out "${ICONSET_DIR}/icon_16x16.png" >/dev/null
sips -z 32 32 "${ICON_SOURCE}" --out "${ICONSET_DIR}/icon_16x16@2x.png" >/dev/null
sips -z 32 32 "${ICON_SOURCE}" --out "${ICONSET_DIR}/icon_32x32.png" >/dev/null
sips -z 64 64 "${ICON_SOURCE}" --out "${ICONSET_DIR}/icon_32x32@2x.png" >/dev/null
sips -z 128 128 "${ICON_SOURCE}" --out "${ICONSET_DIR}/icon_128x128.png" >/dev/null
sips -z 256 256 "${ICON_SOURCE}" --out "${ICONSET_DIR}/icon_128x128@2x.png" >/dev/null
sips -z 256 256 "${ICON_SOURCE}" --out "${ICONSET_DIR}/icon_256x256.png" >/dev/null
sips -z 512 512 "${ICON_SOURCE}" --out "${ICONSET_DIR}/icon_256x256@2x.png" >/dev/null
sips -z 512 512 "${ICON_SOURCE}" --out "${ICONSET_DIR}/icon_512x512.png" >/dev/null
cp "${ICON_SOURCE}" "${ICONSET_DIR}/icon_512x512@2x.png"
iconutil -c icns "${ICONSET_DIR}" -o "${RESOURCES_DIR}/${ICON_FILE}"

cat > "${CONTENTS_DIR}/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>${EXECUTABLE_NAME}</string>
  <key>CFBundleIdentifier</key>
  <string>${BUNDLE_ID}</string>
  <key>CFBundleIconFile</key>
  <string>${ICON_FILE}</string>
  <key>CFBundleName</key>
  <string>${BUNDLE_NAME}</string>
  <key>CFBundleDisplayName</key>
  <string>${BUNDLE_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.developer-tools</string>
  <key>LSUIElement</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

echo "Built ${APP_DIR}"
