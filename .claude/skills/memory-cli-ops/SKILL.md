---
name: memory-cli-ops
description: Master memory-cli operations, commands, and usage patterns. Use when working with CLI commands, troubleshooting CLI issues, adding new commands, or helping users understand CLI functionality.
---

# Memory CLI Operations

Comprehensive guide for working with the memory-cli command-line interface.

## Purpose

Provide expert knowledge of memory-cli commands, operations, usage patterns, and best practices for the self-learning memory system CLI tool.

## Command Reference

### Episode Management Commands

#### `episode start`
Start a new learning episode with context.

**Usage**:
```bash
memory-cli episode start <DESCRIPTION> [OPTIONS]

Options:
  --language <LANG>      Programming language (rust, python, javascript, etc.)
  --framework <FW>       Framework being used
  --domain <DOMAIN>      Problem domain (required)
  --tags <TAG>...        Tags for categorization
  --complexity <LEVEL>   Complexity level (simple, moderate, complex, very-complex)
  --format <FORMAT>      Output format (json, yaml, table, plain)

Example:
memory-cli episode start "Implement authentication" \
  --language rust \
  --framework tokio \
  --domain backend \
  --tags auth,security \
  --complexity moderate
```

**Output**: Episode ID for tracking

#### `episode complete`
Complete an episode with outcome and reflection.

**Usage**:
```bash
memory-cli episode complete <EPISODE_ID> [OPTIONS]

Options:
  --verdict <TEXT>       Completion verdict/summary
  --artifacts <PATH>...  Paths to generated artifacts
  --success              Mark as successful (default: true)
  --format <FORMAT>      Output format

Example:
memory-cli episode complete ep-12345 \
  --verdict "Successfully implemented JWT authentication" \
  --artifacts src/auth.rs tests/auth_tests.rs \
  --success
```

**Output**: Completion summary with patterns extracted

#### `episode log-step`
Log an execution step during episode.

**Usage**:
```bash
memory-cli episode log-step <EPISODE_ID> [OPTIONS]

Options:
  --step-number <NUM>    Step sequence number (auto-increments if not provided)
  --tool <TOOL>          Tool used (cargo, git, grep, etc.)
  --action <ACTION>      Action taken
  --latency <MS>         Latency in milliseconds
  --tokens <COUNT>       Token count if applicable
  --success              Step succeeded (default: true)
  --observation <TEXT>   Observation/result

Example:
memory-cli episode log-step ep-12345 \
  --step-number 1 \
  --tool cargo \
  --action "cargo build" \
  --latency 1250 \
  --success \
  --observation "Build completed successfully"
```

#### `episode list`
List episodes with optional filtering.

**Usage**:
```bash
memory-cli episode list [OPTIONS]

Options:
  --language <LANG>      Filter by language
  --domain <DOMAIN>      Filter by domain
  --tags <TAG>...        Filter by tags (OR logic)
  --task-type <TYPE>     Filter by task type
  --limit <N>            Maximum results (default: 50)
  --offset <N>           Skip first N results
  --sort <FIELD>         Sort by field (timestamp, duration)
  --order <ORDER>        Sort order (asc, desc)
  --format <FORMAT>      Output format

Example:
memory-cli episode list \
  --language rust \
  --domain backend \
  --limit 20 \
  --sort timestamp \
  --order desc \
  --format table
```

**Output**: Table or structured list of episodes

#### `episode view`
View detailed information about an episode.

**Usage**:
```bash
memory-cli episode view <EPISODE_ID> [OPTIONS]

Options:
  --format <FORMAT>      Output format
  --include-steps        Include execution steps
  --include-patterns     Include extracted patterns

Example:
memory-cli episode view ep-12345 \
  --include-steps \
  --include-patterns \
  --format yaml
```

**Output**: Complete episode details with steps and patterns

### Pattern Management Commands

#### `pattern list`
List all patterns with filtering.

