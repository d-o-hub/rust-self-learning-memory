#!/bin/bash
# Quality Gates Runner
#
# Run quality gates locally before committing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Quality Gates - Local Runner                        ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Configuration
COVERAGE_THRESHOLD=${QUALITY_GATE_COVERAGE_THRESHOLD:-90}
PATTERN_THRESHOLD=${QUALITY_GATE_PATTERN_ACCURACY_THRESHOLD:-70}
COMPLEXITY_THRESHOLD=${QUALITY_GATE_COMPLEXITY_THRESHOLD:-10}
SECURITY_THRESHOLD=${QUALITY_GATE_SECURITY_THRESHOLD:-0}
SKIP_OPTIONAL=${QUALITY_GATE_SKIP_OPTIONAL:-false}

echo -e "${BLUE}Configuration:${NC}"
echo "  Coverage Threshold: ${COVERAGE_THRESHOLD}%"
echo "  Pattern Accuracy Threshold: ${PATTERN_THRESHOLD}%"
echo "  Complexity Threshold: ${COMPLEXITY_THRESHOLD}"
echo "  Security Vuln Threshold: ${SECURITY_THRESHOLD}"
echo "  Skip Optional: ${SKIP_OPTIONAL}"
echo ""

# Check required tools
echo -e "${BLUE}Checking required tools...${NC}"
MISSING_TOOLS=()

if ! command -v cargo &> /dev/null; then
    MISSING_TOOLS+=("cargo")
fi

if ! cargo llvm-cov --version &> /dev/null 2>&1; then
    echo -e "  ${YELLOW}⚠${NC}  cargo-llvm-cov not installed"
    if [ "$SKIP_OPTIONAL" != "true" ]; then
        MISSING_TOOLS+=("cargo-llvm-cov")
    fi
fi

if ! cargo audit --version &> /dev/null 2>&1; then
    echo -e "  ${YELLOW}⚠${NC}  cargo-audit not installed"
    if [ "$SKIP_OPTIONAL" != "true" ]; then
        MISSING_TOOLS+=("cargo-audit")
    fi
fi

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo -e "${RED}Missing required tools: ${MISSING_TOOLS[*]}${NC}"
    echo ""
    echo "Install with:"
    for tool in "${MISSING_TOOLS[@]}"; do
        if [ "$tool" = "cargo-llvm-cov" ]; then
            echo "  cargo install cargo-llvm-cov"
        elif [ "$tool" = "cargo-audit" ]; then
            echo "  cargo install cargo-audit"
        fi
    done
    echo ""
    echo "Or set QUALITY_GATE_SKIP_OPTIONAL=true to skip optional tools"
    exit 1
fi

echo -e "${GREEN}✓${NC} All required tools available"
echo ""

# Run quality gates
echo -e "${BLUE}Running quality gates...${NC}"
echo ""

