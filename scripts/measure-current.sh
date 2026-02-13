#!/usr/bin/env bash
# measure-current.sh - Baseline performance measurement for build-compile
# Establishes BEFORE metrics to justify optimization decisions

set -euo pipefail

# Colors
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
METRICS_DIR="$PROJECT_ROOT/metrics"
BASELINE_FILE="$METRICS_DIR/baseline.json"

echo -e "${BLUE}🔍 Phase 1: Baseline Performance Measurement${NC}"
echo "============================================================"
echo ""
echo -e "${BLUE}📂 Project Root: $PROJECT_ROOT${NC}"

# Create metrics directory
mkdir -p "$METRICS_DIR"

# Function to count tokens (approximate: token ≈ 0.75 words)
count_tokens() {
    local file="$1"
    if [[ ! -f "$file" ]]; then
        echo "0"
        return
    fi
    # Count words (more accurate than characters)
    local words=$(wc -w < "$file" 2>/dev/null || echo "0")
    # Token estimate: words * 0.75
    echo "$words" | awk '{print int($1 * 0.75)}'
}

# Function to count characters
count_chars() {
    local file="$1"
    if [[ ! -f "$file" ]]; then
        echo "0"
        return
    fi
    wc -c < "$file" 2>/dev/null || echo "0"
}

# Initialize JSON
cat > "$BASELINE_FILE" <<EOF
{
  "timestamp": "",
  "agent_tokens": 0,
  "agent_chars": 0,
  "skill_tokens": 0,
  "skill_chars": 0,
  "total_tokens": 0,
  "total_chars": 0,
  "check_time": 0,
  "dev_time": 0,
  "target_size_mb": 0
}
EOF

# Measure agent file
AGENT_FILE="$PROJECT_ROOT/.opencode/agent/build-compile.md"
echo -e "\n${BLUE}📄 Agent: build-compile.md${NC}"

if [[ -f "$AGENT_FILE" ]]; then
    agent_tokens=$(count_tokens "$AGENT_FILE")
    agent_chars=$(count_chars "$AGENT_FILE")
    agent_lines=$(wc -l < "$AGENT_FILE")
    
    echo "   Lines: $agent_lines"
    echo "   Tokens: $agent_tokens"
    echo "   Characters: $agent_chars"
    echo "   Token/Char: $(awk "BEGIN {printf \"%.3f\", $agent_tokens/$agent_chars}" <<< "")"
    
    # Update JSON
    jq --arg ts "$(date -Iseconds)" \
       --argjson tokens "$agent_tokens" \
       --argjson chars "$agent_chars" \
       '.timestamp = $ts | .agent_tokens = $tokens | .agent_chars = $chars' \
       "$BASELINE_FILE" > "${BASELINE_FILE}.tmp" && mv "${BASELINE_FILE}.tmp" "$BASELINE_FILE"
else
    echo -e "${YELLOW}   ⚠️  Not found${NC}"
fi

# Measure skill file
SKILL_FILE="$PROJECT_ROOT/.opencode/skill/build-rust/SKILL.md"
echo -e "\n${BLUE}📄 Skill: build-rust/SKILL.md${NC}"

if [[ -f "$SKILL_FILE" ]]; then
    skill_tokens=$(count_tokens "$SKILL_FILE")
    skill_chars=$(count_chars "$SKILL_FILE")
    skill_lines=$(wc -l < "$SKILL_FILE")
    
    echo "   Lines: $skill_lines"
    echo "   Tokens: $skill_tokens"
    echo "   Characters: $skill_chars"
    echo "   Token/Char: $(awk "BEGIN {printf \"%.3f\", $skill_tokens/$skill_chars}" <<< "")"
    
    # Update JSON
    jq --argjson tokens "$skill_tokens" \
       --argjson chars "$skill_chars" \
       '.skill_tokens = $tokens | .skill_chars = $chars' \
       "$BASELINE_FILE" > "${BASELINE_FILE}.tmp" && mv "${BASELINE_FILE}.tmp" "$BASELINE_FILE"
else
    echo -e "${YELLOW}   ⚠️  Not found${NC}"
fi

# Calculate totals
total_tokens=$(jq '.total_tokens = .agent_tokens + .skill_tokens' "$BASELINE_FILE")
total_chars=$(jq '.total_chars = .agent_chars + .skill_chars' "$BASELINE_FILE")

