#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_BIN="$ROOT_DIR/target/release/do-memory-mcp-server"

if [[ ! -x "$RELEASE_BIN" ]]; then
  echo "[do-memory-mcp preflight] release binary missing; building do-memory-mcp..." >&2
  cargo build \
    --release \
    --package do-memory-mcp \
    --bin do-memory-mcp-server \
    --manifest-path "$ROOT_DIR/Cargo.toml" >&2
fi

exec "$RELEASE_BIN"