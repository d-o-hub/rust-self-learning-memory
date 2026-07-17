# Memory CLI Configuration Guide

This guide covers all configuration options for the Memory CLI, including progressive disclosure, configuration wizard, caching, and examples for different use cases and deployment scenarios.

## Quick Start

### Interactive Wizard (Recommended for First-Time Setup)

```bash
do-memory-cli config wizard
```

The wizard will guide you through:
1. Preset selection (quick setup, local, cloud, custom)
2. Database configuration
3. Storage optimization
4. CLI preferences
5. Validation and saving

### Simple Mode

For one-call configuration:

```bash
# Use in your code
use memory_cli::config::Config;

// Automatic detection (recommended)
let config = Config::simple().await?;

// Or specify storage type
let config = Config::simple_with_storage(DatabaseType::Local).await?;

// Or specify performance level
let config = Config::simple_with_performance(PerformanceLevel::High).await?;
```

### Configuration File

Manual configuration via file:

## Configuration Precedence (issue #846)

Settings resolve highest-wins in this order:

| Priority | Source | Examples |
|----------|--------|----------|
| 1 (highest) | CLI flags | `--db-path`, `--storage-mode` |
| 2 | Environment variables | `MEMORY_DB_PATH`, `MEMORY_STORAGE_MODE` |
| 3 | Explicit config file | `--config /path/to.toml` |
| 4 | Auto-discovered config in CWD | `do-memory-cli.toml`, `memory-cli.toml` (and `.memory-cli.toml` / JSON / YAML variants) |
| 5 (lowest) | Built-in defaults | XDG paths, local storage defaults |

`--db-path` / `MEMORY_DB_PATH` and `--storage-mode` / `MEMORY_STORAGE_MODE` are clap options with env fallbacks: a CLI flag always beats the env var for that option, and both beat values loaded from a config file.

## Configuration File Locations

When no `--config` path is given, the CLI searches the current working directory for:

1. `memory-cli.toml` / `do-memory-cli.toml`
2. `memory-cli.json` / `do-memory-cli.json`
3. `memory-cli.yaml` / `do-memory-cli.yaml`
4. `.memory-cli.toml` / `.do-memory-cli.toml`
5. `.memory-cli.json` / `.do-memory-cli.json`
6. `.memory-cli.yaml` / `.do-memory-cli.yaml`

Explicit path via `--config /path/to/config.toml` always takes precedence over auto-discovery (level 3 above).


## Progressive Disclosure Configuration

Memory CLI uses progressive disclosure to reveal configuration options based on usage patterns.

### How It Works

1. **Initial Setup**: Only essential options shown (database URL/path)
2. **Usage-Based**: Additional options appear as you use advanced features
3. **Context-Aware**: Suggestions based on detected patterns

### Progressive Levels

```toml
# Level 1: Essentials (shown to everyone)
[database]
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"
# OR
redb_path = "~/.local/share/do-memory-cli/memory.redb"

# Level 2: Performance (shown after first use)
[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

# Level 3: Advanced (shown when using advanced features)
[cli]
default_format = "json"
progress_bars = true
batch_size = 100
```

### Usage Pattern Detection

The CLI automatically:
- Detects your database type (local vs cloud)
- Adjusts performance settings based on system resources
- Suggests optimizations based on usage patterns
- Validates recommendations against historical data

## Configuration Caching

Memory CLI caches configuration files to avoid repeated parsing, providing 200-500x speedup.

### How Caching Works

1. **First Load**: Configuration file is parsed and cached with mtime (modification time)
2. **Subsequent Loads**: Cache is checked first; if mtime matches, cached config is used
3. **Invalidation**: If file is modified, cache is invalidated and file is re-parsed

### Cache Statistics

```bash
# View cache statistics (programmatic)
use memory_cli::config::cache_stats;

let stats = cache_stats();
println!("Hit rate: {:.1}%", stats.hit_rate * 100.0);
```

Output:
```json
{
  "hits": 145,
  "misses": 3,
  "entries": 2,
  "hit_rate": 0.98
}
```

### Implementation Details

- **Thread-Safe**: Cache uses `Mutex` for safe concurrent access
- **Automatic**: No manual cache management required
- **Efficient**: O(1) lookup time with HashMap
- **Safe**: mtime-based validation prevents stale data
- **Performance**: 200-500x faster than repeated file parsing

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
# do-memory-cli.toml - Development setup
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
# do-memory-cli.toml - Production setup
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
# do-memory-cli.toml - Testing setup
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
# .do-memory-cli.toml - CI/CD setup
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
do-memory-cli storage health
```

### Complete Environment Variable Reference

| Environment Variable | Configuration Path | Description |
|---------------------|-------------------|-------------|
| `TURSO_URL` | `database.turso_url` | Turso database URL |
| `TURSO_TOKEN` | `database.turso_token` | Turso authentication token |
| `REDB_PATH` | `database.redb_path` | redb database file path |

## Quick Start: Generate a Config

```bash
# Write a starter local config (refuses to overwrite)
do-memory-cli config init
# → creates do-memory-cli.toml with storage_mode = "local"

# Or print a template to stdout
do-memory-cli config show-template