**Usage**:
```bash
memory-cli pattern list [OPTIONS]

Options:
  --pattern-type <TYPE>  Filter by type (tool-sequence, decision-point, error-recovery, success-factor)
  --language <LANG>      Filter by language
  --domain <DOMAIN>      Filter by domain
  --min-frequency <N>    Minimum frequency count
  --min-success <RATE>   Minimum success rate (0.0-1.0)
  --limit <N>            Maximum results
  --sort <FIELD>         Sort by field (frequency, success_rate, last_seen)
  --format <FORMAT>      Output format

Example:
memory-cli pattern list \
  --pattern-type tool-sequence \
  --language rust \
  --min-frequency 5 \
  --min-success 0.8 \
  --sort success_rate \
  --order desc \
  --format table
```

**Output**: Table of patterns with statistics

#### `pattern view`
View detailed pattern information.

**Usage**:
```bash
memory-cli pattern view <PATTERN_ID> [OPTIONS]

Options:
  --format <FORMAT>      Output format
  --include-examples     Include example episodes

Example:
memory-cli pattern view pattern-abc123 \
  --include-examples \
  --format yaml
```

**Output**: Complete pattern details with context and examples

#### `pattern analyze`
Analyze pattern effectiveness over time.

**Usage**:
```bash
memory-cli pattern analyze <PATTERN_ID> [OPTIONS]

Options:
  --time-window <DAYS>   Analysis time window (default: 30)
  --format <FORMAT>      Output format

Example:
memory-cli pattern analyze pattern-abc123 \
  --time-window 90 \
  --format json
```

**Output**: Analysis with trends, success rates, and recommendations

#### `pattern effectiveness`
Calculate overall pattern effectiveness metrics.

**Usage**:
```bash
memory-cli pattern effectiveness [OPTIONS]

Options:
  --language <LANG>      Filter by language
  --domain <DOMAIN>      Filter by domain
  --pattern-type <TYPE>  Filter by pattern type
  --format <FORMAT>      Output format

Example:
memory-cli pattern effectiveness \
  --language rust \
  --domain backend \
  --format table
```

**Output**: Effectiveness metrics aggregated by criteria

#### `pattern decay`
Apply decay to aging patterns.

**Usage**:
```bash
memory-cli pattern decay [OPTIONS]

Options:
  --decay-rate <RATE>    Decay rate (0.0-1.0, default: 0.1)
  --older-than <DAYS>    Apply to patterns older than N days
  --dry-run              Show what would be decayed without applying
  --format <FORMAT>      Output format

Example:
memory-cli pattern decay \
  --decay-rate 0.15 \
  --older-than 90 \
  --dry-run
```

**Output**: Summary of patterns decayed

### Storage Operations Commands

#### `storage stats`
Display storage statistics and metrics.

**Usage**:
```bash
memory-cli storage stats [OPTIONS]

Options:
  --detailed             Show detailed backend statistics
  --format <FORMAT>      Output format

Example:
memory-cli storage stats --detailed --format table
```

**Output**: Storage metrics including episode count, pattern count, database sizes, cache hit rates

#### `storage sync`
Synchronize Turso (durable) and redb (cache) storage.

**Usage**:
```bash
memory-cli storage sync [OPTIONS]

Options:
  --direction <DIR>      Sync direction (turso-to-redb, redb-to-turso, bidirectional)
  --dry-run              Show what would be synced
  --force                Force sync even if timestamps match
  --format <FORMAT>      Output format

Example:
memory-cli storage sync \
  --direction bidirectional \
  --format json
```

**Output**: Sync summary with items updated

#### `storage vacuum`
Optimize database storage and reclaim space.

**Usage**:
```bash
memory-cli storage vacuum [OPTIONS]

Options:
  --backend <NAME>       Vacuum specific backend (turso, redb, all)
  --aggressive           Perform aggressive optimization
  --analyze              Run ANALYZE after vacuum
  --format <FORMAT>      Output format

Example:
memory-cli storage vacuum \
  --backend all \
  --aggressive \
  --analyze
```

