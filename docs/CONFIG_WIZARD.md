# Configuration Wizard Guide

The memory-cli configuration wizard is an interactive tool that guides you through setting up your memory system with optimal defaults and sensible validation.

## Overview

The configuration wizard simplifies the setup process by:
- Providing pre-configured presets for common use cases
- Guiding you through each configuration option
- Validating your settings for correctness
- Saving configurations to files

## Quick Start

### Running the Wizard

```bash
memory config wizard
```

This launches an interactive wizard that will:
1. Present configuration presets
2. Collect your preferences
3. Validate your choices
4. Save the configuration file

### Alternative Commands

```bash
# With specific config file
memory --config my-config.toml config wizard

# Show template without interaction
memory config show-template  # (if implemented)
```

## Configuration Presets

The wizard offers four pre-configured presets:

### 1. Local Development (Recommended)

Best for: Development, testing, and personal projects

**Configuration:**
- Database: Local SQLite file (`./data/memory.db`)
- Cache: Local redb file (`./data/cache.redb`)
- Cache Size: 1,000 episodes (~50MB)
- Cache TTL: 30 minutes (1,800 seconds)
- Pool Size: 5 connections
- Output Format: Human-readable
- Progress Bars: Enabled
- Batch Size: 100

**When to use:**
- Local development
- Feature testing
- Personal projects
- Learning and experimentation

### 2. Cloud Setup

Best for: Production workloads with remote storage

**Configuration:**
- Database: Remote Turso database
- Cache: Local redb file for fast access
- Cache Size: 5,000 episodes (~250MB)
- Cache TTL: 2 hours (7,200 seconds)
- Pool Size: 10 connections
- Output Format: Human-readable
- Progress Bars: Enabled
- Batch Size: 100

**When to use:**
- Production applications
- Multi-team environments
- Scenarios requiring remote data access
- High-availability setups

### 3. Memory Only

Best for: CI/CD, automated testing, temporary workspaces

**Configuration:**
- Database: In-memory only
- Cache: In-memory (`:memory:`)
- Cache Size: 100 episodes (~10MB)
- Cache TTL: 5 minutes (300 seconds)
- Pool Size: 2 connections
- Output Format: Human-readable
- Progress Bars: Disabled (for clean output)
- Batch Size: 100

**When to use:**
- CI/CD pipelines
- Automated testing
- Temporary containers
- Quick experiments without persistence

### 4. Custom Configuration

Best for: Advanced users with specific requirements

**Configuration:**
- All options configurable
- Starts with sensible defaults
- Full control over every setting

**When to use:**
- Specific performance requirements
- Custom deployment environments
- Integration with existing systems
- Non-standard storage backends

## Wizard Workflow

### Step 1: Configuration Preset Selection

```
📋 Step 1 of 5: Configuration Preset
────────────────────────────────────
Choose a configuration preset to get started quickly.
💡 Tip: Each preset provides optimized defaults for different use cases.

Select configuration preset:
> ⭐ Local Development (Recommended) - SQLite + redb cache
  ☁️  Cloud Setup - Remote Turso DB + local cache
  🧪 Memory Only - Testing/CI, no persistence
  ⚙️  Custom Configuration - Full control
```

### Step 2: Database Configuration

Configure where your memory data will be stored.

```
💾 Step 2 of 5: Database Configuration
───────────────────────────────────────
Configure where your memory data will be stored.
💡 Tip: You can use local storage, cloud storage, or both for redundancy.

📡 Turso Database Setup
   Example formats:
   • libsql://your-database.turso.io/db  (Remote Turso)
   • file:./data/memory.db               (Local SQLite)

  Turso database URL [file:./data/memory.db]: _
```

The wizard validates your input for:
- Required format (must start with `libsql://` or `file:`)
- Path traversal prevention (no `..` allowed in paths)
- Empty input prevention

For remote Turso databases:
```
🔑 Authentication Token
   Get your token from: https://turso.tech/

  Turso authentication token (or press Enter to skip) [**********]: _
```

Local cache configuration:
```
💾 Local Cache Configuration
   The local cache provides fast access to recent episodes.
   Example paths:
   • ./data/cache.redb  (Recommended: Local file)
   • :memory:           (In-memory only, no persistence)

  Local cache database path [./data/cache.redb]: _
```

### Step 3: Storage Configuration

Configure caching and connection settings.

```
⚙️  Step 3 of 5: Storage Configuration
──────────────────────────────────────
Configure how much data to cache and connection settings.
💡 Tip: Larger cache = better performance, but uses more memory.

📊 Cache Size Configuration
   Recommended values:
   • Testing/CI:    100-200 episodes   (~10MB memory)
   • Development:   500-1000 episodes  (~50MB memory)
   • Production:    1000-5000 episodes (~100-500MB memory)

  Maximum episodes to cache (recommended: 1000) [1000]: _
```

Input validation:
- Must be greater than 0
- Maximum: 100,000 episodes

