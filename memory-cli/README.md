# Memory CLI

A full-featured command-line interface for the Self-Learning Memory System, providing direct access to episode management, pattern analysis, storage operations, monitoring, diagnostics, backup/restore, log analysis, and evaluation tools.

## Documentation

- **[CLI User Guide](CLI_USER_GUIDE.md)** - Complete command reference and usage examples
- **[Configuration Guide](CONFIGURATION_GUIDE.md)** - Detailed configuration options and examples
- **[Configuration Reference](CONFIGURATION.md)** - Configuration options and defaults

## Overview

Memory CLI provides a comprehensive command-line interface for managing the self-learning memory system with:

- **10 Core Commands**: episode, pattern, storage, config, health, backup, monitor, logs, eval, completion
- **9 Command Aliases**: ep, pat, st, cfg, hp, bak, mon, log, comp for faster typing
- **Multiple Output Formats**: human-readable, JSON, YAML, and table formats
- **Intelligent Configuration**: Progressive disclosure, wizard setup, and smart defaults
- **Production Features**: Health monitoring, backup/restore, metrics export, and log analysis
- **Developer Tools**: Validation, diagnostics, calibration, and threshold management

## Installation

### From Source

```bash
# Build with CLI support
cargo build --release --features full

# Or install globally
cargo install --path do-memory-cli --features full
```

### Feature Flags

- `turso`: Enable Turso database backend
- `redb`: Enable redb database backend
- `full`: Enable both backends (recommended)

## Configuration

The CLI supports configuration via:

1. **Default locations** (searched in order):
    - `do-memory-cli.toml` / `do-memory-cli.json` / `do-memory-cli.yaml`
    - `.do-memory-cli.toml` / `.do-memory-cli.json` / `.do-memory-cli.yaml`
    - `config.toml` / `config.json` / `config.yaml`
    - `~/.config/do-memory-cli/config.*` (user config directory)

2. **Explicit path**:
    ```bash
    do-memory-cli --config /path/to/config.toml <command>
    ```

3. **Configuration wizard**:
    ```bash
    do-memory-cli config wizard
    ```

### Key Configuration Features

- **Progressive Disclosure**: Automatically reveals options based on usage patterns
- **Multi-Format Support**: TOML, JSON, and YAML configuration files
- **Configuration Wizard**: Interactive step-by-step setup for new users
- **Configuration Caching**: Fast configuration loading with automatic mtime-based cache invalidation (200-500x speedup)
- **Smart Defaults**: Automatic detection of optimal settings based on system resources
- **Simple Mode API**: One-call configuration for common scenarios

For complete configuration documentation, see **[Configuration Guide](CONFIGURATION_GUIDE.md)** and **[Configuration Reference](CONFIGURATION.md)**.

### Quick Configuration

#### TOML
```toml
[database]
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"
redb_path = "memory.redb"

[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "human"
progress_bars = true
batch_size = 100
```

#### JSON
```json
{
  "database": {
    "turso_url": "libsql://your-db.turso.io",
    "turso_token": "your-auth-token",
    "redb_path": "memory.redb"
  },
  "storage": {
    "max_episodes_cache": 1000,
    "cache_ttl_seconds": 3600,
    "pool_size": 10
  },
  "cli": {
    "default_format": "human",
    "progress_bars": true,
    "batch_size": 100
  }
}
```

## Commands

### Episode Management

#### Create Episode
```bash
do-memory-cli episode create --task "Implement user authentication" --context user-context.json
```

#### List Episodes
```bash
# List all episodes
do-memory-cli episode list

# Filter by task type and limit results
do-memory-cli episode list --task-type "feature" --limit 10

# Filter by status
do-memory-cli episode list --status completed
```

#### View Episode
```bash
do-memory-cli episode view 12345678-1234-1234-1234-123456789abc
```

#### Complete Episode
```bash
do-memory-cli episode complete 12345678-1234-1234-1234-123456789abc --outcome success
```

#### Search Episodes
```bash
do-memory-cli episode search "authentication" --limit 5
```

#### Log Step
```bash
do-memory-cli episode log-step 12345678-1234-1234-1234-123456789abc \
  --tool "grep" \
  --action "Search for authentication patterns" \
  --success true \
  --latency-ms 150 \
  --tokens 25 \
  --observation "Found 3 relevant patterns"
```

### Pattern Analysis

#### List Patterns
```bash
# List all patterns
do-memory-cli pattern list

# Filter by confidence and type
do-memory-cli pattern list --min-confidence 0.8 --pattern-type ToolSequence

# Limit results
do-memory-cli pattern list --limit 20
```

