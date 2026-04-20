---
name: memory-harness
description: Universal agent memory harness — record, replay, and benchmark real agent sessions. Use when testing memory system learning, generating test fixtures, or benchmarking CLI performance.
version: "1.0"
category: testing
---

# Memory Harness

Record, replay, and benchmark real agent sessions against `do-memory-cli`.

## Modes

| Mode | Purpose | Command |
|------|---------|---------|
| **record** | Capture live session as JSON trace | During normal agent work |
| **replay** | Replay traces, measure latency | CI, benchmarking |
| **evaluate** | Run N traces, measure learning | Quality gates |

## Record Mode

```bash
# Create trace file
TRACE_DIR=".memory-traces"
mkdir -p "$TRACE_DIR"
TRACE_FILE="$TRACE_DIR/$(date +%Y%m%d-%H%M%S)-$(echo "$TASK" | tr ' ' '-' | head -c 40).json"

# Create episode via CLI
EPISODE_ID=$(do-memory-cli --format json episode create --task "$TASK" --domain "$DOMAIN" 2>/dev/null | jq -r '.id')

# Write trace header
echo '{"version":"1.0","episode_id":"'$EPISODE_ID'","steps":[]}' > "$TRACE_FILE"

# Log steps (after each tool use)
do-memory-cli episode log-step "$EPISODE_ID" --tool "$TOOL" --action "$ACTION" --latency-ms "$MS" --success

# Complete episode
do-memory-cli episode complete "$EPISODE_ID" success
```

## Replay Mode

```bash
# Single trace replay (script in skill directory)
bash .agents/skills/memory-harness/replay-trace.sh "$TRACE"

# All traces
for trace in .memory-traces/*.json; do bash replay-trace.sh "$trace"; done
```

## Evaluate Mode

Measure learning effectiveness over N episodes:
- Pattern count should increase with more data
- Search latency should stay <100ms
- Retrieval relevance should improve

## Tools to Record

| Tool | Record As |
|------|-----------|
| Read | `read` |
| Grep | `grep` |
| Edit | `edit` |
| Bash (cargo test) | `test` |
| Bash (cargo build) | `build` |
| Bash (git) | `git` |

## Trace Format

```json
{
  "version": "1.0",
  "episode_id": "...",
  "steps": [{"tool":"read","action":"...","latency_ms":12,"success":true}]
}
```

## Performance Targets

| Operation | Target |
|-----------|--------|
| Episode create | < 50ms |
| Step logging | < 20ms |
| Episode complete | < 500ms |
| Search (100 episodes) | < 100ms |
