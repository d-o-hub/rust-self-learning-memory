---
name: memory-cli-ops
description: Execute and troubleshoot memory-cli commands for episode/pattern/storage management. Use when running CLI commands, debugging CLI issues, explaining command usage, or guiding users through CLI workflows.
---

# Memory CLI Operations

Expert guidance for using the memory-cli command-line interface to manage the self-learning memory system.

## When to Use

Use this skill when:
- Executing memory-cli commands for episode, pattern, or storage operations
- Troubleshooting CLI command failures or unexpected behavior
- Explaining CLI command usage and options to users
- Guiding users through common CLI workflows
- Diagnosing storage connection or configuration issues
- Optimizing CLI performance and output formatting

## Core Concepts

### CLI Architecture

The memory-cli provides complete control over the self-learning memory system through **4 command categories**:

1. **Episode Management** (5 commands) - Track learning episodes from start to completion
2. **Pattern Management** (5 commands) - Analyze and manage extracted patterns
3. **Storage Operations** (5 commands) - Monitor and maintain Turso/redb storage
4. **Operational Commands** (9 commands) - Backup, config, health, logs, monitoring

### Output Formats

All commands support 4 output formats:
- **JSON** (`--format json`) - For scripting and automation
- **YAML** (`--format yaml`) - For readability and configuration
- **Table** (`--format table`) - For terminal viewing (default)
- **Plain** (`--format plain`) - For simple text output

### Configuration Hierarchy

Config files are loaded in order of precedence:
1. `$MEMORY_CLI_CONFIG` environment variable
2. `./memory-cli.toml` (current directory)
3. `~/.config/memory-cli/config.toml` (user config)
4. `/etc/memory-cli/config.toml` (system config)

## Essential Commands

### Episode Management

**Start Episode**:
```bash
memory-cli episode start "Task description" \
  --language rust \
  --domain backend \
  --tags tag1,tag2 \
  --complexity moderate
```

**Complete Episode**:
```bash
memory-cli episode complete <EPISODE_ID> \
  --verdict "Summary of outcome" \
  --success
```

**Log Step**:
```bash
memory-cli episode log-step <EPISODE_ID> \
  --tool cargo \
  --action "cargo build" \
  --success \
  --observation "Build completed"
```

**List Episodes**:
```bash
memory-cli episode list \
  --language rust \
  --limit 20 \
  --sort timestamp \
  --order desc
```

**View Episode**:
```bash
memory-cli episode view <EPISODE_ID> \
  --include-steps \
  --include-patterns \
  --format yaml
```

### Pattern Management

**List Patterns**:
```bash
memory-cli pattern list \
  --pattern-type tool-sequence \
  --min-frequency 5 \
  --min-success 0.8 \
  --sort success_rate
```

**Analyze Pattern**:
```bash
memory-cli pattern analyze <PATTERN_ID> \
  --time-window 90 \
  --format json
```

**Pattern Effectiveness**:
```bash
memory-cli pattern effectiveness \
  --language rust \
  --domain backend
```

**Decay Patterns**:
```bash
memory-cli pattern decay \
  --decay-rate 0.15 \
  --older-than 90 \
  --dry-run
```

### Storage Operations

**Storage Stats**:
```bash
memory-cli storage stats --detailed
```

**Sync Storage**:
```bash
memory-cli storage sync --direction bidirectional
```

**Vacuum Storage**:
```bash
memory-cli storage vacuum --backend all --aggressive
```

**Storage Health**:
```bash
memory-cli storage health --detailed
```

**Connection Status**:
```bash
memory-cli storage connection-status --backend all
```

### Operational Commands

**Create Backup**:
```bash
memory-cli backup create \
  --output backups/$(date +%Y%m%d).tar.gz \
  --compress \
  --include-cache
```

**Restore Backup**:
```bash
memory-cli backup restore <BACKUP_PATH> \
  --clear-existing \
  --confirm
```

