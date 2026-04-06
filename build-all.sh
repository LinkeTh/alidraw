#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/desktop"

for tool in npm cargo cargo-xwin clang-cl lld-link llvm-rc makensis; do
  if ! command -v "$tool" >/dev/null; then
    echo "Missing $tool. See the root README.md for release prerequisites." >&2
    exit 1
  fi
done

npm ci
npm run tauri build -- --no-bundle
npm run tauri build -- --target x86_64-pc-windows-msvc --runner cargo-xwin --bundles nsis
artifact_dir="$(cd "${CARGO_TARGET_DIR:-src-tauri/target}" && pwd)"
echo "Linux binary: $artifact_dir/release/alidraw-desktop"
echo "Windows binary: $artifact_dir/x86_64-pc-windows-msvc/release/alidraw-desktop.exe"
echo "Windows installer: $artifact_dir/x86_64-pc-windows-msvc/release/bundle/nsis/"
