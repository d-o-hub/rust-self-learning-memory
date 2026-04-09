---
name: memory-context
description: Retrieve semantically relevant past learnings and analysis outputs using the do-memory-cli for hybrid retrieval over project knowledge
version: "1.0"
template_version: "0.2"
category: KnowledgeManagement
---

# Memory Context

Retrieve semantically relevant past learnings, analysis outputs, and project knowledge using the `do-memory-cli`.

## Prerequisites

```bash
# Build CLI with turso feature for remote storage, or use redb-only (default)
cargo build -p do-memory-cli --release
```

## When to Use

- At session start to recall previous work
- When facing a problem that might have been solved before
- To retrieve specific findings from `agent_docs/` or `plans/`

## Environment Setup

```bash
# Local development (redb, no server needed)
export LOCAL_DATABASE_URL="file:./data/memory.db"

# Turso dev server (if using turso storage)
turso dev --db-file ./data/memory.db --port 8080
export TURSO_URL="http://127.0.0.1:8080"
export TURSO_TOKEN=""
```

## Querying Episodes

```bash
# List recent episodes
cargo run -p do-memory-cli -- episode list --limit 10

# Search episodes by task description
cargo run -p do-memory-cli -- episode search "git worktree" --limit 5

# Get episode details
cargo run -p do-memory-cli -- episode get <episode-id>

# Analyze patterns
cargo run -p do-memory-cli -- pattern analyze --top-k 10
```

## Output Formats

The CLI supports table output (default) and JSON for machine parsing:

```bash
# JSON output for agent consumption
cargo run -p do-memory-cli -- episode list --format json --limit 5
```

## Token Budget

Use a hard post-query cap from environment:

```bash
cargo run -p do-memory-cli -- episode search "error handling" --limit 5 |
awk -v max_tokens="$MAX_CONTEXT_TOKENS" '
{
    for (i = 1; i <= NF; i++) {
        if (token_count < max_tokens) {
            printf "%s%s", $i, (token_count + 1 < max_tokens ? " " : "\n")
            token_count++
        } else {
            exit
        }
    }
}
'
```

## Reference Files

- `agent_docs/LESSONS.md` - Project-wide lessons log
- `agent_docs/database_schema.md` - Database structure
- `do-memory-cli/CLI_USER_GUIDE.md` - Full CLI documentation