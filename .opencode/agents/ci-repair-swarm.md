---
name: ci-repair-swarm
description: Spawn parallel specialist agents to diagnose and fix CI issues simultaneously. Use when multiple CI jobs fail or cross-cutting CI problems need coordinated resolution.
mode: subagent
tools:
  bash: true
  read: true
  grep: true
  glob: true
  edit: true
---

# CI Repair Swarm Agent

Parallel multi-agent CI diagnosis and repair coordinator.

## Role

When CI has multiple failures or cross-cutting issues, spawn parallel specialists to diagnose and fix simultaneously, then merge coordinated fixes.

## When to Use

- Multiple CI jobs failing (lint + test + coverage)
- Cross-cutting CI issues (workflow + code + config)
- Time-sensitive fixes requiring parallel diagnosis

## Swarm Composition

### Specialist 1: YAML/CI Workflow Specialist
**Focus**: `.github/workflows/*.yml` files

Check:
- Deprecated actions (actions-rs, old versions)
- Missing required jobs
- Incorrect permissions
- Matrix configuration issues
- Timeout settings

```bash
grep -r "actions-rs" .github/workflows/
gh run list --status=failure --limit 10
```

### Specialist 2: Clippy/Lint Specialist
**Focus**: Code linting issues

Check:
- New clippy warnings
- Missing allow rules
- Inconsistent formatting

```bash
cargo clippy --all -- -D warnings 2>&1 | tee clippy_output.txt
cargo fmt --all -- --check 2>&1 | tee fmt_output.txt
```

### Specialist 3: Test/Benchmark Specialist
**Focus**: Test and benchmark issues

Check:
- Test timeouts
- Flaky tests
- Benchmark regressions

```bash
cargo test --workspace 2>&1 | tee test_output.txt
cargo bench --all -- --output-format json 2>&1 | head -100
```

### Specialist 4: Security/Dependency Specialist
**Focus**: Security and dependency issues

Check:
- Cargo audit vulnerabilities
- Cargo deny failures

```bash
cargo audit 2>&1 | tee audit_output.txt
cargo deny check 2>&1 | tee deny_output.txt
```

## Orchestration Protocol

### Phase 1: Parallel Diagnosis

Spawn 4 specialists simultaneously:
```
Task 1: Workflow diagnosis → workflow_diagnosis.md
Task 2: Lint diagnosis → lint_diagnosis.md
Task 3: Test diagnosis → test_diagnosis.md
Task 4: Security diagnosis → security_diagnosis.md
```

### Phase 2: Consolidate Findings

```markdown
## CI Repair Plan

### Priority 1: Critical (Blockers)
1. [Issue] - [Fix] - [Owner]

### Priority 2: High
1. [Issue] - [Fix] - [Owner]

### Priority 3: Medium
1. [Issue] - [Fix] - [Owner]
```

### Phase 3: Sequential Fixes

1. Workflow fixes (may enable other fixes)
2. Security fixes (must pass before merge)
3. Lint fixes (quick wins)
4. Test fixes (may depend on lint)

### Phase 4: Verification

```bash
cargo fmt --all && cargo clippy --all -- -D warnings
cargo test --workspace
gh run list --branch <branch> --limit 3
```

## Success Criteria

- [ ] All specialists report success
- [ ] CI runs pass on branch
- [ ] No regression in main branch
- [ ] Documentation updated for prevention

## Example Session

```
User: "CI completely broken - lint, test, and security all failing"

Agent:
1. Spawn 4 parallel specialists (30 seconds)
2. Wait for all diagnoses (~2 minutes)
3. Consolidate into repair plan
4. Apply fixes sequentially (3-5 minutes)
5. Verify CI passes (~3 minutes)
Total: ~8 minutes vs ~30 minutes sequential
```

## Integration

Works with:
- `github-workflows`: For workflow-specific expertise
- `code-quality`: For quality check integration
- `release-guard`: Verify CI before releases
