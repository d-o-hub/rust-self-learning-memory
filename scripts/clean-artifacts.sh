#!/usr/bin/env bash
# Clean Rust build artifacts with configurable aggressiveness
#
# Usage:
#   ./scripts/clean-artifacts.sh [quick|standard|full]
#
# Modes:
#   quick    - Clean incremental caches only (~38GB)
#   standard - Clean incremental + coverage + release (~48GB)
#   full     - Complete cargo clean (~74GB)
#
# See ADR-032 for details: plans/adr/ADR-032-Disk-Space-Optimization.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

get_size() {
    local path="$1"
    if [[ -d "$path" ]]; then
        du -sh "$path" 2>/dev/null | cut -f1 || echo "0"
    else
        echo "0"
    fi
}

show_disk_usage() {
    log_info "Current target/ disk usage:"
    echo ""
    
    local total_size=$(get_size "$PROJECT_ROOT/target")
    echo "  Total target/:           $total_size"
    
    if [[ -d "$PROJECT_ROOT/target/debug" ]]; then
        echo "  target/debug/:           $(get_size "$PROJECT_ROOT/target/debug")"
        echo "    incremental/:          $(get_size "$PROJECT_ROOT/target/debug/incremental")"
        echo "    deps/:                 $(get_size "$PROJECT_ROOT/target/debug/deps")"
        echo "    examples/:             $(get_size "$PROJECT_ROOT/target/debug/examples")"
        echo "    build/:                $(get_size "$PROJECT_ROOT/target/debug/build")"
    fi
    
    if [[ -d "$PROJECT_ROOT/target/release" ]]; then
        echo "  target/release/:         $(get_size "$PROJECT_ROOT/target/release")"
    fi
    
    if [[ -d "$PROJECT_ROOT/target/llvm-cov-target" ]]; then
        echo "  target/llvm-cov-target/: $(get_size "$PROJECT_ROOT/target/llvm-cov-target")"
    fi
    
    echo ""
}

clean_quick() {
    log_info "Quick clean - removing incremental caches..."
    
    local freed=0
    
    if [[ -d "$PROJECT_ROOT/target/debug/incremental" ]]; then
        local size=$(get_size "$PROJECT_ROOT/target/debug/incremental")
        rm -rf "$PROJECT_ROOT/target/debug/incremental"
        log_info "Removed target/debug/incremental/ ($size)"
    fi
    
    if [[ -d "$PROJECT_ROOT/target/release/incremental" ]]; then
        local size=$(get_size "$PROJECT_ROOT/target/release/incremental")
        rm -rf "$PROJECT_ROOT/target/release/incremental"
        log_info "Removed target/release/incremental/ ($size)"
    fi
    
    log_info "Quick clean completed"
}

clean_standard() {
    log_info "Standard clean - removing incremental, coverage, and release artifacts..."
    
    # Incremental caches
    clean_quick
    
    # Coverage artifacts
    if [[ -d "$PROJECT_ROOT/target/llvm-cov-target" ]]; then
        local size=$(get_size "$PROJECT_ROOT/target/llvm-cov-target")
        rm -rf "$PROJECT_ROOT/target/llvm-cov-target"
        log_info "Removed target/llvm-cov-target/ ($size)"
    fi
    
    # Release build
    if [[ -d "$PROJECT_ROOT/target/release" ]]; then
        local size=$(get_size "$PROJECT_ROOT/target/release")
        rm -rf "$PROJECT_ROOT/target/release"
        log_info "Removed target/release/ ($size)"
    fi
    
    # Profraw/profdata files
    local prof_count=$(find "$PROJECT_ROOT" -name "*.profraw" -o -name "*.profdata" 2>/dev/null | wc -l)
    if [[ $prof_count -gt 0 ]]; then
        find "$PROJECT_ROOT" -name "*.profraw" -delete 2>/dev/null || true
        find "$PROJECT_ROOT" -name "*.profdata" -delete 2>/dev/null || true
        log_info "Removed $prof_count coverage files"
    fi
    
    log_info "Standard clean completed"
}

clean_full() {
    log_info "Full clean - running cargo clean..."
    
    cd "$PROJECT_ROOT"
    cargo clean
    
    # Also remove any stray coverage files
    find "$PROJECT_ROOT" -name "*.profraw" -delete 2>/dev/null || true
    find "$PROJECT_ROOT" -name "*.profdata" -delete 2>/dev/null || true
    
    log_info "Full cargo clean completed"
}

main() {
    local mode="${1:-standard}"
    
    echo ""
    echo "========================================"
    echo "  Rust Build Artifact Cleaner"
    echo "========================================"
    echo ""
    
    # Check if target directory exists
    if [[ ! -d "$PROJECT_ROOT/target" ]]; then
        log_info "No target/ directory found - nothing to clean"
        exit 0
    fi
    
    # Show current usage
    show_disk_usage
    
    case "$mode" in
        quick)
            clean_quick
            ;;
        standard)
            clean_standard
            ;;
        full)
            clean_full
            ;;
        *)
            log_error "Unknown mode: $mode"
            echo ""
            echo "Usage: $0 [quick|standard|full]"
            echo ""
            echo "Modes:"
            echo "  quick    - Clean incremental caches only (~38GB)"
            echo "  standard - Clean incremental + coverage + release (~48GB)"
            echo "  full     - Complete cargo clean (~74GB)"
            exit 1
            ;;
    esac
    
    echo ""
    log_info "Remaining disk usage:"
    show_disk_usage
}

main "$@"
