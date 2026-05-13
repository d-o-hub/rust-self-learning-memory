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
  echo "Usage: $(basename "$0") [operation] [options]"
  echo ""
  echo "Operations:"
  echo "  fmt           Format code (default)"
  echo "  clippy        Lint with clippy"
  echo "  audit         Security audit"
  echo "  check         Run all quality gates"
  echo "  fix           Auto-fix common issues"
  echo ""
  echo "Options:"
  echo "  --workspace   Apply to entire workspace"
  echo "  --package <P> Apply to specific package"
  echo "  --strict      Clippy: deny warnings"
  echo "  --fix         Clippy: auto-fix issues"
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
}

# Default operations
OP="${1:-fmt}"
WORKSPACE_FLAG=""
PACKAGE_FLAG=""
STRICT=""
FIX=""

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --workspace)
      WORKSPACE_FLAG="--workspace"
      ;;
    --package)
      shift
      PACKAGE_FLAG="--package $1"
      ;;
    --strict)
      STRICT="-D warnings"
      ;;
    --fix)
      FIX="true"
      ;;
    fmt|clippy|audit|check|fix)
      OP="$1"
      ;;
    *)
      # If it's not a known flag or operation, error out
      # (Unless it's the very first arg which we already assigned to OP)
      if [[ "$1" != "${OP:-}" ]]; then
        echo -e "${RED}Error: Unknown argument: $1${NC}"
        show_help
        exit 1
      fi
      ;;
  esac
  shift
done

# Ensure OP is valid, otherwise default to fmt
case "$OP" in
  fmt|clippy|audit|check|fix) ;;
  *) OP="fmt" ;;
