#!/usr/bin/env bash
# validate-plan-evidence.sh — Validate metric/plan evidence provenance schema (T0.1c)
#
# Usage:
#   ./scripts/validate-plan-evidence.sh PATH_TO_EVIDENCE.json
#   ./scripts/validate-plan-evidence.sh --fixtures
#   ./scripts/validate-plan-evidence.sh --help
#
# Minimal evidence object requires: command, scope, commit, timestamp, result
# Optional but recommended: artifact

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/validate-plan-evidence.sh PATH_TO_EVIDENCE.json
  ./scripts/validate-plan-evidence.sh --fixtures
  ./scripts/validate-plan-evidence.sh --help

Required evidence fields:
  command, scope, commit, timestamp, result

Optional:
  artifact

--fixtures  Create temp good/bad fixtures and verify pass/fail behavior.
            Missing SHA/time/scope/artifact-style incompleteness must fail.
EOF
}

fail() {
  echo "HARNESS VIOLATION: plan-evidence — $1" >&2
  exit 1
}

# Returns 0 on success, 1 on validation failure (does not exit the shell).
validate_evidence() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "HARNESS VIOLATION: plan-evidence — missing evidence file: $path" >&2
    return 1
  fi

  if ! python3 - "$path" <<'PY'
import json, sys
from pathlib import Path

path = Path(sys.argv[1])
try:
    data = json.loads(path.read_text(encoding="utf-8"))
except Exception as e:
    print(f"invalid JSON: {e}", file=sys.stderr)
    sys.exit(1)

# Accept a single object or {records: [...]}
records = []
if isinstance(data, list):
    records = data
elif isinstance(data, dict) and "records" in data:
    records = data["records"]
    if not isinstance(records, list):
        print("records must be a list", file=sys.stderr)
        sys.exit(1)
elif isinstance(data, dict):
    records = [data]
else:
    print("evidence must be object, list, or {records: [...]}", file=sys.stderr)
    sys.exit(1)

if not records:
    print("no evidence records to validate", file=sys.stderr)
    sys.exit(1)

required = ["command", "scope", "commit", "timestamp", "result"]
errors = []
for i, rec in enumerate(records):
    if not isinstance(rec, dict):
        errors.append(f"record[{i}] is not an object")
        continue
    for k in required:
        if k not in rec or rec[k] is None or rec[k] == "":
            errors.append(f"record[{i}] missing/empty field: {k}")

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    sys.exit(1)

print(f"OK: plan-evidence valid ({len(records)} record(s))")
sys.exit(0)
PY
  then
    echo "HARNESS VIOLATION: plan-evidence — evidence validation failed for $path" >&2
    return 1
  fi
  return 0
}

run_fixtures() {
  local tmp
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' RETURN

  local good="$tmp/good.json"
  local bad_missing_sha="$tmp/bad-missing-sha.json"
  local bad_missing_time="$tmp/bad-missing-time.json"
  local bad_missing_scope="$tmp/bad-missing-scope.json"
  local bad_empty="$tmp/bad-empty.json"

  cat >"$good" <<'JSON'
{
  "command": "cargo metadata --format-version 1 --no-deps",
  "scope": "workspace",
  "commit": "a8b7d6d6a350c3f431b5564332b8a5c1365aefb9",
  "timestamp": "2026-07-18T12:00:00Z",
  "result": "pass",
  "artifact": "plans/STATUS/VALIDATION_LATEST.md"
}
JSON

  cat >"$bad_missing_sha" <<'JSON'
{
  "command": "cargo test",
  "scope": "workspace",
  "timestamp": "2026-07-18T12:00:00Z",
  "result": "pass"
}
JSON

  cat >"$bad_missing_time" <<'JSON'
{
  "command": "cargo test",
  "scope": "workspace",
  "commit": "abc123",
  "result": "pass"
}
JSON

  cat >"$bad_missing_scope" <<'JSON'
{
  "command": "cargo test",
  "commit": "abc123",
  "timestamp": "2026-07-18T12:00:00Z",
  "result": "pass"
}
JSON

  cat >"$bad_empty" <<'JSON'
{
  "command": "",
  "scope": "workspace",
  "commit": "abc",
  "timestamp": "2026-07-18T12:00:00Z",
  "result": "pass"
}
JSON

  echo "── fixture: good evidence must pass ──"
  validate_evidence "$good" || fail "good fixture should pass"

  for bad in "$bad_missing_sha" "$bad_missing_time" "$bad_missing_scope" "$bad_empty"; do
    name=$(basename "$bad")
    echo "── fixture: $name must fail ──"
    if validate_evidence "$bad" 2>/dev/null; then
      fail "$name should fail"
    fi
    echo "OK: $name failed as expected"
  done

  echo "OK: validate-plan-evidence fixtures passed"
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
  --fixtures)
    run_fixtures
    exit 0
    ;;
  "")
    echo "FATAL: PATH_TO_EVIDENCE.json or --fixtures required" >&2
    usage >&2
    exit 2
    ;;
  *)
    if ! validate_evidence "$1"; then
      exit 1
    fi
    ;;
esac
