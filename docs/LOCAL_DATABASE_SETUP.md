# Local Database Setup Guide

This guide explains how to configure and use a local SQLite database with the Memory CLI system when Turso cloud database is not available or desired.

## Overview

The Memory CLI system supports multiple storage backends:
- **Turso Cloud** (default) - Remote libSQL database
- **Local SQLite** - Local file-based database (fallback option)
- **In-memory** - Temporary storage for testing

When no Turso configuration is provided, the system automatically falls back to local SQLite storage.

## Quick Start

### 1. Automatic Setup

Run the provided setup script to configure everything automatically:

```bash
# From the project root
./scripts/setup-local-db.sh
```

This script will:
- Create necessary directories (`./data`, `./backups`)
- Initialize SQLite database with proper schema
- Create configuration file
- Test database connectivity

### 2. Manual Setup

If you prefer to set up manually:

#### Step 1: Environment Configuration

Create a `.env` file in the project root:

```bash
# Copy the template
cp .env.example .env
```

Edit `.env` with your local database preferences:

```env
# Leave Turso variables empty for local-only setup
TURSO_DATABASE_URL=
TURSO_AUTH_TOKEN=

# Local database configuration
LOCAL_DATABASE_URL=sqlite:./data/memory.db
MEMORY_REDB_PATH=./data/memory.redb

# Data directories
MEMORY_DATA_DIR=./data
MEMORY_BACKUP_DIR=./backups

# Other settings...
RUST_LOG=info
MEMORY_MAX_EPISODES_CACHE=1000
```

#### Step 2: Create Directories

```bash
mkdir -p ./data ./backups
```

#### Step 3: Initialize Database

```bash
# The CLI will auto-create the database on first run
# Or manually initialize with sqlite3
sqlite3 ./data/memory.db < scripts/schema.sql
```

## Configuration Options

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LOCAL_DATABASE_URL` | Path to local SQLite database | `sqlite:./data/memory.db` |
| `MEMORY_REDB_PATH` | Path to redb cache file | `./data/memory.redb` |
| `MEMORY_DATA_DIR` | Data directory root | `./data` |
| `MEMORY_BACKUP_DIR` | Backup directory | `./backups` |
| `MEMORY_MAX_EPISODES_CACHE` | Max episodes in memory cache | `1000` |
| `MEMORY_CACHE_TTL_SECONDS` | Cache TTL in seconds | `3600` |

### Configuration File

You can also use a `memory-cli.toml` file:

```toml
[database]
# Local SQLite database
turso_url = "file:./data/memory.db"
turso_token = ""

# Local redb cache
redb_path = "./data/memory.redb"