#### View Pattern
```bash
do-memory-cli pattern view pattern-123
```

#### Analyze Pattern
```bash
do-memory-cli pattern analyze pattern-123 --episodes 100
```

#### Pattern Effectiveness
```bash
# Show top 10 most effective patterns
do-memory-cli pattern effectiveness --top 10

# Filter by minimum uses
do-memory-cli pattern effectiveness --min-uses 5
```

#### Pattern Decay
```bash
# Preview decay operation
do-memory-cli pattern decay --dry-run

# Apply decay (requires confirmation)
do-memory-cli pattern decay --force
```

### Storage Operations

#### Storage Statistics
```bash
do-memory-cli storage stats
```

#### Sync Storage
```bash
# Normal sync
do-memory-cli storage sync

# Force full sync
do-memory-cli storage sync --force

# Preview sync operation
do-memory-cli storage sync --dry-run
```

#### Vacuum Storage
```bash
# Preview vacuum operation
do-memory-cli storage vacuum --dry-run

# Execute vacuum
do-memory-cli storage vacuum
```

#### Health Check
```bash
do-memory-cli storage health
```

#### Connection Status
```bash
do-memory-cli storage connections
```

### Configuration Management

#### Validate Configuration
```bash
# Basic validation
do-memory-cli config validate

# Check with recommendations
do-memory-cli config check

# Show current configuration
do-memory-cli config show
```

### Health Monitoring

#### Health Check
```bash
# Comprehensive health check
do-memory-cli health check

# Show current status
do-memory-cli health status

# Monitor continuously
do-memory-cli health monitor --interval 30 --duration 300
```

### Backup and Restore

#### Create Backup
```bash
# Create JSON backup
do-memory-cli backup create ./backups --format json --compress

# Create SQL backup
do-memory-cli backup create ./backups --format sql
```

#### List Backups
```bash
do-memory-cli backup list ./backups
```

#### Restore from Backup
```bash
# Restore specific backup
do-memory-cli backup restore ./backups --backup-id backup_20251117_120000

# Force restore (overwrite existing data)
do-memory-cli backup restore ./backups --backup-id backup_20251117_120000 --force
```

#### Verify Backup
```bash
do-memory-cli backup verify ./backups --backup-id backup_20251117_120000
```

### Monitoring and Metrics

#### Show Status
```bash
do-memory-cli monitor status
```

#### Export Metrics
```bash
# Export as Prometheus format
do-memory-cli monitor export --format prometheus

# Export as JSON
do-memory-cli monitor export --format json
```

### Log Analysis (alias: `log`)

#### Analyze Logs
```bash
# Analyze last 24 hours
do-memory-cli logs analyze --since 24h

# Analyze with custom filter
do-memory-cli logs analyze --since 7d --filter "error"
```

#### Search Logs
```bash
# Search for specific terms
do-memory-cli logs search "authentication" --limit 20 --since 24h

# Search with multiple terms
do-memory-cli logs search "timeout connection" --since 1h
```

#### Export Logs
```bash
# Export as JSON
do-memory-cli logs export ./exports/logs.json --format json --since 24h

# Export as CSV
do-memory-cli logs export ./exports/logs.csv --format csv --since 7d
```

#### Log Statistics
```bash
do-memory-cli logs stats --since 24h
```

### Evaluation and Calibration (alias: `ev`)

#### Calibration Statistics
```bash
# View all domains
do-memory-cli eval calibration --all

# View specific domain
do-memory-cli eval calibration --domain web-development

# View reliable domains only
do-memory-cli eval calibration --min-episodes 10
```

#### Domain Statistics
```bash
do-memory-cli eval stats web-development
```

#### Set Threshold
```bash
# Set duration threshold
do-memory-cli eval set-threshold --domain web-development --duration 300

# Set step count threshold
do-memory-cli eval set-threshold --domain web-development --steps 15
```

### Meta Commands

#### Generate Completions (alias: `comp`)
```bash
# Bash
do-memory-cli completion bash > do-memory-cli.bash

# Zsh
do-memory-cli completion zsh > _do-memory-cli

# Fish
do-memory-cli completion fish > do-memory-cli.fish
```

## Command Aliases

The CLI provides convenient shortcuts for frequently used commands:

| Alias | Full Command | Description |
|-------|--------------|-------------|
| `ep` | `episode` | Episode management |
| `pat` | `pattern` | Pattern analysis |
| `st` | `storage` | Storage operations |
| `cfg` | `config` | Configuration management |
| `hp` | `health` | Health monitoring |
| `bak` | `backup` | Backup and restore |
| `mon` | `monitor` | Monitoring and metrics |
| `log` | `logs` | Log analysis |
| `comp` | `completion` | Shell completions |
| `ev` | `eval` | Evaluation and calibration |

