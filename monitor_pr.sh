#!/bin/bash

for i in $(seq 1 60); do
  echo "Iteration $i"
  output=$(gh pr checks 103 2>&1)
  echo "$output"
  
  # Check for failure
  if echo "$output" | grep -q -i "fail"; then
    echo "Checks failed:"
    echo "$output" | grep -i "fail"
    exit 1
  fi
  
  # Check for success: no pending/running/queued/in_progress, and all others are pass
  if echo "$output" | grep -q -E "(pending|running|queued|in_progress)"; then
    echo "Some checks are still running or queued"
  else
    # Check if any line has status other than pass (excluding headers)
    if echo "$output" | grep -v "pass" | grep -v "^NAME" | grep -v "^--" | grep -v "^$" | grep -v "Iteration" | grep -v "Some checks" | grep -v "All checks" | grep -v "Checks failed" > /dev/null; then
      echo "Some checks have unknown or failed status"
    else
      echo "All checks passed successfully"
      exit 0
    fi
  fi
  
  if [ $i -eq 60 ]; then
    echo "Monitoring timed out after 60 iterations"
    exit 2
  fi
  
  sleep 10
done