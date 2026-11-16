---
name: memory-harness
description: "Universal agent memory harness — record, replay, and benchmark real agent sessions. Use when testing memory system learning, generating test fixtures, or benchmarking CLI performance."
version: "1.1"
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

All three modes must share one database. Export `MEMORY_DB_PATH` (and
`DO_MEMORY_CLI` when the binary is not on `PATH`) before recording or
replaying; each CLI invocation is a separate process.

```bash
export MEMORY_DB_PATH="$PWD/data/memory.redb"   # CLI creates sibling .db with --features turso
export DO_MEMORY_CLI="$PWD/target/release/do-memory-cli"
```

## Record Mode

Use the helper so the trace schema stays in sync with `replay-trace.sh` and
`export-fixtures.sh`. It creates the episode, appends every step, and completes
the episode:

```bash
bash .agents/skills/memory-harness/record-trace.sh start \
  --task "Fix the parser bug" --domain debugging

bash .agents/skills/memory-harness/record-trace.sh step \
  --tool read --action "inspect parser" --latency-ms 12 --success

bash .agents/skills/memory-harness/record-trace.sh step \
  --tool test --action "run parser tests" --latency-ms 840 --success

bash .agents/skills/memory-harness/record-trace.sh complete success
```

`start` prints the episode ID and trace path; the active session lives in
`.memory-traces/.active`. Use `status` to inspect it and a distinct
`--dir`/`MEMORY_TRACE_DIR` for a parallel session.

Equivalent manual commands (the helper's trace schema is authoritative):

```bash
EPISODE_ID=$(do-memory-cli --format json episode create --task "$TASK" --domain "$DOMAIN" | jq -r '.episode_id')
do-memory-cli episode log-step "$EPISODE_ID" --tool "$TOOL" --action "$ACTION" --latency-ms "$MS" --success
do-memory-cli episode complete "$EPISODE_ID" success
```

## Replay Mode

```bash
# Single trace replay (script in skill directory)
bash .agents/skills/memory-harness/replay-trace.sh "$TRACE"

# All traces
for trace in .memory-traces/*.json; do
  [[ "$trace" == *.results.json ]] && continue
  bash .agents/skills/memory-harness/replay-trace.sh "$trace"
done
```

`replay-trace.sh` exits non-zero when episode creation, a step, or completion
fails; a results file is written only after a successful replay.

## Evaluate Mode

Measure learning effectiveness over N episodes:
- Pattern count should increase with more data
- Core search latency should stay <100ms (the CLI wall-time budget is separate)
- Retrieval relevance should improve

```bash
bash .agents/skills/memory-harness/evaluate-learning.sh .memory-traces
```

## Tools to Record

| Tool | Record As |
|------|-----------|
| Read | `read` |
| Grep | `grep` |
| Edit | `edit` |
| Bash (cargo test) | `test` |
| Bash (cargo build) | `build` |
| Bash (git) | `git` |
| Harness verify | `harness` |

## Trace Format

`record-trace.sh` writes this schema; `task`, `domain`, and `outcome` are
required by replay/export (a header with only `episode_id` is rejected):

```json
{
  "version": "1.0",
  "episode_id": "...",
  "task": "Fix the parser bug",
  "domain": "debugging",
  "project": "rust-self-learning-memory",
  "outcome": "success",
  "steps": [{"tool":"read","action":"...","latency_ms":12,"success":true}]
}
```

## Performance Targets

These apply to the in-process core APIs (`SelfLearningMemory`, not the CLI):

| Operation | Target |
|-----------|--------|
| Episode create | < 50ms |
| Step logging | < 20ms |
| Episode complete | < 500ms |
| Search (100 episodes) | < 100ms |

`replay-trace.sh` measures **CLI process wall time**, which includes process
startup, config load, and database open; report it as a diagnostic, not as the
core targets above.

## See also

- `agent_docs/LESSONS.md` LESSON-084 — execute skill examples against the built
  binary; the trace schema and command reference drift silently otherwise.
