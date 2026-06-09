#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Building validate-encoding (release)..."
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

RUST_TOOLS_DIR="$HOME/.config/opencode/rust-tools"
mkdir -p "$RUST_TOOLS_DIR"

cp "$SCRIPT_DIR/target/release/validate-encoding" "$RUST_TOOLS_DIR/validate-encoding"

cat > "$RUST_TOOLS_DIR/validate-encoding.description" << 'DESC'
Validate file encoding and detect mojibake (garbled text) in Nordic/European text files.
Accepts a file path and optional --encoding <enc> and --json flags.
Detects: invalid byte sequences, UTF-8→Latin-1 mojibake,
Latin-1→UTF-8 mojibake, replacement characters (U+FFFD), binary files.
DESC

echo "Installed to $RUST_TOOLS_DIR/validate-encoding"
echo "Restart opencode for the tool to be available."
