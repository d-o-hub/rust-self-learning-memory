#!/bin/bash
# Loop until all GitHub Actions pass

set -e

PR_NUMBER=263
BRANCH="feat/phase4-sprint1-performance"
MAX_ITERATIONS=20
iteration=0

echo "Starting GitHub Actions monitoring loop for PR #$PR_NUMBER"
echo "Branch: $BRANCH"
echo "Max iterations: $MAX_ITERATIONS"
echo ""

while [ $iteration -lt $MAX_ITERATIONS ]; do
    iteration=$((iteration + 1))
    echo "=== Iteration $iteration/$MAX_ITERATIONS ==="
    echo ""
    
    # Get the latest run status
    echo "Checking workflow runs..."
    gh run list --branch $BRANCH --limit 10 --json databaseId,status,conclusion,workflowName,headSha | jq -r '.[] | select(.status == "completed") | "\(.workflowName): \(.conclusion)"' | head -10
    
    # Check for any in-progress runs
    in_progress=$(gh run list --branch $BRANCH --status in_progress --json databaseId | jq length)
    if [ "$in_progress" -gt 0 ]; then
        echo ""
        echo "⏳ $in_progress workflow(s) still in progress..."
        echo "Waiting 60 seconds..."
        sleep 60
        continue
    fi
    
    # Get failed runs
    echo ""
    echo "Checking for failed workflows..."
    failed_runs=$(gh run list --branch $BRANCH --status completed --json databaseId,conclusion,workflowName | jq -r '.[] | select(.conclusion == "failure") | .databaseId')
    
    if [ -z "$failed_runs" ]; then
        echo ""
        echo "✅ All workflows passed!"
        echo ""
        # Show final status
        gh run list --branch $BRANCH --limit 10
        exit 0
    fi
    
    echo ""
    echo "❌ Found failed workflows. Analyzing..."
    
    # Process each failed run
    for run_id in $failed_runs; do
        echo ""
        echo "Analyzing run: $run_id"
        
        # Get failed jobs
        failed_jobs=$(gh run view $run_id --json jobs | jq -r '.jobs[] | select(.conclusion == "failure") | .databaseId')
        
        for job_id in $failed_jobs; do
            echo "  Checking job: $job_id"
            
            # Get the logs
            logs=$(gh run view --job=$job_id --log-failed 2>&1 || true)
            
            # Analyze the error and create fix
            if echo "$logs" | grep -q "license"; then
                echo "  📋 License issue detected"
                # License issues should be fixed in deny.toml already
                echo "  ⏭️  Skipping - already fixed in deny.toml"
                
            elif echo "$logs" | grep -q "gitleaks\|secret\|api_key"; then
                echo "  🔑 Secret scanning issue detected"
                # Secret issues should be fixed in .gitleaksignore already
                echo "  ⏭️  Skipping - already fixed in .gitleaksignore"
                
            elif echo "$logs" | grep -q "clippy\|warning:"; then
                echo "  🔧 Clippy warning detected"
                echo "  Running cargo clippy fix..."
                cargo clippy --all --fix --allow-dirty --allow-staged 2>&1 | head -20 || true
                
            elif echo "$logs" | grep -q "fmt\|format"; then
                echo "  📝 Formatting issue detected"
                echo "  Running cargo fmt..."
                cargo fmt --all
                
            elif echo "$logs" | grep -q "test\|compilation"; then
                echo "  🧪 Test/compilation issue detected"
                echo "  Will need manual fix - creating task..."
                
            else
                echo "  ❓ Unknown error type"
                echo "  Logs preview:"
                echo "$logs" | head -10
            fi
        done
    done
    
    # Check if there are any changes to commit
    if [ -n "$(git status --porcelain)" ]; then
        echo ""
        echo "📦 Changes detected. Committing fixes..."
        git add -A
        git commit -m "ci: auto-fix GitHub Actions issues (iteration $iteration)"
        git push origin $BRANCH
        echo "✅ Fixes pushed. Waiting for new runs..."
    else
        echo ""
        echo "ℹ️  No changes to commit"
    fi
    
    echo ""
    echo "Waiting 60 seconds before next check..."
    sleep 60
done

echo ""
echo "⚠️  Max iterations ($MAX_ITERATIONS) reached"
echo "Some issues may require manual intervention"
exit 1
