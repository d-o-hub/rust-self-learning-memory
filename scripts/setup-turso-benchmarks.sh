#!/bin/bash

# Turso Benchmark Environment Setup Script
# This script sets up a local Turso dev server or configures cloud connection

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
DB_FILE="${TURSO_DB_FILE:-/tmp/turso_benchmark.db}"
TURSO_URL="${TURSO_DATABASE_URL:-libsql://127.0.0.1:8080}"
TURSO_TOKEN="${TURSO_AUTH_TOKEN:-}"

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

# Check if Turso CLI is installed
check_turso_cli() {
    log_info "Checking for Turso CLI..."

    if command -v turso &> /dev/null; then
        local version=$(turso --version 2>/dev/null || echo "unknown")
        log_success "Turso CLI found: $version"
        return 0
    else
        log_error "Turso CLI not found"
        echo
        echo "Please install Turso CLI using one of these methods:"
        echo
        echo "1. Shell script (Linux/macOS):"
        echo "   curl -sSfL https://get.turso.dev | sh"
        echo
        echo "2. Homebrew (macOS):"
        echo "   brew install tursodatabase/tap/turso"
        echo
        echo "3. Go (if Go is installed):"
        echo "   go install github.com/tursodatabase/turso-cli/cmd/turso@latest"
        echo
        return 1
    fi
}

# Start local Turso dev server
start_local_server() {
    log_info "Starting local Turso dev server..."

    # Check if port 8080 is already in use
    if lsof -i :8080 >/dev/null 2>&1; then
        log_warning "Port 8080 is already in use"
        echo "Either:"
        echo "  1. Stop the existing service using port 8080"
        echo "  2. Use a different port: turso dev --db-file $DB_FILE --port 8081"
        echo "  3. Use Turso cloud database instead"
        echo
        read -p "Try different port? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            TURSO_URL="libsql://127.0.0.1:8081"
            turso dev --db-file "$DB_FILE" --port 8081 &
            TURSO_PID=$!
        else
            return 1
        fi
    else
        # Start Turso dev server in background
        turso dev --db-file "$DB_FILE" &
        TURSO_PID=$!
        log_success "Turso dev server started (PID: $TURSO_PID)"
    fi

    # Wait for server to be ready
    log_info "Waiting for server to be ready..."
    sleep 3

    # Test connection
    log_info "Testing connection to $TURSO_URL..."
    # Note: We can't test without sqlite3 or turso CLI
    log_success "Server should be running on $TURSO_URL"

    return 0
}

# Configure Turso cloud connection
configure_cloud_connection() {
    log_info "Configuring Turso cloud connection..."

    if [ -z "$TURSO_AUTH_TOKEN" ]; then
        echo "TURSO_AUTH_TOKEN not set. Please provide your Turso cloud authentication token."
        echo
        echo "To get your token:"
        echo "  1. Login to https://turso.tech"
        echo "  2. Create a database"
        echo "  3. Copy the connection URL and auth token"
        echo
        read -p "Enter TURSO_DATABASE_URL: " TURSO_INPUT_URL
        read -p "Enter TURSO_AUTH_TOKEN: " TURSO_INPUT_TOKEN

        export TURSO_DATABASE_URL="$TURSO_INPUT_URL"
        export TURSO_AUTH_TOKEN="$TURSO_INPUT_TOKEN"
    fi

    log_success "Turso cloud configured"
    log_info "Database URL: $TURSO_DATABASE_URL"
    log_warning "Auth token is set (will not be displayed)"

    return 0
}

