# Memory CLI

A command-line interface for the Self-Learning Memory System, providing direct access to episode management, pattern analysis, and storage operations.

## Documentation

- **[CLI User Guide](CLI_USER_GUIDE.md)** - Complete command reference and usage examples
- **[Configuration Guide](CONFIGURATION_GUIDE.md)** - Detailed configuration options and examples

## Installation

### From Source

```bash
# Build with CLI support
cargo build --release --features full

# Or install globally
cargo install --path memory-cli --features full
```

### Feature Flags

- `turso`: Enable Turso database backend
- `redb`: Enable redb database backend
- `full`: Enable both backends

## Configuration

The CLI supports configuration via:

1. **Default locations** (searched in order):
    - `memory-cli.toml`
    - `memory-cli.json`
    - `memory-cli.yaml`
    - `.memory-cli.toml`
    - `.memory-cli.json`
    - `.memory-cli.yaml`

2. **Explicit path**:
    ```bash
    memory-cli --config /path/to/config.toml <command>
    ```

For complete configuration documentation, see **[Configuration Guide](CONFIGURATION_GUIDE.md)**.

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
memory-cli episode create --task "Implement user authentication" --context user-context.json
```

#### List Episodes
```bash
# List all episodes
memory-cli episode list

# Filter by task type and limit results
memory-cli episode list --task-type "feature" --limit 10

# Filter by status
memory-cli episode list --status completed
```

#### View Episode
```bash
memory-cli episode view 12345678-1234-1234-1234-123456789abc
```

#### Complete Episode
```bash
memory-cli episode complete 12345678-1234-1234-1234-123456789abc --outcome success
```

#### Search Episodes
```bash
memory-cli episode search "authentication" --limit 5
```

#### Log Step
```bash
memory-cli episode log-step 12345678-1234-1234-1234-123456789abc \
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
memory-cli pattern list

# Filter by confidence and type
memory-cli pattern list --min-confidence 0.8 --pattern-type ToolSequence

# Limit results
memory-cli pattern list --limit 20
```

#### View Pattern
```bash
memory-cli pattern view pattern-123
```

#### Analyze Pattern
```bash
memory-cli pattern analyze pattern-123 --episodes 100
```

#### Pattern Effectiveness
```bash
# Show top 10 most effective patterns
memory-cli pattern effectiveness --top 10

# Filter by minimum uses
memory-cli pattern effectiveness --min-uses 5
```

#### Pattern Decay
```bash
# Preview decay operation
memory-cli pattern decay --dry-run

# Apply decay (requires confirmation)
memory-cli pattern decay --force
```

### Storage Operations

#### Storage Statistics
```bash
memory-cli storage stats
```

#### Sync Storage
```bash
# Normal sync
memory-cli storage sync

# Force full sync
memory-cli storage sync --force

# Preview sync operation
memory-cli storage sync --dry-run
```

#### Vacuum Storage
```bash
# Preview vacuum operation
memory-cli storage vacuum --dry-run

# Execute vacuum
memory-cli storage vacuum
```

#### Health Check
```bash
memory-cli storage health
```

#### Connection Status
```bash
memory-cli storage connections
```

### Configuration Management

#### Validate Configuration
```bash
# Basic validation
memory-cli config validate

# Check with recommendations
memory-cli config check

# Show current configuration
memory-cli config show
```

### Health Monitoring

#### Health Check
```bash
# Comprehensive health check
memory-cli health check

# Show current status
memory-cli health status

# Monitor continuously
memory-cli health monitor --interval 30 --duration 300
```

### Backup and Restore

#### Create Backup
```bash
# Create JSON backup
memory-cli backup create ./backups --format json --compress

# Create SQL backup
memory-cli backup create ./backups --format sql
```

#### List Backups
```bash
memory-cli backup list ./backups
```

#### Restore from Backup
```bash
# Restore specific backup
memory-cli backup restore ./backups --backup-id backup_20251117_120000

# Force restore (overwrite existing data)
memory-cli backup restore ./backups --backup-id backup_20251117_120000 --force
```

#### Verify Backup
```bash
memory-cli backup verify ./backups --backup-id backup_20251117_120000
```

### Monitoring and Metrics

#### Show Status
```bash
memory-cli monitor status
```

#### Export Metrics
```bash
# Export as Prometheus format
memory-cli monitor export --format prometheus