jq --argjson tokens "$total_tokens" \
   --argjson chars "$total_chars" \
   '.total_tokens = $tokens | .total_chars = $chars' \
   "$BASELINE_FILE" > "${BASELINE_FILE}.tmp" && mv "${BASELINE_FILE}.tmp" "$BASELINE_FILE"

# Measure build performance
echo ""
echo -e "${BLUE}⏱️  Measuring Build Performance${NC}"

# Check mode (fastest)
echo -n "   Check mode: "
if start_time=$(date +%s.%N 2>/dev/null); then
    if cargo check --all > /dev/null 2>&1; then
        end_time=$(date +%s.%N 2>/dev/null)
        check_time=$(awk "BEGIN {print $end_time - $start_time}" <<< "$start_time $end_time")
        echo -e "${GREEN}✅${NC} ${check_time}s"
        jq --argjson time "$check_time" '.check_time = $time' "$BASELINE_FILE" > "${BASELINE_FILE}.tmp" && mv "${BASELINE_FILE}.tmp" "$BASELINE_FILE"
    else
        echo -e "${YELLOW}❌ Failed${NC}"
        jq '.check_time = -1' "$BASELINE_FILE" > "${BASELINE_FILE}.tmp" && mv "${BASELINE_FILE}.tmp" "$BASELINE_FILE"
    fi
else
    echo -e "${YELLOW}Skipped (date command unavailable)${NC}"
fi

# Target size
echo -n "   Target size: "
if [[ -d "$PROJECT_ROOT/target" ]]; then
    target_size=$(du -sb "$PROJECT_ROOT/target" 2>/dev/null | awk '{print $1}')
    size_mb=$(awk "BEGIN {print $target_size / (1024*1024)}" <<< "$target_size")
    echo "${size_mb} MB"
    jq --argjson size "$size_mb" '.target_size_mb = $size' "$BASELINE_FILE" > "${BASELINE_FILE}.tmp" && mv "${BASELINE_FILE}.tmp" "$BASELINE_FILE"
else
    echo -e "${YELLOW}0 MB (not built yet)${NC}"
fi

# Display summary
echo ""
echo "============================================================"
echo -e "${GREEN}📊 BASELINE METRICS${NC}"
echo "============================================================"

echo -e "\n${BLUE}📝 Token Count:${NC}"
total_tokens=$(jq -r '.total_tokens' "$BASELINE_FILE")
agent_tokens=$(jq -r '.agent_tokens' "$BASELINE_FILE")
skill_tokens=$(jq -r '.skill_tokens' "$BASELINE_FILE")
echo "   Total: $total_tokens tokens"
echo "   Agent: $agent_tokens tokens"
echo "   Skill: $skill_tokens tokens"

echo -e "\n${BLUE}⏱️  Build Performance:${NC}"
check_time=$(jq -r '.check_time' "$BASELINE_FILE")
echo "   Check: ${check_time}s"

echo -e "\n${BLUE}💾 Target Size:${NC}"
target_size=$(jq -r '.target_size_mb' "$BASELINE_FILE")
echo "   ${target_size} MB"

echo ""
echo -e "${BLUE}💾 Results saved: $BASELINE_FILE${NC}"

# Calculate optimization decision criteria
echo ""
echo "============================================================"
echo -e "${GREEN}🎯 OPTIMIZATION DECISION CRITERIA${NC}"
echo "============================================================"

echo ""
echo "Proceed to Phase 2 (Agent+Skill split) IF:"
echo "  ✓ Token reduction > 50%"
echo "  ✓ OR Timing improvement > 20%"
echo "  ✓ AND Implementation cost < 4 hours"

echo ""
echo "Current status:"
echo "  - Baseline established: $total_tokens tokens"
echo "  - Next step: Implement optimization"
echo "  - Target for optimization: $((total_tokens / 2)) tokens (50%)"

echo ""
echo "============================================================"
# Decision
if [[ $total_tokens -gt 2000 ]]; then
    echo -e "${GREEN}✅ PROCEED to Phase 2${NC}"
    echo "   Token count ($total_tokens) justifies optimization"
    echo ""
    echo "   Next: ./scripts/measure-after.sh (after implementation)"
else
    echo -e "${YELLOW}⚠️  DEFER optimization${NC}"
    echo "   Token count ($total_tokens) may not justify effort"
    echo ""
    echo "   Consider: Developer productivity instead"
fi
echo "============================================================"
