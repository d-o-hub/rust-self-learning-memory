#!/usr/bin/env bash
# evaluate-learning.sh — Measure if the memory system actually learns
# Usage: bash evaluate-learning.sh [trace-dir]
set -euo pipefail

TRACE_DIR="${1:-.memory-traces}"
CLI="${DO_MEMORY_CLI:-do-memory-cli}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Collect traces (skip result files)
mapfile -t TRACES < <(find "$TRACE_DIR" -name "*.json" ! -name "*.results.json" | sort)
TOTAL=${#TRACES[@]}

if [ "$TOTAL" -lt 3 ]; then
  echo "Need ≥3 recorded traces to evaluate learning. Have: $TOTAL"
  echo "Record more sessions using memory-harness skill in record mode."
  exit 1
fi

echo "╔══════════════════════════════════════════╗"
echo "║   Learning Effectiveness Evaluation      ║"
echo "╠══════════════════════════════════════════╣"
echo "║  Traces: $(printf '%-33s' "$TOTAL")║"
echo "╚══════════════════════════════════════════╝"
echo ""

# Checkpoint intervals: measure at 33%, 66%, 100%
INTERVAL=$(( TOTAL / 3 ))
[ "$INTERVAL" -lt 1 ] && INTERVAL=1

CHECKPOINT_PATTERNS=()
CHECKPOINT_SEARCH_MS=()
CHECKPOINT_EPISODES=()

for i in "${!TRACES[@]}"; do
  trace="${TRACES[$i]}"
  IDX=$((i + 1))

  # Replay trace silently
  bash "$SCRIPT_DIR/replay-trace.sh" "$trace" > /dev/null 2>&1 || true

  # Checkpoint measurement
  if (( IDX % INTERVAL == 0 )) || (( IDX == TOTAL )); then
    echo "── Checkpoint: $IDX/$TOTAL episodes ──"

    # Count patterns
    PATTERN_COUNT=$($CLI --format json pattern list --limit 10000 2>/dev/null | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if isinstance(data, list): print(len(data))
    elif isinstance(data, dict): print(len(data.get('patterns', data.get('data', []))))
    else: print(0)
except: print(0)
" 2>/dev/null || echo "0")
    CHECKPOINT_PATTERNS+=("$PATTERN_COUNT")
    echo "  Patterns extracted: $PATTERN_COUNT"

    # Measure search latency
    FIRST_DOMAIN=$(python3 -c "import json; print(json.load(open('${TRACES[0]}')).get('domain','general'))" 2>/dev/null || echo "general")
    SEARCH_START=$(date +%s%N)
    SEARCH_HITS=$($CLI --format json episode search "$FIRST_DOMAIN" --limit 10 2>/dev/null | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if isinstance(data, list): print(len(data))
    elif isinstance(data, dict): print(len(data.get('episodes', data.get('results', data.get('data', [])))))
    else: print(0)
except: print(0)
" 2>/dev/null || echo "0")
    SEARCH_MS=$(( ($(date +%s%N) - SEARCH_START) / 1000000 ))
    CHECKPOINT_SEARCH_MS+=("$SEARCH_MS")
    CHECKPOINT_EPISODES+=("$IDX")
    echo "  Search hits for '$FIRST_DOMAIN': $SEARCH_HITS"
    echo "  Search latency: ${SEARCH_MS}ms (target <100ms)"
    echo ""
  fi
done

# --- Summary ---
echo "╔══════════════════════════════════════════╗"
echo "║              Summary                     ║"
echo "╠══════════════════════════════════════════╣"

# Check pattern growth
PASS_PATTERN=true
if [ "${#CHECKPOINT_PATTERNS[@]}" -ge 2 ]; then
  FIRST_P=${CHECKPOINT_PATTERNS[0]}
  LAST_P=${CHECKPOINT_PATTERNS[-1]}
  if [ "$LAST_P" -gt "$FIRST_P" ]; then
    echo "║  ✅ Patterns grew: $FIRST_P → $LAST_P          ║"
  else
    echo "║  ⚠  Patterns flat: $FIRST_P → $LAST_P          ║"
    PASS_PATTERN=false
  fi
fi

# Check search latency stability
PASS_LATENCY=true
for ms in "${CHECKPOINT_SEARCH_MS[@]}"; do
  if [ "$ms" -ge 100 ]; then
    PASS_LATENCY=false
    break
  fi
done
if $PASS_LATENCY; then
  echo "║  ✅ Search latency: all checkpoints <100ms ║"
else
  echo "║  ⚠  Search latency: exceeded 100ms         ║"
fi

echo "╚══════════════════════════════════════════╝"

# Exit code
if $PASS_PATTERN && $PASS_LATENCY; then
  echo ""
  echo "Result: PASS — system learns and stays performant"
  exit 0
else
  echo ""
  echo "Result: NEEDS ATTENTION — check warnings above"
  exit 1
fi
