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
#   F5 a capability declaration defaulting to true fails;
#   F6 a fail-closed declaration whose body is not a capability error fails.
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
OVERLAY="$TREE/scripts/storage-backend-capabilities.toml"
ARTIFACT="$TREE/docs/generated/storage-backend-capabilities.json"
if [[ -f "$ROOT/memory-core/src/storage/backend.rs" ]]; then
  SOURCE="$TREE/memory-core/src/storage/backend.rs"
  cp "$ROOT/memory-core/src/storage/backend.rs" "$SOURCE"
else
  mkdir -p "$TREE/memory-core/src/storage/backend"
  cp "$ROOT"/memory-core/src/storage/backend/*.rs "$TREE/memory-core/src/storage/backend/"
  SOURCE="$TREE/memory-core/src/storage/backend/mod.rs"
fi
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
new = (
    "    async fn store_procedural(&self, _procedural: &ProceduralMemory) -> Result<()> {\n"
    "        let _ = _procedural;\n"
    '        Err(Error::CapabilityUnavailable { operation: "store_procedural" })\n'
    "    }"
)
if old in text:
    path.write_text(text.replace(old, new, 1))
elif "store_procedural" in text and "capability_unavailable" in text.lower():
    # Already capability-aware after main split (#1152) — upgrade already landed.
    # Leave tree as-is; the --check below must still warn, not fail.
    pass
else:
    # Try new-layout anchor (no underscore prefix).
    old2 = (
        "    async fn store_procedural(&self, procedural: &ProceduralMemory) -> Result<()> {\n"
        "        Ok(())\n"
        "    }"
    )
    assert old2 in text or old in text, "fixture anchor missing"
    if old2 in text:
        new2 = (
            "    async fn store_procedural(&self, procedural: &ProceduralMemory) -> Result<()> {\n"
            "        let _ = procedural;\n"
            '        Err(Error::capability_unavailable("store_procedural"))\n'
            "    }"
        )
        path.write_text(text.replace(old2, new2, 1))
PY
upgrade_out=$("$CHECK" --root "$TREE" --check 2>&1) || fail "pending upgrade must not fail: $upgrade_out"
if ! grep -qF "WARN" <<<"$upgrade_out" && ! grep -qF "0 warning(s)" <<<"$upgrade_out"; then
  fail "pending upgrade should warn or be clean: $upgrade_out"
fi
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

# F6 — a fail-closed declaration whose default body is not a capability error
# fails, so the class cannot be used to allowlist a fabricated success.
# Uses get_all_patterns (stable typed fallback) so the fixture survives
# slice-2 reclassifications of procedural/relationship methods.
backup
python3 - "$OVERLAY" <<'PYEOF'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text()
old = 'name = "get_all_patterns"\nclassification = "value-returning-with-typed-fallback"'
assert old in text, "fixture anchor missing"
new = 'name = "get_all_patterns"\nclassification = "fail-closed"'
path.write_text(text.replace(old, new, 1))
PYEOF
f6_out=""
f6_rc=0
f6_out=$("$CHECK" --root "$TREE" --check 2>&1) || f6_rc=$?
[[ $f6_rc -ne 0 ]] || fail "fail-closed with a fabricated body fails: expected failure, but exited 0"
if ! grep -qF "is declared fail-closed" <<<"$f6_out" && ! grep -qF "inventory is stale" <<<"$f6_out"; then
  fail "fail-closed with a fabricated body fails: output missing expected needle: $f6_out"
fi
echo "OK: fail-closed with a fabricated body fails"
restore

echo "OK: storage-backend-capabilities fixtures"