**Output**: Space reclaimed and optimization summary

#### `storage health`
Check health of storage backends.

**Usage**:
```bash
memory-cli storage health [OPTIONS]

Options:
  --detailed             Show detailed health metrics
  --format <FORMAT>      Output format

Example:
memory-cli storage health --detailed --format json
```

**Output**: Health status for each backend (connection, latency, disk space)

#### `storage connection-status`
Verify storage connection status.

**Usage**:
```bash
memory-cli storage connection-status [OPTIONS]

Options:
  --backend <NAME>       Check specific backend (turso, redb, all)
  --format <FORMAT>      Output format

Example:
memory-cli storage connection-status --backend all
```

**Output**: Connection status and latency for each backend

### Operational Commands

#### `backup create`
Create a backup of the memory system.

**Usage**:
```bash
memory-cli backup create [OPTIONS]

Options:
  --output <PATH>        Backup output path
  --compress             Compress backup file
  --include-cache        Include redb cache in backup
  --format <FORMAT>      Backup format (tar.gz, zip)

Example:
memory-cli backup create \
  --output backups/memory-2025-11-20.tar.gz \
  --compress \
  --include-cache
```

**Output**: Backup file path and size

#### `backup restore`
Restore from a backup.

**Usage**:
```bash
memory-cli backup restore <BACKUP_PATH> [OPTIONS]

Options:
  --confirm              Skip confirmation prompt
  --clear-existing       Clear existing data before restore
  --format <FORMAT>      Output format

Example:
memory-cli backup restore backups/memory-2025-11-20.tar.gz \
  --clear-existing \
  --confirm
```

**Output**: Restore summary with items restored

#### `config init`
Initialize CLI configuration.

**Usage**:
```bash
memory-cli config init [OPTIONS]

Options:
  --config-path <PATH>   Custom config file path
  --interactive          Interactive configuration wizard
  --overwrite            Overwrite existing configuration

Example:
memory-cli config init --interactive
```

**Output**: Configuration file path

#### `config show`
Display current configuration.

**Usage**:
```bash
memory-cli config show [OPTIONS]

Options:
  --format <FORMAT>      Output format
  --mask-secrets         Mask sensitive values

Example:
memory-cli config show --format yaml --mask-secrets
```

**Output**: Current configuration with masked secrets

#### `health check`
Run comprehensive health checks.

**Usage**:
```bash
memory-cli health check [OPTIONS]

Options:
  --detailed             Show detailed check results
  --format <FORMAT>      Output format

Example:
memory-cli health check --detailed --format json
```

**Output**: Health check results for all components

#### `logs analyze`
Analyze CLI logs for issues.

**Usage**:
```bash
memory-cli logs analyze [OPTIONS]

Options:
  --log-file <PATH>      Path to log file
  --level <LEVEL>        Filter by level (error, warn, info, debug)
  --since <DURATION>     Analyze logs from last N hours/days
  --pattern <REGEX>      Filter by pattern
  --format <FORMAT>      Output format

Example:
memory-cli logs analyze \
  --level error \
  --since 24h \
  --format table
```

**Output**: Log analysis summary with error patterns

#### `monitor start`
Start monitoring dashboard.

**Usage**:
```bash
memory-cli monitor start [OPTIONS]

Options:
  --port <PORT>          Dashboard port (default: 8080)
  --refresh <SECONDS>    Refresh interval (default: 5)
  --metrics <METRICS>... Metrics to display

Example:
memory-cli monitor start \
  --port 8080 \
  --refresh 5 \
  --metrics episodes,patterns,storage
```

**Output**: Monitoring dashboard URL

## Output Formats

### JSON Format
Structured data for scripting and automation.

```bash
memory-cli episode list --format json
```

```json
{
  "episodes": [
    {
      "id": "ep-12345",
      "task_description": "Implement auth",
      "language": "rust",
      "domain": "backend",
      "start_time": "2025-11-20T10:00:00Z",
      "end_time": "2025-11-20T11:30:00Z",
      "verdict": "Success"
    }
  ],
  "total": 1,
  "limit": 50,
  "offset": 0
}
```

