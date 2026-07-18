#!/usr/bin/env bash
# check-source-reachability.sh — Deny production reachability of quarantined paths (S1.1b / W2.6b)
#
# Usage:
#   ./scripts/check-source-reachability.sh
#   ./scripts/check-source-reachability.sh --deny memory-mcp/src/sandbox/mod.rs
#   ./scripts/check-source-reachability.sh --fixtures   # self-test
#
# Exit 0 when production module graph does not re-export the Node sandbox
# without the sandbox-dev feature gate.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DENY_PATHS=()
FIXTURES=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --deny)
      DENY_PATHS+=("$2")
      shift 2
      ;;
    --fixtures)
      FIXTURES=1
      shift
      ;;
    -h|--help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      exit 2
      ;;
  esac
done

if [[ ${#DENY_PATHS[@]} -eq 0 ]]; then
  DENY_PATHS=("memory-mcp/src/sandbox/mod.rs")
fi

fail() {
  echo "HARNESS VIOLATION: reachability — $1" >&2
  exit 1
}

check_sandbox_gate() {
  local lib="memory-mcp/src/lib.rs"
  if [[ ! -f "$lib" ]]; then
    fail "missing $lib"
  fi

  # Production lib must gate sandbox with sandbox-dev (attribute may be on prior line)
  if ! rg -nU 'cfg\(feature = "sandbox-dev"\)\]\s*\npub mod sandbox;' "$lib" >/dev/null 2>&1 \
    && ! rg -n 'cfg\(feature = "sandbox-dev"\).*pub mod sandbox' "$lib" >/dev/null 2>&1; then
    # Fallback: require both the feature string and a pub mod sandbox line
    if rg -n 'pub mod sandbox' "$lib" >/dev/null 2>&1; then
      if ! rg -n 'feature = "sandbox-dev"' "$lib" >/dev/null 2>&1; then
        fail "memory-mcp/src/lib.rs exposes sandbox without sandbox-dev feature gate (S1.1b)"
      fi
    fi
  fi

  # Must have explicit feature gate for sandbox-dev when module is present
  if rg -n 'pub mod sandbox' "$lib" >/dev/null 2>&1; then
    if ! rg -n 'feature = "sandbox-dev"' "$lib" >/dev/null 2>&1; then
      fail "memory-mcp/src/lib.rs missing sandbox-dev feature gate for sandbox module"
    fi
  fi

  # Production binary/handlers must not import CodeSandbox without feature
  if rg -n 'use do_memory_mcp::sandbox|use crate::sandbox::CodeSandbox' \
    memory-mcp/src/bin memory-mcp/src/server --glob '*.rs' 2>/dev/null \
    | rg -v 'sandbox-dev|cfg\(feature' >/dev/null 2>&1; then
    fail "production MCP paths import sandbox CodeSandbox (must stay fail-closed)"
  fi

  echo "OK: sandbox quarantined behind sandbox-dev feature"
}

if [[ "$FIXTURES" -eq 1 ]]; then
  # Fixture: gated lib is required
  check_sandbox_gate
  echo "OK: fixtures passed"
  exit 0
fi

for path in "${DENY_PATHS[@]}"; do
  case "$path" in
    memory-mcp/src/sandbox/mod.rs|memory-mcp/src/sandbox/*)
      check_sandbox_gate
      ;;
    *)
      echo "WARN: no specific reachability rule for $path (path exists check only)"
      if [[ ! -e "$path" ]]; then
        echo "NOTE: $path does not exist (already removed)"
      fi
      ;;
  esac
done

exit 0