# Export as JSON
memory-cli monitor export --format json
```

### Log Analysis

#### Analyze Logs
```bash
# Analyze last 24 hours
memory-cli logs analyze --since 24h

# Analyze with custom filter
memory-cli logs analyze --since 7d --filter "error"
```

#### Search Logs
```bash
# Search for specific terms
memory-cli logs search "authentication" --limit 20 --since 24h

# Search with multiple terms
memory-cli logs search "timeout connection" --since 1h
```

#### Export Logs
```bash
# Export as JSON
memory-cli logs export ./exports/logs.json --format json --since 24h

# Export as CSV
memory-cli logs export ./exports/logs.csv --format csv --since 7d
```

#### Log Statistics
```bash
memory-cli logs stats --since 24h
```

### Meta Commands

#### Generate Completions
```bash
# Bash
memory-cli completion bash > memory-cli.bash

# Zsh
memory-cli completion zsh > _memory-cli

# Fish
memory-cli completion fish > memory-cli.fish
```

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
episodes=$(memory-cli episode list --limit 10 --format json)

# Process each episode
echo "$episodes" | jq -r '.episodes[].episode_id' | while read episode_id; do
    echo "Processing episode: $episode_id"
    memory-cli episode view "$episode_id" --format json > "episode_$episode_id.json"
done
```

### Pattern Effectiveness Monitoring
```bash
#!/bin/bash

# Get pattern effectiveness as JSON
effectiveness=$(memory-cli pattern effectiveness --top 5 --format json)

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
if ! memory-cli storage health --format json | jq -e '.overall == "healthy"' > /dev/null; then
    echo "Storage health check failed!"
    memory-cli storage health
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
docker-compose -f docker/docker-compose.yml logs -f memory-cli

# Scale monitoring services
docker-compose -f docker/docker-compose.yml up -d --scale prometheus=2
```

#### Systemd Service
```bash
# Enable and start service
sudo systemctl enable memory-cli
sudo systemctl start memory-cli

# Check status
sudo systemctl status memory-cli

# View logs
sudo journalctl -u memory-cli -f
```

### Health Monitoring

#### Automated Health Checks
```bash
# Continuous monitoring
memory-cli health monitor --interval 30 --duration 3600

# Health check in scripts
if memory-cli health check --format json | jq -e '.overall_status == "Healthy"'; then
    echo "System is healthy"
else
    echo "Health check failed"
    exit 1
fi
```

#### Integration with Monitoring Systems
```bash
# Export Prometheus metrics
memory-cli monitor export --format prometheus > metrics.txt

# Export for external analysis
memory-cli monitor export --format json > metrics.json
```

### Backup and Recovery

#### Automated Backups
```bash
#!/bin/bash
# Daily backup script
BACKUP_DIR="./backups"
DATE=$(date +%Y%m%d_%H%M%S)

memory-cli backup create "$BACKUP_DIR" \
    --format json \
    --compress \
    --backup-id "daily_$DATE"

# Cleanup old backups (keep last 30 days)
find "$BACKUP_DIR" -name "daily_*" -mtime +30 -delete
```

#### Disaster Recovery
```bash
# Verify backup integrity
memory-cli backup verify ./backups --backup-id daily_20251117_020000

# Restore from backup
memory-cli backup restore ./backups --backup-id daily_20251117_020000 --force
```

### Log Analysis and Troubleshooting

#### Automated Log Analysis
```bash
# Daily log analysis
memory-cli logs analyze --since 24h > daily_report.json

# Error trend analysis
memory-cli logs search "error timeout" --since 7d --format json | \
    jq '.results | group_by(.episode_id) | map({episode: .[0].episode_id, errors: length})'
```

#### Performance Monitoring
```bash
# Export performance metrics
memory-cli logs stats --since 1h --format json

# Identify slow operations
memory-cli logs analyze --since 24h --format json | \
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
memory-cli --verbose <command>
```

### Dry Run

Preview operations without executing them:
```bash
memory-cli --dry-run <command>
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