```
⏰ Cache TTL (Time-To-Live)
   How long cached episodes remain valid before refresh:
   • Short (300s/5min):    Fresh data, more DB queries
   • Medium (1800s/30min): Balanced (recommended for dev)
   • Long (7200s/2hr):     Less queries (recommended for prod)

  Cache time-to-live in seconds (recommended: 1800) [1800]: _
```

Input validation:
- Must be greater than 0
- Maximum: 86,400 seconds (24 hours)

```
🔌 Connection Pool Size
   Number of simultaneous database connections:
   • Small (2-5):   Low concurrency, minimal resources
   • Medium (5-10): Balanced (recommended for most uses)
   • Large (10-20): High concurrency, more resources

  Database connection pool size (recommended: 5) [5]: _
```

Input validation:
- Must be greater than 0
- Maximum: 200 connections

### Step 4: CLI Configuration

Configure display and behavior settings.

```
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
```

```
📊 Progress Bars
   Show progress bars for long-running operations?
   • Yes: Visual feedback (recommended for interactive use)
   • No:  Clean output (recommended for CI/scripts)

  Enable progress bars [Y/n]: _
```

```
📦 Batch Size
   Number of items to process in a single batch operation:
   • Small (10-50):    Safe, less memory, slower
   • Medium (50-200):  Balanced (recommended)
   • Large (200-1000): Fast, more memory

  Default batch size (recommended: 100) [100]: _
```

Input validation:
- Must be greater than 0
- Maximum: 10,000 items

### Step 5: Review and Validate

Review your configuration before saving.

```
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
```

If validation fails:
```
❌ Configuration Validation Failed
══════════════════════════════════

The following errors must be fixed:

1. ❌ storage.max_episodes_cache: Cache size must be greater than 0
   💡 How to fix: Set max_episodes_cache to a positive value
   ℹ️  Context: Cache size controls how many episodes are kept in memory

⚠️  Configuration has errors. Do you want to continue anyway? (Not recommended)
```

### Saving the Configuration

```
💾 Save Configuration
Choose where to save your configuration file:

Save location:
> ⭐ memory-cli.toml (Current directory - Recommended)
  🔒 .memory-cli.toml (Hidden file in current directory)
  📁 data/memory-cli.toml (Data directory)
  ⚙️  Custom path (Specify your own location)
```

After saving:
```
✅ Configuration successfully saved to: memory-cli.toml

💡 Next steps:
   • Test your configuration: memory --config memory-cli.toml
   • Edit manually if needed: memory-cli.toml
   • Run the wizard again to update: memory config wizard
```

## Using Generated Configuration

### Load Configuration Automatically

Place `memory-cli.toml` (or `.memory-cli.toml`) in your project directory.
The CLI will automatically load it:

```bash
memory episode list
memory pattern analyze --pattern-id <uuid>
```

### Specify Configuration File Explicitly

```bash
memory --config /path/to/my-config.toml episode list
```

### Use Environment Variable

```bash
export MEMORY_CONFIG=/path/to/my-config.toml
memory episode list
```

## Validating Configuration

After running the wizard, validate your configuration:

```bash
memory config validate
```

This checks:
- Configuration syntax
- Required fields
- Value ranges
- Connectivity (if configured)

Check configuration status:

```bash
memory config check
```

Shows:
- Current configuration
- Recommendations
- Security issues

Display current config:

```bash
memory config show
```

Shows:
- Database settings (sensitive data masked)
- Storage settings
- CLI settings
- Enabled features

## Common Scenarios

### Scenario 1: First-Time Setup

```bash
# Run wizard
memory config wizard

# Follow prompts:
# 1. Select "Local Development" preset
# 2. Keep defaults for database (file:./data/memory.db)
# 3. Keep defaults for storage (1000 episodes, 30min TTL)
# 4. Keep defaults for CLI (human format, progress bars)
# 5. Save to memory-cli.toml

# Validate
memory config validate

# Test
memory episode list
```

### Scenario 2: Production Setup

```bash
# Run wizard
memory config wizard

# Follow prompts:
# 1. Select "Cloud Setup" preset
# 2. Enter Turso URL: libsql://production.turso.io/db
# 3. Enter Turso token (get from https://turso.tech/)
# 4. Confirm storage defaults (5000 episodes, 2hr TTL)
# 5. Keep CLI defaults
# 6. Save to .memory-cli.toml (hidden file)

# Validate
memory config validate

# Check connectivity
memory config check
```

### Scenario 3: CI/CD Pipeline

```bash
# Run wizard
memory config wizard

# Follow prompts:
# 1. Select "Memory Only" preset
# 2. Confirm :memory: database
# 3. Confirm minimal cache (100 episodes, 5min TTL)
# 4. Disable progress bars for clean logs
# 5. Change output format to json
# 6. Save to ci-config.toml

# Test in CI
memory --config ci-config.toml episode list --format json
```

### Scenario 4: Updating Configuration

```bash
# Run wizard again
memory config wizard

# Choose "Custom Configuration" preset
# The wizard will use current config as defaults:
# - Press Enter to keep current values
# - Type new values to override
# The wizard asks for confirmation before saving
```