Example:
```bash
# Long form
do-memory-cli episode list

# Short form
do-memory-cli ep list
```

## Recent Improvements (v0.1.4)

### Interactive Confirmations
- Safety prompts for destructive operations (pattern decay, storage sync, vacuum)
- Preview operations before execution with `--dry-run`
- Safe defaults (No) with `--force` or `--yes` flags for automation

### Enhanced Error Messages
- Color-coded error output (red errors, yellow suggestions, cyan numbering)
- Context-rich error messages with helpful suggestions
- Pre-defined helper messages for common error scenarios
- Enhanced error handling infrastructure in `errors.rs` module

### Command Aliases
All 10 commands now have convenient aliases for faster CLI usage (see table above).

### Fixed Duplicate Storage Initialization
- Resolved issues with multiple storage backend initialization
- Improved storage backend detection and management

## Output Formats

### Human (Default)
Human-readable output with colors and formatting:
```
Episodes: 150 (showing 10)
────────────────────────────────────────────────────────────────────────────────
12345678 completed Implement user authentication
23456789 in_progress Refactor database layer
...
```

### JSON
Machine-readable JSON output:
```json
{
  "episodes": [
    {
      "episode_id": "12345678-1234-1234-1234-123456789abc",
      "task_description": "Implement user authentication",
      "status": "completed",
      "created_at": "2025-11-17T10:00:00Z",
      "duration_ms": 1500,
      "steps_count": 5
    }
  ],
  "total_count": 150
}
```

### YAML
Configuration-friendly YAML output:
```yaml
episodes:
- episode_id: 12345678-1234-1234-1234-123456789abc
  task_description: Implement user authentication
  status: completed
  created_at: "2025-11-17T10:00:00Z"
  duration_ms: 1500
  steps_count: 5
total_count: 150
```

## Scripting Examples

### Batch Episode Processing
```bash
#!/bin/bash

# Get recent episodes as JSON
episodes=$(do-memory-cli episode list --limit 10 --format json)

# Process each episode
echo "$episodes" | jq -r '.episodes[].episode_id' | while read episode_id; do
    echo "Processing episode: $episode_id"
    do-memory-cli episode view "$episode_id" --format json > "episode_$episode_id.json"
done
```

### Pattern Effectiveness Monitoring
```bash
#!/bin/bash

# Get pattern effectiveness as JSON
effectiveness=$(do-memory-cli pattern effectiveness --top 5 --format json)

# Check for patterns below threshold
echo "$effectiveness" | jq '.patterns[] | select(.effectiveness < 0.7)' | while read pattern; do
    echo "Low effectiveness pattern found:"
    echo "$pattern" | jq .
done
```

### Health Monitoring
```bash
#!/bin/bash

# Check storage health
if ! do-memory-cli storage health --format json | jq -e '.overall == "healthy"' > /dev/null; then
    echo "Storage health check failed!"
    do-memory-cli storage health
    exit 1
fi

echo "All systems healthy"
```

## Error Handling

The CLI provides clear error messages and appropriate exit codes:

- `0`: Success
- `1`: General error
- `2`: Configuration error
- `3`: Validation error

Use `--verbose` for detailed error information and stack traces.

## Operational Features

### Production Deployment

The CLI includes comprehensive operational features for production deployment:

#### Automated Deployment
```bash
# Run the deployment script
./deploy.sh

# Deploy with systemd service
./deploy.sh --systemd

# Deploy with Docker
./deploy.sh --docker
```

#### Docker Deployment
```bash
# Start all services
docker-compose -f docker/docker-compose.yml up -d

# View logs
docker-compose -f docker/docker-compose.yml logs -f do-memory-cli

# Scale monitoring services
docker-compose -f docker/docker-compose.yml up -d --scale prometheus=2
```

#### Systemd Service
```bash
# Enable and start service
sudo systemctl enable do-memory-cli
sudo systemctl start do-memory-cli

# Check status
sudo systemctl status do-memory-cli

# View logs
sudo journalctl -u do-memory-cli -f
```

### Health Monitoring

#### Automated Health Checks
```bash
# Continuous monitoring
do-memory-cli health monitor --interval 30 --duration 3600

# Health check in scripts
if do-memory-cli health check --format json | jq -e '.overall_status == "Healthy"'; then
    echo "System is healthy"
else
    echo "Health check failed"
    exit 1
fi
```

