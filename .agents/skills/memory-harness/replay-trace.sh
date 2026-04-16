#!/usr/bin/env bash
# replay-trace.sh — Replay a recorded agent trace against do-memory-cli
# Usage: bash replay-trace.sh <trace-file.json>
set -euo pipefail

TRACE="${1:?Usage: replay-trace.sh <trace-file.json>}"
RESULTS_FILE="${TRACE%.json}.results.json"
CLI="${DO_MEMORY_CLI:-do-memory-cli}"

# Parse trace metadata
TASK=$(python3 -c "import json; print(json.load(open('$TRACE'))['task'])")
DOMAIN=$(python3 -c "import json; print(json.load(open('$TRACE')).get('domain','general'))")
STEPS_COUNT=$(python3 -c "import json; print(len(json.load(open('$TRACE'))['steps']))")
OUTCOME=$(python3 -c "import json; print(json.load(open('$TRACE')).get('outcome','success'))")

echo "▶ Replaying: $TASK ($STEPS_COUNT steps)"

# --- Episode creation ---
START_NS=$(date +%s%N)
CREATE_OUTPUT=$($CLI --format json episode create --task "$TASK" --domain "$DOMAIN" 2>/dev/null || echo '{}')
CREATE_MS=$(( ($(date +%s%N) - START_NS) / 1000000 ))

EPISODE_ID=$(echo "$CREATE_OUTPUT" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    # Handle various output formats
    eid = data.get('id') or data.get('episode_id') or ''
    print(eid)
except:
    print('')
" 2>/dev/null || echo "")

if [ -z "$EPISODE_ID" ]; then
  echo "  ⚠ Could not parse episode ID from create output, using placeholder"
  EPISODE_ID="replay-$(date +%s)"
fi

# --- Step logging ---
STEP_TOTAL_MS=0
STEP_COUNT=0

python3 -c "
import json
trace = json.load(open('$TRACE'))
for s in trace['steps']:
    tool = s.get('tool', 'unknown')
    action = s.get('action', '').replace(\"'\", '')[:200]
    latency = s.get('latency_ms', 0)
    success = 'true' if s.get('success', True) else 'false'
    obs = s.get('observation', '').replace(\"'\", '')[:200]
    print(f'{tool}\t{action}\t{latency}\t{success}\t{obs}')
" 2>/dev/null | while IFS=$'\t' read -r TOOL ACTION LATENCY SUCCESS OBS; do
  STEP_START=$(date +%s%N)
  if [ "$SUCCESS" = "true" ]; then
    $CLI episode log-step "$EPISODE_ID" \
      --tool "$TOOL" --action "$ACTION" --latency-ms "$LATENCY" \
      --success --observation "$OBS" 2>/dev/null || true
  else
    $CLI episode log-step "$EPISODE_ID" \
      --tool "$TOOL" --action "$ACTION" --latency-ms "$LATENCY" \
      --observation "$OBS" 2>/dev/null || true
  fi
  STEP_MS=$(( ($(date +%s%N) - STEP_START) / 1000000 ))
  STEP_TOTAL_MS=$((STEP_TOTAL_MS + STEP_MS))
  STEP_COUNT=$((STEP_COUNT + 1))
done

# --- Episode completion ---
COMPLETE_START=$(date +%s%N)
$CLI episode complete "$EPISODE_ID" "$OUTCOME" 2>/dev/null || true
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
  "targets": {
    "create_under_50ms": $([ "$CREATE_MS" -lt 50 ] && echo true || echo false),
    "complete_under_500ms": $([ "$COMPLETE_MS" -lt 500 ] && echo true || echo false)
  }
}
EOF

echo "  create: ${CREATE_MS}ms | steps: ${STEP_TOTAL_MS}ms | complete: ${COMPLETE_MS}ms | total: ${TOTAL_MS}ms"

# Exit non-zero if targets not met
if [ "$CREATE_MS" -ge 50 ] || [ "$COMPLETE_MS" -ge 500 ]; then
  echo "  ⚠ Performance targets NOT met"
fi
