#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != Darwin || "$(uname -m)" != arm64 ]]; then
  echo "Run this script natively on an Apple Silicon Mac." >&2
  exit 1
fi
export PATH="$HOME/.cargo/bin:$PATH"
export MACOSX_DEPLOYMENT_TARGET=11.0

# Keep the legacy dependencies buildable without changing other build targets.
rust_toolchain=1.77.2
if ! rustup run "$rust_toolchain" rustc --version >/dev/null 2>&1; then
  echo "Install the build toolchain with: rustup toolchain install $rust_toolchain --profile minimal" >&2
  exit 1
fi

# Skip the legacy Tauri CLI's installer, which fetches an Intel executable.
npm exec --yes --package yarn@1.22.22 -- yarn install \
  --frozen-lockfile --ignore-engines --ignore-scripts --non-interactive
(
  cd desktop
  TAURI_BUILD=true NODE_OPTIONS="${NODE_OPTIONS:-} --openssl-legacy-provider" \
    node ../node_modules/@vue/cli-service/bin/vue-cli-service.js build
)
(
  cd desktop/src-tauri
  cargo +"$rust_toolchain" build --release --locked --target aarch64-apple-darwin
)

app="$PWD/releases/N-Link.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp desktop/src-tauri/target/aarch64-apple-darwin/release/n-link "$app/Contents/MacOS/n-link"
cp desktop/src-tauri/icons/icon.icns "$app/Contents/Resources/icon.icns"
cp LICENSE "$app/Contents/Resources/LICENSE"
python3 - "$app/Contents/Info.plist" <<'PY'
import json
import os
import plistlib
import sys
with open('desktop/package.json') as f:
    version = json.load(f)['version']
with open('desktop/src-tauri/tauri.conf.json') as f:
    bundle = json.load(f)['tauri']['bundle']
with open(sys.argv[1], 'wb') as f:
    plistlib.dump({
        'CFBundleDevelopmentRegion': 'en',
        'CFBundleDisplayName': 'N-Link',
        'CFBundleExecutable': 'n-link',
        'CFBundleIconFile': 'icon.icns',
        'CFBundleIdentifier': bundle['identifier'],
        'CFBundleInfoDictionaryVersion': '6.0',
        'CFBundleName': 'N-Link',
        'CFBundlePackageType': 'APPL',
        'CFBundleShortVersionString': version,
        'CFBundleVersion': version,
        'LSApplicationCategoryType': 'public.app-category.utilities',
        'LSMinimumSystemVersion': os.environ['MACOSX_DEPLOYMENT_TARGET'],
        'NSHighResolutionCapable': True,
        'NSHumanReadableCopyright': bundle['copyright'],
        'NSPrincipalClass': 'NSApplication',
    }, f)
PY
codesign --force --sign - --timestamp=none "$app"
codesign --verify --deep --strict "$app"
[[ "$(lipo -archs "$app/Contents/MacOS/n-link")" == arm64 ]]
"$app/Contents/MacOS/n-link" --version
version="$(node -p 'require("./desktop/package.json").version')"
archive="$PWD/releases/N-Link_${version}_macOS_arm64.zip"
ditto -c -k --sequesterRsrc --keepParent "$app" "$archive"
(
  cd releases
  shasum -a 256 "$(basename "$archive")" > "$(basename "$archive").sha256"
)
echo "Built: $app"
echo "Archive: $archive"
