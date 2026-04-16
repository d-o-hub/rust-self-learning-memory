---
name: memory-harness
description: Universal agent memory harness — record, replay, and benchmark real agent sessions against do-memory-cli. Deploy in any codebase to capture real-life usage data, replay as benchmarks, and measure learning effectiveness. Use when you need to test whether the memory system actually learns, generate realistic test fixtures, or benchmark CLI performance with real data.
version: "1.0"
category: testing
---

# Memory Harness

Universal harness for recording, replaying, and benchmarking real agent sessions against `do-memory-cli`. Works in any codebase.

## Modes

| Mode | Purpose | When |
|------|---------|------|
| **record** | Capture live agent session as a trace file | During normal agent work |
| **replay** | Replay recorded traces against CLI, measure latency | CI, benchmarking |
| **evaluate** | Run N traces, measure learning effectiveness | Quality gates, regression |
| **export** | Generate test fixtures from recorded traces | Test data generation |

---

## Mode 1: Record

Capture a real agent session as a portable JSON trace. Run this **during normal task work** — it wraps the episode lifecycle.

### Step 1: Create trace file

At session start, create `.memory-traces/<timestamp>-<slug>.json`:

```bash
TRACE_DIR=".memory-traces"
mkdir -p "$TRACE_DIR"
TRACE_FILE="$TRACE_DIR/$(date +%Y%m%d-%H%M%S)-$(echo "$TASK" | tr ' ' '-' | head -c 40).json"
```

### Step 2: Record episode start

```bash
# Create episode via CLI — capture the ID
EPISODE_ID=$(do-memory-cli --format json episode create \
  --task "$TASK_DESCRIPTION" \
  --domain "$DOMAIN" 2>/dev/null | grep -o '"id":"[^"]*"' | cut -d'"' -f4)

# Write trace header
cat > "$TRACE_FILE" <<EOF
{
  "version": "1.0",
  "recorded_at": "$(date -Iseconds)",
  "project": "$(basename $(git rev-parse --show-toplevel 2>/dev/null || pwd))",
  "task": "$TASK_DESCRIPTION",
  "domain": "$DOMAIN",
  "episode_id": "$EPISODE_ID",
  "steps": [],
  "outcome": null,
  "metrics": {}
}
EOF
```

### Step 3: Log steps during work

After each significant tool use, append to the trace:

```bash
# Log step to CLI
do-memory-cli episode log-step "$EPISODE_ID" \
  --tool "$TOOL_NAME" \
  --action "$ACTION_DESCRIPTION" \
  --latency-ms "$LATENCY_MS" \
  --success \
  --observation "$OBSERVATION"

# Append to trace file (using jq or Python)
python3 -c "
import json, sys
with open('$TRACE_FILE', 'r+') as f:
    trace = json.load(f)
    trace['steps'].append({
        'tool': '$TOOL_NAME',
        'action': '$ACTION_DESCRIPTION',
        'latency_ms': $LATENCY_MS,
        'success': True,
        'observation': '$OBSERVATION',
        'timestamp': '$(date -Iseconds)'
    })
    f.seek(0)
    json.dump(trace, f, indent=2)
    f.truncate()
"
```

### Step 4: Complete episode

```bash
do-memory-cli episode complete "$EPISODE_ID" success

# Update trace
python3 -c "
import json
with open('$TRACE_FILE', 'r+') as f:
    trace = json.load(f)
    trace['outcome'] = 'success'
    trace['metrics'] = {
        'total_steps': len(trace['steps']),
        'duration_ms': sum(s.get('latency_ms', 0) for s in trace['steps']),
        'success_rate': sum(1 for s in trace['steps'] if s.get('success')) / max(len(trace['steps']), 1)
    }
    f.seek(0)
    json.dump(trace, f, indent=2)
    f.truncate()
"
```

### What tools to record

Log these agent actions as steps:

