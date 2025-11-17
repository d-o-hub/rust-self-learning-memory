# Memory CLI

A command-line interface for the Self-Learning Memory System, providing direct access to episode management, pattern analysis, and storage operations.

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

### Configuration Format

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

#### Validate Configuration
```bash
memory-cli config
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