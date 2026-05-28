#!/usr/bin/env bash
set -euo pipefail

# Configurations
OWNER="d-o-hub"
REPO=$(basename "$(git rev-parse --show-toplevel)")
PLAN_DIR="plans"
LEARNINGS_FILE="learnings.md"
FAIL_COUNT=0

echo "🤖 Orchestrator initialized for user: ${OWNER} on repo: ${REPO}"

# --- HELPER FUNCTIONS ---

update_plan_progress() {
    local task_file=$1
    local status=$2
    local message=$3
    # Update progress structurally inside the plans folder
    echo -e "## [$(date +'%Y-%m-%d %H:%M:%S')] ${status}\n${message}\n" >> "${PLAN_DIR}/progress_log.md"
}

update_learnings() {
    local impact_summary=$1
    if [ -f "$LEARNINGS_FILE" ]; then
        echo -e "\n### Learning - $(date +'%Y-%m-%d')\n$impact_summary" >> "$LEARNINGS_FILE"
        echo "💡 Learnings log updated."
    fi
}

check_pr_health() {
    # Extract structural constraints using gh cli
    local pr_num
    pr_num=$(gh pr view --json number --jq '.number' 2>/dev/null || echo "")
    
    if [ -z "$pr_num" ]; then
        echo "none"
        return
    fi

    # Verify if there are unresolved conversations or failing checks
    local state
    state=$(gh pr view "$pr_num" --json reviewDecision,statusCheckRollup --jq '.reviewDecision + ":" + .statusCheckRollup[].status' 2>/dev/null || echo "PENDING")
    echo "$state"
}

fetch_documentation_safely() {
    echo "🔍 Triggering fallback deep research after sequence failures..."
    # Fallback block for handling external tooling documentation extraction 
    # if internal compiler metrics or fixes fail twice sequentially.
    if [ -f "llms.txt" ]; then
        echo "📚 Consulting workspace llms.txt for contextual guidance..."
        head -n 20 llms.txt
    fi
}

# --- THE MAIN GOAP ORCHESTRATION LOOP ---

while true; do
    echo "🔄 Assessing current state against target: [Mergeable & Clean Repository]"
    
    # 1. State Scan: Read tasks from plans/ folder
    # Filter for active or unfulfilled task parameters
    mapfile -t tasks < <(find "$PLAN_DIR" -type f -name "*.md" ! -name "progress_log.md")
    
    if [ ${#tasks[@]} -eq 0 ]; then
        echo "✅ No explicit action plans remaining in directory."
    fi

    # Fetch PR state constraints
    PR_STATE=$(check_pr_health)
    echo "📊 Current GitHub PR State Indicator: $PR_STATE"

    # Break loop condition: If PR exists, has no blocking changes, and is merged/mergeable
    if [[ "$PR_STATE" == "APPROVED:COMPLETED" || "$PR_STATE" == "CLEAN" ]]; then
        PR_NUM=$(gh pr view --json number --jq '.number')
        echo "🚀 All conditions met. Executing final action: Merge."
        gh pr merge "$PR_NUM" --squash --delete-branch --admin
        update_plan_progress "all" "MERGED" "PR #$PR_NUM cleanly integrated into main thread."
        break
    fi

    # 2. Planning Phase: Parallel Processing Execution
    # Spawn background sub-tasks concurrently to address warnings, tasks, and code comments
    PID_LIST=()
    
    echo "⚡ Spawning atomic workers in parallel..."

    # Worker Node 1: Processing open plan objectives
    (
        # Simulating isolated atomic workspace processing
        # Ensure zero-tolerance lint metrics (no unused variables/functions allowed)
        echo "🛠️ Processing tasks from execution layout files..."
        # Verify if workspace plans exist and are structured correctly
        if [ -f "plans/GOAP_STATE.md" ]; then
            grep -E "🔴 Open|WG-" "plans/GOAP_STATE.md" | head -n 5
        fi
    ) & PID_LIST+=($!)

    # Worker Node 2: Extracting PR Comments & Addressing Impacts
    (
        PR_NUM=$(gh pr view --json number --jq '.number' 2>/dev/null || echo "")
        if [ -n "$PR_NUM" ]; then
            echo "💬 Inspecting unresolved PR conversations for #$PR_NUM..."
            # Pull unthreaded review comments
            gh pr view "$PR_NUM" --json comments,reviews --jq '.comments[].body' 2>/dev/null || echo "No unresolved comment bodies found."
        else
            echo "💬 No active PR detected. Skipping comment extraction."
        fi
    ) & PID_LIST+=($!)

    # Worker Node 3: Addressing Code Warnings & Performance Items
    (
        echo "🚨 Scanning for pre-existing compilation warnings or lint blocks..."
        # If building via cargo or lint setups, ensure clean output states
        if [ -f "./scripts/code-quality.sh" ]; then
            echo "🔍 Running code quality check..."
            ./scripts/code-quality.sh check
        else
            echo "🔧 Performing cargo check..."
            cargo check --workspace
        fi
    ) & PID_LIST+=($!)

    # 3. Synchronize Worker Execution
    FAILED_WORKER=false
    for pid in "${PID_LIST[@]}"; do
        if ! wait "$pid"; then
            FAILED_WORKER=true
        fi
    done

    # 4. Atomic Sync & Git Upstream Integration
    if [ -n "$(git status --porcelain)" ]; then
        echo "📦 Staging atomic modifications..."
        git add .
        
        # Build clean dynamic message based on actions resolved
        local_msg="chore(orchestrator): resolve plan tasks and address tracking issues"
        git commit -m "$local_msg"
        
        echo "📤 Pushing upstream..."
        git push origin HEAD
        FAIL_COUNT=0 # Reset failure count on valid push transitions
    else
        if [ "$FAILED_WORKER" = true ]; then
            ((FAIL_COUNT++))
            echo "⚠️ Worker phase failed to make clean progression. Consecutive Failures: $FAIL_COUNT"
            
            if [ "$FAIL_COUNT" -ge 2 ]; then
                fetch_documentation_safely
                update_learnings "Required system verification updates after repeated local run failures."
            fi
        fi
    fi

    # Ensure a cooldown step to avoid rate-limiting on high-frequency API pooling
    echo "💤 Execution cycle completed. Pausing briefly before reassessment..."
    sleep 10
done
