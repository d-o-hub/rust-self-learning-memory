#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_PATH="$ROOT_DIR/target/release/memory-mcp-server"
DEBUG_BIN_PATH="$ROOT_DIR/target/debug/memory-mcp-server"

if [[ -x "$BIN_PATH" ]]; then
  exec "$BIN_PATH"
fi

if [[ -x "$DEBUG_BIN_PATH" ]]; then
  exec "$DEBUG_BIN_PATH"
fi

echo "[memory-mcp] release/debug binary not found, building release..." >&2
cargo build --release --bin memory-mcp-server --manifest-path "$ROOT_DIR/memory-mcp/Cargo.toml" >&2
exec "$BIN_PATH"
