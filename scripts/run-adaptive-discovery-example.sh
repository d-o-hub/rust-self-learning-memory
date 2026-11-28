#!/usr/bin/env bash
# run-adaptive-discovery-example.sh — run the adaptive source-discovery example end to end.
#
# The example is deliberately split across processes so the restart check is real:
#
#   1. seed      records the fixture history into a fresh database (process 1)
#   2. recommend opens that database in a new process, chooses bounded plans, and
#                writes report-recommend.json (process 2)
#   3. seed      replays the same fixture rows onto the same database (process 3).
#                It must resolve every row to the episode that already records it,
#                so the completed evidence count does not grow.
#   4. verify    recomputes the recommendation and compares the fingerprint with
#                report-recommend.json (process 4). A replay that minted a second
#                observation per row would change the evidence counts here.
#
# When a do-memory-cli binary is available the script also replays one emitted
# memory-harness trace and proves that an incomplete trace fails loudly.
#
# Usage: ./scripts/run-adaptive-discovery-example.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$ROOT/target/adaptive-discovery-example"
DB="$WORK/memory.db"
REPORT_DIR="$WORK/report"
TRACE_DIR="$WORK/traces"
CLI="${DO_MEMORY_CLI:-$ROOT/target/debug/do-memory-cli}"
HARNESS_DIR="$ROOT/.agents/skills/memory-harness"
# The CLI is chatty at INFO on every call; keep the runner's output readable.
export RUST_LOG="${RUST_LOG:-error}"

rm -rf "$WORK"
mkdir -p "$WORK" "$REPORT_DIR"

run_phase() {
  local phase="$1"
  shift
  cargo run --quiet -p do-memory-examples --example adaptive_source_discovery -- \
    --phase "$phase" --db "$DB" --report-dir "$REPORT_DIR" "$@"
}

echo "== seed (process 1) =="
run_phase seed --emit-traces "$TRACE_DIR"

echo
echo "== recommend (process 2) =="
run_phase recommend --require-evidence

echo
echo "== re-seed (process 3, replay) =="
RESEED_LOG="$WORK/reseed.log"
run_phase seed --emit-traces "$TRACE_DIR" | tee "$RESEED_LOG"
# Every row must resolve to the episode recorded by process 1. A created row here
# is the double-counting regression: it would add a second observation and move
# the recommendation fingerprint.
if grep -Eq '[[:space:]][1-9][0-9]* created' "$RESEED_LOG"; then
  echo "ERROR: the replay created new episodes instead of resolving to the seeded ones" >&2
  exit 1
fi
if ! grep -Eq '[[:space:]][1-9][0-9]* replayed' "$RESEED_LOG"; then
  echo "ERROR: the replay reported no replayed attempts; nothing was resolved" >&2
  exit 1
fi

echo
echo "== verify (process 4, restart check after the replay) =="
run_phase verify --expect "$REPORT_DIR/report-recommend.json"

echo
if [[ -x "$CLI" ]]; then
  echo "== memory-harness replay =="
  success_trace="$TRACE_DIR/basketball-scores-keyword-beta.json"
  MEMORY_DB_PATH="$WORK/replay.db" DO_MEMORY_CLI="$CLI" \
    bash "$HARNESS_DIR/replay-trace.sh" "$success_trace"

  incomplete_trace="$WORK/incomplete-trace.json"
  python3 - "$success_trace" "$incomplete_trace" <<'PY'
import json
import sys

source, destination = sys.argv[1], sys.argv[2]
with open(source, encoding="utf-8") as handle:
    trace = json.load(handle)
# An episode that never completed must be rejected by the replay path.
trace["outcome"] = "in_progress"
with open(destination, "w", encoding="utf-8") as handle:
    json.dump(trace, handle, indent=2)
PY

  if MEMORY_DB_PATH="$WORK/replay.db" DO_MEMORY_CLI="$CLI" \
    bash "$HARNESS_DIR/replay-trace.sh" "$incomplete_trace" >/dev/null 2>&1; then
    echo "ERROR: replay accepted a trace with an incomplete episode" >&2
    exit 1
  fi
  echo "replay rejected the incomplete trace as expected"
else
  echo "note: no CLI at $CLI; skipping memory-harness replay"
  echo "      build it with: cargo build -p do-memory-cli"
fi

echo
echo "== ok =="
echo "report:  $REPORT_DIR/report-recommend.json"
echo "traces:  $TRACE_DIR"
echo "database: $DB (reused by every phase above)"