### Scenario 5: Multiple Environments

```bash
# Create dev config
memory config wizard
# Save as dev.toml

# Create production config
memory --config prod.toml config wizard
# Save as prod.toml

# Switch between environments
memory --config dev.toml episode list
memory --config prod.toml episode list
```

## Configuration File Reference

### TOML Format

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

# Optional - embeddings (currently defaults)
[embeddings]
enabled = false
# ... embeddings config (if enabled)
```

### JSON Format (Alternative)

```json
{
  "database": {
    "turso_url": "file:./data/memory.db",
    "turso_token": null,
    "redb_path": "./data/cache.redb"
  },
  "storage": {
    "max_episodes_cache": 1000,
    "cache_ttl_seconds": 1800,
    "pool_size": 5
  },
  "cli": {
    "default_format": "human",
    "progress_bars": true,
    "batch_size": 100
  }
}
```

## Troubleshooting

### Configuration Not Found

```
Error: Configuration file not found
```

**Solutions:**
1. Create config with wizard: `memory config wizard`
2. Specify config file explicitly: `memory --config path/to/config.toml <command>`
3. Create default config file: `memory-cli.toml` in current directory

### Validation Errors

```
❌ Configuration Validation Failed
The following errors must be fixed:
1. ❌ storage.max_episodes_cache: Cache size must be greater than 0
```

**Solutions:**
1. Run wizard again: `memory config wizard`
2. Edit config file manually and fix the error
3. Check for typos or invalid values

### Connection Failures

```
Error: Turso connection failed
```

**Solutions:**
1. Verify Turso URL format (must start with `libsql://`)
2. Check Turso token is correct
3. Test connectivity: `memory config check`
4. Verify network access to Turso

### Permission Denied

```
Error: Failed to write configuration to 'memory-cli.toml'
```

**Solutions:**
1. Check write permissions on directory
2. Use different save location: `data/memory-cli.toml`
3. Run with appropriate permissions

## Best Practices

1. **Start with Defaults**: The wizard provides sensible defaults. Only change what you need.

2. **Version Control Config**: Add configuration to version control (except secrets like tokens).

3. **Environment-Specific Configs**: Use separate config files for dev, staging, production.

4. **Document Changes**: Add comments to config files explaining non-standard choices.

5. **Validate Regularly**: Run `memory config validate` after any changes.

6. **Use Presets When Possible**: Presets are optimized and tested. Use them as starting points.

7. **Monitor Performance**: Adjust cache size and TTL based on actual usage patterns.

8. **Security**: Never commit tokens to version control. Use `.gitignore` for sensitive configs.

## Advanced Usage

### Programmatic Configuration

```rust
use memory_cli::config::{Config, ConfigPreset};

let config = ConfigPreset::Local.create_config();
// Customize as needed
let builder = memory_cli::config::create_writer(config);
builder.write_to_file(std::path::Path::new("custom-config.toml"))?;
```

### Configuration Merging

```bash
# Start with a preset, then override specific values
# Using environment variables or multiple config files
memory --config base.toml --config overrides.toml <command>
```

### Dynamic Configuration

```bash
# Use config generation in CI/CD
cat <<EOF > ci-config.toml
[database]
redb_path = ":memory:"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 300

[cli]
default_format = "json"
progress_bars = false
EOF

memory --config ci-config.toml episode list
```

## Related Commands

- `memory config validate` - Validate current configuration
- `memory config check` - Check configuration for issues
- `memory config show` - Display current configuration
- `memory episode list` - Test configuration with episode listing
- `memory config wizard` - Run configuration wizard (this command)

## Examples

### Example 1: Minimal Configuration

```bash
$ memory config wizard
# Select: Memory Only preset
# Keep all defaults
# Save to .memory-cli.toml

$ memory --config .memory-cli.toml episode list
# Works with in-memory storage
```

### Example 2: Custom Storage Path

```bash
$ memory config wizard
# Select: Local Development preset
# Database: file:/var/lib/memory/memory.db
# Cache: /var/lib/memory/cache.redb
# Rest defaults
# Save to /etc/memory-cli/config.toml

$ memory --config /etc/memory-cli/config.toml episode list
# Uses custom storage paths
```

### Example 3: Automated Setup Script

```bash
#!/bin/bash
cat > setup-config.sh << 'SCRIPT'
echo "Setting up memory-cli configuration..."

cat > memory-cli.toml << 'CONFIG'
[database]
redb_path = "./data/cache.redb"

[storage]
max_episodes_cache = 2000
cache_ttl_seconds = 3600
pool_size = 8

[cli]
default_format = "json"
progress_bars = false
batch_size = 200
CONFIG

echo "Configuration saved to memory-cli.toml"
memory config validate
SCRIPT

chmod +x setup-config.sh
./setup-config.sh
```

## Support and Feedback

- **Issues**: Report bugs or issues on GitHub
- **Documentation**: Check main README for additional info
- **Community**: Join discussions for tips and best practices

---

**Note**: The wizard provides interactive guidance. For scripted or automated setups, consider creating config files manually or using templates.
