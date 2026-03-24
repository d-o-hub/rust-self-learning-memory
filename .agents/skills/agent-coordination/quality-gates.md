# Quality Gates in Agent Coordination

## Overview

Quality gates are validation checkpoints that ensure work meets required standards before proceeding.

## Gate Types

### Pre-Execution Gates

Validate prerequisites before starting work.

```
Checks:
- Required tools available
- Environment configured
- Dependencies resolved
- Input data valid

Example:
- Before build: check cargo, rustc versions
- Before test: check database connection
- Before deploy: check credentials
```

### During-Execution Gates

Monitor progress and catch issues early.

```
Checks:
- Progress milestones met
- Resource usage within limits
- Intermediate results valid
- No blocking errors

Example:
- During test: memory usage, timeout warnings
- During build: compilation warnings
- During analysis: progress percentage
```

### Post-Execution Gates

Validate final results before marking complete.

```
Checks:
- All tests pass
- Coverage threshold met
- No clippy warnings
- Documentation updated

Example:
- After implementation: cargo test, cargo clippy
- After review: all comments addressed
- After deploy: health checks pass
```

### Integration Gates

Verify cross-component compatibility.

```
Checks:
- API contracts honored
- Data formats compatible
- No breaking changes
- Integration tests pass

Example:
- After API change: integration test suite
- After schema change: migration test
- After dependency update: full test suite
```

## Gate Configuration

### Severity Levels

| Level | Behavior |
|-------|----------|
| **Blocking** | Stop execution, require fix |
| **Warning** | Log issue, allow continuation |
| **Info** | Record for reporting |

### Threshold Configuration

```yaml
quality_gates:
  coverage:
    threshold: 90
    severity: blocking
  complexity:
    threshold: 10
    severity: warning
  security:
    threshold: 0
    severity: blocking
```

## Integration with Agents

### Task Agents with Gates

```
Task Agent Execution:
1. Check pre-execution gates
2. Execute work
3. Check during-execution gates (periodic)
4. Check post-execution gates
5. Report results
```

### Loop Agent with Gates

```
Loop Agent Execution:
1. Execute work
2. Check quality gates
3. If gates fail: refine and retry
4. If gates pass: complete
5. Max iterations: configurable
```

## Common Quality Gates

### Rust Project Gates

| Gate | Command | Threshold |
|------|---------|-----------|
| Format | `./scripts/code-quality.sh fmt` | 0 violations |
| Clippy | `./scripts/code-quality.sh clippy --workspace` | 0 warnings |
| Tests | `cargo nextest run` | 100% pass |
| Coverage | `cargo tarpaulin` | ≥90% |
| Docs | `cargo doc` | No warnings |

### CI/CD Gates

| Gate | Check |
|------|-------|
| Build | Compiles on all platforms |
| Security | No vulnerabilities |
| Performance | No regression |
| Integration | E2E tests pass |

## See Also

- [strategies.md](./strategies.md) - Execution patterns
- [skills-agents.md](./skills-agents.md) - Skills vs Agents
- [examples.md](./examples.md) - Real-world examples