### YAML Format
Human-readable structured format.

```bash
memory-cli episode view ep-12345 --format yaml
```

```yaml
episode:
  id: ep-12345
  task_description: Implement authentication
  context:
    language: rust
    framework: tokio
    domain: backend
    tags:
      - auth
      - security
  steps:
    - step_number: 1
      tool: cargo
      action: build
      success: true
```

### Table Format
Formatted table for terminal viewing.

```bash
memory-cli pattern list --format table
```

```
┌──────────────┬─────────────────┬───────────┬─────────────┬──────────────┐
│ Pattern ID   │ Type            │ Frequency │ Success Rate│ Last Seen    │
├──────────────┼─────────────────┼───────────┼─────────────┼──────────────┤
│ pattern-abc  │ ToolSequence    │ 42        │ 0.95        │ 2 hours ago  │
│ pattern-def  │ DecisionPoint   │ 28        │ 0.88        │ 5 hours ago  │
└──────────────┴─────────────────┴───────────┴─────────────┴──────────────┘
```

### Plain Format
Simple text output for pipes and grep.

```bash
memory-cli episode list --format plain
```

```
ep-12345: Implement authentication (Success)
ep-12346: Add logging (Success)
ep-12347: Fix bug #42 (Partial Success)
```

## Configuration

### Config File Location

**Default Paths** (in order of precedence):
1. `$MEMORY_CLI_CONFIG` (environment variable)
2. `./memory-cli.toml` (current directory)
3. `~/.config/memory-cli/config.toml` (user config)
4. `/etc/memory-cli/config.toml` (system config)

### Config File Format

```toml
[storage]
# Turso (durable storage)
turso_url = "libsql://your-database.turso.io"
turso_token = "${TURSO_AUTH_TOKEN}"  # Use env var for security

# redb (cache)
redb_path = "~/.local/share/memory-cli/cache.redb"

[output]
# Default output format
default_format = "table"

# Enable colored output
color = true

# Verbose output by default
verbose = false

[logging]
# Log level (error, warn, info, debug, trace)
level = "info"

# Log file path
file = "~/.local/share/memory-cli/logs/cli.log"

# Max log file size (MB)
max_size = 100

# Number of log files to keep
max_backups = 5

[backup]
# Default backup directory
directory = "~/.local/share/memory-cli/backups"

# Automatic backup interval (hours)
auto_interval = 24

# Keep last N backups
keep_last = 7

[performance]
# Query result cache size
cache_size = 1000

# Cache TTL (seconds)
cache_ttl = 300

# Max concurrent operations
max_concurrent = 10
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MEMORY_CLI_CONFIG` | Custom config file path | (see defaults above) |
| `TURSO_DATABASE_URL` | Turso database URL | (required) |
| `TURSO_AUTH_TOKEN` | Turso auth token | (required) |
| `MEMORY_CLI_LOG_LEVEL` | Logging level | `info` |
| `NO_COLOR` | Disable colored output | `false` |
| `MEMORY_CLI_VERBOSE` | Enable verbose output | `false` |

## Common Workflows

### Complete Episode Workflow

```bash
# 1. Start episode
EPISODE_ID=$(memory-cli episode start "Implement feature X" \
  --language rust \
  --domain backend \
  --tags feature,api \
  --format plain | awk '{print $1}')

# 2. Log steps as you work
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
# 1. List patterns by success rate
memory-cli pattern list \
  --sort success_rate \
  --order desc \
  --min-frequency 5 \
  --format table

# 2. Analyze top pattern
memory-cli pattern analyze pattern-abc123 \
  --time-window 90 \
  --format yaml

# 3. Check effectiveness across domain
memory-cli pattern effectiveness \
  --domain backend \
  --format json > backend_patterns.json
```

### Storage Maintenance Workflow

