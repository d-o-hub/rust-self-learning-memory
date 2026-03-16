#!/bin/bash

# Local Database Setup Script for Memory CLI
# This script sets up a local SQLite database when Turso cloud is not available

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DATA_DIR="${MEMORY_DATA_DIR:-$PROJECT_ROOT/data}"
BACKUP_DIR="${MEMORY_BACKUP_DIR:-$PROJECT_ROOT/backups}"
DB_FILE="$DATA_DIR/memory.db"
REDB_FILE="$DATA_DIR/memory.redb"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are available
check_dependencies() {
    log_info "Checking dependencies..."
    
    if ! command -v sqlite3 &> /dev/null; then
        log_warning "sqlite3 not found. Installing sqlite3..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y sqlite3
        elif command -v brew &> /dev/null; then
            brew install sqlite3
        elif command -v cargo &> /dev/null; then
            log_info "Installing sqlite3 via cargo..."
            cargo install sqlite3
        else
            log_error "Please install sqlite3 manually"
            exit 1
        fi
    fi
    
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo is required but not installed"
        exit 1
    fi
    
    log_success "Dependencies check passed"
}

# Create necessary directories
create_directories() {
    log_info "Creating data directories..."
    
    mkdir -p "$DATA_DIR"
    mkdir -p "$BACKUP_DIR"
    
    # Set proper permissions
    chmod 755 "$DATA_DIR"
    chmod 755 "$BACKUP_DIR"
    
    log_success "Directories created: $DATA_DIR, $BACKUP_DIR"
}

# Initialize SQLite database with schema
initialize_sqlite_db() {
    log_info "Initializing SQLite database at $DB_FILE..."
    
    # Create SQLite database with Turso-compatible schema
    sqlite3 "$DB_FILE" << 'EOF'
-- Memory System Schema for Local SQLite Database (Turso Compatible)

-- Episodes table
CREATE TABLE IF NOT EXISTS episodes (
    episode_id TEXT PRIMARY KEY NOT NULL,
    task_type TEXT NOT NULL,
    task_description TEXT NOT NULL,
    context TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    steps TEXT NOT NULL,
    outcome TEXT,
    reward TEXT,
    reflection TEXT,
    patterns TEXT NOT NULL,
    heuristics TEXT NOT NULL DEFAULT '[]',
    metadata TEXT NOT NULL,
    domain TEXT NOT NULL,
    language TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Patterns table
CREATE TABLE IF NOT EXISTS patterns (
    pattern_id TEXT PRIMARY KEY NOT NULL,
    pattern_type TEXT NOT NULL,
    pattern_data TEXT NOT NULL,
    success_rate REAL NOT NULL,
    context_domain TEXT,
    context_language TEXT,
    context_tags TEXT,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Heuristics table
CREATE TABLE IF NOT EXISTS heuristics (
    heuristic_id TEXT PRIMARY KEY NOT NULL,
    condition_text TEXT NOT NULL,
    action_text TEXT NOT NULL,
    confidence REAL NOT NULL,
    evidence TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Execution records table for monitoring
CREATE TABLE IF NOT EXISTS execution_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_name TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    duration_ms INTEGER NOT NULL,
    started_at INTEGER NOT NULL,
    task_description TEXT,
    error_message TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Agent metrics table for monitoring
CREATE TABLE IF NOT EXISTS agent_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_name TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    total_tasks INTEGER NOT NULL DEFAULT 0,
    successful_tasks INTEGER NOT NULL DEFAULT 0,
    failed_tasks INTEGER NOT NULL DEFAULT 0,
    average_duration_ms REAL NOT NULL DEFAULT 0.0,
    last_updated INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_episodes_task_type ON episodes(task_type);
CREATE INDEX IF NOT EXISTS idx_episodes_start_time ON episodes(start_time DESC);
CREATE INDEX IF NOT EXISTS idx_episodes_domain ON episodes(domain);
CREATE INDEX IF NOT EXISTS idx_patterns_context ON patterns(context_domain, context_language);
CREATE INDEX IF NOT EXISTS idx_heuristics_confidence ON heuristics(confidence DESC);

EOF

    if [ $? -eq 0 ]; then
        log_success "SQLite database initialized successfully"
    else
        log_error "Failed to initialize SQLite database"
        exit 1
    fi
}

# Initialize redb cache
initialize_redb_cache() {
    log_info "Initializing redb cache at $REDB_FILE..."
    
    # The redb cache will be initialized by the Rust application
    # We just need to ensure the directory exists and file can be created
    touch "$REDB_FILE" 2>/dev/null || true
    
    if [ -f "$REDB_FILE" ]; then
        log_success "redb cache file created"
    else
        log_warning "redb cache file will be created on first run"
    fi
}

# Create configuration file
create_config_file() {
    log_info "Creating configuration file..."
    
    local config_file="$PROJECT_ROOT/memory-cli.toml"
    
    if [ ! -f "$config_file" ]; then
        cat > "$config_file" << EOF
# Memory CLI Configuration - Local Development Setup

[database]
# Local SQLite database (fallback when Turso is not available)
turso_url = "file://$DB_FILE"
turso_token = ""

# Local redb cache
redb_path = "$REDB_FILE"

[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "human"
progress_bars = true
batch_size = 100

[monitoring]
enabled = true
health_check_interval_seconds = 30

[backup]
backup_dir = "$BACKUP_DIR"
max_backup_age_days = 30
compress_backups = true

[logging]
level = "info"
max_log_size_mb = 10
max_log_files = 5
EOF
        log_success "Configuration file created: $config_file"
    else
        log_warning "Configuration file already exists, skipping creation"
    fi
}

# Test database connection
test_database() {
    log_info "Testing database connection..."
    
    # Test SQLite
    if sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM episodes;" >/dev/null 2>&1; then
        log_success "SQLite database connection test passed"
    else
        log_error "SQLite database connection test failed"
        return 1
    fi
}

# Main setup function
main() {
    log_info "Starting local database setup for Memory CLI..."
    
    check_dependencies
    create_directories
    initialize_sqlite_db
    initialize_redb_cache
    create_config_file
    test_database
    
    log_success "Local database setup completed successfully!"
    echo
    log_info "Database files created:"
    echo "  - SQLite DB: $DB_FILE"
    echo "  - redb Cache: $REDB_FILE"
    echo "  - Config: $PROJECT_ROOT/memory-cli.toml"
    echo
    log_info "You can now use Memory CLI with local storage:"
    echo "  cd $PROJECT_ROOT && cargo run --bin memory-cli -- --help"
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Local Database Setup Script for Memory CLI"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --test-only    Only test existing database"
        echo "  --clean        Remove existing database files and recreate"
        echo
        exit 0
        ;;
    --test-only)
        test_database
        exit $?
        ;;
    --clean)
        log_warning "Removing existing database files..."
        rm -f "$DB_FILE" "$REDB_FILE"
        log_success "Database files removed"
        main
        ;;
    *)
        main
        ;;
esac