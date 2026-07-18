#!/usr/bin/env bash
# run-evals.sh — Discover, validate, and run skill evaluations (K3.1)
#
# Usage:
#   ./scripts/run-evals.sh [--verbose] [--validate-only] [--strict] [--changed] [skill_name]
#   ./scripts/run-evals.sh --fixtures   # schema rejection fixtures (must exit 1 overall if any fixture misbehaves)
#
# Schema (canonical):
#   { "tests": [ { "name": "...", "description": "...", "exec": "<non-noop bash>" }, ... ] }
#
# Rejected (fail validation):
#   - missing evals.json when skill is requested
#   - top-level key "evals" instead of "tests"
#   - zero tests
#   - missing name/exec on a test
#   - noop exec: true, "true", ":", empty, or whitespace-only

set -euo pipefail

VERBOSE=false
SKILL_FILTER=""
VALIDATE_ONLY=false
STRICT=true
CHANGED_ONLY=false
RUN_FIXTURES=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --verbose|-v)
      VERBOSE=true
      shift
      ;;
    --validate-only)
      VALIDATE_ONLY=true
      shift
      ;;
    --strict)
      STRICT=true
      shift
      ;;
    --no-strict)
      STRICT=false
      shift
      ;;
    --changed)
      CHANGED_ONLY=true
      shift
      ;;
    --fixtures)
      RUN_FIXTURES=true
      shift
      ;;
    -*)
      echo "Unknown option: $1" >&2
      exit 2
      ;;
    *)
      SKILL_FILTER="$1"
      shift
      ;;
  esac
done

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKILL_DIR="$PROJECT_ROOT/.agents/skills"
FIXTURE_DIR="$PROJECT_ROOT/scripts/fixtures/skill-evals"

if ! command -v jq &>/dev/null; then
  echo -e "${RED}Error: jq is required to parse evals.json${NC}" >&2
  exit 1
fi

is_noop_exec() {
  local cmd="$1"
  # trim
  cmd="${cmd#"${cmd%%[![:space:]]*}"}"
  cmd="${cmd%"${cmd##*[![:space:]]}"}"
  case "$cmd" in
    "" | "true" | ":" | "true;" | "/bin/true" | "exit 0")
      return 0
      ;;
  esac
  return 1
}

# Validate one evals.json. Prints problems to stderr. Returns 0 if valid.
validate_eval_file() {
  local eval_file="$1"
  local problems=0

  if [[ ! -f "$eval_file" ]]; then
    echo "missing file: $eval_file" >&2
    return 1
  fi

  if ! jq -e 'type == "object"' "$eval_file" >/dev/null 2>&1; then
    echo "$eval_file: root must be a JSON object" >&2
    return 1
  fi

  # Reject unknown top-level keys that confuse the runner (evals legacy)
  local keys
  keys=$(jq -r 'keys[]' "$eval_file")
  while IFS= read -r key; do
    [[ -z "$key" ]] && continue
    case "$key" in
      tests) ;;
      evals)
        echo "$eval_file: uses legacy top-level key 'evals' (required: 'tests')" >&2
        problems=$((problems + 1))
        ;;
      *)
        # Allow metadata keys only if documented later; for K3.1 keep strict.
        if [[ "$STRICT" == true ]]; then
          echo "$eval_file: unknown top-level key '$key' (only 'tests' allowed)" >&2
          problems=$((problems + 1))
        fi
        ;;
    esac
  done <<<"$keys"

  if ! jq -e 'has("tests") and (.tests | type == "array")' "$eval_file" >/dev/null 2>&1; then
    echo "$eval_file: missing required array field 'tests'" >&2
    return 1
  fi

  local test_count
  test_count=$(jq '.tests | length' "$eval_file")
  if [[ "$test_count" -eq 0 ]]; then
    echo "$eval_file: zero tests" >&2
    problems=$((problems + 1))
  fi

  local i
  for ((i = 0; i < test_count; i++)); do
    local name exec_cmd
    name=$(jq -r ".tests[$i].name // empty" "$eval_file")
    exec_cmd=$(jq -r ".tests[$i].exec // empty" "$eval_file")
    if [[ -z "$name" ]]; then
      echo "$eval_file: tests[$i] missing name" >&2
      problems=$((problems + 1))
    fi
    if [[ -z "$exec_cmd" || "$exec_cmd" == "null" ]]; then
      echo "$eval_file: tests[$i] missing exec" >&2
      problems=$((problems + 1))
    elif is_noop_exec "$exec_cmd"; then
      echo "$eval_file: tests[$i] ($name) has noop exec ('$exec_cmd')" >&2
      problems=$((problems + 1))
    fi
  done

  return "$problems"
}

find_evals() {
  if [[ -n "${1:-}" ]]; then
    find "$SKILL_DIR/$1" -name "evals.json" 2>/dev/null || true
  else
    find "$SKILL_DIR" -name "evals.json" 2>/dev/null || true
  fi
}

find_changed_skills() {
  # Skills with any change under .agents/skills/<name>/ vs origin/main (or HEAD~1).
  # Only directories that contain SKILL.md count as skills.
  # Root-level files under .agents/skills/ (e.g. skill-rules.json routing config)
  # are NOT skills and must never be treated as skill names by --changed.
  local base_ref="origin/main"
  if ! git -C "$PROJECT_ROOT" rev-parse --verify "$base_ref" >/dev/null 2>&1; then
    base_ref="HEAD~1"
  fi
  local name
  while IFS= read -r name; do
    [[ -z "$name" ]] && continue
    # Require a real skill directory: .agents/skills/<name>/SKILL.md
    if [[ -f "$SKILL_DIR/$name/SKILL.md" ]]; then
      printf '%s\n' "$name"
    fi
  done < <(
    git -C "$PROJECT_ROOT" diff --name-only "$base_ref"...HEAD -- .agents/skills 2>/dev/null \
      | awk -F/ '/\.agents\/skills\// {print $3}' \
      | sort -u
  )
}

