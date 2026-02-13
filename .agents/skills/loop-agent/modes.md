# Loop Termination Modes

## 1. Fixed Iteration Count

```markdown
Run exactly N iterations regardless of results
Use when: Known number of refinement passes needed
Example: "Run 3 quality improvement iterations"
```

## 2. Criteria-Based Termination

```markdown
Continue until success criteria met (with max limit)
Use when: Specific quality/performance targets exist
Example: "Optimize until response time < 100ms (max 10 iterations)"
```

## 3. Convergence Detection

```markdown
Stop when improvements become negligible
Use when: Optimal result unknown, stop at diminishing returns
Example: "Refactor until <5% quality improvement over 3 iterations"
```

## 4. Hybrid Mode

```markdown
Combine multiple termination conditions
Use when: Complex requirements with multiple stop signals
Example: "Min 2 iterations, max 15, stop when quality > 90% OR converged"
```