| Tool | Record As | Example Action |
|------|-----------|----------------|
| Read | `read` | `"Read src/lib.rs lines 1-80"` |
| Grep | `grep` | `"Search for 'fn start_episode' in memory-core"` |
| edit_file | `edit` | `"Edit src/storage/mod.rs: add batch_insert method"` |
| Bash (cargo test) | `test` | `"cargo nextest run -p do-memory-core"` |
| Bash (cargo build) | `build` | `"cargo build -p do-memory-cli"` |
| Bash (git) | `git` | `"git commit -m 'feat(core): add batch insert'"` |
| finder | `search` | `"Find where episode completion triggers pattern extraction"` |

---

## Mode 2: Replay

Replay recorded traces against CLI to benchmark real-world performance.

### Replay a single trace

```bash
#!/usr/bin/env bash
# replay-trace.sh <trace-file>
set -euo pipefail

TRACE="$1"
RESULTS_FILE="${TRACE%.json}.results.json"

# Parse trace
TASK=$(python3 -c "import json; print(json.load(open('$TRACE'))['task'])")
DOMAIN=$(python3 -c "import json; print(json.load(open('$TRACE'))['domain'])")
STEPS=$(python3 -c "import json; print(len(json.load(open('$TRACE'))['steps']))")

echo "▶ Replaying: $TASK ($STEPS steps)"

# Time episode creation
START_NS=$(date +%s%N)
EPISODE_ID=$(do-memory-cli --format json episode create \
  --task "$TASK" --domain "$DOMAIN" 2>/dev/null | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
CREATE_MS=$(( ($(date +%s%N) - START_NS) / 1000000 ))

# Time step logging
STEP_TOTAL_MS=0
python3 -c "import json; [print(f\"{s['tool']}|{s['action']}|{s.get('latency_ms',0)}|{s.get('success',True)}|{s.get('observation','')}\") for s in json.load(open('$TRACE'))['steps']]" | \
while IFS='|' read -r TOOL ACTION LATENCY SUCCESS OBS; do
  STEP_START=$(date +%s%N)
  do-memory-cli episode log-step "$EPISODE_ID" \
    --tool "$TOOL" --action "$ACTION" --latency-ms "$LATENCY" \
    ${SUCCESS:+--success} --observation "$OBS" 2>/dev/null || true
  STEP_MS=$(( ($(date +%s%N) - STEP_START) / 1000000 ))
  STEP_TOTAL_MS=$((STEP_TOTAL_MS + STEP_MS))
done

# Time completion
COMPLETE_START=$(date +%s%N)
OUTCOME=$(python3 -c "import json; print(json.load(open('$TRACE')).get('outcome','success'))")
do-memory-cli episode complete "$EPISODE_ID" "$OUTCOME" 2>/dev/null || true
COMPLETE_MS=$(( ($(date +%s%N) - COMPLETE_START) / 1000000 ))

# Write results
cat > "$RESULTS_FILE" <<EOF
{
  "trace": "$TRACE",
  "episode_id": "$EPISODE_ID",
  "timings": {
    "create_ms": $CREATE_MS,
    "steps_total_ms": $STEP_TOTAL_MS,
    "complete_ms": $COMPLETE_MS,
    "total_ms": $((CREATE_MS + STEP_TOTAL_MS + COMPLETE_MS))
  },
  "steps_count": $STEPS,
  "targets": {
    "create_under_50ms": $([ $CREATE_MS -lt 50 ] && echo true || echo false),
    "complete_under_500ms": $([ $COMPLETE_MS -lt 500 ] && echo true || echo false)
  }
}
EOF

echo "  create: ${CREATE_MS}ms | steps: ${STEP_TOTAL_MS}ms | complete: ${COMPLETE_MS}ms"
```

### Replay all traces (benchmark suite)

