#!/usr/bin/env bash
# run-evals.sh — Discover and run skill evaluations
# Usage: ./scripts/run-evals.sh [--verbose] [skill_name]

set -e

VERBOSE=false
SKILL_FILTER=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --verbose|-v)
      VERBOSE=true
      shift
      ;;
    -*)
      echo "Unknown option: $1"
      exit 1
      ;;
    *)
      SKILL_FILTER="$1"
      shift
      ;;
  esac
done

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKILL_DIR="$PROJECT_ROOT/.agents/skills"

find_evals() {
  if [[ -n "$1" ]]; then
    find "$SKILL_DIR/$1" -name "evals.json" 2>/dev/null
  else
    find "$SKILL_DIR" -name "evals.json"
  fi
}

run_eval() {
  local eval_file="$1"
  local skill_path
  skill_path=$(dirname "$(dirname "$eval_file")")
  local skill_name
  skill_name=$(basename "$skill_path")

  echo -e "${BLUE}Running evals for skill: ${YELLOW}$skill_name${NC}"

  # Basic validation of evals.json structure
  if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is required to parse evals.json${NC}"
    exit 1
  fi

  local test_count
  test_count=$(jq '.tests | length' "$eval_file")
  echo -e "Found ${YELLOW}$test_count${NC} tests"

  # Execute each test
  # For now, we assume tests have an 'exec' field with a bash command
  local failed=0
  for (( i=0; i<test_count; i++ )); do
    local name
    name=$(jq -r ".tests[$i].name" "$eval_file")
    local cmd
    cmd=$(jq -r ".tests[$i].exec" "$eval_file")

    echo -ne "  - $name ... "

    # Run the command relative to skill directory or project root?
    # Usually eval commands should be runnable from project root.
    local output
    if [ "$VERBOSE" = true ]; then
      if eval "$cmd"; then
        echo -e "${GREEN}PASS${NC}"
      else
        echo -e "${RED}FAIL${NC}"
        echo -e "    Command: $cmd"
        failed=$((failed + 1))
      fi
    else
      output=$(eval "$cmd" 2>&1) || {
        echo -e "${RED}FAIL${NC}"
        echo -e "    Command: $cmd"
        echo -e "    Output:\n$output"
        failed=$((failed + 1))
        continue
      }
      echo -e "${GREEN}PASS${NC}"
    fi
  done

  return $failed
}

EVAL_FILES=$(find_evals "$SKILL_FILTER")

if [[ -z "$EVAL_FILES" ]]; then
  echo -e "${YELLOW}No evals.json found.${NC}"
  exit 0
fi

TOTAL_FAILED=0
for f in $EVAL_FILES; do
  run_eval "$f" || TOTAL_FAILED=$((TOTAL_FAILED + 1))
done

if [[ $TOTAL_FAILED -gt 0 ]]; then
  echo -e "${RED}Evaluations failed for $TOTAL_FAILED skill(s).${NC}"
  exit 1
else
  echo -e "${GREEN}All skill evaluations passed!${NC}"
  exit 0
fi
