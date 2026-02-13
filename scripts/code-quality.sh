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
  echo "  --workspace   Format entire workspace"
  echo "  --package     Format specific package"
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
OP="${1:-fmt}"
WORKSPACE_FLAG=""
PACKAGE_FLAG=""
STRICT=""
FIX=""

# Parse arguments
shift
while [[ $# -gt 0 ]]; do
  case "$1" in
    --workspace)
      WORKSPACE_FLAG="--workspace"
      ;;
    --package)
      PACKAGE_FLAG="--package"
      shift
      OP="${2:?fmt}"
      ;;
    --strict)
      STRICT="--deny-warnings"
      ;;
    --fix)
      FIX="true"
      shift
      OP="${2:?fmt}"
      ;;
    fmt|clippy|audit|check)
      OP="$1"
      ;;
    *)
      echo -e "${RED}Error: Unknown operation: $1${NC}"
      show_help
      exit 1
      ;;
  esac
  shift
done

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
    if [[ -n "$WORKSPACE_FLAG" && -n "$PACKAGE_FLAG" ]]; then
      echo -e "${YELLOW}Formatting entire workspace...${NC}"
      cargo fmt --all
    elif [[ -n "$WORKSPACE_FLAG" && -n "$PACKAGE_FLAG" ]]; then
      pkg="${PACKAGE_FLAG#--package}"
      echo -e "${YELLOW}Formatting package: ${pkg#--package=}${NC}"
      cargo fmt --package "${pkg#--package=}"
    else
      echo -e "${YELLOW}Formatting check only...${NC}"
      cargo fmt --all -- --check
    fi
    
    if [[ $? -eq 0 ]]; then
      echo -e "${GREEN}✅ Formatting check passed${NC}"
    else
      echo -e "${RED}❌ Formatting check failed${NC}"
      echo -e "${YELLOW}Run 'cargo fmt' to fix${NC}"
    fi
    ;;

  clippy)
    echo -e "${BLUE}🔍 Linting with Clippy...${NC}"
    if [[ -n "$WORKSPACE_FLAG" && -n "$PACKAGE_FLAG" ]]; then
      echo -e "${YELLOW}Linting entire workspace...${NC}"
      cargo clippy --workspace -- ${STRICT:-D warnings}
    elif [[ -n "$WORKSPACE_FLAG" && -n "$PACKAGE_FLAG" ]]; then
      pkg="${PACKAGE_FLAG#--package}"
      echo -e "${YELLOW}Linting package: ${pkg#--package=}${NC}"
      cargo clippy --package "${pkg#--package=}" -- ${STRICT:-D warnings}
    else
      echo -e "${YELLOW}Linting current package only...${NC}"
      cargo clippy -- ${STRICT:-D warnings}
    fi
    
    clippy_result=$?
    if [[ $clippy_result -eq 0 ]]; then
      echo -e "${GREEN}✅ No Clippy warnings${NC}"
    else
      echo -e "${YELLOW}⚠️  Clippy found ${clippy_result} warnings${NC}"
    fi
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
    ;;

  check)
    echo -e "${BLUE}🔎 Running quality gates...${NC}"
    
    # Run formatting check
    echo -e "${BLUE}  Formatting check...${NC}"
    cargo fmt --all -- --check
    fmt_result=$?
    
    # Run clippy
    echo -e "${BLUE}  Clippy check (strict mode)...${NC}"
    cargo clippy --all -D warnings
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
      echo -e "  Clippy: ${YELLOW}⚠️  WARNINGS: ${clippy_result}${NC}"
    fi
    
    if [[ $audit_result -eq 0 ]]; then
      echo -e "  Security: ${GREEN}✅ PASS${NC}"
    else
      echo -e "  Security: ${RED}FAIL${NC}"
    fi
    
    echo ""
    echo "Next steps:"
    echo "  • Run 'cargo fmt' to fix formatting"
    echo "  • Run 'cargo clippy --fix' to auto-fix warnings"
    echo "  • Update dependencies with 'cargo update'"
    echo ""
    echo "============================================"
    ;;

  *)
    echo -e "${RED}Error: Unknown operation: $OP${NC}"
    show_help
    exit 1
    ;;
esac
