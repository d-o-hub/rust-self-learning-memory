---
name: do-memory-cli-ops
description: "Execute and troubleshoot do-memory-cli commands for episode management, pattern analysis, and storage operations. Use this skill when running CLI commands, debugging CLI issues, explaining command usage, or guiding users through CLI workflows."
---

# Memory CLI Operations

Execute and troubleshoot the do-memory-cli for the self-learning memory system.

## Quick Reference

- **[Commands](commands.md)** - Full command reference
- **[Troubleshooting](troubleshooting.md)** - Debugging guide
- **[Examples](examples.md)** - Common workflows

## When to Use

- Running CLI commands for episode/pattern management
- Debugging CLI command failures
- Understanding command syntax and options
- Guiding users through CLI workflows

## CLI Overview

**Location**: `./target/release/do-memory-cli`
**Output Formats**: human (default), json, yaml

## Global Options

```bash
do-memory-cli [OPTIONS] <COMMAND>

Options:
  -c, --config <FILE>         Configuration file path
  -f, --format <FORMAT>       Output format (human|json|yaml)
  -v, --verbose               Enable verbose output
  --dry-run                   Show what would be done
  --storage-mode <MODE>       Storage mode: remote | local | memory
                              (env: MEMORY_STORAGE_MODE)
  --db-path <PATH>            Project-local DB path (env: MEMORY_DB_PATH)
```

### Storage / DB path notes (issues #830, #832)

| Flag / env | Effect |
|------------|--------|
| `--storage-mode` / `MEMORY_STORAGE_MODE` | Sets `[database].storage_mode` (`remote`, `local`, `memory`) |
| `--db-path` / `MEMORY_DB_PATH` | **Always** sets `redb_path` (and `db_path`) to the given path. Local default backend when no Turso URL is set. |

**Config placement for `storage_mode`**:
- Canonical: `[database].storage_mode`
- Alias: `[storage].storage_mode` is accepted and copied into `[database]` if unset
- `[storage]` is otherwise for cache size / TTL / pool size — not backend selection

```bash
# Project-local redb (recommended for multi-process CLI smoke)
do-memory-cli --storage-mode local --db-path ./data/memory.redb episode list
# Or:
MEMORY_DB_PATH=./data/memory.redb MEMORY_STORAGE_MODE=local do-memory-cli episode list
```

## Config discovery (issue #829)

```bash
# Print a full TOML template to stdout
do-memory-cli config show-template

# Write a starter config (default: do-memory-cli.toml)
do-memory-cli config init
do-memory-cli config init --path ./my-project.toml

# Inspect / validate resolved config
do-memory-cli config show
do-memory-cli config validate
```

Partial TOML is valid — missing sections use defaults. Minimal local example:

```toml
[database]
redb_path = "./.do-memory-cli/cache/memory.redb"
storage_mode = "local"
```

## Cross-process pattern workflow (issue #831)

Each CLI invocation is a **separate process**. Patterns must be durable (postcard + redb/Turso), not only in-memory.

```bash
DB=./data/memory.redb
CLI="do-memory-cli --storage-mode local --db-path $DB"

# 1. Create
ID=$($CLI episode create -t "Implement auth" --format json | jq -r .episode_id)

# 2. Log steps (use --success for tool-sequence patterns)
$CLI episode log-step "$ID" --tool compiler --action "build" --success
$CLI episode log-step "$ID" --tool test --action "run tests" --success

# 3. Complete (triggers pattern extraction + durable cache)
$CLI episode complete "$ID" success

# 4. List in a fresh process (must be > 0)
$CLI pattern list
$CLI pattern search auth
```

**Pattern-type warning**:
- `create` + `complete` alone (no successful tool steps) still yields a **ContextPattern**
- Tool-sequence / multi-step patterns need `episode log-step ... --success` (flag sets success=true; omit → failure path)
- Episodes with 0 steps extract little useful pattern data — always log steps before complete when testing pattern pipelines

## Commands Overview

| Command | Alias | Purpose |
|---------|-------|---------|
| episode | ep | Episode management |
| pattern | pat | Pattern analysis |
| storage | st | Storage operations |
| config | cfg | Configuration (`init`, `show-template`, `show`, `validate`) |
| health | hp | Health monitoring |
| backup | bak | Backup/restore |
| monitor | mon | Metrics |

See **[commands.md](commands.md)** for detailed command documentation and **[examples.md](examples.md)** for common workflows.