#### Integration with Monitoring Systems
```bash
# Export Prometheus metrics
do-memory-cli monitor export --format prometheus > metrics.txt

# Export for external analysis
do-memory-cli monitor export --format json > metrics.json
```

### Backup and Recovery

#### Automated Backups
```bash
#!/bin/bash
# Daily backup script
BACKUP_DIR="./backups"
DATE=$(date +%Y%m%d_%H%M%S)

do-memory-cli backup create "$BACKUP_DIR" \
    --format json \
    --compress \
    --backup-id "daily_$DATE"

# Cleanup old backups (keep last 30 days)
find "$BACKUP_DIR" -name "daily_*" -mtime +30 -delete
```

#### Disaster Recovery
```bash
# Verify backup integrity
do-memory-cli backup verify ./backups --backup-id daily_20251117_020000

# Restore from backup
do-memory-cli backup restore ./backups --backup-id daily_20251117_020000 --force
```

### Log Analysis and Troubleshooting

#### Automated Log Analysis
```bash
# Daily log analysis
do-memory-cli logs analyze --since 24h > daily_report.json

# Error trend analysis
do-memory-cli logs search "error timeout" --since 7d --format json | \
    jq '.results | group_by(.episode_id) | map({episode: .[0].episode_id, errors: length})'
```

#### Performance Monitoring
```bash
# Export performance metrics
do-memory-cli logs stats --since 1h --format json

# Identify slow operations
do-memory-cli logs analyze --since 24h --format json | \
    jq '.performance_trends[] | select(.average_latency_ms > 1000)'
```

## Integration with MCP Server

The CLI complements the MCP server by providing:

- **Direct Access**: Command-line operations without MCP protocol overhead
- **Scripting**: Easy integration into automation scripts and CI/CD pipelines
- **Debugging**: Detailed inspection capabilities for troubleshooting
- **Administration**: Storage maintenance and monitoring tools

Both interfaces share the same core logic and validation, ensuring consistency.

## Troubleshooting

### Common Issues

1. **Configuration not found**
   ```
   Error: Failed to read config file
   ```
   Solution: Create a configuration file or ensure database credentials are set.

2. **Database connection failed**
   ```
   Error: Connection refused
   ```
   Solution: Check database URL and credentials in configuration.

3. **Permission denied**
   ```
   Error: Permission denied (os error 13)
   ```
   Solution: Ensure proper file permissions for redb database file.

### Debug Mode

Enable verbose logging for troubleshooting:
```bash
do-memory-cli --verbose <command>
```

### Dry Run

Preview operations without executing them:
```bash
do-memory-cli --dry-run <command>
```

## Contributing

When adding new commands:

1. Implement the command logic in the appropriate module (`episode.rs`, `pattern.rs`, or `storage.rs`)
2. Add comprehensive tests
3. Update this documentation
4. Ensure consistent output formatting
5. Follow the existing error handling patterns

## License

This CLI is part of the Self-Learning Memory System and follows the same MIT license.
## Architecture

### Key Modules

- **commands/**: Command implementations for all CLI operations
  - `episode.rs`: Episode management commands
  - `pattern.rs`: Pattern analysis commands
  - `storage.rs`: Storage operations
  - `config.rs`: Configuration validation and management
  - `health.rs`: Health monitoring and diagnostics
  - `backup.rs`: Backup and restore operations
  - `monitor.rs`: Monitoring and metrics export
  - `logs.rs`: Log analysis and search
  - `eval.rs`: Evaluation and calibration

- **config/**: Configuration system with progressive disclosure
  - `loader.rs`: Configuration loading with caching
  - `validator.rs`: Configuration validation
  - `wizard.rs`: Interactive configuration wizard
  - `simple.rs`: Simple mode API
  - `progressive.rs`: Progressive disclosure
  - `types.rs`: Configuration types

- **output.rs**: Output formatting (human, JSON, YAML, table)
- **errors.rs`: Enhanced error handling with context
- **main.rs`: CLI entry point and command routing

### Dependencies

- **clap** 4.4: CLI framework with derive features
- **clap_complete** 4.5: Shell completion generation
- **dialoguer** 0.12: Interactive terminal prompts
- **indicatif** 0.18: Progress bars for long operations
- **colored** 3.0: Colorized console output
- **serde_yaml** 0.9: YAML configuration support
- **dirs** 5.0: Cross-platform directory paths
- **sysinfo** 0.30: System resource detection
- **regex** 1.10: Pattern matching for log analysis

### Core Integrations

- **do-memory-core**: Core memory operations and APIs
- **do-memory-storage-turso**: Turso database backend (optional)
- **do-memory-storage-redb**: redb database backend (optional)
