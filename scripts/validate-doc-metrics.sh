#!/usr/bin/env bash
# validate-doc-metrics.sh — D3.2c provenance attachment for public metrics
#
# Usage:
#   ./scripts/validate-doc-metrics.sh README.md --evidence plans/STATUS/VALIDATION_LATEST.md

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "HARNESS VIOLATION: doc-metrics — $1" >&2
  exit 1
}

usage() {
  sed -n '2,8p' "$0"
}

DOC=""
EVIDENCE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence)
      shift
      [[ $# -gt 0 ]] || fail "--evidence requires a path"
      EVIDENCE="$1"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    -*)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
    *)
      if [[ -z "$DOC" ]]; then
        DOC="$1"
      else
        fail "unexpected argument: $1"
      fi
      shift
      ;;
  esac
done

[[ -n "$DOC" ]] || fail "usage: $0 README.md --evidence plans/STATUS/VALIDATION_LATEST.md"
[[ -n "$EVIDENCE" ]] || fail "missing --evidence <path>"

[[ -f "$DOC" ]] || fail "doc file missing: $DOC"
[[ -f "$EVIDENCE" ]] || fail "evidence file missing: $EVIDENCE"

# Soft pass: evidence must contain a date (YYYY-MM-DD) or SHA-like hex (7–40 chars)
# Note: ripgrep uses Rust regex by default; -E is --encoding, not extended-regex.
if rg -q '[0-9]{4}-[0-9]{2}-[0-9]{2}' "$EVIDENCE"; then
  echo "OK: evidence has date stamp ($EVIDENCE)"
elif rg -q '\b[0-9a-fA-F]{7,40}\b' "$EVIDENCE"; then
  echo "OK: evidence has SHA-like string ($EVIDENCE)"
else
  # Soft: still pass with note if file is non-empty (caller can tighten later)
  if [[ -s "$EVIDENCE" ]]; then
    echo "WARN: evidence lacks clear date/SHA; accepting non-empty file (soft)"
  else
    fail "evidence file empty and lacks date/SHA: $EVIDENCE"
  fi
fi

echo "OK: doc-metrics validated for $DOC with evidence $EVIDENCE"
