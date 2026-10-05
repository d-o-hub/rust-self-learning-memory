#!/usr/bin/env bash
# test-docs-integrity.sh — fixtures for check-docs-integrity.sh link scanning.
#
# Guards two regressions found on 2026-10-04:
#   F1 angle-bracket-wrapped URLs (the AGENTS.md-mandated style) were reported as
#      broken relative paths, which blocked `release-manager.sh ship`.
#   F2 under `set -euo pipefail` a link failure aborted the whole script, so the
#      script-reference / version-sync / fail-closed phases never ran.
#
# Usage:
#   ./scripts/test-docs-integrity.sh --fixtures

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:---fixtures}"

fail() {
  echo "HARNESS VIOLATION: docs-integrity — $1" >&2
  exit 1
}

[[ "$MODE" == "--fixtures" ]] || fail "unknown mode: $MODE (expected --fixtures)"

CHECK="$ROOT/scripts/check-docs-integrity.sh"
[[ -x "$CHECK" ]] || fail "check-docs-integrity.sh missing or not executable"

command -v git >/dev/null || fail "git required"

make_fixture_repo() {
  local dir="$1"
  rm -rf "$dir"
  mkdir -p "$dir"
  # Cargo.toml gates the script's early exit; the checks under test ignore it.
  printf '[workspace.package]\nversion = "0.0.0"\n' > "$dir/Cargo.toml"
  printf '# exists\n' > "$dir/exists.md"
  cat > "$dir/guide.md" <<'MD'
# Guide

An external reference written the mandated way: [upstream](<https://example.com/verify-a-release.html>).

A quoted SUMMARY contract, literal rather than live: `- [Verify a Release](./verify-a-release.md)`.

A real relative link: [exists](./exists.md).
MD
  git -C "$dir" init -q
  git -C "$dir" add -A
  git -C "$dir" -c user.email=t@example.com -c user.name=t commit -qm fixture
}

make_broken_repo() {
  local dir="$1"
  make_fixture_repo "$dir"
  printf 'A genuinely broken link: [nope](./missing-target.md).\n' > "$dir/bad.md"
  git -C "$dir" add -A
  git -C "$dir" -c user.email=t@example.com -c user.name=t commit -qm broken
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# F1 — clean tree: none of the mandated-style or quoted links may be flagged.
CLEAN="$WORK/clean"
make_fixture_repo "$CLEAN"
clean_out="$("$CHECK" --root "$CLEAN" 2>&1 || true)"
printf '%s\n' "$clean_out" | grep -q "Markdown link check passed" \
  || fail "clean fixture did not pass the link check: $clean_out"
if printf '%s\n' "$clean_out" | grep -q "^BROKEN:"; then
  fail "false-positive broken link reported: $(printf '%s\n' "$clean_out" | grep '^BROKEN:')"
fi
echo "OK: angle-bracket URLs and backtick-quoted links are not flagged"

# F2 — broken link: still reported, and the later phases must still run.
BROKEN="$WORK/broken"
make_broken_repo "$BROKEN"
broken_out="$("$CHECK" --root "$BROKEN" 2>&1 || true)"
printf '%s\n' "$broken_out" | grep -q "^BROKEN:bad.md:1:./missing-target.md" \
  || fail "real broken link was not reported: $broken_out"
printf '%s\n' "$broken_out" | grep -q "Checking script references in markdown" \
  || fail "a link failure aborted the script before the later phases ran"
echo "OK: a real broken link is reported and the remaining phases still execute"

# The gate must fail on the broken fixture, or the ship path stays unguarded.
if "$CHECK" --root "$BROKEN" >/dev/null 2>&1; then
  fail "check-docs-integrity.sh exited 0 despite a broken link"
fi
echo "OK: broken link still fails the gate"

echo "OK: docs-integrity fixtures"
