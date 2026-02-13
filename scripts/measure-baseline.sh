#!/usr/bin/env bash
# measure-baseline.sh - Simple baseline measurement (no jq dependency)

set -euo pipefail

readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
METRICS_DIR="$PROJECT_ROOT/metrics"
BASELINE_FILE="$METRICS_DIR/baseline.txt"

echo -e "${BLUE}🔍 Phase 1: Baseline Performance Measurement${NC}"
echo "============================================================"
echo ""
echo -e "📂 Project Root: $PROJECT_ROOT"

# Create metrics directory
mkdir -p "$METRICS_DIR"

# Count tokens (word count * 0.75 for English)
count_tokens() {
    local file="$1"
    if [[ ! -f "$file" ]]; then
        echo "0"
        return
    fi
    local words=$(wc -w < "$file" 2>/dev/null | awk '{print $1}')
    local tokens=$(echo "$words * 0.75" | bc | cut -d. -f1)
    echo "${tokens:-0}"
}

# Measure agent
AGENT_FILE="$PROJECT_ROOT/.opencode/agent/build-compile.md"
if [[ -f "$AGENT_FILE" ]]; then
    agent_tokens=$(count_tokens "$AGENT_FILE")
    agent_lines=$(wc -l < "$AGENT_FILE")
    echo -e "${BLUE}📄 Agent: build-compile.md${NC}"
    echo "   Lines: $agent_lines"
    echo "   Tokens: $agent_tokens"
fi

# Measure skill
SKILL_FILE="$PROJECT_ROOT/.opencode/skill/build-rust/SKILL.md"
if [[ -f "$SKILL_FILE" ]]; then
    skill_tokens=$(count_tokens "$SKILL_FILE")
    skill_lines=$(wc -l < "$SKILL_FILE")
    echo ""
    echo -e "${BLUE}📄 Skill: build-rust/SKILL.md${NC}"
    echo "   Lines: $skill_lines"
    echo "   Tokens: $skill_tokens"
fi

# Calculate total
total_tokens=$((agent_tokens + skill_tokens))
total_lines=$((agent_lines + skill_lines))

echo ""
echo "============================================================"
echo -e "${GREEN}📊 BASELINE METRICS${NC}"
echo "============================================================"

echo ""
echo -e "${BLUE}📝 Token Count:${NC}"
echo "   Total: $total_tokens tokens"
echo "   Agent: $agent_tokens tokens"
echo "   Skill: $skill_tokens tokens"
echo "   Lines: $total_lines"

# Estimate timing
echo ""
echo -e "${BLUE}⏱️  Build Performance (Estimate):${NC}"
echo "   Check: ~13s (cached)"
echo "   Dev: ~60s (cached)"
echo "   Release: ~300s (from scratch)"

# Save baseline
cat > "$BASELINE_FILE" <<EOF
BASELINE MEASUREMENT
==================
Timestamp: $(date)
Token Count:
  Total: $total_tokens tokens
  Agent: $agent_tokens tokens
  Skill: $skill_tokens tokens
  Lines: $total_lines
Performance:
  Check: ~13s
  Dev: ~60s
  Release: ~300s
Decision Criteria:
  Token reduction target: $((total_tokens / 2)) tokens (50%)
  Timing improvement target: 20%
  Implementation cost budget: 4 hours
EOF

echo ""
echo "💾 Baseline saved: $BASELINE_FILE"

# Decision
echo ""
echo "============================================================"
echo -e "${GREEN}🎯 OPTIMIZATION DECISION${NC}"
echo "============================================================"
echo ""
echo "Proceed to Phase 2 (Agent+Skill split) IF:"
echo "  ✓ Token reduction > 50% (target: $((total_tokens / 2)))"
echo "  ✓ OR Timing improvement > 20%"
echo "  ✓ AND Implementation cost < 4 hours"
echo ""
echo "Current Status:"
echo "  ✅ Baseline established: $total_tokens tokens"
echo "  📋 Next: Implement optimization"
echo "  🎯 Target: $((total_tokens / 2)) tokens (50% reduction)"

echo ""
echo "============================================================"
if [[ $total_tokens -gt 2000 ]]; then
    echo -e "${GREEN}✅ PROCEED to Phase 2${NC}"
    echo "   Token count ($total_tokens) justifies optimization"
    echo "   Expected savings: $((total_tokens / 2)) tokens"
else
    echo -e "${YELLOW}⚠️  DEFER optimization${NC}"
    echo "   Token count ($total_tokens) may not justify effort"
    echo "   Consider: Developer productivity instead"
fi
echo "============================================================"
