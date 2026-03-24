#!/usr/bin/env bash
# clean-artifacts.sh - Workspace disk hygiene helper
#
# See ADR-032 for background:
# plans/adr/ADR-032-Disk-Space-Optimization.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

MODE="standard"
INCLUDE_NODE_MODULES="false"
DRY_RUN="false"
TARGET_DIR_OVERRIDE=""

log_info() {
  echo -e "${GREEN}[INFO]${NC} $*"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $*"
}

usage() {
  cat <<EOF
Usage: ./scripts/clean-artifacts.sh [mode] [options]

Modes:
  quick      Remove incremental caches only (fastest)
  standard   Remove incremental + release + coverage artifacts (default)
  full       Run cargo clean, then remove coverage artifacts

Options:
  --node-modules        Also remove root-level node_modules directories
  --target-dir <path>   Override target directory (default: CARGO_TARGET_DIR or ./target)
  --dry-run             Show what would be removed without deleting
  -h, --help            Show this help text

Notes:
  - If CARGO_TARGET_DIR is set, this script uses it automatically.
  - Relative target dirs are resolved from repository root.
  - node_modules cleanup is opt-in to avoid surprising JS/tooling workflows.

Examples:
  ./scripts/clean-artifacts.sh
  ./scripts/clean-artifacts.sh quick
  ./scripts/clean-artifacts.sh standard --node-modules
  CARGO_TARGET_DIR=.cargo-target ./scripts/clean-artifacts.sh full
  ./scripts/clean-artifacts.sh --target-dir /tmp/rslm-target --dry-run
EOF
}