```bash
# 1. Check storage health
memory-cli storage health --detailed

# 2. Sync storages
memory-cli storage sync --direction bidirectional

# 3. Vacuum to reclaim space
memory-cli storage vacuum --backend all --aggressive

# 4. Verify stats
memory-cli storage stats --detailed --format table
```

### Backup and Restore Workflow

```bash
# 1. Create backup before major changes
memory-cli backup create \
  --output backups/pre-migration-$(date +%Y%m%d).tar.gz \
  --compress \
  --include-cache

# 2. Perform operations...

# 3. If needed, restore from backup
memory-cli backup restore backups/pre-migration-20251120.tar.gz \
  --confirm
```

## Troubleshooting Guide

### Command Fails with "Storage Connection Failed"

**Symptoms**: Commands hang or fail with connection errors

**Diagnosis**:
```bash
# Check connection status
memory-cli storage connection-status

# Check configuration
memory-cli config show --mask-secrets
```

**Solutions**:
1. Verify `TURSO_DATABASE_URL` and `TURSO_AUTH_TOKEN` are set
2. Check network connectivity to Turso
3. Verify credentials with: `curl -H "Authorization: Bearer $TURSO_AUTH_TOKEN" $TURSO_DATABASE_URL`
4. Try syncing: `memory-cli storage sync --force`

### "Config File Not Found"

**Symptoms**: CLI can't find configuration file

**Diagnosis**:
```bash
# Check which config file is being used
memory-cli config show 2>&1 | grep "config file"
```

**Solutions**:
1. Initialize config: `memory-cli config init --interactive`
2. Set custom path: `export MEMORY_CLI_CONFIG=~/my-config.toml`
3. Create config manually in default location

### Command Returns "Permission Denied"

**Symptoms**: Can't write to data directories

**Diagnosis**:
```bash
# Check permissions
ls -la ~/.local/share/memory-cli/
ls -la ~/.config/memory-cli/
```

**Solutions**:
1. Fix permissions: `chmod -R u+rw ~/.local/share/memory-cli/`
2. Run with sudo if needed (not recommended)
3. Change data directory in config

### Slow Command Execution

**Symptoms**: Commands take a long time to complete

**Diagnosis**:
```bash
# Check storage health
memory-cli storage health --detailed

# Check storage stats
memory-cli storage stats --detailed
```

**Solutions**:
1. Vacuum databases: `memory-cli storage vacuum --aggressive`
2. Increase cache size in config
3. Sync storages: `memory-cli storage sync`
4. Check network latency to Turso

### Pattern Decay Not Working

**Symptoms**: Old patterns not being decayed

**Diagnosis**:
```bash
# List old patterns
memory-cli pattern list --sort last_seen --order asc

# Dry run decay
memory-cli pattern decay --older-than 90 --dry-run
```

**Solutions**:
1. Run decay manually: `memory-cli pattern decay --older-than 90`
2. Set up automated decay with cron/systemd timer
3. Check decay rate is appropriate (not 0.0)

## Performance Tips

### Optimize List Commands

```bash
# Use pagination for large result sets
memory-cli episode list --limit 20 --offset 0

# Use specific filters to reduce results
memory-cli pattern list --language rust --domain backend

# Cache results for repeated queries
memory-cli pattern effectiveness --format json > cache.json
```

### Batch Operations

```bash
# Process multiple episodes from a file
while read episode_id; do
  memory-cli episode view $episode_id --format json >> episodes.jsonl
done < episode_ids.txt

# Parallel processing (GNU parallel)
cat pattern_ids.txt | parallel -j 4 \
  memory-cli pattern analyze {} --format json
```

### Monitoring Performance

```bash
# Time commands
time memory-cli episode list --limit 1000

# Enable profiling
RUST_LOG=trace memory-cli storage stats 2>&1 | grep "duration"

# Use monitoring dashboard
memory-cli monitor start --metrics episodes,patterns,storage,latency
```

## Best Practices