# Verify vector extensions
verify_vector_extensions() {
    log_info "Verifying vector extensions..."

    echo "Attempting to connect to $TURSO_URL..."
    echo "This requires sqlite3 or turso CLI to test..."

    if command -v sqlite3 &> /dev/null; then
        if [[ "$TURSO_URL" == file://* ]]; then
            log_error "Cannot verify vector extensions with file:// URLs"
            log_error "Vector extensions are only available in Turso, not local SQLite"
            return 1
        fi

        # Note: We can't actually test libsql:// URLs with sqlite3
        log_warning "Cannot verify libsql:// URLs with sqlite3"
        log_info "Vector extensions will be verified when benchmarks run"
        return 0
    else
        log_warning "sqlite3 not available - skipping verification"
        log_info "Vector extensions will be verified when benchmarks run"
        return 0
    fi
}

# Run benchmarks
run_benchmarks() {
    log_info "Running Turso vector search benchmarks..."

    cd "$PROJECT_ROOT"

    # Export environment variables if not already set
    export TURSO_DATABASE_URL="${TURSO_DATABASE_URL:-libsql://127.0.0.1:8080}"
    export TURSO_AUTH_TOKEN="${TURSO_AUTH_TOKEN:-}"

    echo
    echo "Configuration:"
    echo "  TURSO_DATABASE_URL: $TURSO_DATABASE_URL"
    echo "  TURSO_AUTH_TOKEN: ${TURSO_AUTH_TOKEN:+<set>}"
    echo "  DB File: $DB_FILE"
    echo

    # Run benchmarks
    cargo bench --bench turso_vector_performance \
      --features memory-storage-turso/turso_multi_dimension

    return $?
}

# Cleanup function
cleanup() {
    if [ -n "${TURSO_PID:-}" ]; then
        log_info "Stopping Turso dev server (PID: $TURSO_PID)..."
        kill $TURSO_PID 2>/dev/null || true
        log_success "Turso dev server stopped"
    fi
}

# Set trap for cleanup
trap cleanup EXIT INT TERM

# Main function
main() {
    echo "========================================"
    echo "Turso Benchmark Environment Setup"
    echo "========================================"
    echo

    # Check for Turso CLI
    if ! check_turso_cli; then
        log_error "Turso CLI required. Please install it first."
        exit 1
    fi

    echo
    echo "Choose setup mode:"
    echo "  1) Local dev server (turso dev)"
    echo "  2) Turso cloud database"
    echo "  3) Use existing environment variables"
    echo
    read -p "Enter choice [1-3]: " choice

    case $choice in
        1)
            if start_local_server; then
                echo
                verify_vector_extensions || echo "Note: Verification deferred to benchmark execution"
                echo
                read -p "Run benchmarks now? (y/n) " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    run_benchmarks
                fi
            else
                log_error "Failed to start local server"
                exit 1
            fi
            ;;
        2)
            if configure_cloud_connection; then
                verify_vector_extensions || echo "Note: Verification deferred to benchmark execution"
                echo
                read -p "Run benchmarks now? (y/n) " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    run_benchmarks
                fi
            else
                log_error "Failed to configure cloud connection"
                exit 1
            fi
            ;;
        3)
            if [ -z "${TURSO_DATABASE_URL:-}" ]; then
                log_error "TURSO_DATABASE_URL not set"
                echo "Please set environment variables:"
                echo "  export TURSO_DATABASE_URL=\"libsql://your-db.turso.io\""
                echo "  export TURSO_AUTH_TOKEN=\"your-auth-token\""
                exit 1
            fi

            log_info "Using existing environment variables"
            log_info "Database URL: $TURSO_DATABASE_URL"
            verify_vector_extensions || echo "Note: Verification deferred to benchmark execution"
            echo
            read -p "Run benchmarks now? (y/n) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                run_benchmarks
            fi
            ;;
        *)
            log_error "Invalid choice"
            exit 1
            ;;
    esac
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Turso Benchmark Environment Setup Script"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --local         Use local dev server (skip prompt)"
        echo "  --cloud          Use Turso cloud (skip prompt)"
        echo "  --env            Use existing environment variables (skip prompt)"
        echo "  --run            Run benchmarks after setup (skip prompt)"
        echo
        exit 0
        ;;
    --local)
        check_turso_cli || exit 1
        start_local_server || exit 1
        verify_vector_extensions || true
        [[ "${2:-}" == "--run" ]] && run_benchmarks
        ;;
    --cloud)
        configure_cloud_connection || exit 1
        verify_vector_extensions || true
        [[ "${2:-}" == "--run" ]] && run_benchmarks
        ;;
    --env)
        if [ -z "${TURSO_DATABASE_URL:-}" ]; then
            log_error "TURSO_DATABASE_URL not set"
            exit 1
        fi
        verify_vector_extensions || true
        [[ "${2:-}" == "--run" ]] && run_benchmarks
        ;;
    --run)
        # Skip setup, just run benchmarks
        run_benchmarks
        ;;
    *)
        main
        ;;
esac
