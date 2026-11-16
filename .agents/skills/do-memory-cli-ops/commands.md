# Memory CLI Commands

## Episode Commands

### Create Episode

```bash
do-memory-cli episode create --task "task description" [--domain DOMAIN] [--context FILE]
# Alias
do-memory-cli ep create -t "task description" [--domain DOMAIN] [-c FILE]
```

**Options**:
- `-t, --task <TASK>`: Task description (required)
- `--domain <DOMAIN>`: Domain for episode (e.g., 'web-api', 'testing')
- `-c, --context <FILE>`: Context file in JSON (optional)

**Note**: There is NO `--type` argument. Task type is inferred automatically.

### List Episodes

```bash
do-memory-cli episode list [OPTIONS]
# Alias
do-memory-cli ep list [OPTIONS]
```

**Options**:
- `-t, --task-type <TYPE>`: Filter by task type
- `-l, --limit <N>`: Max episodes [default: 10]
- `-s, --status <STATUS>`: Filter by status

### View Episode Details

```bash
do-memory-cli episode view <EPISODE_ID>
do-memory-cli ep view <EPISODE_ID>
```

### Complete Episode

```bash
do-memory-cli episode complete <EPISODE_ID> <OUTCOME>
do-memory-cli ep complete <EPISODE_ID> <OUTCOME>
```

**Outcomes**: `success`, `partial`, `failed`

### Log Execution Step

```bash
do-memory-cli episode log-step --tool <TOOL> --action <ACTION> EPISODE_ID [--success]
do-memory-cli ep log-step --tool <TOOL> --action <ACTION> EPISODE_ID [--success]
```

**Options**:
- `--tool <TOOL>`: Tool name (required)
- `--action <ACTION>`: Action description (required)
- `EPISODE_ID`: Episode UUID (required, positional at END)
- `--success`: Whether step was successful
- `--latency-ms <MS>`: Step latency
- `--tokens <N>`: Token count
- `--observation <TEXT>`: Step observation

**IMPORTANT**: Episode ID is positional at the END, not a flag argument.

## Pattern Commands

### List Patterns

```bash
do-memory-cli pattern list [OPTIONS]
do-memory-cli pat list [OPTIONS]
```

**Options**:
- `--min-confidence <FLOAT>`: Min confidence [default: 0.0]
- `-p, --pattern-type <TYPE>`: Filter by type
- `-l, --limit <N>`: Max patterns [default: 20]

### View Pattern Details

```bash
do-memory-cli pattern view <PATTERN_ID>
do-memory-cli pat view <PATTERN_ID>
```

## Storage Commands

### Statistics

```bash
do-memory-cli storage stats
do-memory-cli st stats
```

### Health

```bash
do-memory-cli storage health
do-memory-cli st health
```

### Synchronize

```bash
do-memory-cli storage sync [--force] [--dry-run]
do-memory-cli st sync [--force] [--dry-run]
```

### Vacuum

```bash
do-memory-cli storage vacuum [--dry-run]
do-memory-cli st vacuum [--dry-run]
```

### Connections

```bash
do-memory-cli storage connections
do-memory-cli st connections
```

### Operation Journal / Repair

```bash
do-memory-cli storage journal [--pending] [--repair]
do-memory-cli st journal [--pending] [--repair]
```

`storage journal --repair` is the repair entry point. There is no
`storage status` or `storage repair` command; use `storage stats` / `storage health`
and `storage journal --repair`.

## Config Commands

### Show Template / Init (issue #829)

```bash
# Print full TOML template to stdout
do-memory-cli config show-template

# Write starter config (default path: do-memory-cli.toml)
do-memory-cli config init
do-memory-cli config init --path ./my-project.toml
```

`storage_mode` belongs under `[database]` (canonical). `[storage].storage_mode` is an accepted alias.

### Show Config

```bash
do-memory-cli config show
do-memory-cli cfg show
```

### Validate Config

```bash
do-memory-cli config validate
do-memory-cli cfg validate
```

## Health Commands

### Check Health

```bash
do-memory-cli health check
do-memory-cli hp check
```

The `check` subcommand is required; `do-memory-cli health` alone is rejected.

### Detailed status

```bash
do-memory-cli health status
do-memory-cli hp status
```

### Monitor

```bash
do-memory-cli health monitor [--interval SECONDS] [--duration SECONDS]
do-memory-cli hp monitor [--interval SECONDS] [--duration SECONDS]
```

## See also

- `agent_docs/LESSONS.md` LESSON-084 — command references are verified against the
  clap surface, not inferred from the skill prose.