esac

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
    if [[ -n "$WORKSPACE_FLAG" ]]; then
      echo -e "${YELLOW}Formatting entire workspace...${NC}"
      if cargo fmt --all; then fmt_res=0; else fmt_res=$?; fi
    elif [[ -n "$PACKAGE_FLAG" ]]; then
      pkg="${PACKAGE_FLAG#--package }"
      echo -e "${YELLOW}Formatting package: $pkg${NC}"
      if cargo fmt --package "$pkg"; then fmt_res=0; else fmt_res=$?; fi
    else
      echo -e "${YELLOW}Formatting check only...${NC}"
      if cargo fmt --all -- --check; then fmt_res=0; else fmt_res=$?; fi
    fi
    
    if [[ $fmt_res -eq 0 ]]; then
      echo -e "${GREEN}✅ Formatting check passed${NC}"
    else
      echo -e "${RED}❌ Formatting check failed (exit code: $fmt_res)${NC}"
      echo -e "${YELLOW}Run 'cargo fmt' to fix${NC}"
      exit $fmt_res
    fi
    ;;

  clippy)
    echo -e "${BLUE}🔍 Linting with Clippy...${NC}"
    CLIPPY_FLAGS="-A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used"
    if [[ -n "$STRICT" ]]; then
      CLIPPY_FLAGS="-D warnings $CLIPPY_FLAGS"
    fi
    
    EXTRA_ARGS=""
    if [[ -n "$FIX" ]]; then
      EXTRA_ARGS="--fix --allow-dirty --allow-staged"
    fi

    if [[ -n "$WORKSPACE_FLAG" ]]; then
      echo -e "${YELLOW}Linting entire workspace (lib + tests)...${NC}"
      if cargo clippy --workspace --tests $EXTRA_ARGS -- $CLIPPY_FLAGS; then clippy_result=0; else clippy_result=$?; fi
    elif [[ -n "$PACKAGE_FLAG" ]]; then
      pkg="${PACKAGE_FLAG#--package }"
      echo -e "${YELLOW}Linting package: $pkg (lib + tests)...${NC}"
      if cargo clippy --package "$pkg" --tests $EXTRA_ARGS -- $CLIPPY_FLAGS; then clippy_result=0; else clippy_result=$?; fi
    else
      echo -e "${YELLOW}Linting current package (lib + tests)...${NC}"
      if cargo clippy --tests $EXTRA_ARGS -- $CLIPPY_FLAGS; then clippy_result=0; else clippy_result=$?; fi
    fi
    
    if [[ $clippy_result -eq 0 ]]; then
      echo -e "${GREEN}✅ No Clippy warnings${NC}"
    else
      echo -e "${YELLOW}⚠️  Clippy found warnings (exit code: $clippy_result)${NC}"
      exit $clippy_result
    fi
    ;;

  audit)
    echo -e "${BLUE}🔒 Security audit...${NC}"
    if cargo audit; then audit_result=0; else audit_result=$?; fi
    
    if [[ $audit_result -eq 0 ]]; then
      echo -e "${GREEN}✅ No security vulnerabilities${NC}"
    else
      echo -e "${RED}❌ Security vulnerabilities found! (exit code: $audit_result)${NC}"
      exit $audit_result
    fi
    ;;

  check)
    echo -e "${BLUE}🔎 Running quality gates...${NC}"
    
    # CI parity: Clippy flags matching .github/workflows/quick-check.yml
    CLIPPY_FLAGS="-D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used"

    # Run formatting check
    echo -e "${BLUE}  Formatting check...${NC}"
    if cargo fmt --all -- --check; then fmt_result=0; else fmt_result=$?; fi
    
    # Run clippy on lib + tests (CI parity)
    echo -e "${BLUE}  Clippy check (lib + tests, strict mode)...${NC}"
    if cargo clippy --workspace --tests -- $CLIPPY_FLAGS; then clippy_result=0; else clippy_result=$?; fi
    
    # Run audit
    echo -e "${BLUE}  Security audit...${NC}"
    if cargo audit; then audit_result=0; else audit_result=$?; fi
    
    # Summary
    echo ""
    echo "============================================"
    echo -e "${GREEN}📊 Quality Gates Summary${NC}"
    echo "============================================"

    if [[ $fmt_result -eq 0 ]]; then
      echo -e "  Formatting: ${GREEN}✅ PASS${NC}"
    else
      echo -e "  Formatting: ${RED}FAIL (exit code: $fmt_result)${NC}"
    fi

    if [[ $clippy_result -eq 0 ]]; then
      echo -e "  Clippy: ${GREEN}✅ PASS${NC}"
    else
      echo -e "  Clippy: ${RED}FAIL (exit code: ${clippy_result})${NC}"
    fi

    if [[ $audit_result -eq 0 ]]; then
      echo -e "  Security: ${GREEN}✅ PASS${NC}"
    else
      echo -e "  Security: ${RED}FAIL (exit code: $audit_result)${NC}"
    fi

    echo ""
    echo "Next steps:"
    echo "  • Run 'cargo fmt' to fix formatting"
    echo "  • Run 'cargo clippy --tests --fix' to auto-fix warnings"
    echo "  • Update dependencies with 'cargo update'"
    echo ""
    echo "============================================"
    
    # Exit with non-zero if any gate failed
    if [[ $fmt_result -ne 0 || $clippy_result -ne 0 || $audit_result -ne 0 ]]; then
      exit 1
    fi
    ;;

  fix)
    echo -e "${BLUE}🛠️  Auto-fixing issues...${NC}"
    echo -e "${BLUE}  Formatting...${NC}"
    cargo fmt --all
    
    echo -e "${BLUE}  Clippy auto-fix...${NC}"
    cargo clippy --workspace --tests --fix --allow-dirty --allow-staged -- -D warnings -A clippy::expect_used -A clippy::uninlined_format_args -A clippy::unwrap_used
    
    echo -e "${GREEN}✅ Auto-fix complete${NC}"
    ;;

  *)
    echo -e "${RED}Error: Unknown operation: $OP${NC}"
    show_help
    exit 1
    ;;
esac
