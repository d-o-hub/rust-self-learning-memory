#!/usr/bin/env bash
# validate-feature-spike.sh — Validate a feature-spike decision JSON artifact
#
# Usage:
#   ./scripts/validate-feature-spike.sh PATH_TO_SPIKE_JSON
#   ./scripts/validate-feature-spike.sh --fixtures
#   ./scripts/validate-feature-spike.sh --help
#
# Fails if required fields are missing or decision is not GO|NO-GO.
# Optional GO checks: metrics checks must not report failed required gates.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/validate-feature-spike.sh PATH_TO_SPIKE_JSON
  ./scripts/validate-feature-spike.sh --fixtures
  ./scripts/validate-feature-spike.sh --help

Validates schema:
  {id, commit, timestamp, owner, commands, metrics, preapproved_thresholds,
   result, decision, reviewers}

decision must be GO or NO-GO. Missing fields fail validation.

--fixtures  Create temp good/bad fixtures and verify pass/fail behavior.
EOF
}

fail() {
  echo "HARNESS VIOLATION: feature-spike — $1" >&2
  exit 1
}

# Returns 0 on success, 1 on validation failure (does not exit the shell).
validate_spike() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "HARNESS VIOLATION: feature-spike — missing spike artifact: $path" >&2
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

if not isinstance(data, dict):
    print("artifact must be a JSON object", file=sys.stderr)
    sys.exit(1)

required = [
    "id", "commit", "timestamp", "owner", "commands", "metrics",
    "preapproved_thresholds", "result", "decision", "reviewers",
]
missing = [k for k in required if k not in data]
if missing:
    print(f"missing required fields: {', '.join(missing)}", file=sys.stderr)
    sys.exit(1)

decision = data.get("decision")
if decision not in ("GO", "NO-GO"):
    print(f"decision must be GO or NO-GO, got: {decision!r}", file=sys.stderr)
    sys.exit(1)

# Type checks
if not isinstance(data["commands"], list):
    print("commands must be a list", file=sys.stderr)
    sys.exit(1)
if not isinstance(data["metrics"], dict):
    print("metrics must be an object", file=sys.stderr)
    sys.exit(1)
if not isinstance(data["preapproved_thresholds"], dict):
    print("preapproved_thresholds must be an object", file=sys.stderr)
    sys.exit(1)
if not isinstance(data["reviewers"], list):
    print("reviewers must be a list", file=sys.stderr)
    sys.exit(1)
for key in ("id", "commit", "timestamp", "owner", "result"):
    if data[key] is None or data[key] == "":
        print(f"field {key} must be non-empty", file=sys.stderr)
        sys.exit(1)

print(f"OK: feature-spike schema valid decision={decision} id={data['id']}")
sys.exit(0)
PY
  then
    echo "HARNESS VIOLATION: feature-spike — schema validation failed for $path" >&2
    return 1
  fi
  return 0
}

run_fixtures() {
  local tmp
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' RETURN

  local good="$tmp/good.json"
  local bad_missing="$tmp/bad-missing.json"
  local bad_decision="$tmp/bad-decision.json"

  cat >"$good" <<'JSON'
{
  "id": "F4.fixture",
  "commit": "abc123",
  "timestamp": "2026-07-18T00:00:00Z",
  "owner": "test",
  "commands": ["true"],
  "metrics": {"checks": []},
  "preapproved_thresholds": {"p95_overhead_pct": 2},
  "result": "pass",
  "decision": "GO",
  "reviewers": ["fixture"]
}
JSON

  cat >"$bad_missing" <<'JSON'
{
  "id": "F4.bad",
  "decision": "GO"
}
JSON

  cat >"$bad_decision" <<'JSON'
{
  "id": "F4.bad2",
  "commit": "abc",
  "timestamp": "2026-07-18T00:00:00Z",
  "owner": "test",
  "commands": [],
  "metrics": {},
  "preapproved_thresholds": {},
  "result": "weird",
  "decision": "MAYBE",
  "reviewers": []
}
JSON

  echo "── fixture: good must pass ──"
  validate_spike "$good" || fail "good fixture should pass"

  echo "── fixture: missing fields must fail ──"
  if validate_spike "$bad_missing" 2>/dev/null; then
    fail "missing-fields fixture should fail"
  fi
  echo "OK: missing-fields fixture failed as expected"

  echo "── fixture: invalid decision must fail ──"
  if validate_spike "$bad_decision" 2>/dev/null; then
    fail "invalid-decision fixture should fail"
  fi
  echo "OK: invalid-decision fixture failed as expected"

  echo "OK: validate-feature-spike fixtures passed"
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
    echo "FATAL: PATH_TO_SPIKE_JSON or --fixtures required" >&2
    usage >&2
    exit 2
    ;;
  *)
    if ! validate_spike "$1"; then
      exit 1
    fi
    ;;
esac
