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

**Flag placement & logs** (verified against release v0.1.38):
- `-f/--format`, `-v/--verbose`, `-c/--config`, `--storage-mode`, `--db-path` are **top-level** flags — place them **before** the subcommand: `do-memory-cli --format json episode list` works, `do-memory-cli episode list --format json` is rejected.
- **Logs go to stderr**; stdout carries only command output, so `--format json` piped to `jq` stays machine-parseable.
- `RUST_LOG=off` silences all logs; `RUST_LOG=debug` adds detail (equivalent to `-v`).
  Note: an *empty* `RUST_LOG` also silences logs — prefer an explicit `RUST_LOG=off`.

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

### Verifying Turso/SQLite entries

With the `turso` feature, `--db-path x.redb` opens **two sibling files**: the
Turso/SQLite database at `x.db` (durable) and the redb cache at `x.redb`.

Inspect the SQLite side directly (no `sqlite3` CLI needed — python3 ships sqlite3):

```bash
python3 - <<'PY'
import sqlite3
con = sqlite3.connect("data/cache.db")   # sibling of data/cache.redb
cur = con.cursor()
print([t[0] for t in cur.execute("SELECT name FROM sqlite_master WHERE type='table'")])
print("episodes:", cur.execute("SELECT COUNT(*) FROM episodes").fetchone()[0])
print(cur.execute("SELECT episode_id, substr(task_description,1,40) FROM episodes").fetchall())
PY
```

Notes:

- `episodes.outcome` is a **JSON string** (e.g. `{"Success":{"verdict":"..."}}`); there is no `status` column.
- Steps live inside the `episodes.steps` JSON column, not a separate table.
- CLI-side verification: `episode list`, `pattern list`, `storage stats`, `health check`.
- **`storage sync` is Turso → redb only** (ADR-076 reconciliation): it refreshes the cache from durable storage and can never push cache-only episodes into Turso. `backup create` also reads Turso only. Episodes that exist only in the redb cache stay cache-only — recreate them or add a cache→Turso path to make them durable.

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

# 1. Create (--format is a top-level flag: place it BEFORE the subcommand)
ID=$($CLI --format json episode create -t "Implement auth" | jq -r .episode_id)

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
| episode | ep | Episode management (create, list, view, log-step, complete, bulk, search) |
| pattern | pat | Pattern analysis (list, view, analyze, effectiveness, extract, decay) |
| storage | st | Storage operations (stats, sync, vacuum, health, connections) |
| config | cfg | Configuration (`init`, `show-template`, `show`, `validate`) |
| health | hp | Health monitoring (`check`, `status`, `monitor`) |
| backup | bak | Backup/restore (`create`, `list`, `restore`, `verify`) |
| monitor | mon | Metrics (`status`, `export`) |
| logs | log | Log analysis (`analyze`, `search`, `export`, `stats`) |
| eval | ev | Evaluation & calibration (`stats`, `calibration`) |
| embedding | emb | Embedding providers (`list`, `test`, `configure`) |
| tag | tg | Episode tags (`add`, `remove`, `set`, `get`, `search`) |
| relationship | rel | Episode relationships (DAG + `info`) |
| playbook | pb | Playbook recommendations & management |
| feedback | fb | Recommendation feedback tracking |
| external-signal | sig | External signal provider management |
| completion | comp | Shell completions (`bash`, `zsh`, `fish`, …) |

Subcommand help is authoritative: `do-memory-cli <cmd> --help`. Some leaf commands
(`backup create`, `logs export`, `monitor export`) accept a **local** `--format`;
all others use the top-level flag only.

See **[commands.md](commands.md)** for detailed command documentation and **[examples.md](examples.md)** for common workflows.
