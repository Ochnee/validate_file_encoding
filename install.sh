#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Building validate-encoding (release)..."
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

RUST_TOOLS_DIR="$HOME/.config/rust-tools"
mkdir -p "$RUST_TOOLS_DIR"

cp "$SCRIPT_DIR/target/release/libvalidate_encoding.so" "$RUST_TOOLS_DIR/libvalidate_encoding.so"

echo "Installed to $RUST_TOOLS_DIR/libvalidate_encoding.so"
echo "Restart the MCP host (mcp-host) for the tool to be available."
