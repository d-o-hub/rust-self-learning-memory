# Memory CLI Configuration Guide

This guide covers all configuration options for the Memory CLI, including examples for different use cases and deployment scenarios.

## Configuration File Locations

The CLI searches for configuration files in this order:

1. **Explicit path**: `--config /path/to/config.toml`
2. `memory-cli.toml`
3. `memory-cli.json`
4. `memory-cli.yaml`
5. `.memory-cli.toml`
6. `.memory-cli.json`
7. `.memory-cli.yaml`

## Configuration Schema

### Complete Configuration Reference

```toml
[database]
# Turso database configuration
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"

# redb cache configuration
redb_path = "memory.redb"

[storage]
# Cache settings
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
# CLI behavior
default_format = "human"
progress_bars = true
batch_size = 100
```

### Database Section

#### Turso Configuration

```toml
[database]
# Required: Turso database URL
turso_url = "libsql://your-db.turso.io"

# Optional: Authentication token (can also use MEMORY_TURSO_TOKEN env var)
turso_token = "your-auth-token"
```

**Environment Variables:**
- `MEMORY_TURSO_URL`: Override turso_url
- `MEMORY_TURSO_TOKEN`: Override turso_token

#### redb Configuration

```toml
[database]
# Optional: Path to redb database file (default: "memory.redb")
redb_path = "memory.redb"

# Use in-memory database (for testing)
redb_path = ":memory:"
```

### Storage Section

```toml
[storage]
# Maximum episodes to keep in cache (default: 1000)
max_episodes_cache = 1000

# Cache entry time-to-live in seconds (default: 3600 = 1 hour)
cache_ttl_seconds = 3600

# Database connection pool size (default: 10)
pool_size = 10
```

### CLI Section

```toml
[cli]
# Default output format: "human", "json", or "yaml" (default: "human")
default_format = "human"

# Enable progress bars for long operations (default: true)
progress_bars = true

# Batch size for bulk operations (default: 100)
batch_size = 100
```

## Configuration Examples

### Development Configuration

```toml
# memory-cli.toml - Development setup
[database]
turso_url = "libsql://dev-db.turso.io"
turso_token = "dev-token-123"

[storage]
max_episodes_cache = 500
cache_ttl_seconds = 1800  # 30 minutes
pool_size = 5

[cli]
default_format = "human"
progress_bars = true
batch_size = 50
```

### Production Configuration

```toml
# memory-cli.toml - Production setup
[database]
turso_url = "libsql://prod-db.turso.io"
turso_token = "prod-token-456"

[storage]
max_episodes_cache = 5000
cache_ttl_seconds = 7200  # 2 hours
pool_size = 20

[cli]
default_format = "json"  # Machine-readable for automation
progress_bars = false   # Disable for headless operation
batch_size = 200
```

### Testing Configuration

```toml
# memory-cli.toml - Testing setup
[database]
# Use in-memory redb for testing
redb_path = ":memory:"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 300   # 5 minutes
pool_size = 2

[cli]
default_format = "json"
progress_bars = false
batch_size = 10
```

### CI/CD Configuration

```toml
# .memory-cli.toml - CI/CD setup
[database]
turso_url = "libsql://ci-db.turso.io"
turso_token = "ci-token-789"

[storage]
max_episodes_cache = 200
cache_ttl_seconds = 600   # 10 minutes
pool_size = 3

[cli]
default_format = "json"
progress_bars = false
batch_size = 25
```

## Alternative Formats

### JSON Configuration

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

### YAML Configuration

```yaml
database:
  turso_url: "libsql://your-db.turso.io"
  turso_token: "your-auth-token"
  redb_path: "memory.redb"

storage:
  max_episodes_cache: 1000
  cache_ttl_seconds: 3600
  pool_size: 10

cli:
  default_format: "human"
  progress_bars: true
  batch_size: 100
```

## Environment Variable Overrides

You can override any configuration value using environment variables:

```bash
# Override database URL
export MEMORY_TURSO_URL="libsql://override-db.turso.io"

# Override token
export MEMORY_TURSO_TOKEN="override-token"

# Run CLI with overrides
memory-cli storage health
```

### Complete Environment Variable Reference

| Environment Variable | Configuration Path | Description |
|---------------------|-------------------|-------------|
| `MEMORY_TURSO_URL` | `database.turso_url` | Turso database URL |
| `MEMORY_TURSO_TOKEN` | `database.turso_token` | Turso authentication token |
| `MEMORY_REDB_PATH` | `database.redb_path` | redb database file path |

## Configuration Validation

The CLI validates configuration on startup. Use the `config` command to check your configuration:

```bash
# Validate configuration
memory-cli config

# Validate specific config file
memory-cli --config custom.toml config
```

### Validation Rules

- **Database**: Either `turso_url` or `redb_path` must be configured
- **Storage**: `max_episodes_cache` and `pool_size` must be > 0
- **CLI**: `default_format` must be "human", "json", or "yaml"
- **CLI**: `batch_size` must be > 0

