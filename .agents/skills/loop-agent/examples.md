# Loop Examples

## Example 1: Quality Improvement Loop

```markdown
Task: Improve code quality metrics

## Configuration
- Mode: Hybrid
- Min Iterations: 3
- Max Iterations: 10
- Success: Clippy warnings = 0, Coverage > 85%
- Convergence: <1% improvement over 2 iterations

## Execution
Iteration 1 (Baseline):
  - refactorer: Initial quality pass
  - Metrics: Warnings: 45, Coverage: 72%
  - Decision: Continue (below thresholds)

Iteration 2:
  - refactorer: Fix clippy issues
  - test-runner: Run coverage
  - Metrics: Warnings: 28, Coverage: 76%
  - Decision: Continue

Iteration 3:
  - refactorer: Fix remaining warnings
  - test-runner: Coverage check
  - Metrics: Warnings: 12, Coverage: 80%
  - Decision: Continue

Iteration 4-5:
  - Progressive fixes
  - Metrics: Warnings: 5, Coverage: 83%

Iteration 6:
  - refactorer: Final fixes
  - test-runner: Full validation
  - Metrics: Warnings: 0, Coverage: 86%
  - Decision: Success! (criteria met)

Result: 6 iterations, all targets achieved
```

## Example 2: Documentation Enhancement Loop

```markdown
Task: Improve documentation coverage

## Configuration
- Mode: Fixed iterations
- Iterations: 5
- Success: API docs complete, Examples: 10+
- Agents: code-reviewer, refactorer

## Execution
Each iteration focuses on different documentation aspect:
1. API documentation for public functions
2. Code comments for complex logic
3. Usage examples for main features
4. README and getting started guide
5. Cross-reference and links check

Result: 5 iterations, complete documentation
```

## Progress Tracking

Track across iterations:
- Iteration number
- Current metrics vs baseline
- Improvement rate
- Remaining gap to target
- Estimated iterations to completion
