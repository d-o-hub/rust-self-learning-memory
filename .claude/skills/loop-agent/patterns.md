# Loop Patterns

## Pattern 1: Simple Refinement Loop

```markdown
Task: "Iteratively improve code quality"

Configuration:
- Max Iterations: 5
- Success: All clippy warnings resolved + rustfmt clean
- Agent: refactorer

Loop:
Iteration 1:
  - refactorer: Fix issues
  - Validate: Check clippy + rustfmt
  - Metrics: 15 warnings → Continue

Iteration 2:
  - refactorer: Fix remaining issues
  - Validate: Check clippy + rustfmt
  - Metrics: 3 warnings → Continue

Iteration 3:
  - refactorer: Final cleanup
  - Validate: Check clippy + rustfmt
  - Metrics: 0 warnings ✓ → Success (criteria met)

Result: 3 iterations, quality standards achieved
```

## Pattern 2: Test-Fix-Validate Loop

```markdown
Task: "Fix all failing tests"

Configuration:
- Mode: Criteria with max iterations
- Max Iterations: 10
- Success: All tests passing
- Agents: test-runner, debugger

Loop:
Iteration 1:
  - test-runner: Run test suite
  - Result: 23 failures
  - debugger: Diagnose top 5 failures
  - Metrics: 23/23 failures → Continue

Iteration 2-5:
  - debugger: Fix diagnosed failures
  - test-runner: Verify fixes
  - Metrics: 23 → 18 → 12 → 7 → 3 failures → Continue

Iteration 6:
  - debugger: Fix remaining failures
  - test-runner: Run full suite
  - Metrics: 0 failures ✓ → Success

Result: 6 iterations, all tests passing
```

## Pattern 3: Performance Optimization Loop

```markdown
Task: "Optimize response time"

Configuration:
- Mode: Convergence with max cap
- Max Iterations: 20
- Convergence: <2% improvement over 3 iterations
- Success: Response time < 50ms
- Agents: refactorer, test-runner

Loop:
Iteration 1: 250ms baseline
Iteration 5: 120ms (51% improvement)
Iteration 10: 75ms (37% improvement from iter 5)
Iteration 15: 58ms (23% improvement from iter 10)
Iteration 17: 52ms (10% improvement)
Iteration 18: 48ms (8% improvement)
Iteration 19: 47ms (2% improvement)
Iteration 20: 47ms (0% improvement) → Convergence

Result: 20 iterations, converged at 47ms
```
