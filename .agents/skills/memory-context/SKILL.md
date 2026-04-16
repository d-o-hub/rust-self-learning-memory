---
name: memory-context
description: Retrieve relevant context from memory and preserve essential state. Use for episode retrieval, semantic search, or context compaction when window fills.
version: "2.0"
template_version: "0.2"
category: KnowledgeManagement
---

# Memory Context

Unified skill for retrieving episodic context and preserving essential session state.

## CLI Operations

### Prerequisites

```bash
# Build CLI (redb-only default, or --features turso for remote)
cargo build -p do-memory-cli --release
```

### Environment Setup

```bash
# Local development (redb, no server)
export LOCAL_DATABASE_URL="file:./data/memory.db"

# Turso dev server
turso dev --db-file ./data/memory.db --port 8080
export TURSO_URL="http://127.0.0.1:8080"
export TURSO_TOKEN=""
```

### Episode Queries

```bash
# List recent episodes
cargo run -p do-memory-cli -- episode list --limit 10

# Search by description
cargo run -p do-memory-cli -- episode search "git worktree" --limit 5

# Get episode details
cargo run -p do-memory-cli -- episode get <episode-id>

# JSON output for parsing
cargo run -p do-memory-cli -- episode search "error" --format json --limit 5
```

## Retrieval Methods

### Semantic Search (Programmatic)

When embeddings available, use semantic search for best relevance:

```rust
let context = memory
    .retrieve_relevant_context("implement async batch updates", task_context, 5)
    .await?;
```

### Keyword Search (Fallback)

Fast, deterministic SQL-based retrieval:

```sql
SELECT * FROM episodes
WHERE task_type = ? AND tags LIKE ?
ORDER BY timestamp DESC LIMIT ?;
```

### Filtering Strategies

```rust
// By domain
TaskContext { domain: "storage".to_string(), .. }

// By task type
task_type_filter: Some("implementation")

// By recency (last 30 days)
since: Some(now - Duration::days(30))

// By success only
verdict: Some(Verdict::Success)
```

## Context Compaction

When context window fills, preserve essential state.

### Always Preserve

| Item | Why |
|------|-----|
| Test names + output (pass/fail) | Regression detection |
| Build status (success/error + msg) | Know if in broken state |
| Files modified (path + why) | Track changes, rollback |
| Open TODOs with state | Prevent losing WIP |
| Env vars set this session | Re-establish environment |
| Decisions made (WHY) | Contextual future decisions |

### DO NOT Compress

- Exact file paths
- Error messages (verbatim)
- Test names
- Numeric results

### Compaction Format

```
Tests: 2 fail (test_embed, test_turso_insert)
Build: error - "missing trait impl Write for &str"
Files: do-memory-core/src/embed.rs (added async embed)
TODOs: [in_progress] refactor embed.rs - 60%
Env: TURSO_DATABASE_URL=libsql://...
Decision: switched to async embed for hot path
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Low recall | Check embeddings, expand tags, increase limit |
| Slow retrieval | Check cache, verify indexes, reduce result set |
| Poor relevance | Use semantic search, improve query, filter by domain |

## References

- `agent_docs/LESSONS.md` - Project-wide lessons
- `do-memory-cli/CLI_USER_GUIDE.md` - Full CLI documentation