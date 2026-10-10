#!/usr/bin/env bash
# test-storage-backend-capabilities.sh — fixtures for the #1087 capability guard.
#
# check-storage-backend-capabilities.sh must fail closed on a new defaulted
# StorageBackend method, so this exercises the exact failure modes on a throwaway
# copy of the trait source:
#   F0 clean tree passes;
#   F1 a new defaulted method with no classification fails;
#   F2 an edited justification makes the machine-readable inventory stale;
#   F3 a capability-gated declaration whose body is not a capability error fails;
#   F4 a `pending` method becoming capability-aware only warns (in-flight #1087);
#   F5 a capability declaration defaulting to true fails.
#
# Usage:
#   ./scripts/test-storage-backend-capabilities.sh --fixtures

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:---fixtures}"
[[ "$MODE" == "--fixtures" ]] || {
  echo "unknown mode: $MODE (expected --fixtures)" >&2
  exit 2
}

fail() {
  echo "HARNESS VIOLATION: storage-capabilities — $1" >&2
  exit 1
}

CHECK="$ROOT/scripts/check-storage-backend-capabilities.sh"
[[ -x "$CHECK" ]] || fail "check-storage-backend-capabilities.sh missing or not executable"
command -v python3 >/dev/null || fail "python3 required"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

TREE="$WORK/tree"
mkdir -p "$TREE/memory-core/src/storage" "$TREE/docs/generated" "$TREE/scripts"
SOURCE="$TREE/memory-core/src/storage/backend.rs"
OVERLAY="$TREE/scripts/storage-backend-capabilities.toml"
ARTIFACT="$TREE/docs/generated/storage-backend-capabilities.json"
cp "$ROOT/memory-core/src/storage/backend.rs" "$SOURCE"
cp "$ROOT/scripts/storage-backend-capabilities.toml" "$OVERLAY"
cp "$ROOT/docs/generated/storage-backend-capabilities.json" "$ARTIFACT"

backup() {
  cp "$SOURCE" "$SOURCE.bak"
  cp "$OVERLAY" "$OVERLAY.bak"
  cp "$ARTIFACT" "$ARTIFACT.bak"
}

restore() {
  cp "$SOURCE.bak" "$SOURCE"
  cp "$OVERLAY.bak" "$OVERLAY"
  cp "$ARTIFACT.bak" "$ARTIFACT"
}

expect_pass() {
  local name="$1"
  local out
  if ! out=$("$CHECK" --root "$TREE" --check 2>&1); then
    fail "$name: expected pass, got failure: $out"
  fi
  echo "OK: $name"
}

expect_fail() {
  local name="$1" needle="$2"
  local out rc=0
  out=$("$CHECK" --root "$TREE" --check 2>&1) || rc=$?
  [[ $rc -ne 0 ]] || fail "$name: expected failure, but the checker exited 0"
  grep -qF "$needle" <<<"$out" || fail "$name: output missing '$needle': $out"
  echo "OK: $name"
}

# F0 — the checked-in tree is clean.
expect_pass "clean tree passes"

# F1 — a brand-new defaulted method with no classification must fail.
backup
python3 - "$SOURCE" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text()
needle = "    /// Query procedural memories"
assert needle in text, "fixture anchor missing"
injected = (
    "    /// Fixture: a fabricated success that must not land silently.\n"
    "    async fn fake_capability_probe(&self, _id: Uuid) -> Result<bool> {\n"
    "        Ok(false)\n"
    "    }\n\n"
)
path.write_text(text.replace(needle, injected + needle, 1))
PY
expect_fail "new unclassified default fails" "has no classification"
restore

# F2 — editing a justification changes the generated inventory, so it is stale.
backup
python3 - "$OVERLAY" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text()
old = "Typed empty-list fallback for single-pattern-only backends; durable backends override it."
assert old in text, "fixture anchor missing"
path.write_text(text.replace(old, old + " Edited for the fixture.", 1))
PY
expect_fail "edited justification is stale" "inventory is stale"
restore

# F3 — a capability-gated declaration whose body fabricates success must fail.
backup
python3 - "$SOURCE" "$OVERLAY" <<'PY'
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
text = source.read_text()
needle = "    /// Query procedural memories"
assert needle in text, "fixture anchor missing"
injected = (
    "    /// Fixture: declared capability-gated but still fabricates success.\n"
    "    async fn fake_capability_probe(&self, _id: Uuid) -> Result<bool> {\n"
    "        Ok(false)\n"
    "    }\n\n"
)
source.write_text(text.replace(needle, injected + needle, 1))

overlay = pathlib.Path(sys.argv[2])
overlay.write_text(
    overlay.read_text()
    + "\n[[method]]\n"
    'name = "fake_capability_probe"\n'
    'classification = "capability-gated"\n'
    'capability = "supports_recommendation_attribution"\n'
    'gate_kind = "error"\n'
    'justification = "fixture"\n'
)
PY
expect_fail "gated body must be a capability error" "not a CapabilityUnavailable error"
restore

# F4 — a `pending` method that becomes capability-aware only warns.
backup
python3 - "$SOURCE" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text()
old = (
    "    async fn store_procedural(&self, _procedural: &ProceduralMemory) -> Result<()> {\n"
    "        Ok(())\n"
    "    }"
)
assert old in text, "fixture anchor missing"
new = (
    "    async fn store_procedural(&self, _procedural: &ProceduralMemory) -> Result<()> {\n"
    "        let _ = _procedural;\n"
    '        Err(Error::CapabilityUnavailable { operation: "store_procedural" })\n'
    "    }"
)
path.write_text(text.replace(old, new, 1))
PY
upgrade_out=$("$CHECK" --root "$TREE" --check 2>&1) || fail "pending upgrade must not fail: $upgrade_out"
grep -qF "WARN" <<<"$upgrade_out" || fail "pending upgrade should warn: $upgrade_out"
echo "OK: pending method becoming capability-aware warns instead of failing"
restore

# F5 — a capability declaration must default to false.
backup
python3 - "$SOURCE" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text()
needle = "    /// Query procedural memories"
assert needle in text, "fixture anchor missing"
injected = (
    "    /// Fixture: a capability declaration that defaults to true.\n"
    "    fn supports_fixture_fake(&self) -> bool {\n"
    "        true\n"
    "    }\n\n"
)
path.write_text(text.replace(needle, injected + needle, 1))
PY
expect_fail "capability declaration must default false" "must default to false"
restore

echo "OK: storage-backend-capabilities fixtures"
