#!/usr/bin/env bash
set -euo pipefail

WINDOWS_TARGET="${WINDOWS_TARGET:-x86_64-pc-windows-gnu}"

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: '$cmd' is not installed or not on PATH" >&2
    exit 1
  fi
}

print_mingw_install_hint() {
  cat >&2 <<'EOF'
Install a MinGW cross-toolchain, then re-run:
  Debian/Ubuntu: sudo apt install gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64
  Fedora:        sudo dnf install mingw64-gcc mingw64-binutils
  Arch:          sudo pacman -S mingw-w64-gcc
EOF
}

require_cmd cargo
require_cmd rustup

target_installed=false
while IFS= read -r installed_target; do
  if [[ "$installed_target" == "$WINDOWS_TARGET" ]]; then
    target_installed=true
    break
  fi
done < <(rustup target list --installed)

if [[ "$target_installed" == false ]]; then
  echo "Installing Rust target: $WINDOWS_TARGET"
  rustup target add "$WINDOWS_TARGET"
fi

echo "Building Linux release binary..."
cargo build --release

echo "Checking Windows cross-compile tools..."
if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
  echo "error: missing linker 'x86_64-w64-mingw32-gcc' required for $WINDOWS_TARGET" >&2
  print_mingw_install_hint
  exit 1
fi

DLLTOOL_CMD="x86_64-w64-mingw32-dlltool"
if ! command -v "$DLLTOOL_CMD" >/dev/null 2>&1; then
  if command -v llvm-dlltool >/dev/null 2>&1; then
    DLLTOOL_CMD="llvm-dlltool"
    echo "warning: using fallback dlltool: $DLLTOOL_CMD"
  else
    echo "error: missing dlltool ('x86_64-w64-mingw32-dlltool' or 'llvm-dlltool')" >&2
    print_mingw_install_hint
    exit 1
  fi
fi

echo "Building Windows release binary ($WINDOWS_TARGET)..."
if [[ -n "${RUSTFLAGS:-}" ]]; then
  RUSTFLAGS="$RUSTFLAGS -Cdlltool=$DLLTOOL_CMD" cargo build --release --target "$WINDOWS_TARGET"
else
  RUSTFLAGS="-Cdlltool=$DLLTOOL_CMD" cargo build --release --target "$WINDOWS_TARGET"
fi

echo
echo "Build complete."
echo "Linux:   target/release/alidraw"
echo "Windows: target/$WINDOWS_TARGET/release/alidraw.exe"
