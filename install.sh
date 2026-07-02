#!/usr/bin/env sh
# LIPI one-command installer for Linux / macOS.
#   curl -fsSL https://raw.githubusercontent.com/<you>/lipi-lang/main/install.sh | sh
#
# Builds from source with the default Rust toolchain and installs `lipi` to
# ~/.local/bin (override with LIPI_PREFIX). Requires: git, cargo, a C toolchain.
set -eu

REPO="${LIPI_REPO:-https://github.com/naraxcel/lipi-lang.git}"
PREFIX="${LIPI_PREFIX:-$HOME/.local/bin}"
WORK="${TMPDIR:-/tmp}/lipi-build-$$"

echo "LIPI installer"
echo "=============="

command -v cargo >/dev/null 2>&1 || { echo "error: cargo (Rust) not found — install from https://rustup.rs"; exit 1; }
command -v git   >/dev/null 2>&1 || { echo "error: git not found"; exit 1; }

echo "→ cloning $REPO"
git clone --depth 1 "$REPO" "$WORK"

echo "→ building (release) — this takes a minute"
( cd "$WORK" && cargo build --release )

mkdir -p "$PREFIX"
cp "$WORK/target/release/lipi" "$PREFIX/lipi"
chmod +x "$PREFIX/lipi"
rm -rf "$WORK"

echo
echo "✓ installed to $PREFIX/lipi"
case ":$PATH:" in
  *":$PREFIX:"*) : ;;
  *) echo "  add it to PATH:  export PATH=\"$PREFIX:\$PATH\"" ;;
esac
echo "  try:  lipi --help   (or)   echo 'बताओ \"नमस्ते\"' > hi.swami && lipi hi.swami"
