#!/usr/bin/env bash
# harness-check.sh — Agent-optimized sensor wrapper
# Usage: ./scripts/harness-check.sh <sensor>

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SENSOR="${1:-}"

show_help() {
  echo "Usage: $(basename "$0") <sensor>"
  echo ""
  echo "Available Sensors:"
  echo "  fmt              Check code formatting"
  echo "  clippy           Run lints (strict)"
  echo "  build            Check if workspace compiles"
  echo "  test             Run all unit/integration tests"
  echo "  doc              Check doctests and doc integrity"
  echo "  docs-integrity   Verify cross-links and metadata"
  echo "  quality          Run coverage and LOC quality gates"
  echo "  release          Verify release state (version, changelog)"
  echo "  ignored-tests    Check ignored-test ceiling"
  echo "  memory           Run memory learning evaluation"
}

if [[ -z "$SENSOR" ]]; then
  show_help
  exit 1
fi

case "$SENSOR" in
  fmt)
    echo -e "${BLUE}Running fmt sensor...${NC}"
    if ! ./scripts/code-quality.sh fmt; then
      echo -e "${RED}FAILURE HINT:${NC} Run './scripts/code-quality.sh fix' or 'cargo fmt --all' to fix formatting."
      exit 1
    fi
    ;;

  clippy)
    echo -e "${BLUE}Running clippy sensor...${NC}"
    if ! ./scripts/code-quality.sh clippy --workspace; then
      echo -e "${RED}FAILURE HINT:${NC} Fix the reported warnings. Zero warnings policy is enforced. Try './scripts/code-quality.sh clippy --fix' for auto-fixable issues."
      exit 1
    fi
    ;;

  build)
    echo -e "${BLUE}Running build sensor...${NC}"
    if ! ./scripts/build-rust.sh check; then
      echo -e "${RED}FAILURE HINT:${NC} The code does not compile. Check the error messages above and fix syntax/type errors."
      exit 1
    fi
    ;;

  test)
    echo -e "${BLUE}Running test sensor...${NC}"
    if command -v cargo-nextest &> /dev/null; then
      if ! cargo nextest run --all; then
        echo -e "${RED}FAILURE HINT:${NC} One or more tests failed. Run with '--nocapture' for details."
        exit 1
      fi
    else
      if ! cargo test --workspace; then
        echo -e "${RED}FAILURE HINT:${NC} One or more tests failed. Use 'cargo test --workspace -- --nocapture' to see detailed output or target a specific test with '-p <crate> --test <name>'."
        exit 1
      fi
    fi
    ;;

  doc)
    echo -e "${BLUE}Running doc sensor...${NC}"
    if ! ./scripts/check-doctests.sh; then
      echo -e "${RED}FAILURE HINT:${NC} Doctests failed or documentation has warnings. Ensure all URLs are wrapped in <angle brackets> and examples are valid."
      exit 1
    fi
    ;;

  docs-integrity)
    echo -e "${BLUE}Running docs-integrity sensor...${NC}"
    if ! ./scripts/check-docs-integrity.sh; then
      echo -e "${RED}FAILURE HINT:${NC} Documentation metadata or cross-links are broken. Check YAML frontmatter and file references."
      exit 1
    fi
    ;;

  quality)
    echo -e "${BLUE}Running quality sensor...${NC}"
    if ! ./scripts/quality-gates.sh; then
      echo -e "${RED}FAILURE HINT:${NC} Quality gates failed. This usually means coverage is below the threshold or a file exceeds 500 LOC. Review the logs for specifics."
      exit 1
    fi
    ;;

  release)
    echo -e "${BLUE}Running release sensor...${NC}"
    if ! ./scripts/verify-release-state.sh; then
      echo -e "${RED}FAILURE HINT:${NC} Release state is inconsistent. Ensure VERSION matches Cargo.toml and CHANGELOG.md is updated."
      exit 1
    fi
    ;;

  ignored-tests)
    echo -e "${BLUE}Running ignored-tests sensor...${NC}"
    if ! ./scripts/check-ignored-tests.sh; then
      echo -e "${RED}FAILURE HINT:${NC} Too many tests are ignored. Fix and enable ignored tests or document why they must remain ignored in ADR-027."
      exit 1
    fi
    ;;

  memory)
    echo -e "${BLUE}Running memory sensor...${NC}"
    if ! .agents/skills/memory-harness/evaluate-learning.sh; then
      echo -e "${RED}FAILURE HINT:${NC} Memory system failed to demonstrate learning. Ensure you have enough traces recorded in '.memory-traces/' (minimum 3)."
      exit 1
    fi
    ;;

  *)
    echo -e "${RED}Unknown sensor: $SENSOR${NC}"
    show_help
    exit 1
    ;;
esac

echo -e "${GREEN}Sensor '$SENSOR' PASSED${NC}"