**Initialize Config**:
```bash
memory-cli config init --interactive
```

**Show Config**:
```bash
memory-cli config show --mask-secrets
```

**Health Check**:
```bash
memory-cli health check --detailed
```

## Common Workflows

### Complete Episode Workflow

```bash
# 1. Start episode and capture ID
EPISODE_ID=$(memory-cli episode start "Implement feature X" \
  --language rust \
  --domain backend \
  --tags feature,api \
  --format plain | awk '{print $1}')

# 2. Log work steps
memory-cli episode log-step $EPISODE_ID \
  --tool cargo \
  --action "cargo build" \
  --success

memory-cli episode log-step $EPISODE_ID \
  --tool cargo \
  --action "cargo test" \
  --success

# 3. Complete episode
memory-cli episode complete $EPISODE_ID \
  --verdict "Feature X implemented successfully" \
  --success
```

### Pattern Analysis Workflow

```bash
# 1. List top patterns
memory-cli pattern list \
  --sort success_rate \
  --order desc \
  --min-frequency 5

# 2. Analyze specific pattern
memory-cli pattern analyze pattern-abc123 --time-window 90

# 3. Check overall effectiveness
memory-cli pattern effectiveness --domain backend --format json
```

### Storage Maintenance Workflow

```bash
# 1. Check health
memory-cli storage health --detailed

# 2. Sync storages
memory-cli storage sync --direction bidirectional

# 3. Vacuum to reclaim space
memory-cli storage vacuum --backend all --aggressive

# 4. Verify stats
memory-cli storage stats --detailed
```

## Troubleshooting

### Storage Connection Failed

**Symptoms**: Commands hang or fail with connection errors

**Diagnosis**:
```bash
memory-cli storage connection-status
memory-cli config show --mask-secrets
```

**Solutions**:
1. Verify `TURSO_DATABASE_URL` and `TURSO_AUTH_TOKEN` are set
2. Check network connectivity: `curl -H "Authorization: Bearer $TURSO_AUTH_TOKEN" $TURSO_DATABASE_URL`
3. Try force sync: `memory-cli storage sync --force`

### Config File Not Found

**Symptoms**: CLI can't find configuration

**Solutions**:
1. Initialize config: `memory-cli config init --interactive`
2. Set custom path: `export MEMORY_CLI_CONFIG=~/my-config.toml`
3. Create config manually in `~/.config/memory-cli/config.toml`

### Permission Denied

**Symptoms**: Can't write to data directories

**Solutions**:
1. Fix permissions: `chmod -R u+rw ~/.local/share/memory-cli/`
2. Change data directory in config

### Slow Command Execution

**Diagnosis**:
```bash
memory-cli storage health --detailed
memory-cli storage stats --detailed
```

**Solutions**:
1. Vacuum databases: `memory-cli storage vacuum --aggressive`
2. Increase cache size in config
3. Sync storages: `memory-cli storage sync`

## Configuration

### Essential Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `TURSO_DATABASE_URL` | Turso database URL | Yes |
| `TURSO_AUTH_TOKEN` | Turso auth token | Yes |
| `MEMORY_CLI_CONFIG` | Custom config file path | No |
| `MEMORY_CLI_LOG_LEVEL` | Logging level | No (default: info) |

### Minimal Config File

```toml
[storage]
turso_url = "libsql://your-database.turso.io"
turso_token = "${TURSO_AUTH_TOKEN}"  # Use env var
redb_path = "~/.local/share/memory-cli/cache.redb"

[output]
default_format = "table"
color = true

[logging]
level = "info"
file = "~/.local/share/memory-cli/logs/cli.log"
```

## Best Practices

### Security
✓ Store credentials in environment variables, not config files
✓ Use `--mask-secrets` when sharing config output
✓ Restrict config permissions: `chmod 600 ~/.config/memory-cli/config.toml`
✓ Regularly rotate Turso auth tokens