resolve_target_dir() {
  local configured
  if [[ -n "$TARGET_DIR_OVERRIDE" ]]; then
    configured="$TARGET_DIR_OVERRIDE"
  elif [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    configured="${CARGO_TARGET_DIR}"
  else
    configured="target"
  fi

  if [[ "$configured" = /* ]]; then
    printf '%s\n' "$configured"
  else
    printf '%s\n' "$PROJECT_ROOT/$configured"
  fi
}

get_size() {
  local path="$1"
  if [[ -e "$path" ]]; then
    du -sh "$path" 2>/dev/null | cut -f1 || echo "0"
  else
    echo "0"
  fi
}

remove_path() {
  local path="$1"
  local label="$2"

  if [[ -z "$path" || "$path" = "/" ]]; then
    log_error "Refusing to remove unsafe path: '$path'"
    exit 1
  fi

  if [[ ! -e "$path" ]]; then
    return 0
  fi

  local size
  size="$(get_size "$path")"

  if [[ "$DRY_RUN" = "true" ]]; then
    log_info "[dry-run] Would remove $label ($path, $size)"
  else
    rm -rf "$path"
    log_info "Removed $label ($size)"
  fi
}

remove_file_if_exists() {
  local file="$1"
  local label="$2"

  if [[ ! -f "$file" ]]; then
    return 0
  fi

  local size
  size="$(get_size "$file")"

  if [[ "$DRY_RUN" = "true" ]]; then
    log_info "[dry-run] Would remove $label ($file, $size)"
  else
    rm -f "$file"
    log_info "Removed $label ($size)"
  fi
}

show_disk_usage() {
  local target_dir="$1"

  log_info "Disk usage snapshot"
  echo "  target dir: $target_dir"
  echo "  total target:            $(get_size "$target_dir")"
  echo "  debug incremental:       $(get_size "$target_dir/debug/incremental")"
  echo "  release incremental:     $(get_size "$target_dir/release/incremental")"
  echo "  release:                 $(get_size "$target_dir/release")"
  echo "  llvm-cov target:         $(get_size "$target_dir/llvm-cov-target")"
  echo "  coverage/:               $(get_size "$PROJECT_ROOT/coverage")"
  echo "  node_modules/:           $(get_size "$PROJECT_ROOT/node_modules")"
  echo "  memory-cli/node_modules: $(get_size "$PROJECT_ROOT/memory-cli/node_modules")"
  echo ""
}

clean_coverage_artifacts() {
  log_info "Cleaning coverage artifacts"

  local target_dir="$1"
  remove_path "$target_dir/llvm-cov-target" "target llvm-cov artifacts"
  remove_path "$PROJECT_ROOT/coverage" "coverage report directory"
  remove_path "$PROJECT_ROOT/coverage-html" "coverage-html directory"

  remove_file_if_exists "$PROJECT_ROOT/lcov.info" "lcov report"
  remove_file_if_exists "$PROJECT_ROOT/cobertura.xml" "cobertura report"

  local prof_count
  prof_count="$(find "$PROJECT_ROOT" \( -name "*.profraw" -o -name "*.profdata" \) 2>/dev/null | wc -l | tr -d ' ')"
  if [[ "$prof_count" -gt 0 ]]; then
    if [[ "$DRY_RUN" = "true" ]]; then
      log_info "[dry-run] Would remove $prof_count profile files (*.profraw/*.profdata)"
    else
      find "$PROJECT_ROOT" -name "*.profraw" -delete 2>/dev/null || true
      find "$PROJECT_ROOT" -name "*.profdata" -delete 2>/dev/null || true
      log_info "Removed $prof_count profile files (*.profraw/*.profdata)"
    fi
  fi
}

clean_quick() {
  local target_dir="$1"
  log_info "Quick clean: incremental caches"
  remove_path "$target_dir/debug/incremental" "debug incremental cache"
  remove_path "$target_dir/release/incremental" "release incremental cache"
}

clean_standard() {
  local target_dir="$1"
  log_info "Standard clean: incremental + release + coverage"
  clean_quick "$target_dir"
  remove_path "$target_dir/release" "release artifacts"
  clean_coverage_artifacts "$target_dir"
}

clean_full() {
  local target_dir="$1"
  log_info "Full clean: cargo clean + coverage cleanup"

  if [[ "$DRY_RUN" = "true" ]]; then
    log_info "[dry-run] Would run: cargo clean --target-dir $target_dir"
  else
    cargo clean --target-dir "$target_dir"
  fi

  clean_coverage_artifacts "$target_dir"
}

clean_node_modules() {
  if [[ "$INCLUDE_NODE_MODULES" != "true" ]]; then
    return 0
  fi

  log_warn "Optional cleanup enabled: removing node_modules directories"
  remove_path "$PROJECT_ROOT/node_modules" "root node_modules"
  remove_path "$PROJECT_ROOT/memory-cli/node_modules" "memory-cli node_modules"
}

parse_args() {
  if [[ $# -eq 0 ]]; then
    return 0
  fi

  while [[ $# -gt 0 ]]; do
    case "$1" in
      quick|standard|full)
        MODE="$1"
        shift
        ;;
      --node-modules)
        INCLUDE_NODE_MODULES="true"
        shift
        ;;
      --target-dir)
        if [[ $# -lt 2 ]]; then
          log_error "--target-dir requires a value"
          usage
          exit 1
        fi
        TARGET_DIR_OVERRIDE="$2"
        shift 2
        ;;
      --dry-run)
        DRY_RUN="true"
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        log_error "Unknown argument: $1"
        usage
        exit 1
        ;;
    esac
  done
}

main() {
  parse_args "$@"

  local target_dir
  target_dir="$(resolve_target_dir)"

  echo ""
  echo "========================================"
  echo "  Workspace Artifact Cleaner"
  echo "========================================"
  echo ""
  echo "Mode:               $MODE"
  echo "Target directory:   $target_dir"
  echo "Node modules clean: $INCLUDE_NODE_MODULES"
  echo "Dry run:            $DRY_RUN"
  echo ""

  show_disk_usage "$target_dir"

  case "$MODE" in
    quick)
      clean_quick "$target_dir"
      ;;
    standard)
      clean_standard "$target_dir"
      ;;
    full)
      clean_full "$target_dir"
      ;;
    *)
      log_error "Unknown mode: $MODE"
      usage
      exit 1
      ;;
  esac

  clean_node_modules

  echo ""
  log_info "Remaining disk usage"
  show_disk_usage "$target_dir"
}

main "$@"
