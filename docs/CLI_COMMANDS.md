# CLI Commands - Config Wizard

The configuration wizard command provides an interactive, guided experience for setting up memory-cli configuration files with sensible defaults and validation.

## Command Reference

### `memory config wizard`

Launches the interactive configuration wizard.

```bash
memory config wizard
```

**Description:**
Guides you through creating a configuration file step-by-step with:
- Configuration presets for common use cases
- Input validation and helpful error messages
- Configuration preview before saving
- Multiple save location options

**Examples:**

```bash
# Quick start with wizard
memory config wizard

# Select preset and accept all defaults
# (1) Choose "Local Development" preset
# (2) Press Enter for all defaults
# (3) Confirm and save

# Full interactive setup
memory config wizard
# Follow prompts to customize each setting
```

**Preset Options:**

1. **Local Development** (Recommended)
   - Local SQLite database at `./data/memory.db`
   - Local redb cache at `./data/cache.redb`
   - 1,000 episode cache (~50MB)
   - 30-minute cache TTL
   - 5 database connections
   - Human-readable output
   - Progress bars enabled

2. **Cloud Setup**
   - Remote Turso database
   - Local redb cache for performance
   - 5,000 episode cache (~250MB)
   - 2-hour cache TTL
   - 10 database connections
   - Optimized for production

3. **Memory Only**
   - In-memory database only
   - No persistence
   - 100 episode cache (~10MB)
   - 5-minute cache TTL
   - 2 database connections
   - Progress bars disabled (clean logs)
   - Perfect for CI/CD

4. **Custom Configuration**
   - Full control over all settings
   - Sensible defaults to start with
   - Configure each option manually

**Wizard Steps:**

1. **Preset Selection** - Choose a starting preset
2. **Database Configuration** - Configure storage backends
3. **Storage Configuration** - Set cache size, TTL, and pool size
4. **CLI Configuration** - Choose output format and display options
5. **Review & Validate** - Preview config, validate, and save

**Save Locations:**

- `memory-cli.toml` - Current directory (recommended)
- `.memory-cli.toml` - Hidden file in current directory
- `data/memory-cli.toml` - Data subdirectory
- Custom path - Specify any location

## Related Commands

```bash
# Validate existing configuration
memory config validate

# Check configuration for issues and recommendations
memory config check

# Show current configuration
memory config show

# Show configuration template
memory config show-template  # (if implemented)
```

## Configuration File Format

The wizard generates TOML configuration files:

```toml
[database]
turso_url = "file:./data/memory.db"
turso_token = null
redb_path = "./data/cache.redb"

[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 1800
pool_size = 5

[cli]
default_format = "human"
progress_bars = true
batch_size = 100
```

## Environment Variables

You can specify a custom configuration file location:

```bash
# Use specific config file
memory --config my-config.toml config wizard

# Or set environment variable
export MEMORY_CONFIG=/path/to/config.toml
memory config wizard
```

## Aliases

```bash
# Short form using alias
memory cfg wizard
```

## Examples

### Example 1: First-Time Setup