# Optional: Non-blocking GOAP checks (documentation hygiene + feedback loop markers)
run_goap_checks() {
  echo -e "${BLUE}Running optional GOAP checks (non-blocking)...${NC}"
  local failed=0

  # Helper: ensure file exists and under 500 lines
  check_doc() {
    local f="$1"
    if [ -f "$f" ]; then
      local lines
      lines=$(wc -l < "$f" | tr -d ' ')
      if [ "$lines" -gt 500 ]; then
        echo -e "  ${YELLOW}⚠${NC} $f exceeds 500 lines ($lines)"
      else
        echo -e "  ${GREEN}✓${NC} $f within 500 lines ($lines)"
      fi
    else
      echo -e "  ${YELLOW}⚠${NC} $f not found"
    fi
  }

  # Check canonical GOAP docs
  check_doc plans/GOAP_AGENT_IMPROVEMENT_PLAN.md
  check_doc plans/GOAP_AGENT_QUALITY_GATES.md
  check_doc plans/GOAP_AGENT_EXECUTION_TEMPLATE.md
  check_doc plans/GOAP_AGENT_ROADMAP.md
  check_doc plans/GOAP_AGENT_CODEBASE_VERIFICATION.md

  # Light template completeness checks (headings exist) for current execution plans
  for f in plans/GOAP_EXECUTION_PLAN_*.md; do
    [ -e "$f" ] || continue
    local ok=1
    grep -q "^## Objective" "$f" || ok=0
    grep -q "^## Validation" "$f" || grep -q "^## Validation Plan" "$f" || ok=0
    grep -q "^## Risks" "$f" || grep -q "^## Risks & Mitigations" "$f" || ok=0
    grep -q "^## Rollback" "$f" || ok=0
    if [ $ok -eq 1 ]; then
      echo -e "  ${GREEN}✓${NC} Template sections present in $f"
    else
      echo -e "  ${YELLOW}⚠${NC} Consider aligning $f with GOAP template sections"
    fi
    check_doc "$f"
  done

  # Check for non-permanent documentation files in root directory
  echo -e "${BLUE}Checking for non-permanent documentation in root...${NC}"
  local root_docs=()
  for f in *.md *.txt *.rst; do
    [ -e "$f" ] || continue
    case "$f" in
      README.md|CHANGELOG.md|CONTRIBUTING.md|SECURITY.md|DEPLOYMENT.md|TESTING.md|AGENTS.md|CLAUDE.md|GEMINI.md|ADVANCED_PATTERNS_QUICK_START.md)
        # These are permanent project documentation files allowed in root
        ;;
      *)
        root_docs+=("$f")
        ;;
    esac
  done
  
  if [ ${#root_docs[@]} -gt 0 ]; then
    echo -e "  ${YELLOW}⚠${NC} Non-permanent documentation files found in root:"
    for f in "${root_docs[@]}"; do
      echo -e "    - $f (should be in plans/)"
    done
    failed=1
  else
    echo -e "  ${GREEN}✓${NC} No non-permanent documentation files in root"
  fi

  # Optional memory-mcp feedback loop markers (informational only)
  echo -e "${BLUE}memory-mcp feedback loop markers:${NC}"
  echo "  - health_check: ensure MCP server is reachable during plan execution"
  echo "  - get_metrics: capture a short snapshot in plan notes when applicable"
  echo "  - advanced_pattern_analysis/analyze_patterns: run when data is available"

  echo -e "${GREEN}GOAP checks complete (non-blocking).${NC}"

  # Dependency metrics tracking (ADR-036 Tier 5)
  echo ""
  echo -e "${BLUE}📊 Dependency metrics (ADR-036 Tier 5):${NC}"
  if command -v cargo &> /dev/null; then
    # Count duplicate dependency roots
    local dupes
    dupes=$(cargo tree -d 2>/dev/null | grep -cE "^[a-z]" || echo "0")
    echo "  Duplicate dependency roots: $dupes"

    # Count total packages (workspace members + all dependencies)
    local total
    if command -v jq &> /dev/null; then
      total=$(cargo metadata --format-version=1 2>/dev/null | jq '.packages | length' 2>/dev/null || echo "0")
    else
      # Fallback: count unique package names in tree output
      total=$(cargo tree --workspace 2>/dev/null | wc -l)
    fi
    echo "  Total dependency packages: $total"

    # Warning if duplicates increasing
    if [ "$dupes" -gt 130 ]; then
      echo -e "  ${YELLOW}⚠${NC}  WARNING: Duplicate dependencies increasing (target: <100, current: $dupes, alert: >130)"
      echo "      Run 'cargo tree -d' to see duplicates"
      # Non-blocking - just warn
    fi

    # Success criteria check
    if [ "$dupes" -lt 100 ]; then
      echo -e "  ${GREEN}✓${NC} Dependency deduplication target met (<100 duplicates)"
    else
      echo -e "  ${YELLOW}📝${NC} Dependency deduplication in progress (current: $dupes, target: <100)"
    fi
  else
    echo "  cargo not available - skipping dependency metrics"
  fi
}

# Blocking: enforce source file size limit (AGENTS.md invariant)
run_source_file_size_gate() {
  echo -e "${BLUE}Running source file size gate (<=500 LOC, source files only)...${NC}"

  local limit=500
  local oversized_source=()
  local oversized_tests=()
  local file
  local lines

  while IFS= read -r file; do
    [ -n "$file" ] || continue

    case "$file" in
      benches/*|target/*|.git/*)
        continue
        ;;
    esac

    [ -f "$file" ] || continue

    lines=$(wc -l < "$file" | tr -d ' ')
    if [ "$lines" -gt "$limit" ]; then
      case "$file" in
        tests/*|*/tests/*|*_test.rs|*_tests.rs)
          oversized_tests+=("$file:$lines")
          ;;
        *)
          oversized_source+=("$file:$lines")
          ;;
      esac
    fi
  done < <(
    {
      git ls-files '*.rs'
      git ls-files --others --exclude-standard '*.rs'
    } | awk '!seen[$0]++'
  )

  if [ ${#oversized_source[@]} -gt 0 ]; then
    echo -e "  ${RED}✗${NC} Source file size gate failed: ${#oversized_source[@]} source file(s) exceed ${limit} LOC"
    for entry in "${oversized_source[@]}"; do
      echo "    - $entry"
    done
    echo ""
    echo "  Split oversized files to restore <=${limit} LOC compliance."
    if [ ${#oversized_tests[@]} -gt 0 ]; then
      echo "  Note: ${#oversized_tests[@]} oversized test file(s) detected (non-blocking)."
    fi
    return 1
  fi

  if [ ${#oversized_tests[@]} -gt 0 ]; then
    echo -e "  ${YELLOW}⚠${NC} Oversized test files detected (non-blocking): ${#oversized_tests[@]}"
    for entry in "${oversized_tests[@]}"; do
      echo "    - $entry"
    done
    echo ""
  fi

  echo -e "  ${GREEN}✓${NC} Source file size gate passed (all Rust source files <=${limit} LOC)"
  echo ""
  return 0
}

export QUALITY_GATE_COVERAGE_THRESHOLD=$COVERAGE_THRESHOLD
export QUALITY_GATE_PATTERN_ACCURACY_THRESHOLD=$PATTERN_THRESHOLD
export QUALITY_GATE_COMPLEXITY_THRESHOLD=$COMPLEXITY_THRESHOLD
export QUALITY_GATE_SECURITY_THRESHOLD=$SECURITY_THRESHOLD
export QUALITY_GATE_SKIP_OPTIONAL=$SKIP_OPTIONAL

# Execute GOAP checks unless disabled
if [ "${QUALITY_GATE_SKIP_GOAP:-false}" != "true" ]; then
  run_goap_checks || true
else
  echo -e "${YELLOW}Skipping GOAP checks (${NC}QUALITY_GATE_SKIP_GOAP=true${YELLOW}).${NC}"
fi

# Blocking source file-size compliance gate
if ! run_source_file_size_gate; then
    echo ""
    echo -e "${RED}────────────────────────────────────────────────────────────────────────${NC}"
    echo -e "${RED}│          ✗ Quality Gates FAILED                               │${NC}"
    echo -e "${RED}────────────────────────────────────────────────────────────────────────${NC}"
    echo ""
    echo "Review the output above to identify which gates failed."
    echo "See docs/QUALITY_GATES.md for troubleshooting guidance."
    exit 1
fi

echo ""
if RUSTFLAGS="-D warnings" cargo test --test quality_gates -- --nocapture; then
    echo ""
    echo -e "${GREEN}────────────────────────────────────────────────────────────────────────${NC}"
    echo -e "${GREEN}│          ✓ All Quality Gates PASSED                          │${NC}"
    echo -e "${GREEN}────────────────────────────────────────────────────────────────────────${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}────────────────────────────────────────────────────────────────────────${NC}"
    echo -e "${RED}│          ✗ Quality Gates FAILED                               │${NC}"
    echo -e "${RED}────────────────────────────────────────────────────────────────────────${NC}"
    echo ""
    echo "Review the output above to identify which gates failed."
    echo "See docs/QUALITY_GATES.md for troubleshooting guidance."
    exit 1
fi