### Reliability
✓ Set up automated backups (cron/systemd)
✓ Monitor storage health regularly
✓ Vacuum databases periodically
✓ Sync storages after major operations

### Performance
✓ Use pagination for large results (`--limit`, `--offset`)
✓ Filter queries as much as possible
✓ Use JSON output for scripting (faster parsing)
✓ Cache expensive query results

### Usability
✓ Use table format for interactive use
✓ Use JSON/YAML for automation
✓ Set up shell aliases for common commands
✓ Use `--verbose` for debugging

## Shell Integration

### Bash Aliases

```bash
# Add to ~/.bashrc
alias mcli='memory-cli'
alias mep='memory-cli episode'
alias mpat='memory-cli pattern'
alias mstor='memory-cli storage'

# Quick episode start
estart() {
  memory-cli episode start "$1" \
    --language rust \
    --domain backend \
    --format plain
}
```

### Bash Completion

```bash
# Enable completion
memory-cli completion bash > ~/.local/share/bash-completion/completions/memory-cli
source ~/.bashrc
```

## Advanced Usage

### Scripting with JSON

```bash
#!/bin/bash
# Export all episodes to JSON

OUTPUT_DIR="exports"
mkdir -p "$OUTPUT_DIR"

# Get all episode IDs
EPISODES=$(memory-cli episode list --format json | jq -r '.episodes[].id')

# Export each episode
for ep in $EPISODES; do
  memory-cli episode view "$ep" --format json > "$OUTPUT_DIR/${ep}.json"
done

echo "Exported $(echo "$EPISODES" | wc -l) episodes"
```

### Integration with Tools

```bash
# Filter with jq
memory-cli episode list --format json | \
  jq '.episodes[] | select(.language == "rust")'

# Export to CSV
memory-cli episode list --format json | \
  jq -r '.episodes[] | [.id, .task_description, .verdict] | @csv' > episodes.csv

# Parallel processing
cat pattern_ids.txt | parallel -j 4 \
  memory-cli pattern analyze {} --format json
```

## Quick Reference

```
Episode Commands:
  start       Start new episode
  complete    Complete episode
  log-step    Log execution step
  list        List episodes
  view        View episode details

Pattern Commands:
  list          List patterns
  view          View pattern details
  analyze       Analyze pattern
  effectiveness Calculate metrics
  decay         Apply decay

Storage Commands:
  stats             Storage statistics
  sync              Sync storages
  vacuum            Optimize storage
  health            Check health
  connection-status Connection status

Operations:
  backup    Create/restore backups
  config    Manage configuration
  health    Health checks
  logs      Log analysis
  monitor   Start monitoring

Output Formats:
  --format json   JSON output
  --format yaml   YAML output
  --format table  Table output (default)
  --format plain  Plain text

Help:
  memory-cli --help              Show general help
  memory-cli <command> --help    Show command help
  memory-cli --version           Show version
```

## Resources

- **User Guide**: [memory-cli/CLI_USER_GUIDE.md](../../memory-cli/CLI_USER_GUIDE.md)
- **Configuration**: [memory-cli/CONFIGURATION_GUIDE.md](../../memory-cli/CONFIGURATION_GUIDE.md)
- **Memory Core**: [memory-core/README.md](../../memory-core/README.md)
- **Issue Tracker**: [GitHub Issues](https://github.com/d-o-hub/rust-self-learning-memory/issues)

## Integration

**Works with**:
- `memory-cli` agent - For implementing new CLI commands
- `test-runner` skill - For testing CLI commands
- `build-compile` skill - For building CLI binary
- `debug-troubleshoot` skill - For debugging CLI issues

**Use Cases**:
- Execute CLI commands during development workflows
- Guide users through CLI operations
- Troubleshoot CLI configuration and connectivity
- Automate memory system operations with scripts
- Monitor and maintain storage health

Remember: The memory-cli is designed for both interactive use and automation. Choose the right output format for your use case!