## Security Considerations

### Token Management

- **Never commit tokens** to version control
- Use environment variables for sensitive values
- Rotate tokens regularly
- Use different tokens for different environments

### File Permissions

```bash
# Secure configuration file permissions
chmod 600 memory-cli.toml

# Secure directory permissions
chmod 700 ~/.config/memory-cli/
```

### Example Secure Setup

```bash
# Create secure config directory
mkdir -p ~/.config/memory-cli
cd ~/.config/memory-cli

# Create configuration file
cat > memory-cli.toml << EOF
[database]
turso_url = "libsql://secure-db.turso.io"
# Token will be provided via environment variable

[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "human"
progress_bars = true
batch_size = 100
EOF

# Set secure permissions
chmod 600 memory-cli.toml

# Set environment variable
export MEMORY_TURSO_TOKEN="your-secure-token"

# Test configuration
memory-cli config
```

## Troubleshooting Configuration

### Common Issues

1. **"Configuration file not found"**
   ```bash
   # Check if file exists
   ls -la memory-cli.toml

   # Try explicit path
   memory-cli --config ./memory-cli.toml config
   ```

2. **"Invalid configuration format"**
   ```bash
   # Validate TOML syntax
   python3 -c "import tomllib; tomllib.load(open('memory-cli.toml', 'rb'))"

   # Check for common mistakes:
   # - Missing quotes around strings
   # - Invalid boolean values (use true/false, not yes/no)
   # - Incorrect indentation (TOML doesn't use indentation)
   ```

3. **"Database connection failed"**
   ```bash
   # Check URL format
   grep turso_url memory-cli.toml

   # Verify token
   echo $MEMORY_TURSO_TOKEN | head -c 10  # Should show start of token

   # Test connectivity (if you have curl)
   curl -H "Authorization: Bearer $MEMORY_TURSO_TOKEN" "$TURSO_URL"
   ```

4. **"Permission denied"**
   ```bash
   # Check file permissions
   ls -la memory-cli.toml

   # Fix permissions
   chmod 600 memory-cli.toml
   ```

### Debug Configuration

Enable verbose logging to see configuration loading:

```bash
memory-cli --verbose config
```

This will show:
- Configuration file search paths
- Which file was loaded
- Configuration parsing details
- Validation results

## Advanced Configuration

### Multiple Environment Configurations

Create separate configuration files for different environments:

```bash
# Development
cp memory-cli.toml memory-cli.dev.toml

# Production
cp memory-cli.toml memory-cli.prod.toml

# Use with aliases
alias memory-cli-dev='memory-cli --config memory-cli.dev.toml'
alias memory-cli-prod='memory-cli --config memory-cli.prod.toml'
```

### Configuration Templates

Use a base configuration with environment-specific overrides:

```bash
# Base configuration
cat > memory-cli.base.toml << EOF
[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "human"
progress_bars = true
batch_size = 100
EOF

# Environment-specific overrides
cat > memory-cli.dev.toml << EOF
import = "memory-cli.base.toml"

[database]
turso_url = "libsql://dev-db.turso.io"
turso_token = "dev-token"

[storage]
max_episodes_cache = 500
EOF
```

### Configuration Management Scripts

#### Configuration Backup
```bash
#!/bin/bash
# Backup current configuration

BACKUP_DIR="$HOME/.memory-cli-backups"
mkdir -p "$BACKUP_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/memory-cli_$TIMESTAMP.toml"

if [ -f "memory-cli.toml" ]; then
    cp memory-cli.toml "$BACKUP_FILE"
    echo "Configuration backed up to: $BACKUP_FILE"
else
    echo "No configuration file found to backup"
    exit 1
fi
```

#### Configuration Validation Script
```bash
#!/bin/bash
# Validate configuration and show details

echo "=== Memory CLI Configuration Validation ==="
echo

# Check if config exists
if [ -f "memory-cli.toml" ]; then
    echo "✓ Configuration file found: memory-cli.toml"
else
    echo "✗ No configuration file found"
    echo "  Create one with: memory-cli --help"
    exit 1
fi

# Validate with CLI
echo
echo "Validating configuration..."
if memory-cli config > /dev/null 2>&1; then
    echo "✓ Configuration is valid"
else
    echo "✗ Configuration validation failed"
    memory-cli config
    exit 1
fi

# Show configuration summary
echo
echo "Configuration summary:"
echo "- Database: $(grep -c 'turso_url\|redb_path' memory-cli.toml) storage backend(s) configured"
echo "- Cache: $(grep 'max_episodes_cache' memory-cli.toml | cut -d'=' -f2 | tr -d ' ') episodes max"
echo "- CLI: $(grep 'default_format' memory-cli.toml | cut -d'=' -f2 | tr -d ' \"') output format"

echo
echo "=== Validation Complete ==="
```