### Security
- ✅ Store credentials in environment variables, not config files
- ✅ Use `--mask-secrets` when sharing config output
- ✅ Restrict config file permissions: `chmod 600 ~/.config/memory-cli/config.toml`
- ✅ Regularly rotate Turso auth tokens
- ✅ Use backup encryption for sensitive data

### Reliability
- ✅ Set up automated backups with cron/systemd timers
- ✅ Monitor storage health regularly
- ✅ Vacuum databases periodically
- ✅ Sync storages after major operations
- ✅ Keep CLI version up to date

### Performance
- ✅ Use appropriate pagination limits
- ✅ Filter queries as much as possible
- ✅ Vacuum databases regularly
- ✅ Monitor cache hit rates
- ✅ Use JSON output for scripting (faster parsing)

### Usability
- ✅ Use table format for interactive use
- ✅ Use JSON/YAML for automation
- ✅ Set up shell aliases for common commands
- ✅ Create scripts for common workflows
- ✅ Use `--verbose` when debugging

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

# Quick pattern lookup
plookup() {
  memory-cli pattern view "$1" --format yaml | less
}
```

### Bash Completion

```bash
# Enable completion
memory-cli completion bash > ~/.local/share/bash-completion/completions/memory-cli

# Reload shell
source ~/.bashrc
```

### ZSH Integration

```zsh
# Add to ~/.zshrc
alias mcli='memory-cli'

# Enable completion
memory-cli completion zsh > ~/.zsh/completion/_memory-cli
```

## Advanced Usage

### Scripting with JSON Output

```bash
#!/bin/bash
# Script to export all episodes to JSON

OUTPUT_DIR="exports"
mkdir -p "$OUTPUT_DIR"

# Get all episode IDs
EPISODES=$(memory-cli episode list --format json | jq -r '.episodes[].id')

# Export each episode
for ep in $EPISODES; do
  memory-cli episode view "$ep" --format json > "$OUTPUT_DIR/${ep}.json"
done

echo "Exported $(echo "$EPISODES" | wc -l) episodes to $OUTPUT_DIR"
```

### Integration with Other Tools

```bash
# Use with jq for JSON processing
memory-cli episode list --format json | \
  jq '.episodes[] | select(.language == "rust")'

# Use with yq for YAML processing
memory-cli config show --format yaml | \
  yq '.storage.turso_url'

# Pipe to grep and awk
memory-cli pattern list --format plain | \
  grep "ToolSequence" | \
  awk '{print $1}'

# Export to CSV
memory-cli episode list --format json | \
  jq -r '.episodes[] | [.id, .task_description, .verdict] | @csv' > episodes.csv
```

## Resources

- **User Guide**: [CLI_USER_GUIDE.md](../../memory-cli/CLI_USER_GUIDE.md)
- **Configuration**: [CONFIGURATION_GUIDE.md](../../memory-cli/CONFIGURATION_GUIDE.md)
- **README**: [memory-cli/README.md](../../memory-cli/README.md)
- **Memory Core**: [memory-core docs](../../memory-core/README.md)
- **Issue Tracker**: [GitHub Issues](https://github.com/d-o-hub/rust-self-learning-memory/issues)

## Quick Reference Card

```
# Episode Commands
start       Start new episode
complete    Complete episode
log-step    Log execution step
list        List episodes
view        View episode details

# Pattern Commands
list          List patterns
view          View pattern details
analyze       Analyze pattern
effectiveness Calculate metrics
decay         Apply decay

# Storage Commands
stats             Storage statistics
sync              Sync storages
vacuum            Optimize storage
health            Check health
connection-status Connection status

# Operations
backup    Create/restore backups
config    Manage configuration
health    Health checks
logs      Log analysis
monitor   Start monitoring

# Output Formats
--format json   JSON output
--format yaml   YAML output
--format table  Table output
--format plain  Plain text

# Help
memory-cli --help              Show general help
memory-cli <command> --help    Show command help
memory-cli --version           Show version
```

Remember: The memory-cli is designed for both interactive use and automation. Choose the right output format for your use case!