```bash
#!/usr/bin/env bash
# replay-all.sh — benchmark all recorded traces
for trace in .memory-traces/*.json; do
  [[ "$trace" == *.results.json ]] && continue
  bash replay-trace.sh "$trace"
done

# Aggregate results
python3 -c "
import json, glob
results = [json.load(open(f)) for f in glob.glob('.memory-traces/*.results.json')]
if results:
    avg_create = sum(r['timings']['create_ms'] for r in results) / len(results)
    avg_complete = sum(r['timings']['complete_ms'] for r in results) / len(results)
    total_steps = sum(r['steps_count'] for r in results)
    print(f'Traces: {len(results)} | Steps: {total_steps}')
    print(f'Avg create: {avg_create:.1f}ms (target <50ms)')
    print(f'Avg complete: {avg_complete:.1f}ms (target <500ms)')
    targets_met = sum(1 for r in results if r['targets']['create_under_50ms'] and r['targets']['complete_under_500ms'])
    print(f'Targets met: {targets_met}/{len(results)}')
"
```

---

## Mode 3: Evaluate (Learning Effectiveness)

The critical test: **does the system actually learn?**

### Evaluate learning over N episodes

```bash
#!/usr/bin/env bash
# evaluate-learning.sh — measure if patterns improve with more data
set -euo pipefail

TRACES=(.memory-traces/*.json)
TOTAL=${#TRACES[@]}

if [ "$TOTAL" -lt 5 ]; then
  echo "Need ≥5 recorded traces to evaluate learning. Record more sessions first."
  exit 1
fi

echo "=== Learning Effectiveness Evaluation ==="
echo "Traces: $TOTAL"

# Phase 1: Replay traces in chronological order
CHECKPOINT_INTERVAL=$(( TOTAL / 3 ))  # Measure at 33%, 66%, 100%

for i in "${!TRACES[@]}"; do
  trace="${TRACES[$i]}"
  [[ "$trace" == *.results.json ]] && continue

  # Replay trace
  bash replay-trace.sh "$trace" 2>/dev/null

  # At each checkpoint, measure pattern quality
  CHECKPOINT=$(( i + 1 ))
  if (( CHECKPOINT % CHECKPOINT_INTERVAL == 0 )) || (( CHECKPOINT == TOTAL )); then
    echo ""
    echo "--- Checkpoint: $CHECKPOINT/$TOTAL episodes ---"

    # Count extracted patterns
    PATTERN_COUNT=$(do-memory-cli --format json pattern list --limit 1000 2>/dev/null | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    print(len(data.get('patterns', data)) if isinstance(data, (dict, list)) else 0)
except: print(0)
")
    echo "  Patterns extracted: $PATTERN_COUNT"

    # Test retrieval quality: search for a known domain
    SEARCH_DOMAIN=$(python3 -c "import json; print(json.load(open('${TRACES[0]}'))['domain'])")
    SEARCH_RESULTS=$(do-memory-cli --format json episode search "$SEARCH_DOMAIN" --limit 5 2>/dev/null | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    results = data.get('episodes', data) if isinstance(data, dict) else data
    print(len(results) if isinstance(results, list) else 0)
except: print(0)
")
    echo "  Search results for '$SEARCH_DOMAIN': $SEARCH_RESULTS"

    # Measure retrieval latency
    SEARCH_START=$(date +%s%N)
    do-memory-cli episode search "$SEARCH_DOMAIN" --limit 5 2>/dev/null > /dev/null
    SEARCH_MS=$(( ($(date +%s%N) - SEARCH_START) / 1000000 ))
    echo "  Search latency: ${SEARCH_MS}ms (target <100ms)"
  fi
done

echo ""
echo "=== Evaluation Complete ==="
echo "Expected: pattern count increases, search results become more relevant, latency stays stable"
```

---

## Mode 4: Export (Test Fixtures)

Generate test fixtures from recorded traces for the `memory-cli` test suite.

