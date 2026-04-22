# Memory CLI Commands

## Episode Commands

### Create Episode

```bash
do-memory-cli episode create --task "task description" [--context context.json]
# Alias
do-memory-cli ep create -t "task description" [-c context.json]
```

**Options**:
- `-t, --task <TASK>`: Task description (required)
- `-c, --context <FILE>`: Context file in JSON (optional)

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
do-memory-cli episode log-step <EPISODE_ID> [OPTIONS]
do-memory-cli ep log-step <EPISODE_ID> [OPTIONS]
```

**Options**:
- `-t, --tool <TOOL>`: Tool name (required)
- `-a, --action <ACTION>`: Action description (required)
- `--success`: Whether step was successful
- `--latency-ms <MS>`: Step latency
- `-o, --observation <TEXT>`: Step observation

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

### Synchronize

```bash
do-memory-cli storage sync
do-memory-cli st sync
```

### Status

```bash
do-memory-cli storage status
do-memory-cli st status
```

### Repair

```bash
do-memory-cli storage repair
do-memory-cli st repair
```

## Config Commands

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
do-memory-cli health
do-memory-cli hp
```

### Detailed Check

```bash
do-memory-cli health --detailed
do-memory-cli hp --detailed
```
