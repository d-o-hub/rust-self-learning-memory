#!/bin/bash
set -e

echo "Monitoring PR #253 until all checks pass..."

MAX_ATTEMPTS=60
ATTEMPT=0

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    echo "================================================================"
    echo "Check #$(($ATTEMPT + 1)) at $(date '+%Y-%m-%d %H:%M:%S')"
    echo "================================================================"
    
    # Get PR check status
    OUTPUT=$(gh pr checks 253 2>&1)
    
    # Count passing, failing, and pending checks
    PASSING=$(echo "$OUTPUT" | grep -c "pass" || echo 0)
    FAILING=$(echo "$OUTPUT" | grep -c "fail" || echo 0)
    PENDING=$(echo "$OUTPUT" | grep -c "pending" || echo 0)
    TOTAL=$((PASSING + FAILING + PENDING))
    
    echo "Passing: $PASSING, Failing: $FAILING, Pending: $PENDING, Total: $TOTAL"
    
    # Show status of key checks
    echo ""
    echo "Key Checks Status:"
    echo "$OUTPUT" | grep -E "(Essential|Quick PR|File Structure|Supply Chain|MCP Build|Tests|Clippy)" | head -15 || true
    
    # Check if all checks are complete
    if [ $PENDING -eq 0 ]; then
        if [ $FAILING -eq 0 ]; then
            echo ""
            echo "================================================================"
            echo "✅ ALL CHECKS PASSED!"
            echo "================================================================"
            exit 0
        fi
    fi
    
    echo ""
    echo "Waiting 30 seconds before next check..."
    echo ""
    sleep 30
    
    ATTEMPT=$((ATTEMPT + 1))
done

echo ""
echo "================================================================"
echo "⚠️  MAX ATTEMPTS REACHED ($MAX_ATTEMPTS)"
echo "================================================================"
echo ""
echo "Final Status:"
echo "Passing: $PASSING, Failing: $FAILING, Pending: $PENDING"
exit 1