# Use a project-local path
do-memory-cli -c do-memory-cli.toml episode list

# Or override path without a config file (also sets redb_path):
do-memory-cli --storage-mode local --db-path ./data/memory.redb episode list
# MEMORY_DB_PATH=./data/memory.redb MEMORY_STORAGE_MODE=local do-memory-cli episode list
```

### Where does `storage_mode` go?

| Location | Supported? | Notes |
|----------|------------|-------|
| `[database].storage_mode` | ✅ Canonical | Preferred; emitted by `config init` |
| `--storage-mode` / `MEMORY_STORAGE_MODE` | ✅ CLI/env override | Wins over config file |
| `[storage].storage_mode` | ✅ Accepted alias | Copied into `[database]` if unset |

`[storage]` is for cache size / TTL / pool size — not backend selection.

### Minimal valid config (issue #829)

Partial files work; missing sections use defaults:

```toml
[database]
redb_path = "./.do-memory-cli/cache/memory.redb"
storage_mode = "local"
```

See also `memory-cli/config/do-memory-cli.example.toml`.

## Configuration Validation

The CLI validates configuration on startup. Use the `config` command to check your configuration:

```bash
# Validate configuration
do-memory-cli config validate

# Validate specific config file
do-memory-cli --config custom.toml config validate
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
chmod 600 do-memory-cli.toml

# Secure directory permissions
chmod 700 ~/.config/do-memory-cli/
```

### Example Secure Setup

```bash
# Create secure config directory
mkdir -p ~/.config/do-memory-cli
cd ~/.config/do-memory-cli

# Create configuration file
cat > do-memory-cli.toml << EOF
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
chmod 600 do-memory-cli.toml

# Set environment variable
export MEMORY_TURSO_TOKEN="your-secure-token"

# Test configuration
do-memory-cli config validate
```

## Troubleshooting Configuration

### Common Issues

1. **"Configuration file not found"**
   ```bash
   # Check if file exists
   ls -la do-memory-cli.toml

   # Try explicit path
   do-memory-cli --config ./do-memory-cli.toml config
   ```

2. **"Invalid configuration format"**
   ```bash
   # Validate TOML syntax
   python3 -c "import tomllib; tomllib.load(open('do-memory-cli.toml', 'rb'))"

   # Check for common mistakes:
   # - Missing quotes around strings
   # - Invalid boolean values (use true/false, not yes/no)
   # - Incorrect indentation (TOML doesn't use indentation)
   ```

3. **"Database connection failed"**
   ```bash
   # Check URL format
   grep turso_url do-memory-cli.toml

   # Verify token
   echo $MEMORY_TURSO_TOKEN | head -c 10  # Should show start of token

   # Test connectivity (if you have curl)
   curl -H "Authorization: Bearer $MEMORY_TURSO_TOKEN" "$TURSO_URL"
   ```

4. **"Permission denied"**
   ```bash
   # Check file permissions
   ls -la do-memory-cli.toml

   # Fix permissions
   chmod 600 do-memory-cli.toml
   ```

### Debug Configuration

Enable verbose logging to see configuration loading:

```bash
do-memory-cli --verbose config
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
cp do-memory-cli.toml do-memory-cli.dev.toml

# Production
cp do-memory-cli.toml do-memory-cli.prod.toml

# Use with aliases
alias do-memory-cli-dev='do-memory-cli --config do-memory-cli.dev.toml'
alias do-memory-cli-prod='do-memory-cli --config do-memory-cli.prod.toml'
```

### Configuration Templates

Use a base configuration with environment-specific overrides:

```bash
# Base configuration
cat > do-memory-cli.base.toml << EOF
[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "human"
progress_bars = true
batch_size = 100
EOF

# Environment-specific overrides (pass a different file with --config)
cat > do-memory-cli.dev.toml << EOF
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

BACKUP_DIR="$HOME/.do-memory-cli-backups"
mkdir -p "$BACKUP_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/do-memory-cli_$TIMESTAMP.toml"

if [ -f "do-memory-cli.toml" ]; then
    cp do-memory-cli.toml "$BACKUP_FILE"
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
if [ -f "do-memory-cli.toml" ]; then
    echo "✓ Configuration file found: do-memory-cli.toml"
else
    echo "✗ No configuration file found"
    echo "  Create one with: do-memory-cli --help"
    exit 1
fi

# Validate with CLI
echo
echo "Validating configuration..."
if do-memory-cli config validate > /dev/null 2>&1; then
    echo "✓ Configuration is valid"
else
    echo "✗ Configuration validation failed"
    do-memory-cli config validate
    exit 1
fi

# Show configuration summary
echo
echo "Configuration summary:"
echo "- Database: $(grep -c 'turso_url\|redb_path' do-memory-cli.toml) storage backend(s) configured"
echo "- Cache: $(grep 'max_episodes_cache' do-memory-cli.toml | cut -d'=' -f2 | tr -d ' ') episodes max"
echo "- CLI: $(grep 'default_format' do-memory-cli.toml | cut -d'=' -f2 | tr -d ' \"') output format"

echo
echo "=== Validation Complete ==="
```