# Memory CLI Troubleshooting

## Common Issues

### Invalid Arguments (Exit code 2)

**Common mistakes**:
- `--type` for episode create → NO `--type` argument exists. Task type is inferred.
- Unknown flags on subcommands → use `do-memory-cli <cmd> --help` (global: `-v`/`--verbose`, `-f`/`--format`, `--db-path`, `--storage-mode`)

**Solution**: Use `--help` to verify correct arguments:
```bash
do-memory-cli episode create --help
do-memory-cli episode log-step --help
do-memory-cli pattern list --help
```

### Command Not Found

```bash
# Ensure binary exists
ls ./target/release/do-memory-cli

# Add to PATH
export PATH="./target/release:$PATH"
```

### Database Errors

```bash
# Check database exists
ls -la data/

# Repair database
do-memory-cli storage repair

# Reinitialize
rm data/memory.db
do-memory-cli init
```

### Authentication Errors

```bash
# Check environment variables
echo $TURSO_TOKEN
echo $TURSO_URL

# Set credentials
export TURSO_TOKEN="your-token"
export TURSO_URL="libsql://your-db.turso.io"
```

### Connection Issues

```bash
# Test connectivity
curl -v $TURSO_URL

# Check network
ping your-db.turso.io

# Verify token
curl -H "Authorization: Bearer $TURSO_TOKEN" $TURSO_URL
```

### Performance Issues

```bash
# Check cache status
do-memory-cli storage status

# Clear cache
rm -rf data/cache.redb

# Rebuild cache
do-memory-cli storage sync
```

### Steps Not Persisting (Episode shows "Steps: 0")

**Root Cause**: CLI uses step batching (50 steps or 5s interval). Each CLI command is a separate process, so buffered steps are lost between invocations.

**Solutions**:
1. Complete the episode immediately after logging steps (completion flushes buffered steps)
2. Or use MCP server for long-running workflows (steps persist across commands)
3. Or log ≥50 steps in one session to trigger batch flush

**Note**: Pattern extraction requires steps. Episodes with 0 steps will not extract patterns.

### Patterns Not Found (Pattern list shows 0)

**Root causes** (issues #830 / #831):

1. **Different DB path across processes** — `--db-path` / `MEMORY_DB_PATH` must be the same for create, complete, and list. The flag always sets `redb_path` (local default backend). Without it, each process may open a different XDG cache path.
2. **In-memory-only pattern map** — a fresh CLI process must load patterns from redb/Turso (`get_all_patterns`). If list is empty after complete logged "Successfully cached pattern", verify durable store + same `--db-path`.
3. **No steps logged** — create+complete alone still yields **ContextPattern**; tool-sequence patterns need `log-step --success`.
4. **Missing `--success` on log-step** — without the flag, steps are recorded as failures; tool-sequence extractors may skip them.

**Verification**:
```bash
DB=./data/memory.redb
CLI="do-memory-cli --storage-mode local --db-path $DB"

$CLI episode view <EPISODE_ID>   # Steps: N (not 0)
$CLI storage stats               # Patterns: Total > 0
$CLI pattern list                # Fresh process must see patterns
```

### Config format hard to discover (issue #829)

```bash
do-memory-cli config show-template   # full TOML template
do-memory-cli config init            # write do-memory-cli.toml
do-memory-cli config show            # resolved values
```

Partial TOML is OK; missing sections use defaults. Put `storage_mode` under `[database]` (canonical). `[storage].storage_mode` is an accepted alias only.

### Wrong `storage_mode` section (issue #832)

| Location | Supported? | Notes |
|----------|------------|-------|
| `[database].storage_mode` | ✅ Canonical | Preferred; emitted by `config init` |
| `--storage-mode` / `MEMORY_STORAGE_MODE` | ✅ CLI/env | Wins over config file |
| `[storage].storage_mode` | ✅ Alias | Copied into `[database]` if unset |

`[storage]` is for cache size / TTL / pool size — not backend selection.

### `--db-path` appears ignored (issue #830)

**Symptom**: Episodes/patterns written but not visible in the next CLI process.

**Cause**: Older builds only set Turso `db_path` and left `redb_path` at the default XDG path. Current CLI **always** sets `redb_path` from `--db-path` / `MEMORY_DB_PATH`.

**Fix**:
```bash
# Same path for every command in the workflow
export MEMORY_DB_PATH=./data/memory.redb
export MEMORY_STORAGE_MODE=local
do-memory-cli episode create -t "task"
# ... log-step / complete / pattern list with same env
```

## Debug Mode

```bash
# Enable verbose output
do-memory-cli -v <command>

# Enable debug logging
RUST_LOG=debug do-memory-cli <command>
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Database error |
| 4 | Network error |
| 5 | Authentication error |

## Log Analysis

```bash
# View recent logs
do-memory-cli logs --lines 100

# Filter by level
do-memory-cli logs --level error

# Search for pattern
do-memory-cli logs --grep "episode"
```
