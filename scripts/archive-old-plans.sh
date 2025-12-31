#!/bin/bash

# Archive Old Plans Script
# Purpose: Automatically move completed plans to archive
# Usage: ./scripts/archive-old-plans.sh [--dry-run]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            echo "Usage: ./scripts/archive-old-plans.sh [--dry-run]"
            exit 1
            ;;
    esac
done

cd /workspaces/feat-phase3/plans

echo -e "${BLUE}=== Archive Old Plans ===${NC}"
if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}DRY RUN MODE - No files will be moved${NC}"
fi
echo ""

# Function to move or show move operation
move_file() {
    local source="$1"
    local dest="$2"
    local dest_dir=$(dirname "$dest")

    if [[ "$DRY_RUN" == "true" ]]; then
        echo -e "${GREEN}[DRY RUN] Would move:${NC} $source → $dest"
    else
        mkdir -p "$dest_dir"
        if mv "$source" "$dest"; then
            echo -e "${GREEN}✓ Moved:${NC} $source → $dest"
        else
            echo -e "${RED}✗ Failed to move:${NC} $source"
            return 1
        fi
    fi
}

# Archive completed GOAP execution plans
echo -e "${BLUE}Archiving completed GOAP execution plans...${NC}"
completed_plans=(
    "GOAP/PHASE1_EXECUTION_PLAN.md"
    "GOAP/PHASE3_ACTION_PLAN.md"
    "GOAP/PHASE4_EXECUTION_PLAN.md"
    "GOAP/PHASE4_GOAP_EXECUTION_PLAN.md"
    "GOAP/PR192_FIX_EXECUTION_PLAN.md"
    "GOAP/PR192_PHASE_1_2_TASKS.md"
    "GOAP/PR192_PHASE_3_5_TASKS.md"
    "GOAP/PR192_QUALITY_GATES.md"
    "GOAP/PR192_RISK_MITIGATION.md"
    "GOAP/PR192_RISK_MITIGATION_PART1.md"
)

for plan in "${completed_plans[@]}"; do
    if [[ -f "$plan" ]]; then
        move_file "$plan" "archive/goap-plans/$(basename $plan)"
    fi
done

# Archive completed status reports
echo ""
echo -e "${BLUE}Archiving superseded status reports...${NC}"
status_reports=(
    "STATUS/IMPLEMENTATION_PHASE1.md"
    "STATUS/IMPLEMENTATION_PHASE2.md"
    "STATUS/PHASE1_CODE_REVIEW_REPORT_2025-12-25.md"
    "STATUS/PHASE1_VALIDATION_REPORT_2025-12-25.md"
    "STATUS/MEMORY_SYSTEM_VERIFICATION_REPORT_2025-12-24.md"
    "STATUS/VALIDATION_LATEST.md"
    "STATUS/V019_STATUS_REPORT.md"
)

for report in "${status_reports[@]}"; do
    if [[ -f "$report" ]]; then
        move_file "$report" "archive/completed/$(basename $report)"
    fi
done

# Archive research phase plans
echo ""
echo -e "${BLUE}Archiving completed research integration plans...${NC}"
research_plans=(
    "research/PHASE1_INTEGRATION_PLAN.md"
    "research/PHASE2_INTEGRATION_PLAN.md"
    "research/PHASE3_INTEGRATION_PLAN.md"
    "research/PHASE3_COMPLETION_REPORT.md"
    "research/DIVERSITY_MAXIMIZER_IMPLEMENTATION_SUMMARY.md"
)

for plan in "${research_plans[@]}"; do
    if [[ -f "$plan" ]]; then
        move_file "$plan" "archive/research/$(basename $plan)"
    fi
done

# Archive one-time reports
echo ""
echo -e "${BLUE}Archiving one-time audit and analysis reports...${NC}"
one_time_reports=(
    "GAP_ANALYSIS_REPORT_2025-12-29.md"
    "IMPLEMENTATION_PRIORITY_PLAN_2025-12-29.md"
    "QUICK_WINS_IMPLEMENTATION_2025-12-29.md"
)

for report in "${one_time_reports[@]}"; do
    if [[ -f "$report" ]]; then
        move_file "$report" "archive/temporary/$(basename $report)"
    fi
done

# Archive Turso AI phase plans
echo ""
echo -e "${BLUE}Archiving Turso AI phase plans...${NC}"
turso_plans=(
    "GOAP/TURSO_AI_CONCRETE_RECOMMENDATIONS.md"
    "GOAP/TURSO_AI_EMBEDDINGS_ANALYSIS_AND_PLAN.md"
    "GOAP/TURSO_AI_EXTENSION_COMPATIBILITY_MATRIX.md"
    "GOAP/TURSO_AI_PERFORMANCE_TEST_FRAMEWORK.md"
    "GOAP/TURSO_AI_PHASE0_HANDOFF_TESTING_QA.md"
    "GOAP/TURSO_AI_PHASE0_PROGRESS_TRACKING.md"
    "GOAP/TURSO_AI_PHASE0_TESTING_QA_SUMMARY.md"
    "GOAP/TURSO_AI_PHASE0_TEST_PLAN.md"
    "GOAP/turso_vector_benchmark_optimization_plan.md"
    "GOAP/MIGRATION_SCRIPT_DESIGN.md"
    "GOAP/MULTI_DIMENSION_ROUTING_PROTOTYPE.rs"
    "GOAP/MULTI_DIMENSION_SCHEMA_DESIGN.md"
    "GOAP/PLANS_ANALYSIS_EXECUTION_2025-12-29.md"
)

for plan in "${turso_plans[@]}"; do
    if [[ -f "$plan" ]]; then
        move_file "$plan" "archive/goap-plans/2025-12-turso-ai/$(basename $plan)"
    fi
done

# Archive completed GOAP agent documentation
echo ""
echo -e "${BLUE}Archiving completed GOAP agent documentation...${NC}"
goap_docs=(
    "GOAP/GOAP_AGENT_CODEBASE_VERIFICATION.md"
    "GOAP/GOAP_AGENT_EXECUTION_TEMPLATE.md"
    "GOAP/GOAP_AGENT_IMPROVEMENT_PLAN.md"
    "GOAP/GOAP_AGENT_QUALITY_GATES.md"
    "GOAP/GOAP_AGENT_ROADMAP.md"
)

for doc in "${goap_docs[@]}"; do
    if [[ -f "$doc" ]]; then
        move_file "$doc" "archive/goap-plans/agent-docs/$(basename $doc)"
    fi
done

# Summary
echo ""
echo -e "${BLUE}=== Archive Summary ===${NC}"
echo "Completed archiving old plans"
if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}DRY RUN - No files were actually moved${NC}"
else
    echo "Files have been moved to archive/"
    echo ""
    echo "Next steps:"
    echo "1. Update archive/ARCHIVE_INDEX.md"
    echo "2. Verify all links still work"
    echo "3. Run ./scripts/update-archive-index.sh"
fi
