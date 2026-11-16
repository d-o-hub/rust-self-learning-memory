#!/usr/bin/env bash
# replay-trace.sh — Replay a recorded agent trace against do-memory-cli
# Usage: bash replay-trace.sh <trace-file.json>
#
# Environment:
#   DO_MEMORY_CLI   CLI binary/command (default: do-memory-cli)
#   MEMORY_DB_PATH  Forwarded to every CLI call so replay uses the recorded DB.
set -euo pipefail

TRACE="${1:?Usage: replay-trace.sh <trace-file.json>}"
RESULTS_FILE="${TRACE%.json}.results.json"
IFS=' ' read -r -a CLI_CMD <<< "${DO_MEMORY_CLI:-do-memory-cli}"

die() {
  echo "replay-trace: $*" >&2
  exit 1
}

run_cli() {
  "${CLI_CMD[@]}" "$@"
}

command -v "${CLI_CMD[0]}" >/dev/null 2>&1 ||
  die "CLI not found: ${CLI_CMD[0]} (set DO_MEMORY_CLI or build do-memory-cli)"
[[ -f "$TRACE" ]] || die "trace not found: $TRACE"

# Parse trace metadata with defaults so an older header fails loudly, not silently.
IFS=$'\t' read -r TASK DOMAIN STEPS_COUNT OUTCOME PROJECT < <(
  TRACE_IN="$TRACE" python3 - <<'PY'
import json, os, sys
try:
    trace = json.load(open(os.environ["TRACE_IN"], encoding="utf-8"))
except (OSError, json.JSONDecodeError) as exc:
    print(f"replay-trace: invalid trace JSON: {exc}", file=sys.stderr)
    raise SystemExit(1)
task = trace.get("task") or "Unknown task"
domain = trace.get("domain") or "general"
steps = trace.get("steps") or []
outcome = trace.get("outcome") or "success"
project = trace.get("project") or "unknown"
print("\t".join([task.replace("\t", " "), domain.replace("\t", " "), str(len(steps)), outcome, project]))
PY
)

case "$OUTCOME" in
  success | partial | failed) ;;
  *) die "trace outcome must be success, partial, or failed (found: $OUTCOME)" ;;
esac

echo "▶ Replaying: $TASK ($STEPS_COUNT steps, $PROJECT)"
if [[ "$STEPS_COUNT" -eq 0 ]]; then
  echo "  ⚠ trace has no steps; pattern extraction will be limited" >&2
fi

# --- Episode creation ---
START_NS=$(date +%s%N)
if ! CREATE_OUTPUT="$(run_cli --format json episode create --task "$TASK" --domain "$DOMAIN")"; then
  die "episode create failed"
fi
CREATE_MS=$(( ($(date +%s%N) - START_NS) / 1000000 ))

EPISODE_ID="$(printf '%s' "$CREATE_OUTPUT" | python3 -c '
import json, sys
try:
    data = json.load(sys.stdin)
    print(data.get("episode_id", ""))
except json.JSONDecodeError:
    print("")
')"
[[ -n "$EPISODE_ID" ]] || die "could not parse episode_id from create output: $CREATE_OUTPUT"

# --- Step logging ---
STEP_TOTAL_MS=0
STEP_COUNT=0
STEP_FAILURES=0

while IFS=$'\t' read -r TOOL ACTION LATENCY SUCCESS OBS; do
  [[ -n "$TOOL" ]] || continue
  STEP_START=$(date +%s%N)
  STEP_ARGS=(episode log-step "$EPISODE_ID" --tool "$TOOL" --action "$ACTION" --latency-ms "$LATENCY")
  if [[ "$SUCCESS" == "true" ]]; then
    STEP_ARGS+=(--success)
  fi
  if [[ -n "$OBS" ]]; then
    STEP_ARGS+=(--observation "$OBS")
  fi
  if ! run_cli "${STEP_ARGS[@]}" >/dev/null; then
    echo "  ⚠ step failed: $TOOL / $ACTION" >&2
    STEP_FAILURES=$((STEP_FAILURES + 1))
  fi
  STEP_MS=$(( ($(date +%s%N) - STEP_START) / 1000000 ))
  STEP_TOTAL_MS=$((STEP_TOTAL_MS + STEP_MS))
  STEP_COUNT=$((STEP_COUNT + 1))
done < <(
  TRACE_IN="$TRACE" python3 - <<'PY'
import json, os
trace = json.load(open(os.environ["TRACE_IN"], encoding="utf-8"))
for step in trace.get("steps") or []:
    fields = [
        str(step.get("tool") or "unknown"),
        str(step.get("action") or "").replace("\t", " ")[:200],
        str(int(step.get("latency_ms") or 0)),
        "true" if step.get("success", True) else "false",
        str(step.get("observation") or "").replace("\t", " ")[:200],
    ]
    print("\t".join(fields))
PY
)

# --- Episode completion ---
COMPLETE_START=$(date +%s%N)
if ! run_cli episode complete "$EPISODE_ID" "$OUTCOME" >/dev/null; then
  die "episode complete failed for $EPISODE_ID"
fi
COMPLETE_MS=$(( ($(date +%s%N) - COMPLETE_START) / 1000000 ))

TOTAL_MS=$((CREATE_MS + STEP_TOTAL_MS + COMPLETE_MS))

# --- Write results ---
cat > "$RESULTS_FILE" <<EOF
{
  "trace": "$(basename "$TRACE")",
  "episode_id": "$EPISODE_ID",
  "timings": {
    "create_ms": $CREATE_MS,
    "steps_total_ms": $STEP_TOTAL_MS,
    "complete_ms": $COMPLETE_MS,
    "total_ms": $TOTAL_MS
  },
  "steps_count": $STEPS_COUNT,
  "step_failures": $STEP_FAILURES
}
EOF

echo "  create: ${CREATE_MS}ms | steps: ${STEP_TOTAL_MS}ms | complete: ${COMPLETE_MS}ms | total: ${TOTAL_MS}ms (CLI process wall time, includes startup)"
if [[ "$STEP_FAILURES" -gt 0 ]]; then
  echo "  ⚠ $STEP_FAILURES step(s) failed to record"
fi

if [[ "$STEP_FAILURES" -gt 0 ]]; then
  die "replay recorded $STEP_FAILURES failed step(s)"
fi