```bash
$ memory config wizard
🚀 Memory CLI Configuration Wizard
===================================
This wizard will guide you through setting up memory-cli with optimal defaults.
You can customize each setting or press Enter to accept recommended values.

📋 Step 1 of 5: Configuration Preset
────────────────────────────────────
Choose a configuration preset to get started quickly.
💡 Tip: Each preset provides optimized defaults for different use cases.

Select configuration preset:
> ⭐ Local Development (Recommended) - SQLite + redb cache
  ☁️  Cloud Setup - Remote Turso DB + local cache
  🧪 Memory Only - Testing/CI, no persistence
  ⚙️  Custom Configuration - Full control

✓ Selected: Local Development
  • Uses local SQLite database (file:./data/memory.db)
  • Local redb cache for fast access
  • Moderate cache size (1000 episodes)
  • Perfect for development and testing

💾 Step 2 of 5: Database Configuration
───────────────────────────────────────
Configure where your memory data will be stored.
💡 Tip: You can use local storage, cloud storage, or both for redundancy.

📡 Turso Database Setup
   Example formats:
   • libsql://your-database.turso.io/db  (Remote Turso)
   • file:./data/memory.db               (Local SQLite)

  Turso database URL [file:./data/memory.db]: 

✓ Using local SQLite file - no authentication needed

💾 Local Cache Configuration
   The local cache provides fast access to recent episodes.
   Example paths:
   • ./data/cache.redb  (Recommended: Local file)
   • :memory:           (In-memory only, no persistence)

  Local cache database path [./data/cache.redb]: 

✓ Database configuration complete

⚙️  Step 3 of 5: Storage Configuration
──────────────────────────────────────
Configure how much data to cache and connection settings.
💡 Tip: Larger cache = better performance, but uses more memory.

📊 Cache Size Configuration
   Recommended values:
   • Testing/CI:    100-200 episodes   (~10MB memory)
   • Development:   500-1000 episodes  (~50MB memory)
   • Production:    1000-5000 episodes (~100-500MB memory)

  Maximum episodes to cache (recommended: 1000) [1000]: 

  Cache time-to-live in seconds (recommended: 1800) [1800]: 

🔌 Connection Pool Size
   Number of simultaneous database connections:
   • Small (2-5):   Low concurrency, minimal resources
   • Medium (5-10): Balanced (recommended for most uses)
   • Large (10-20): High concurrency, more resources

  Database connection pool size (recommended: 5) [5]: 

✓ Storage configuration complete

🎨 Step 4 of 5: CLI Configuration
──────────────────────────────────
Configure how the CLI displays information and handles operations.
💡 Tip: These settings affect the user interface, not functionality.

🎨 Output Format
   Choose how command results are displayed:
   • human - Easy to read, colored output (recommended for interactive use)
   • json  - Machine-readable, great for scripts and automation
   • yaml  - Structured and readable, good for configs and logs

  Default output format:
> human (Recommended)
  json
  yaml

📊 Progress Bars
   Show progress bars for long-running operations?
   • Yes: Visual feedback (recommended for interactive use)
   • No:  Clean output (recommended for CI/scripts)

  Enable progress bars [Y/n]: 

📦 Batch Size
   Number of items to process in a single batch operation:
   • Small (10-50):    Safe, less memory, slower
   • Medium (50-200):  Balanced (recommended)
   • Large (200-1000): Fast, more memory

  Default batch size (recommended: 100) [100]: 

✓ CLI configuration complete

✅ Step 5 of 5: Review & Validate
──────────────────────────────────
Review your configuration before saving.

📋 Configuration Summary
════════════════════════

💾 Database Configuration:
  📁 Local SQLite URL: file:./data/memory.db
  💾 File-based cache Path: ./data/cache.redb

⚙️  Storage Configuration:
  📊 Cache Size:     1000 episodes (~100MB memory)
  ⏰ Cache TTL:      1800 seconds (30min)
  🔌 Pool Size:      5 connections

🎨 CLI Configuration:
  👤 Output Format:  human
  📊 Progress Bars:  ✓ Enabled
  📦 Batch Size:     100 items

✅ Configuration Validation Passed!
═══════════════════════════════════

💾 Save Configuration
Choose where to save your configuration file:

Save location:
> ⭐ memory-cli.toml (Current directory - Recommended)
  🔒 .memory-cli.toml (Hidden file in current directory)
  📁 data/memory-cli.toml (Data directory)
  ⚙️  Custom path (Specify your own location)

✅ Configuration successfully saved to: memory-cli.toml

💡 Next steps:
   • Test your configuration: memory --config memory-cli.toml
   • Edit manually if needed: memory-cli.toml
   • Run the wizard again to update: memory config wizard
```

### Example 2: Production Setup

```bash
$ memory config wizard
# ...
📋 Step 1 of 5: Configuration Preset
Select configuration preset:
  ⭐ Local Development (Recommended) - SQLite + redb cache
> ☁️  Cloud Setup - Remote Turso DB + local cache
  🧪 Memory Only - Testing/CI, no persistence
  ⚙️  Custom Configuration - Full control

✓ Selected: Cloud Setup
  • Uses remote Turso database
  • Local redb cache for performance
  • Large cache size (up to 5000 episodes)
  • Optimized for production workloads

💾 Step 2 of 5: Database Configuration
📡 Turso Database Setup
  Turso database URL [libsql://your-db.turso.io/db]: libsql://prod-123.turso.io/db

🔑 Authentication Token
   Get your token from: https://turso.tech/

  Turso authentication token (or press Enter to skip): eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

💾 Local Cache Configuration
  Local cache database path [./data/cache.redb]: /var/lib/cache/memory.redb

# ... rest of wizard with production-appropriate defaults
```

