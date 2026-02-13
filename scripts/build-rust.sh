#!/usr/bin/env bash
# build-rust.sh - Optimized Rust build operations
# Usage: ./scripts/build-rust.sh [dev|release|profile|check|clean] [crate]

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m' # No Color

# Configuration
readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Logging functions
log_info() {
  echo -e "${GREEN}[build-rust]${NC} $*"
}

log_warn() {
  echo -e "${YELLOW}[build-rust]${NC} $*" >&2
}

log_error() {
  echo -e "${RED}[build-rust]${NC} $*" >&2
}

# Show usage
usage() {
  cat <<EOF
Usage: $(basename "$0") <mode> [crate]

Modes:
  dev       Development build (fast, debug symbols)
  release   Release build (optimized, stripped)
  profile    Build with timing information
  check      Fast type-check only
  clean      Clean build artifacts

Options:
  crate      Optional: Build specific crate (e.g., memory-core)

Examples:
  $0 dev
  $0 release memory-core
  $0 profile
  $0 clean
EOF
  exit 1
}

# Validate crate name
validate_crate() {
  local crate="$1"
  if [[ ! "$crate" =~ ^[a-z0-9_]+$ ]]; then
    log_error "Invalid crate name: $crate"
    return 1
  fi

  # Check if crate exists
  if ! cargo metadata --format-version 1 --no-deps 2>/dev/null | \
    jq -e ".packages[].name | select(. == \"$crate\")" >/dev/null; then
    log_warn "Crate '$crate' not found in workspace"
    log_warn "Available crates:"
    cargo metadata --format-version 1 --no-deps 2>/dev/null | \
      jq -r '.packages[].name' | sed 's/^/  - /' >&2
    return 1
  fi
}

# Main build function
build() {
  local mode="$1"
  local crate="${2:-}"

  log_info "Starting $mode build..."

  case "$mode" in
    dev)
      if [[ -n "$crate" ]]; then
        validate_crate "$crate" || return 1
        cargo build --package "$crate"
      else
        cargo build --workspace
      fi
      ;;

    release)
      local cmd=(cargo build --release)
      if [[ -n "$crate" ]]; then
        validate_crate "$crate" || return 1
        cmd+=(--package "$crate")
      else
        cmd+=(--workspace)
      fi

      log_info "Optimizing for production..."
      "${cmd[@]}"
      ;;

    profile)
      if [[ -n "$crate" ]]; then
        log_warn "Profile mode ignores crate argument"
      fi
      log_info "Building with timing information..."
      cargo build --release --workspace --timings
      ;;

    check)
      if [[ -n "$crate" ]]; then
        validate_crate "$crate" || return 1
        cargo check --package "$crate"
      else
        cargo check --workspace
      fi
      ;;

    clean)
      log_info "Cleaning build artifacts..."
      if [[ -n "$crate" ]]; then
        validate_crate "$crate" || return 1
        cargo clean --package "$crate"
      else
        cargo clean
      fi
      ;;

    *)
      log_error "Unknown mode: $mode"
      usage
      ;;
  esac

  local exit_code=$?
  if [[ $exit_code -eq 0 ]]; then
    log_info "✅ Build successful"
  else
    log_error "❌ Build failed with exit code $exit_code"
  fi

  return $exit_code
}

# Parse arguments
if [[ $# -lt 1 ]]; then
  usage
fi

MODE="$1"
CRATE="${2:-}"

# Validate mode
case "$MODE" in
  dev|release|profile|check|clean)
    build "$MODE" "$CRATE"
    ;;
  *)
    log_error "Invalid mode: $MODE"
    usage
    ;;
esac
