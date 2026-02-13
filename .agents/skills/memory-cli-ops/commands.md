# Memory CLI Commands

## Episode Commands

### Create Episode

```bash
memory-cli episode create --task "task description" [--context context.json]
# Alias
memory-cli ep create -t "task description" [-c context.json]
```

**Options**:
- `-t, --task <TASK>`: Task description (required)
- `-c, --context <FILE>`: Context file in JSON (optional)

### List Episodes

```bash
memory-cli episode list [OPTIONS]
# Alias
memory-cli ep list [OPTIONS]
```

**Options**:
- `-t, --task-type <TYPE>`: Filter by task type
- `-l, --limit <N>`: Max episodes [default: 10]
- `-s, --status <STATUS>`: Filter by status

### View Episode Details

```bash
memory-cli episode view <EPISODE_ID>
memory-cli ep view <EPISODE_ID>
```

### Complete Episode

```bash
memory-cli episode complete <EPISODE_ID> <OUTCOME>
memory-cli ep complete <EPISODE_ID> <OUTCOME>
```

**Outcomes**: `success`, `partial`, `failed`

### Log Execution Step

```bash
memory-cli episode log-step <EPISODE_ID> [OPTIONS]
memory-cli ep log-step <EPISODE_ID> [OPTIONS]
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
memory-cli pattern list [OPTIONS]
memory-cli pat list [OPTIONS]
```

**Options**:
- `--min-confidence <FLOAT>`: Min confidence [default: 0.0]
- `-p, --pattern-type <TYPE>`: Filter by type
- `-l, --limit <N>`: Max patterns [default: 20]

### View Pattern Details

```bash
memory-cli pattern view <PATTERN_ID>
memory-cli pat view <PATTERN_ID>
```

## Storage Commands

### Synchronize

```bash
memory-cli storage sync
memory-cli st sync
```

### Status

```bash
memory-cli storage status
memory-cli st status
```

### Repair

```bash
memory-cli storage repair
memory-cli st repair
```

## Config Commands

### Show Config

```bash
memory-cli config show
memory-cli cfg show
```

### Validate Config

```bash
memory-cli config validate
memory-cli cfg validate
```

## Health Commands

### Check Health

```bash
memory-cli health
memory-cli hp
```

### Detailed Check

```bash
memory-cli health --detailed
memory-cli hp --detailed
```