run_eval() {
  local eval_file="$1"
  local skill_path
  skill_path=$(dirname "$(dirname "$eval_file")")
  local skill_name
  skill_name=$(basename "$skill_path")

  echo -e "${BLUE}Running evals for skill: ${YELLOW}$skill_name${NC}"

  if ! validate_eval_file "$eval_file"; then
    echo -e "  ${RED}SCHEMA FAIL${NC}"
    return 1
  fi

  if [[ "$VALIDATE_ONLY" == true ]]; then
    echo -e "  ${GREEN}SCHEMA OK${NC}"
    return 0
  fi

  local test_count failed
  test_count=$(jq '.tests | length' "$eval_file")
  echo -e "Found ${YELLOW}$test_count${NC} tests"
  failed=0

  local i
  for ((i = 0; i < test_count; i++)); do
    local name cmd
    name=$(jq -r ".tests[$i].name" "$eval_file")
    cmd=$(jq -r ".tests[$i].exec" "$eval_file")
    echo -ne "  - $name ... "

    local output rc=0
    if [[ "$VERBOSE" == true ]]; then
      if (cd "$PROJECT_ROOT" && eval "$cmd"); then
        echo -e "${GREEN}PASS${NC}"
      else
        echo -e "${RED}FAIL${NC}"
        echo -e "    Command: $cmd"
        failed=$((failed + 1))
      fi
    else
      set +e
      output=$(cd "$PROJECT_ROOT" && eval "$cmd" 2>&1)
      rc=$?
      set -e
      if [[ $rc -eq 0 ]]; then
        echo -e "${GREEN}PASS${NC}"
      else
        echo -e "${RED}FAIL${NC}"
        echo -e "    Command: $cmd"
        echo -e "    Output:\n$output"
        failed=$((failed + 1))
      fi
    fi
  done

  return "$failed"
}

run_fixtures() {
  echo -e "${BLUE}K3.1 skill-eval schema fixtures${NC}"
  local failures=0
  local f
  for f in "$FIXTURE_DIR"/*.json; do
    [[ -f "$f" ]] || continue
    local base
    base=$(basename "$f")
    # Files named expect-fail-*.json must fail validation
    # Files named expect-pass-*.json must pass
    if [[ "$base" == expect-fail-* ]]; then
      if validate_eval_file "$f" 2>/dev/null; then
        echo -e "  ${RED}FAIL${NC} $base (expected schema rejection)"
        failures=$((failures + 1))
      else
        echo -e "  ${GREEN}PASS${NC} $base (rejected as expected)"
      fi
    elif [[ "$base" == expect-pass-* ]]; then
      if validate_eval_file "$f"; then
        echo -e "  ${GREEN}PASS${NC} $base"
      else
        echo -e "  ${RED}FAIL${NC} $base (expected schema accept)"
        failures=$((failures + 1))
      fi
    else
      echo -e "  ${YELLOW}SKIP${NC} $base (name must start with expect-fail- or expect-pass-)"
    fi
  done
  if [[ $failures -gt 0 ]]; then
    echo -e "${RED}Fixture failures: $failures${NC}"
    return 1
  fi
  echo -e "${GREEN}All schema fixtures passed${NC}"
  return 0
}

if [[ "$RUN_FIXTURES" == true ]]; then
  run_fixtures
  exit $?
fi

EVAL_FILES=""
if [[ "$CHANGED_ONLY" == true ]]; then
  while IFS= read -r skill; do
    [[ -z "$skill" ]] && continue
    local_file="$SKILL_DIR/$skill/evals/evals.json"
    if [[ -f "$local_file" ]]; then
      EVAL_FILES+="$local_file"$'\n'
    else
      echo -e "${RED}Changed skill '$skill' has no evals/evals.json${NC}" >&2
      TOTAL_FAILED_PRE=1
    fi
  done < <(find_changed_skills)
  EVAL_FILES=$(echo "$EVAL_FILES" | sed '/^$/d')
  if [[ -z "$EVAL_FILES" && -z "${TOTAL_FAILED_PRE:-}" ]]; then
    echo -e "${YELLOW}No changed skills with evals under .agents/skills (vs origin/main).${NC}"
    exit 0
  fi
else
  EVAL_FILES=$(find_evals "$SKILL_FILTER")
fi

if [[ -z "$EVAL_FILES" ]]; then
  if [[ -n "$SKILL_FILTER" ]]; then
    echo -e "${RED}No evals.json found for skill: $SKILL_FILTER${NC}" >&2
    exit 1
  fi
  echo -e "${YELLOW}No evals.json found.${NC}"
  exit 1
fi

TOTAL_FAILED=${TOTAL_FAILED_PRE:-0}
while IFS= read -r f; do
  [[ -z "$f" ]] && continue
  if ! run_eval "$f"; then
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
  fi
done <<<"$EVAL_FILES"

if [[ $TOTAL_FAILED -gt 0 ]]; then
  echo -e "${RED}Evaluations failed for $TOTAL_FAILED skill(s).${NC}"
  exit 1
fi

echo -e "${GREEN}All skill evaluations passed!${NC}"
exit 0