```bash
#!/usr/bin/env bash
# export-fixtures.sh — convert traces to Rust test fixtures
OUTPUT_DIR="memory-cli/tests/fixtures"
mkdir -p "$OUTPUT_DIR"

python3 << 'PYEOF'
import json, glob, os

traces = sorted(glob.glob('.memory-traces/*.json'))
traces = [t for t in traces if not t.endswith('.results.json')]

fixtures = []
for path in traces:
    with open(path) as f:
        trace = json.load(f)
    fixtures.append({
        'task': trace['task'],
        'domain': trace['domain'],
        'steps': [{
            'tool': s['tool'],
            'action': s['action'],
            'latency_ms': s.get('latency_ms', 0),
            'success': s.get('success', True),
            'observation': s.get('observation', ''),
        } for s in trace['steps']],
        'outcome': trace.get('outcome', 'success'),
    })

output = os.path.join('memory-cli/tests/fixtures', 'real_traces.json')
with open(output, 'w') as f:
    json.dump(fixtures, f, indent=2)

print(f"Exported {len(fixtures)} traces to {output}")
PYEOF
```

---

## Deploying in Another Codebase

This skill is **self-contained** and portable. To use in any project:

### 1. Copy the skill

```bash
cp -r .agents/skills/memory-harness /path/to/other-project/.agents/skills/
```

### 2. Ensure CLI is available

```bash
# Option A: Install from crates.io
cargo install do-memory-cli

# Option B: Use from workspace
cargo build -p do-memory-cli --release
export PATH="$PATH:./target/release"
```

### 3. Initialize trace directory

```bash
mkdir -p .memory-traces
echo ".memory-traces/" >> .gitignore  # or commit traces for shared benchmarks
```

### 4. Record sessions during normal work

The agent loads this skill at session start, records tool usage as steps, and saves the trace on completion.

### 5. Run benchmarks in CI

```yaml
# .github/workflows/memory-benchmark.yml
- name: Replay memory traces
  run: |
    cargo install do-memory-cli
    for trace in .memory-traces/*.json; do
      bash .agents/skills/memory-harness/replay-trace.sh "$trace"
    done
```

---

## Integration with Existing Skills

| Existing Skill | Replaces | Notes |
|----------------|----------|-------|
| `episode-start` | Absorbed | Use `memory-harness record` instead |
| `episode-log-steps` | Absorbed | Steps recorded automatically |
| `episode-complete` | Absorbed | Completion captured in trace |
| `memory-context` | Complementary | Harness generates data for context queries |

---

## Trace File Format

```json
{
  "version": "1.0",
  "recorded_at": "2026-04-16T14:30:00+00:00",
  "project": "my-project",
  "task": "Add rate limiting to API endpoints",
  "domain": "web-api",
  "episode_id": "a1b2c3d4-...",
  "steps": [
    {
      "tool": "read",
      "action": "Read src/api/routes.rs lines 1-50",
      "latency_ms": 12,
      "success": true,
      "observation": "Found 3 unprotected endpoints",
      "timestamp": "2026-04-16T14:30:05+00:00"
    },
    {
      "tool": "grep",
      "action": "Search for rate_limit in middleware/",
      "latency_ms": 8,
      "success": true,
      "observation": "No existing rate limiter found",
      "timestamp": "2026-04-16T14:30:10+00:00"
    },
    {
      "tool": "edit",
      "action": "Create src/middleware/rate_limit.rs with token bucket",
      "latency_ms": 45,
      "success": true,
      "observation": "Implemented 100 req/min per IP",
      "timestamp": "2026-04-16T14:31:00+00:00"
    },
    {
      "tool": "test",
      "action": "cargo nextest run -p api-server",
      "latency_ms": 3200,
      "success": true,
      "observation": "42 tests pass, 0 fail",
      "timestamp": "2026-04-16T14:31:45+00:00"
    }
  ],
  "outcome": "success",
  "metrics": {
    "total_steps": 4,
    "duration_ms": 3265,
    "success_rate": 1.0
  }
}
```

## Success Criteria

| Metric | Target | Measured By |
|--------|--------|-------------|
| Episode create latency | < 50ms | replay mode |
| Step logging latency | < 20ms/step | replay mode |
| Episode complete latency | < 500ms | replay mode |
| Search latency (100 episodes) | < 100ms | evaluate mode |
| Pattern count growth | Monotonic increase | evaluate mode |
| Retrieval relevance | Improves with more data | evaluate mode |
