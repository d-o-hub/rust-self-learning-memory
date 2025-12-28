# Memory CLI Configuration Guide

**Version**: 0.1.7
**Last Updated**: 2025-12-28

## Table of Contents

- [Quick Start](#quick-start)
- [Configuration Methods](#configuration-methods)
- [Configuration Reference](#configuration-reference)
- [Simple Mode](#simple-mode)
- [Configuration Wizard](#configuration-wizard)
- [Environment Variables](#environment-variables)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Zero Configuration (Recommended)

```bash
# Use intelligent defaults
memory-cli init

# Start using immediately
memory-cli episode list
```

The CLI will automatically:
- Detect optimal storage backend (local or cloud)
- Configure appropriate performance settings
- Set up data directories
- Initialize the database

### Simple Mode

For explicit control with minimal configuration:

```rust
use memory_cli::config::Config;

// Local storage with standard performance
let config = Config::simple().await?;

// Or specify database type
let config = Config::simple_with_storage(DatabaseType::Local).await?;

// Or specify performance level
let config = Config::simple_with_performance(PerformanceLevel::High).await?;
```

### Configuration File

Create `~/.config/memory-cli/config.toml`:

```toml
[database]
# Local embedded database (recommended for development)
redb_path = "~/.local/share/memory-cli/memory.redb"

# OR cloud database (recommended for production)
# turso_url = "libsql://your-db.turso.io"
# turso_token = "your-auth-token"

[storage]
max_episodes_cache = 5000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "json"
progress_bars = true
batch_size = 50
```

## Configuration Methods

Memory CLI supports multiple configuration methods (in priority order):

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Configuration file** (TOML, JSON, or YAML)
4. **Interactive wizard**
5. **Intelligent defaults** (lowest priority)

### Method 1: Command-Line Arguments

```bash
memory-cli --config /path/to/config.toml episode list
memory-cli --database local episode add "Implement feature"
memory-cli --format json episode list
```

### Method 2: Environment Variables

```bash
# Config file location
export MEMORY_CLI_CONFIG=/path/to/config.toml

# Database settings
export TURSO_DATABASE_URL=libsql://your-db.turso.io
export TURSO_AUTH_TOKEN=your-token

# Performance settings
export MEMORY_POOL_SIZE=10
export MEMORY_CACHE_SIZE=5000

# Run CLI
memory-cli episode list
```

### Method 3: Configuration File

Memory CLI searches for configuration files in these locations (in order):

1. Path specified by `MEMORY_CLI_CONFIG` environment variable
2. `./memory-cli.toml` (current directory)
3. `./config.toml`
4. `~/.config/memory-cli/config.toml` (user config directory)
5. `~/.memory-cli.toml` (user home directory)
6. `/etc/memory-cli/config.toml` (system-wide, Linux/macOS only)

Supported formats:
- **TOML** (recommended): `config.toml`
- **JSON**: `config.json`
- **YAML**: `config.yaml`

### Method 4: Interactive Wizard

```bash
memory-cli config wizard

# Follow the prompts:
# - Choose preset (local/cloud/custom)
# - Configure database
# - Configure storage
# - Configure CLI options
# - Review and save
```

### Method 5: Intelligent Defaults

If no configuration is provided, Memory CLI uses intelligent defaults based on:
- Available system resources (CPU, RAM)
- Detected environment (local vs cloud)
- Platform (Linux, macOS, Windows)

## Configuration Reference

### Database Configuration

Controls where and how episodes are stored.

```toml
[database]
# Local embedded database (redb)
redb_path = "~/.local/share/memory-cli/memory.redb"

# OR Cloud database (Turso)
turso_url = "libsql://your-db.turso.io"
turso_token = "your-auth-token"

# Connection pooling (optional)
pool_size = 10                    # Number of database connections (default: CPU count * 2)
connection_timeout_ms = 5000      # Connection timeout in milliseconds
max_idle_connections = 5          # Maximum idle connections to keep
```

**Database Types:**

| Type | Use Case | Pros | Cons |
|------|----------|------|------|
| **redb** (Local) | Development, single-user | Fast, no network, no setup | Single machine only |
| **Turso** (Cloud) | Production, multi-user | Distributed, replicated | Requires internet, setup |
| **In-Memory** | Testing, temporary | Fastest | Lost on restart |

### Storage Configuration

Controls caching and performance settings.

```toml
[storage]
# Cache settings
max_episodes_cache = 5000         # Maximum episodes to keep in memory cache
cache_ttl_seconds = 3600          # Cache time-to-live (1 hour)

# Connection pool
pool_size = 10                    # Database connection pool size

# Performance tuning
enable_compression = true         # Compress large artifacts (default: true)
enable_metrics = true             # Enable performance metrics (default: true)
```

**Performance Levels:**

| Level | Cache Size | Pool Size | Use Case |
|-------|------------|-----------|----------|
| **Minimal** | 100 | 2 | Resource-constrained (Raspberry Pi, VPS) |
| **Standard** | 1000 | 5 | Typical development machine |
| **High** | 5000+ | 10+ | Production servers, high throughput |

### CLI Configuration

Controls CLI behavior and output formatting.

```toml
[cli]
# Output formatting
default_format = "json"           # Default output format: json, yaml, table, human
color_output = true               # Enable colored output (default: true)
progress_bars = true              # Show progress bars for long operations

# Batch operations
batch_size = 50                   # Number of items to process per batch
parallel_operations = 4           # Number of parallel operations

# Behavior
auto_confirm = false              # Skip confirmation prompts (default: false)
verbose = false                   # Enable verbose logging (default: false)
```

## Simple Mode

Simple Mode provides one-call configuration for common scenarios.

### Available Configurations

```rust
// 1. Automatic (recommended)
Config::simple().await?
// Detects: Local if no Turso env vars, Cloud if Turso detected
// Performance: Based on system resources

// 2. Explicit database type
Config::simple_with_storage(DatabaseType::Local).await?
Config::simple_with_storage(DatabaseType::Cloud).await?
Config::simple_with_storage(DatabaseType::Memory).await?

// 3. Explicit performance level
Config::simple_with_performance(PerformanceLevel::Minimal).await?
Config::simple_with_performance(PerformanceLevel::Standard).await?
Config::simple_with_performance(PerformanceLevel::High).await?

// 4. Both explicit
Config::simple_full(DatabaseType::Local, PerformanceLevel::High).await?
```

### Defaults by Database Type

**Local (redb)**:
- Path: `~/.local/share/memory-cli/memory.redb`
- Pool size: Based on CPU count
- Cache: 1000 episodes (standard), 5000 (high)

**Cloud (Turso)**:
- URL: From `TURSO_DATABASE_URL` env var
- Token: From `TURSO_AUTH_TOKEN` env var
- Pool size: 10 (standard), 20 (high)
- Cache: 1000 episodes (standard), 5000 (high)

**Memory (In-Memory)**:
- No persistence
- Pool size: 1
- Cache: All episodes (no limit)

## Configuration Wizard

Interactive step-by-step configuration setup.

### Running the Wizard

```bash
memory-cli config wizard
```

### Wizard Steps

1. **Choose Preset**
   - Quick setup (recommended)
   - Local development
   - Cloud production
   - Custom configuration

2. **Database Configuration**
   - Local: Choose data directory
   - Cloud: Enter Turso credentials
   - Validate connection

3. **Storage Configuration**
   - Cache size (based on system RAM)
   - Pool size (based on CPU cores)
   - Performance level

4. **CLI Configuration**
   - Default output format
   - Progress bars
   - Batch size

5. **Review and Save**
   - Review all settings
   - Validate configuration
   - Save to file

### Wizard Features

- **Smart Recommendations**: Based on system resources and environment
- **Real-Time Validation**: Check settings as you enter them
- **Helpful Examples**: Shows examples for each field
- **Error Guidance**: Clear messages with fix suggestions
- **Progress Indication**: Shows current step and total steps

## Environment Variables

### Database Variables

```bash
# Turso cloud database
TURSO_DATABASE_URL=libsql://your-db.turso.io
TURSO_AUTH_TOKEN=your-auth-token

# Local database path (alternative)
MEMORY_REDB_PATH=/path/to/memory.redb
```

### Performance Variables

```bash
# Connection pooling
MEMORY_POOL_SIZE=10
MEMORY_CONNECTION_TIMEOUT=5000

# Caching
MEMORY_CACHE_SIZE=5000
MEMORY_CACHE_TTL=3600
```

### CLI Variables

```bash
# Configuration file
MEMORY_CLI_CONFIG=/path/to/config.toml

# Output formatting
MEMORY_CLI_FORMAT=json
MEMORY_CLI_NO_COLOR=1          # Disable colors
MEMORY_CLI_QUIET=1             # Suppress non-essential output
```

## Examples

### Example 1: Local Development

```toml
# config.toml
[database]
redb_path = "./data/dev-memory.redb"

[storage]
max_episodes_cache = 500
cache_ttl_seconds = 1800
pool_size = 2

[cli]
default_format = "human"
progress_bars = true
batch_size = 25
```

### Example 2: Cloud Production

```toml
# config.toml
[database]
turso_url = "libsql://prod-memory.turso.io"
turso_token = "${TURSO_AUTH_TOKEN}"  # Read from env var
pool_size = 20
connection_timeout_ms = 3000

[storage]
max_episodes_cache = 10000
cache_ttl_seconds = 7200
enable_compression = true
enable_metrics = true

[cli]
default_format = "json"
progress_bars = false  # For CI/CD
batch_size = 100
```

### Example 3: Testing/CI

```toml
# config.toml
[database]
# Use in-memory database for fast tests
# (configure via code, not file)

[storage]
max_episodes_cache = 100
pool_size = 1

[cli]
default_format = "json"
progress_bars = false
auto_confirm = true  # Don't prompt in CI
```

## Troubleshooting

### Configuration Not Found

**Problem**: `Config file not found` error

**Solutions**:
1. Specify config path: `memory-cli --config /path/to/config.toml`
2. Set environment variable: `export MEMORY_CLI_CONFIG=/path/to/config.toml`
3. Place config in default location: `~/.config/memory-cli/config.toml`
4. Run wizard: `memory-cli config wizard`

### Database Connection Failed

**Problem**: `Failed to connect to database` error

**For Turso**:
1. Verify `turso_url` is correct: `libsql://your-db.turso.io`
2. Check `turso_token` is valid and not expired
3. Ensure network connectivity to Turso
4. Verify database exists: `turso db list`

**For redb**:
1. Check `redb_path` directory exists and is writable
2. Ensure no other process is using the database
3. Verify disk space is available

### Validation Errors

**Problem**: `Configuration validation failed` error

**Solutions**:
1. Check error message for specific field
2. Run wizard for guided setup: `memory-cli config wizard`
3. Verify all required fields are present
4. Check value ranges (e.g., pool_size > 0)

### Performance Issues

**Problem**: Slow CLI operations

**Solutions**:
1. Increase `pool_size` (default: CPU count * 2)
2. Increase `max_episodes_cache` (default: 1000)
3. Enable compression: `enable_compression = true`
4. Use local database for development (faster than cloud)

## Best Practices

1. **Use Simple Mode** for quick setup and standard configurations
2. **Use Config File** for reproducible deployments and teams
3. **Use Environment Variables** for secrets (Turso tokens, etc.)
4. **Use Wizard** when unsure about settings or for first-time setup
5. **Validate After Changes**: Run `memory-cli config validate` to check
6. **Version Control**: Commit config files (except secrets)
7. **Performance Tuning**: Start with defaults, adjust based on metrics

## Advanced Topics

### Configuration Inheritance

Memory CLI supports layered configuration:

1. Default settings (lowest priority)
2. System config (`/etc/memory-cli/config.toml`)
3. User config (`~/.config/memory-cli/config.toml`)
4. Project config (`./config.toml`)
5. Environment variables
6. Command-line arguments (highest priority)

Later sources override earlier ones.

### Dynamic Configuration

For advanced use cases, configuration can be loaded programmatically:

```rust
use memory_cli::config::{Config, ConfigLoader};

// Load from custom path
let config = ConfigLoader::load_from_path("/custom/config.toml").await?;

// Merge multiple configs
let base = ConfigLoader::load_defaults().await?;
let overrides = ConfigLoader::load_from_env().await?;
let config = base.merge(overrides);

// Validate before use
config.validate()?;
```

### Configuration Presets

Available presets for quick setup:

- **development**: Local redb, standard performance
- **production**: Turso cloud, high performance
- **testing**: In-memory, minimal settings
- **edge**: Turso edge, optimized for latency

Use in wizard or CLI:

```bash
memory-cli config init --preset production
```

## Migration Guide

### From v0.1.6 to v0.1.7

**Breaking Changes**: None

**Deprecated**:
- Old environment variable names (use `MEMORY_*` prefix now)
- Complex manual setup (use Simple Mode instead)

**New Features**:
- Simple Mode API
- Configuration wizard
- Multi-format support (TOML, JSON, YAML)
- Configuration caching

**Migration**:
1. Existing config files work without changes
2. Update environment variables to new names (old ones still work)
3. Consider using Simple Mode for new setups

## Support

For help with configuration:

1. **Documentation**: This file and `memory-cli --help`
2. **Wizard**: Run `memory-cli config wizard` for guided setup
3. **Validation**: Run `memory-cli config validate` to check settings
4. **Examples**: See `examples/` directory for sample configs
5. **Issues**: Report bugs at https://github.com/your-repo/issues
