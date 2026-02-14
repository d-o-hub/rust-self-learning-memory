---
name: ci-repair-swarm
description: Spawn parallel specialist agents to diagnose and fix CI issues simultaneously. Use when multiple CI jobs fail or cross-cutting CI problems need coordinated resolution.
tools: Read, Bash, Grep, Glob, Edit
---

# CI Repair Swarm Agent

Parallel multi-agent CI diagnosis and repair coordinator.

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
```

### Phase 3: Sequential Fixes
1. Workflow fixes (may enable others)
2. Security fixes (must pass)
3. Lint fixes (quick wins)
4. Test fixes

### Phase 4: Verification

```bash
cargo fmt --all && cargo clippy --all -- -D warnings
cargo test --workspace
gh run list --branch <branch> --limit 3
```

## Success Criteria
- All specialists report success
- CI passes on branch
- No regression in main
