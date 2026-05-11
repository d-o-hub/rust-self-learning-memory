#!/usr/bin/env bash
# code-quality.sh - Rust code quality operations
# Optimized CLI for code-quality agent

set -uo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly RED='\033[0;31m'
readonly NC='\033[0m'

show_help() {
  echo "Usage: $(basename "$0") [operation] [options]"
  echo ""
  echo "Operations:"
  echo "  fmt           Format code (fast check)"
  echo "  clippy        Lint with clippy (strict)"
  echo "  audit         Security audit"
  echo "  check          Run all quality gates"
  echo ""
  echo "Options:"
  echo "  --workspace   Format/Lint entire workspace"
  echo "  --package <P> Format/Lint specific package"
  echo "  --strict      Clippy: deny warnings"
  echo "  --fix         Auto-fix (applies to fmt and clippy)"
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

# Default values
OP=""
WORKSPACE_FLAG=""
PACKAGE_FLAG=""
STRICT=""
FIX=""

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --workspace)
      WORKSPACE_FLAG="true"
      ;;
    --package)
      shift
      PACKAGE_FLAG="$1"
      ;;
    --strict)
      STRICT="-D warnings"
      ;;
    --fix)
      FIX="true"
      ;;
    fmt|clippy|audit|check)
      OP="$1"
      ;;
    -h|--help)
      show_help
      exit 0
      ;;
    *)
      echo -e "${RED}Error: Unknown argument: $1${NC}"
      show_help
      exit 1
      ;;
  esac
  shift
done

OP="${OP:-fmt}"

# Check if in project root
if [[ ! -f "$PROJECT_ROOT/Cargo.toml" && ! -f "$PROJECT_ROOT/Cargo.lock" ]]; then
  echo -e "${YELLOW}Warning: Not in project root${NC}"
  echo "Current directory: $PROJECT_ROOT"
  echo "Project root should contain: Cargo.toml and Cargo.lock"
  exit 1
fi

# Operation implementations
case "$OP" in
  fmt)
    echo -e "${BLUE}📐 Formatting code...${NC}"
    FMT_ARGS=""
    if [[ -n "$WORKSPACE_FLAG" ]]; then
      FMT_ARGS="--all"
    elif [[ -n "$PACKAGE_FLAG" ]]; then
      FMT_ARGS="--package $PACKAGE_FLAG"
    else
      FMT_ARGS="--all" # Default to all if nothing specified
    fi

    if [[ -z "$FIX" ]]; then
      echo -e "${YELLOW}Formatting check only...${NC}"
      cargo fmt $FMT_ARGS -- --check
    else
      echo -e "${YELLOW}Applying formatting fixes...${NC}"
      cargo fmt $FMT_ARGS
    fi
    
    fmt_result=$?
    if [[ $fmt_result -eq 0 ]]; then
      echo -e "${GREEN}✅ Formatting check passed${NC}"
    else
      echo -e "${RED}❌ Formatting check failed${NC}"
      if [[ -z "$FIX" ]]; then
        echo -e "${YELLOW}Run '$(basename "$0") fmt --fix' to fix${NC}"
      fi
    fi
    exit $fmt_result
    ;;

  clippy)
    echo -e "${BLUE}🔍 Linting with Clippy...${NC}"
    # CI parity: Run clippy on both lib and tests with same flags as .github/workflows/quick-check.yml
    CLIPPY_FLAGS="-D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used"
    if [[ -n "$STRICT" ]]; then
       CLIPPY_FLAGS="$CLIPPY_FLAGS $STRICT"
    fi
    
    CLIPPY_ARGS=""
    if [[ -n "$WORKSPACE_FLAG" ]]; then
      CLIPPY_ARGS="--workspace --tests"
    elif [[ -n "$PACKAGE_FLAG" ]]; then
      CLIPPY_ARGS="--package $PACKAGE_FLAG --tests"
    else
      CLIPPY_ARGS="--tests"
    fi

    if [[ -n "$FIX" ]]; then
      CLIPPY_ARGS="$CLIPPY_ARGS --fix --allow-dirty"
    fi
    
    echo -e "${YELLOW}Running: cargo clippy $CLIPPY_ARGS -- $CLIPPY_FLAGS${NC}"
    cargo clippy $CLIPPY_ARGS -- $CLIPPY_FLAGS
    
    clippy_result=$?
    if [[ $clippy_result -eq 0 ]]; then
      echo -e "${GREEN}✅ No Clippy warnings${NC}"
    else
      echo -e "${RED}❌ Clippy found warnings/errors (exit code: $clippy_result)${NC}"
    fi
    exit $clippy_result
    ;;

  audit)
    echo -e "${BLUE}🔒 Security audit...${NC}"
    cargo audit
    
    audit_result=$?
    if [[ $audit_result -eq 0 ]]; then
      echo -e "${GREEN}✅ No security vulnerabilities${NC}"
    else
      echo -e "${RED}❌ Security vulnerabilities found!${NC}"
      echo -e "${YELLOW}Review output above for details${NC}"
    fi
    exit $audit_result
    ;;

  check)
    echo -e "${BLUE}🔎 Running quality gates...${NC}"
    
    # CI parity: Clippy flags matching .github/workflows/quick-check.yml
    CLIPPY_FLAGS="-D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used"

    # Run formatting check
    echo -e "${BLUE}  Formatting check...${NC}"
    cargo fmt --all -- --check
    fmt_result=$?
    
    # Run clippy on lib + tests (CI parity)
    echo -e "${BLUE}  Clippy check (lib + tests, strict mode)...${NC}"
    cargo clippy --workspace --tests -- $CLIPPY_FLAGS
    clippy_result=$?
    
    # Run audit
    echo -e "${BLUE}  Security audit...${NC}"
    cargo audit
    audit_result=$?
    
    # Summary
    echo ""
    echo "============================================"
    echo -e "${GREEN}📊 Quality Gates Summary${NC}"
    echo "============================================"

    if [[ $fmt_result -eq 0 ]]; then
      echo -e "  Formatting: ${GREEN}✅ PASS${NC}"
    else
      echo -e "  Formatting: ${RED}FAIL${NC}"
    fi

    if [[ $clippy_result -eq 0 ]]; then
      echo -e "  Clippy: ${GREEN}✅ PASS${NC}"
    else
      echo -e "  Clippy: ${RED}FAIL (exit code: ${clippy_result})${NC}"
    fi

    if [[ $audit_result -eq 0 ]]; then
      echo -e "  Security: ${GREEN}✅ PASS${NC}"
    else
      echo -e "  Security: ${RED}FAIL${NC}"
    fi

    echo ""
    echo "Next steps:"
    echo "  • Run '$(basename "$0") fmt --fix' to fix formatting"
    echo "  • Run '$(basename "$0") clippy --fix' to auto-fix warnings"
    echo "  • Update dependencies with 'cargo update'"
    echo ""
    echo "============================================"
    
    if [[ $fmt_result -ne 0 || $clippy_result -ne 0 || $audit_result -ne 0 ]]; then
      exit 1
    fi
    ;;

  *)
    echo -e "${RED}Error: Unknown operation: $OP${NC}"
    show_help
    exit 1
    ;;
esac
