#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_BIN="$ROOT_DIR/target/release/memory-mcp-server"

if [[ ! -x "$RELEASE_BIN" ]]; then
  echo "[memory-mcp preflight] release binary missing; building memory-mcp..." >&2
  cargo build \
    --release \
    --package memory-mcp \
    --bin memory-mcp-server \
    --manifest-path "$ROOT_DIR/Cargo.toml" >&2
fi

exec "$RELEASE_BIN"
