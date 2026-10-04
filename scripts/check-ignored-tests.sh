#!/usr/bin/env bash
# check-ignored-tests.sh — W2.5b ignored-test ceiling ratchet and inventory validation
#
# Usage:
#   ./scripts/check-ignored-tests.sh
#   ./scripts/check-ignored-tests.sh --ceiling 200
#   ./scripts/check-ignored-tests.sh --fixture ratchet

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CEILING="${QUALITY_GATE_IGNORED_TEST_CEILING:-200}"
INVENTORY_FILE="plans/ignored_tests_inventory.json"
MODE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ceiling)
      CEILING="$2"
      shift 2
      ;;
    --fixture)
      MODE="fixture"
      shift
      if [[ "${1:-}" == "ratchet" ]]; then shift; fi
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

# Count #[ignore] attributes in Rust sources (production + tests).
# Prefer ripgrep; fall back to find+grep when rg is not installed (minimal CI images).
# Never fail with exit 127 when rg is missing — that broke Quick Check (CI).
count_ignores() {
  local n=0
  if command -v rg >/dev/null 2>&1; then
    n=$(rg -c '#\[ignore' --glob '*.rs' -g '!target/**' 2>/dev/null \
      | awk -F: '{s+=$2} END {print s+0}')
  elif command -v find >/dev/null 2>&1 && command -v grep >/dev/null 2>&1; then
    n=$(find . \( -path ./target -o -path ./.git \) -prune -o -name '*.rs' -print 2>/dev/null \
      | xargs grep -c '#\[ignore' 2>/dev/null \
      | awk -F: '{s+=$2} END {print s+0}')
  else
    echo "WARN: neither rg nor find+grep available; treating ignore count as 0" >&2
    n=0
  fi
  # Normalize empty
  echo "${n:-0}"
}

COUNT=$(count_ignores)
# Bash arithmetic requires integer
COUNT=$((COUNT + 0))

echo "ignored_test_attrs=$COUNT ceiling=$CEILING"

if [[ "$MODE" == "fixture" ]]; then
  # Ratchet fixture: ceiling must be numeric and count must not exceed it
  [[ "$CEILING" =~ ^[0-9]+$ ]] || {
    echo "HARNESS VIOLATION: ignored-tests — invalid ceiling" >&2
    exit 1
  }
fi

if (( COUNT > CEILING )); then
  echo "HARNESS VIOLATION: ignored-tests — $COUNT attrs exceed ceiling $CEILING" >&2
  echo "Lower ignores or raise ceiling only with documented evidence." >&2
  exit 1
fi

echo "OK: ignored-test count within ceiling"

# Validate inventory file exists and matches #[ignore] count and schema
if [[ ! -f "$INVENTORY_FILE" ]]; then
  echo "HARNESS VIOLATION: Ignored test inventory file '$INVENTORY_FILE' missing!" >&2
  exit 1
fi

python3 -c "
import os, re, json, sys

inventory_file = '$INVENTORY_FILE'
try:
    with open(inventory_file, 'r', encoding='utf-8') as f:
        inventory = json.load(f)
except Exception as e:
    print(f'HARNESS VIOLATION: Failed to parse {inventory_file}: {e}', file=sys.stderr)
    sys.exit(1)

# Schema validation
required_fields = ['crate', 'file', 'line', 'test', 'reason', 'upstream_tracker', 'owner', 'revalidation_date']
for idx, entry in enumerate(inventory):
    for field in required_fields:
        if field not in entry:
            print(f'HARNESS VIOLATION: Inventory entry #{idx} missing field \"{field}\"', file=sys.stderr)
            sys.exit(1)

# Scan codebase for #[ignore] attributes
fn_pattern = re.compile(r'fn\s+([a-zA-Z0-9_]+)')
codebase_ignores = []

for root, dirs, files in os.walk('.'):
    if 'target' in root or '.git' in root:
        continue
    for f in sorted(files):
        if f.endswith('.rs'):
            path = os.path.join(root, f)
            rel_path = os.path.relpath(path, '.')
            with open(path, 'r', encoding='utf-8', errors='ignore') as file:
                lines = file.readlines()
            for line_idx, line in enumerate(lines):
                if '#[ignore' in line and not line.strip().startswith('//'):
                    fn_name = 'unknown'
                    for j in range(line_idx + 1, min(line_idx + 20, len(lines))):
                        m_fn = fn_pattern.search(lines[j])
                        if m_fn:
                            fn_name = m_fn.group(1)
                            break
                    codebase_ignores.append((rel_path, fn_name))

inv_keys = set((item['file'], item['test']) for item in inventory)
missing_in_inv = []
for file_path, test_name in codebase_ignores:
    if (file_path, test_name) not in inv_keys:
        missing_in_inv.append((file_path, test_name))

if missing_in_inv:
    print('HARNESS VIOLATION: Found undocumented #[ignore] tests in codebase:', file=sys.stderr)
    for file_path, test_name in missing_in_inv:
        print(f'  - {file_path} :: {test_name}', file=sys.stderr)
    sys.exit(1)

if len(inventory) != len(codebase_ignores):
    print(f'HARNESS VIOLATION: Inventory count ({len(inventory)}) does not match codebase #[ignore] count ({len(codebase_ignores)})', file=sys.stderr)
    sys.exit(1)

print(f'OK: Ignored test inventory valid and complete ({len(inventory)} items matched).')
"
