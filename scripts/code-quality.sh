#!/usr/bin/env bash
# code-quality.sh - Rust code quality operations
# Optimized CLI for code-quality agent

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly RED='\033[0;31m'
readonly NC='\033[0m'

show_help() {
  echo "Usage: $(basename "$0") <operation> [options]"
  echo ""
  echo "Operations:"
  echo "  fmt           Format code (fast check)"
  echo "  clippy        Lint with clippy (strict)"
  echo "  audit         Security audit"
  echo "  check          Run all quality gates"
  echo "  fix           Auto-fix common issues"
  echo ""
  echo "Options:"
  echo "  --workspace   Format/Lint entire workspace"
  echo "  --package <P> Format/Lint specific package"
  echo "  --strict      Clippy: deny warnings"
  echo "  --fix         Auto-fix with cargo clippy"
  echo ""
  echo "Examples:"
  echo "  # Quick format check"
  echo "  $(basename "$0") fmt"
  echo ""
  echo "  # Full workspace format"
  echo "  $(basename "$0") fmt --workspace"
  echo ""
  echo "  # Security audit"
  echo "  $(basename "$0") audit"
  echo ""
  echo "  # Run quality gates"
  echo "  $(basename "$0") check"
  echo ""
  echo "  # Auto-fix issues"
  echo "  $(basename "$0") clippy --fix"
}

# Default operations
OP=""
WORKSPACE_FLAG=""
PACKAGE_NAME=""
STRICT=""
FIX_FLAG=""

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --workspace)
      WORKSPACE_FLAG="true"
      shift
      ;;
    --package)
      PACKAGE_NAME="${2:?Package name required}"
      shift 2
      ;;
    --strict)
      STRICT="-D warnings"
      shift
      ;;
    --fix)
      FIX_FLAG="true"
      shift
      ;;
    fmt|clippy|audit|check|fix)
      OP="$1"
      shift
      ;;
    help|--help|-h)
      show_help
      exit 0
      ;;
    *)
      echo -e "${RED}Error: Unknown argument: $1${NC}"
      show_help
      exit 1
      ;;
  esac
done

OP="${OP:-fmt}"

# Check if in project root
if [[ ! -f "$PROJECT_ROOT/Cargo.toml" && ! -f "$PROJECT_ROOT/Cargo.lock" ]]; then
  echo -e "${YELLOW}Warning: Not in project root${NC}"
  echo "Current directory: $PROJECT_ROOT"
  echo "Project root should contain: Cargo.toml and Cargo.lock"
  exit 1
fi

# Helper to build cargo flags
get_cargo_flags() {
  if [[ -n "$WORKSPACE_FLAG" ]]; then
    echo "--workspace"
  elif [[ -n "$PACKAGE_NAME" ]]; then
    echo "--package $PACKAGE_NAME"
  else
    echo ""
  fi
}

# Operation implementations
case "$OP" in
  fmt)
    echo -e "${BLUE}📐 Formatting code...${NC}"
    FLAGS=$(get_cargo_flags)

    if [[ -z "$FLAGS" ]]; then
      # Default to all if no specific flag
      echo -e "${YELLOW}Formatting entire workspace check...${NC}"
      cargo fmt --all -- --check
    else
      echo -e "${YELLOW}Formatting with flags: $FLAGS${NC}"
      # cargo fmt doesn't take --workspace, but --all is usually what's wanted
      if [[ "$FLAGS" == "--workspace" ]]; then
         cargo fmt --all -- --check
      else
         cargo fmt $FLAGS -- --check
      fi
    fi
    
    if [[ $? -eq 0 ]]; then
      echo -e "${GREEN}✅ Formatting check passed${NC}"
    else
      echo -e "${RED}❌ Formatting check failed${NC}"
      echo -e "${YELLOW}Run 'cargo fmt' to fix${NC}"
      exit 1
    fi
    ;;

  clippy)
    echo -e "${BLUE}🔍 Linting with Clippy...${NC}"
    CLIPPY_FLAGS="-D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used"
    if [[ -n "$STRICT" ]]; then
      CLIPPY_FLAGS="$CLIPPY_FLAGS $STRICT"
    fi
    
    CARGO_FLAGS=$(get_cargo_flags)
    [[ -z "$CARGO_FLAGS" ]] && CARGO_FLAGS="--workspace"

    EXTRA_ARGS=""
    if [[ -n "$FIX_FLAG" ]]; then
      EXTRA_ARGS="--fix --allow-dirty --allow-staged"
    fi

    echo -e "${YELLOW}Linting with flags: $CARGO_FLAGS (lib + tests)...${NC}"
    cargo clippy $CARGO_FLAGS $EXTRA_ARGS --tests -- $CLIPPY_FLAGS
    
    clippy_result=$?
    if [[ $clippy_result -eq 0 ]]; then
      echo -e "${GREEN}✅ No Clippy warnings${NC}"
    else
      echo -e "${YELLOW}⚠️  Clippy found warnings (exit code: $clippy_result)${NC}"
      exit $clippy_result
    fi
    ;;

  fix)
    echo -e "${BLUE}🔧 Auto-fixing common issues...${NC}"
    echo -e "${YELLOW}Running cargo fmt...${NC}"
    cargo fmt --all

    echo -e "${YELLOW}Running cargo clippy --fix...${NC}"
    cargo clippy --workspace --tests --fix --allow-dirty --allow-staged -- -A clippy::all

    echo -e "${GREEN}✅ Fixes applied${NC}"
    ;;

  audit)
    echo -e "${BLUE}🔒 Security audit...${NC}"
    cargo audit
    
    audit_result=$?
    if [[ $audit_result -eq 0 ]]; then
      echo -e "${GREEN}✅ No security vulnerabilities${NC}"
    else
      echo -e "${RED}❌ Security vulnerabilities found!${NC}"
      exit $audit_result
    fi
    ;;

  check)
    echo -e "${BLUE}🔎 Running quality gates...${NC}"
    
    # Run formatting check
    "$0" fmt --workspace || exit 1
    
    # Run clippy
    "$0" clippy --workspace || exit 1
    
    # Run audit
    "$0" audit || exit 1
    
    echo ""
    echo "============================================"
    echo -e "${GREEN}📊 Quality Gates Summary: ALL PASSED${NC}"
    echo "============================================"
    ;;

  *)
    echo -e "${RED}Error: Unknown operation: $OP${NC}"
    show_help
    exit 1
    ;;
esac
