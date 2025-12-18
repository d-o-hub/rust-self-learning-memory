# Loop Agent Execution Plan: GitHub Actions Monitoring

## Objective
Monitor GitHub Actions workflows after pushing changes to develop branch and iteratively fix any failures until all workflows pass.

## Loop Configuration
- **Mode**: Criteria-Based (with max iterations)
- **Max Iterations**: 10
- **Min Iterations**: 1 (must check at least once)
- **Success Criteria**:
  - ✅ All GitHub Actions workflows show "success" or "completed" status
  - ✅ No workflow runs show "failure" status
  - ✅ All triggered workflows complete (none stuck in "in_progress")

## Source of Truth
**ONLY** GitHub Actions status from `gh CLI` commands. No assumptions, only actual workflow run data.

## Agent Sequence Per Iteration

### Step 1: Wait & Initial Status Check
```bash
# Wait for workflows to trigger (30 seconds after push)
sleep 30

# Check latest runs on develop branch
gh run list --repo d-o-hub/rust-self-learning-memory --branch develop --limit 20
```

### Step 2: Monitor Until Workflows Complete
```bash
# Get latest commit SHA
COMMIT_SHA=$(git rev-parse HEAD)

# Monitor workflows for this commit
gh run list --repo d-o-hub/rust-self-learning-memory --commit $COMMIT_SHA
```

Wait for all workflows to complete (status != "in_progress"):
- Check every 30 seconds
- Timeout after 30 minutes per iteration
- Display progress updates

### Step 3: Analyze Results
```bash
# Get detailed status of all workflows for the commit
gh run list --repo d-o-hub/rust-self-learning-memory --commit $COMMIT_SHA --json status,conclusion,name,workflowName

# Identify failures
gh run list --repo d-o-hub/rust-self-learning-memory --commit $COMMIT_SHA --json status,conclusion,name --jq '.[] | select(.conclusion == "failure")'
```

Count:
- Total workflows triggered
- Successful workflows
- Failed workflows
- In-progress workflows

### Step 4: Decision Point

**If all workflows passed**:
→ SUCCESS ✅ Exit loop

**If workflows still running**:
→ Continue monitoring (stay in Step 2)

**If workflows failed**:
→ Proceed to Step 5 (Fix)

### Step 5: Fix Failures (Using GOAP Agent)

For each failed workflow:
1. Get failure logs: `gh run view <run-id> --log-failed`
2. Analyze root cause
3. Invoke goap-agent with context:
   - Workflow name
   - Failure logs
   - Files modified in last commit
   - Task: Fix the specific failure

4. Apply fixes from goap-agent
5. Commit and push
6. Return to Step 1 (new iteration)

## Progress Tracking

| Iteration | Workflows Total | Success | Failed | In Progress | Action | Result |
|-----------|----------------|---------|--------|-------------|---------|---------|
| 1         | TBD            | TBD     | TBD    | TBD         | Monitor | TBD     |

## Success Definition
All success criteria met:
- ✅ Total workflows = Success workflows
- ✅ Failed workflows = 0
- ✅ In Progress workflows = 0

## Termination Conditions

### Success (Exit Loop)
All workflows for latest commit show "success" conclusion

### Max Iterations (10)
Iteration limit reached - report final status and remaining issues

### No Progress (3 static iterations)
Same failures repeating with no improvement - escalate to user

### Manual Stop
User intervention required

## Metrics to Track
- **Time per iteration**: From commit to all workflows complete
- **Workflows triggered**: Number of workflows that run
- **Success rate per iteration**: % workflows passing
- **Improvement per iteration**: Δ success count
- **Total time**: From start to all workflows green

## Expected Workflows to Monitor

Based on repository analysis, expect these workflows:
1. **Quick Check** - Fast format/clippy check on PRs
2. **CI** - Comprehensive CI (triggered by Quick Check or direct push)
3. **Security** - Security scanning (triggers on all pushes)
4. **YAML Lint** - YAML validation (triggers on workflow changes)
5. **Benchmarks** - Performance benchmarks (may trigger)
6. **Release** - Only on tags (won't trigger for develop push)

## Iteration 1 Plan

### Initial Check (After 30s)
```bash
sleep 30
gh run list --repo d-o-hub/rust-self-learning-memory --branch develop --limit 10
```

### Monitor Until Complete
Poll every 30 seconds, checking:
- Are all workflows finished?
- Any failures detected?

### Analyze Results
Once complete:
- Count success/failure
- Get logs for any failures
- Determine if fixes needed

### Decision
- All pass → SUCCESS, exit loop
- Failures → Analyze with goap-agent, fix, commit, next iteration
- Still running → Continue monitoring

## Safety Mechanisms
- **Max wait per iteration**: 30 minutes (workflows shouldn't take longer)
- **Total loop timeout**: 5 hours (10 iterations × 30 min max)
- **No-progress detection**: 3 iterations with same failures → escalate
- **Manual intervention**: User can stop loop anytime

## Context Preservation Between Iterations

Each iteration receives:
- Previous iteration results
- Failure patterns observed
- Fixes already attempted
- Current workflow status
- Commit history

This prevents:
- Repeating failed fixes
- Losing progress context
- Missing patterns
- Inefficient iterations

## Expected Outcome

**Best case**: Iteration 1, all workflows pass (our changes are solid)
**Likely case**: Iteration 1-2, minor fixes needed, all pass by iteration 2
**Worst case**: 3-5 iterations with complex failures requiring analysis

**Final state**: All GitHub Actions workflows showing green ✅