[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "human"
progress_bars = true
batch_size = 100

[backup]
backup_dir = "./backups"
max_backup_age_days = 30
compress_backups = true

[logging]
level = "info"
max_log_size_mb = 10
max_log_files = 5
```

## Database Schema

The local SQLite database uses the same schema as Turso:

### Tables

- **`episodes`** - Stores task execution records
- **`execution_steps`** - Individual steps within episodes
- **`patterns`** - Extracted patterns from episodes
- **`heuristics`** - Learned condition-action rules
- **`episode_patterns`** - Relationships between episodes and patterns

### Schema Initialization

The database schema is automatically initialized on first connection. The schema includes:

```sql
-- Episodes table
CREATE TABLE episodes (
    id TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    context TEXT,
    task_type TEXT,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    outcome TEXT,
    verdict TEXT,
    artifacts TEXT,
    reward_score REAL,
    reflection TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Additional tables for steps, patterns, heuristics...
-- (Full schema in scripts/setup-local-db.sh)
```

## Usage Examples

### Basic CLI Usage

```bash
# Initialize and test the setup
cargo run --bin memory-cli -- config show

# Store an episode
cargo run --bin memory-cli -- episode store \
  --description "Implement user authentication" \
  --context '{"language": "rust", "domain": "auth"}' \
  --outcome "success" \
  --verdict "Auth system implemented with JWT tokens"

# Retrieve relevant context
cargo run --bin memory-cli -- context retrieve \
  --query "add user authorization" \
  --limit 5

# List stored episodes
cargo run --bin memory-cli -- episode list --limit 10

# Show patterns
cargo run --bin memory-cli -- pattern list
```

### Programmatic Usage

```rust
use memory_core::{SelfLearningMemory, TaskContext, TaskType, TaskOutcome};
use memory_cli::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration (will auto-detect local SQLite)
    let config = Config::load(None)?;
    let memory = config.create_memory().await?;

    // Use the memory system
    let episode_id = memory.start_episode(
        "Implement feature X".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    ).await;

    // ... log steps ...

    memory.complete_episode(episode_id, TaskOutcome::Success {
        verdict: "Feature implemented successfully".to_string(),
        artifacts: vec!["feature_x.rs".to_string()],
    }).await?;

    Ok(())
}
```

## Migration from Turso

If you want to migrate from Turso to local SQLite:

### 1. Export from Turso

```bash
# Using turso CLI (if available)
turso db shell your-db-name ".dump" > turso_backup.sql
```

### 2. Import to Local SQLite

```bash
# Create local database
sqlite3 ./data/memory.db < turso_backup.sql
```

### 3. Update Configuration

Update your `.env` or `memory-cli.toml` to use local paths instead of Turso URLs.

## Performance Considerations

### Local SQLite Performance

- **Read Performance**: Excellent for local access
- **Write Performance**: Good, but consider batch operations
- **Concurrency**: Limited by SQLite's file-based locking
- **Scalability**: Suitable for development and moderate workloads

### Optimization Tips

1. **Batch Operations**: Use batch size > 100 for bulk operations
2. **Cache Settings**: Increase `MEMORY_MAX_EPISODES_CACHE` for better performance
3. **File Location**: Place database on fast storage (SSD)
4. **Regular Maintenance**: Periodically run `VACUUM` and `ANALYZE`

```bash
# Database maintenance
sqlite3 ./data/memory.db "VACUUM; ANALYZE;"
```

## Backup and Recovery

### Automated Backups

The system supports automatic backups to the configured backup directory:

```bash
# Manual backup
cargo run --bin memory-cli -- backup create --name "manual-backup-$(date +%Y%m%d)"

# List backups
cargo run --bin memory-cli -- backup list

# Restore from backup
cargo run --bin memory-cli -- backup restore --name "manual-backup-20231201"
```

### Manual Backup

```bash
# Simple file copy
cp ./data/memory.db ./backups/memory-$(date +%Y%m%d-%H%M%S).db

# Using SQLite backup command
sqlite3 ./data/memory.db ".backup ./backups/memory-$(date +%Y%m%d).db"
```

## Troubleshooting

### Common Issues

#### 1. Database Lock Errors

**Problem**: "database is locked" errors

**Solution**:
- Ensure only one process is accessing the database
- Check for long-running transactions
- Consider using WAL mode:

```bash
sqlite3 ./data/memory.db "PRAGMA journal_mode=WAL;"
```

#### 2. Permission Errors

**Problem**: Cannot create database files

**Solution**:
```bash
# Check directory permissions
ls -la ./data/

# Fix permissions
chmod 755 ./data
chmod 644 ./data/memory.db 2>/dev/null || true
```

#### 3. Schema Mismatch

**Problem**: "no such table" errors

**Solution**:
```bash
# Re-initialize schema
./scripts/setup-local-db.sh --clean

# Or manually
rm ./data/memory.db
cargo run --bin memory-cli -- config show  # This will recreate the schema
```

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug cargo run --bin memory-cli -- <command>
```

### Database Inspection

Use SQLite CLI to inspect the database:

```bash
# Connect to database
sqlite3 ./data/memory.db

# List tables
.tables

# Show schema
.schema

# Query episodes
SELECT id, description, created_at FROM episodes LIMIT 5;

# Check database stats
.databases
.schema
```

## Security Considerations

### File Permissions

Local SQLite files should have appropriate permissions:

```bash
# Secure database file (read/write for owner only)
chmod 600 ./data/memory.db
chmod 700 ./data/
```

### Backup Security

- Encrypt backup files if they contain sensitive data
- Store backups in secure locations
- Consider using encrypted SQLite extensions for sensitive data

### Environment Variables

Don't commit sensitive configuration to version control:

```bash
# Add to .gitignore
.env
data/
backups/
*.db
*.redb
```

## Integration with CI/CD

### GitHub Actions

```yaml
name: Test with Local Database
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Setup local database
        run: |
          ./scripts/setup-local-db.sh
          
      - name: Run tests
        run: |
          cargo test --all
        env:
          LOCAL_DATABASE_URL: sqlite:./test-data/test.db
          MEMORY_REDB_PATH: ./test-data/cache.redb
```

## Advanced Configuration

### Custom SQLite Options

You can customize SQLite behavior through environment variables:

```env
# Enable WAL mode for better concurrency
SQLITE_PRAGMA_JOURNAL_MODE=WAL

# Set synchronous mode (NORMAL, FULL, OFF)
SQLITE_PRAGMA_SYNCHRONOUS=NORMAL

# Configure cache size (in pages)
SQLITE_PRAGMA_CACHE_SIZE=10000

# Enable foreign key constraints
SQLITE_PRAGMA_FOREIGN_KEYS=ON
```

### Multiple Databases

For advanced use cases, you can use separate databases:

```env
# Episodes database
EPISODES_DB_URL=sqlite:./data/episodes.db

# Patterns database  
PATTERNS_DB_URL=sqlite:./data/patterns.db

# Cache database
CACHE_DB_URL=sqlite:./data/cache.db
```

## Next Steps

1. **Run the setup script**: `./scripts/setup-local-db.sh`
2. **Test the configuration**: `cargo run --bin memory-cli -- config show`
3. **Try basic operations**: Store and retrieve some episodes
4. **Explore the API**: Check the programmatic usage examples
5. **Configure backups**: Set up automated backup schedule

For more information, see:
- [Memory CLI Documentation](../memory-cli/README.md)
- [Core API Documentation](../memory-core/README.md)
- [Configuration Reference](../memory-cli/CONFIGURATION_GUIDE.md)