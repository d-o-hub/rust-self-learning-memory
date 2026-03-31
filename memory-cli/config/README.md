# Memory CLI Configuration Examples

This directory contains example configuration files for various use cases. Choose the one that best matches your needs, copy it, and customize as needed.

## Quick Start

### 1. Choose Your Configuration

| Use Case | Configuration File | Description |
|----------|-------------------|-------------|
| 🚀 **Local Development** | `local-dev.toml` | Local SQLite, debug logging, moderate cache |
| ☁️ **Production Cloud** | `cloud-production.toml` | Turso cloud, large cache, JSON output |
| 🧪 **CI/CD Testing** | `ci-testing.toml` | In-memory, fast, no persistence |
| ⚡ **Minimal Setup** | `minimal.toml` | Bare minimum configuration |
| 📝 **Full Template** | `do-memory-cli.toml` | Complete template with all options |
| 🧑‍💻 **Test Configuration** | `test-config.toml` | Used for testing the CLI itself |

### 2. Copy and Customize

```bash
# For local development
cp do-memory-cli/config/local-dev.toml do-memory-cli.toml

# Edit to customize
nano do-memory-cli.toml

# Verify configuration
do-memory-cli config validate
```

### 3. Use Your Configuration

```bash
# Use specific config file
do-memory-cli --config do-memory-cli.toml health

# Or set environment variable
export MEMORY_CLI_CONFIG=do-memory-cli.toml
do-memory-cli health
```

## Configuration Files Explained

### `local-dev.toml` - Local Development ⭐ Recommended
**Perfect for:** Developers working locally, testing features

**Key Features:**
- Local SQLite database (no cloud dependencies)
- Debug logging for troubleshooting
- Moderate cache size (1000 episodes)
- Human-readable output with progress bars
- 30-minute cache TTL (balanced)

**Use when:**
- You're developing or testing locally
- You don't need cloud storage
- You want detailed debug information

```bash
cp do-memory-cli/config/local-dev.toml do-memory-cli.toml
```

---

### `cloud-production.toml` - Production Setup
**Perfect for:** Production deployments, cloud workloads

**Key Features:**
- Turso cloud database (high availability)
- Large cache (5000 episodes)
- JSON output for logging/automation
- 2-hour cache TTL (performance)
- Connection pool of 20 (high concurrency)

**Use when:**
- Running in production
- Need high availability
- Require cloud database
- High concurrency expected

**Security Note:** Use environment variables for `turso_token`:
```bash
export TURSO_AUTH_TOKEN="your-token-here"
```

---

### `ci-testing.toml` - CI/CD Pipelines
**Perfect for:** GitHub Actions, CI/CD, automated testing

**Key Features:**
- In-memory storage (no persistence)
- Fast, isolated tests
- JSON output for parsing
- Minimal logging (warn level)
- No progress bars (clean logs)

**Use when:**
- Running in CI/CD pipelines
- Need fast, isolated tests
- No persistent data required

```yaml
# .github/workflows/test.yml
- name: Run tests
  run: |
    do-memory-cli --config do-memory-cli/config/ci-testing.toml test
```

---

### `minimal.toml` - Bare Minimum
**Perfect for:** Quick setups, learning, simple use cases

**Key Features:**
- Only required fields
- Sensible defaults
- Easy to understand
- Local SQLite storage

**Use when:**
- You're just getting started
- You want the simplest setup
- You'll use mostly defaults

---

## Configuration Sections

### `[database]` - Storage Configuration

```toml
[database]
# Database URL - where your memory data is stored
turso_url = "file:./data/memory.db"  # Local SQLite
# turso_url = "libsql://db.turso.io"  # Remote Turso

# Authentication token (for remote Turso only)
turso_token = ""

# Local cache database path
redb_path = "./data/cache.redb"  # File-based cache
# redb_path = ":memory:"  # In-memory cache (testing)
```

**Common Patterns:**
- **Local dev:** `file:./data/memory.db`
- **Production:** `libsql://your-db.turso.io`
- **Testing:** `file::memory:?cache=shared`
- **CI/CD:** `:memory:` for both

---

### `[storage]` - Cache & Performance

```toml
[storage]
# How many episodes to cache
max_episodes_cache = 1000

# Cache time-to-live in seconds
cache_ttl_seconds = 3600  # 1 hour

# Database connection pool size
pool_size = 10
```

**Sizing Guide:**
| Environment | Episodes | TTL | Pool Size |
|-------------|----------|-----|-----------|
| Testing | 100-200 | 300s | 2-5 |
| Development | 500-1000 | 1800s | 5-10 |
| Production | 1000-5000 | 7200s | 10-20 |

**Memory Usage:**
- ~10KB per episode
- 1000 episodes ≈ 10MB
- 5000 episodes ≈ 50MB

---

### `[cli]` - User Interface

```toml
[cli]
# Output format for commands
default_format = "human"  # human, json, yaml

# Show progress bars for long operations
progress_bars = true

# Batch size for bulk operations
batch_size = 100
```

**Format Recommendations:**
- **Interactive use:** `human` (colored, readable)
- **Automation/scripts:** `json` (parseable)
- **Configuration files:** `yaml` (structured)

---

### `[monitoring]` - Observability (Optional)

```toml
[monitoring]
enabled = true
health_check_interval_seconds = 30
```

---

### `[backup]` - Backup Settings (Optional)

```toml
[backup]
backup_dir = "./backups"
max_backup_age_days = 30
compress_backups = true
```

---

### `[logging]` - Logging Configuration (Optional)