### Example 3: CI/CD Setup

```bash
$ memory config wizard
# ...
📋 Step 1 of 5: Configuration Preset
Select configuration preset:
  ⭐ Local Development (Recommended) - SQLite + redb cache
  ☁️  Cloud Setup - Remote Turso DB + local cache
> 🧪 Memory Only - Testing/CI, no persistence
  ⚙️  Custom Configuration - Full control

✓ Selected: Memory Only
  • In-memory storage only
  • No persistent data (restarts clear all data)
  • Minimal cache (100 episodes)
  • Ideal for CI/CD and quick tests

💾 Step 2 of 5: Database Configuration
...
  Local cache database path [:memory:]: 

⚙️  Step 3 of 5: Storage Configuration
  Maximum episodes to cache (recommended: 100) [100]: 
  Cache time-to-live in seconds (recommended: 300) [300]: 

🎨 Step 4 of 5: CLI Configuration
🎨 Output Format
  Default output format:
  human (Recommended)
> json
  yaml

📊 Progress Bars
  Enable progress bars [Y/n]: n

# ... Save to ci-config.toml
```

### Example 4: Updating Configuration

```bash
$ memory config wizard
🚀 Memory CLI Configuration Wizard (Update Mode)
═════════════════════════════════════════════════
Updating existing configuration with new values.
Press Enter to keep current values, or type new ones.

💾 Step 1 of 4: Database Configuration
Configure Turso remote database? [Y/n]: 
  Turso database URL [libsql://my-db.turso.io/db]: libsql://my-new-db.turso.io/db
  Turso authentication token [**********]: new-token-here

⚙️  Step 2 of 4: Storage Configuration
Maximum episodes to cache [1000]: 2500
Cache time-to-live (seconds) [1800]: 3600
Database connection pool size [5]: 10

🎨 Step 3 of 4: CLI Configuration
Default output format:
> human
  json
  yaml
  
Enable progress bars [Y/n]: 

# ... Review and confirm changes
```

## Common Workflows

### Quick Setup for Development

```bash
# 1. Run wizard with default preset
memory config wizard

# 2. Accept all defaults by pressing Enter
# (1) Local Development preset ✓
# (2) Default database paths ✓
# (3) Default storage settings ✓
# (4) Default CLI settings ✓
# (5) Save to memory-cli.toml ✓

# 3. Validate configuration
memory config validate

# 4. Test
memory episode list
```

### Setup Multiple Environments

```bash
# Development config
memory config wizard
# Save as dev.toml

# Staging config (use existing as template)
cp dev.toml staging.toml
# Edit staging.toml to use stage database
memory --config staging.toml config wizard
# Update settings and save

# Production config
memory config wizard
# Select Cloud preset and enter production Turso details
# Save as prod.toml

# Use environment-specific configs
memory --config dev.toml     episode list
memory --config prod.toml    episode list
memory --config staging.toml episode list
```

### Validate Configuration

```bash
# After running wizard, always validate
memory config validate

# Output:
✅ Configuration is valid

Connectivity Status:
  ✅ Turso: Connected
  ✅ redb: Accessible
  Latency: 5ms

Issues found: (none)
```

## Troubleshooting

### Wizard Fails to Start

```bash
# Check if config already exists
ls -la memory-cli.toml .memory-cli.toml

# Run wizard with verbose output
memory --verbose config wizard
```

### Validation Errors

```bash
# After wizard completes, if validation fails:
memory config check

# Output shows:
# ⚠️ Configuration Warnings
# 
# These won't prevent usage, but you may want to address them:
# 
# 1. ⚠️  cache_ttl_seconds: Cache TTL is quite short
#    💡 Suggestion: Consider increasing to at least 1800s for better performance
```

### Connection Issues

```bash
# If remote database fails to connect:
memory config check

# Shows connectivity errors, then update config:
memory config wizard
# Reconfigure database with correct URL/token
```

## See Also

<!-- NOTE: CONFIG_WIZARD.md is planned documentation -->
- [Configuration Guide](../memory-cli/CONFIGURATION_GUIDE.md) - Complete configuration reference
- [Config Commands (CLI Reference)](#config-commands) - All config subcommands
- [Local Database Setup](./LOCAL_DATABASE_SETUP.md) - Database implementation details