```toml
[logging]
level = "info"  # error, warn, info, debug, trace
log_file = "./logs/do-memory-cli.log"
max_log_size_mb = 10
max_log_files = 5
```

**Log Levels:**
- **error:** Only errors (production)
- **warn:** Warnings + errors (production)
- **info:** General information (default)
- **debug:** Detailed debugging (development)
- **trace:** Very verbose (troubleshooting)

---

## Environment Variables

Override configuration values using environment variables:

```bash
# Database configuration
export MEMORY_CLI_CONFIG="./my-config.toml"
export TURSO_AUTH_TOKEN="your-token-here"
export TURSO_DATABASE_URL="libsql://db.turso.io"

# CLI options
export MEMORY_CLI_FORMAT="json"
export MEMORY_CLI_LOG_LEVEL="debug"

# Run with environment variables
do-memory-cli health
```

---

## Configuration Validation

Validate your configuration before use:

```bash
# Validate configuration file
do-memory-cli config validate

# Validate specific file
do-memory-cli --config my-config.toml config validate

# Show configuration summary
do-memory-cli config show
```

---

## Interactive Configuration Wizard

Use the interactive wizard to create a configuration:

```bash
# Launch interactive wizard
do-memory-cli config wizard

# Wizard will guide you through:
# 1. Choosing a preset (Local/Cloud/Memory/Custom)
# 2. Database configuration
# 3. Storage settings
# 4. CLI preferences
# 5. Review and save
```

The wizard provides:
- ✅ Contextual help and examples
- ✅ Inline validation
- ✅ Smart defaults
- ✅ Visual feedback

---

## Troubleshooting

### Configuration not found
```bash
# Check current directory for config files
ls -la *.toml

# Use explicit path
do-memory-cli --config /path/to/config.toml health
```

### Validation errors
```bash
# Run validation to see detailed errors
do-memory-cli config validate

# Common issues:
# - Missing required fields
# - Invalid URLs
# - Path traversal in paths
# - Invalid values (negative numbers, etc.)
```

### Database connection issues
```bash
# Test database connection
do-memory-cli health

# Check Turso URL format
# ✅ Correct: libsql://your-db.turso.io
# ❌ Wrong: https://your-db.turso.io

# Verify token (for remote Turso)
# Use environment variable for security
export TURSO_AUTH_TOKEN="your-token"
```

---

## Best Practices

### Security
- ✅ **Never commit tokens** to version control
- ✅ Use environment variables for secrets
- ✅ Use `.env` files (add to `.gitignore`)
- ✅ Rotate tokens regularly

### Performance
- ✅ Use larger cache for production (3000-5000 episodes)
- ✅ Increase `cache_ttl_seconds` to reduce DB queries
- ✅ Tune `pool_size` based on concurrency needs
- ✅ Monitor cache hit rates

### Development
- ✅ Use `debug` log level during development
- ✅ Enable progress bars for visual feedback
- ✅ Use `human` output format interactively
- ✅ Keep cache size moderate (1000 episodes)

### Production
- ✅ Use `info` or `warn` log level
- ✅ Use `json` output for log parsing
- ✅ Disable progress bars (cleaner logs)
- ✅ Set up log rotation
- ✅ Monitor health checks

---

## Migration Guide

### Upgrading from older versions

If you're upgrading from an older version of do-memory-cli, your old configuration should still work (backward compatible). However, you can take advantage of new features:

1. **Add new sections** (optional):
   ```toml
   [monitoring]
   enabled = true
   
   [backup]
   backup_dir = "./backups"
   
   [logging]
   level = "info"
   ```

2. **Update database paths** (optional):
   - Old: `turso_url` and `redb_path` in separate sections
   - New: Both in `[database]` section (still supported)

3. **Validate** your updated config:
   ```bash
   do-memory-cli config validate
   ```

---

## Examples by Use Case

### Example 1: Local Development with Debug Logging

```toml
[database]
turso_url = "file:./dev-data/memory.db"
redb_path = "./dev-data/cache.redb"

[storage]
max_episodes_cache = 500
cache_ttl_seconds = 1800
pool_size = 5

[cli]
default_format = "human"
progress_bars = true
batch_size = 50

[logging]
level = "debug"
log_file = "./dev-data/debug.log"
```

### Example 2: Production with High Availability

```toml
[database]
turso_url = "libsql://prod-memory.turso.io"
# Set TURSO_AUTH_TOKEN environment variable
redb_path = "/var/lib/do-memory-cli/cache.redb"

[storage]
max_episodes_cache = 5000
cache_ttl_seconds = 7200
pool_size = 20

[cli]
default_format = "json"
progress_bars = false
batch_size = 500

[logging]
level = "warn"
log_file = "/var/log/do-memory-cli/app.log"
max_log_size_mb = 100
max_log_files = 10
```

### Example 3: GitHub Actions CI

```toml
[database]
turso_url = "file::memory:?cache=shared"
redb_path = ":memory:"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 300
pool_size = 2

[cli]
default_format = "json"
progress_bars = false
batch_size = 50

[logging]
level = "warn"
```

---

## Getting Help

- 📖 **Documentation:** See [CONFIGURATION.md](../CONFIGURATION.md)
- 🎨 **Wizard:** Run `do-memory-cli config wizard`
- ✅ **Validate:** Run `do-memory-cli config validate`
- 🔍 **Show config:** Run `do-memory-cli config show`
- 💬 **Issues:** https://github.com/d-o-hub/rust-self-learning-memory/issues

---

**Last Updated:** 2025-12-29 (v0.1.9)
