# ADR-046: Claude Code Configuration Improvements

- **Status**: Accepted
- **Date**: 2026-03-15
- **Implementation Date**: 2026-03-15
- **Deciders**: Project maintainers
- **Related**: ADR-022 (GOAP Agent System), ADR-038 (Local CI Parity)

## Context

Analysis of 34 Claude Code sessions (234 messages, 97 commits) revealed significant friction patterns that impact development velocity and code quality:

### Session Friction Analysis

| Friction Type | Count | Root Cause | Mitigation |
|---------------|-------|------------|------------|
| wrong_approach | 8 | Agent proceeded without understanding existing patterns | Read existing code before implementing |
| buggy_code | 6 | Insufficient testing before committing | Run tests after changes |
| excessive_changes | 5 | Large commits without atomic separation | Enforce atomic commits |
| tool_errors | 67 | Bash overuse, insufficient Grep usage | Enforce tool ratio targets |

### Tool Usage Metrics

| Metric | Current | Target | Gap |
|--------|---------|--------|-----|
| Bash:Grep ratio | 17:1 | 2:1 | Over-reliance on Bash |
| Tool errors per session | ~2 | 0 | Need better error handling |
| Pre-commit verification | Inconsistent | 100% | Missing enforcement |

### Common Pitfalls Identified

1. **GitHub Actions Version Issues**: `wait-on-check-action@v1.5.0` deprecated, use v2.0.0+
2. **--all-features Libclang Dependency**: Building with `--all-features` requires libclang, breaking CI
3. **Network-Dependent Test Flakiness**: Integration tests requiring TURSO_DATABASE_URL fail in CI
4. **Clippy Lint Handling**: Test-only lints need allow-list propagation to integration test crates

## Decision

Implement configuration improvements across three areas:

### 1. Update AGENTS.md

Add three sections after "Core Invariants":

**Common Pitfalls** (from session analysis):
- Read existing patterns before implementing new code
- Run tests after every change
- Use atomic commits for each logical change
- Check CI status before proceeding

**Tool Selection Enforcement**:
- Target Bash:Grep ratio of 2:1 (current: 17:1)
- Prefer Grep for file/content searches
- Use Bash only for scripts and git operations

**Atomic Change Rules**:
- One logical change per commit
- Commit message describes exactly what changed
- All tests pass before commit
- git status verification required

### 2. Create agent_docs/common_friction_points.md

Document detailed friction patterns with:
- Root cause analysis
- Prevention strategies
- Quick reference for common issues

### 3. Consolidate Hooks Configuration

Merge `.claude/hooks/hooks.json` into `.claude/settings.json`:
- Single source of truth for hooks
- Add quick compile check hook for Rust files
- Maintain backward compatibility

## Implementation Plan

### Phase 1: Documentation Updates

| File | Change | Priority |
|------|--------|----------|
| AGENTS.md | Add Common Pitfalls section | P0 |
| AGENTS.md | Add Tool Selection Enforcement section | P0 |
| AGENTS.md | Add Atomic Change Rules section | P0 |
| agent_docs/common_friction_points.md | Create new file | P1 |

### Phase 2: Hooks Consolidation

| File | Change | Priority |
|------|--------|----------|
| .claude/settings.json | Merge hooks from hooks.json | P1 |
| .claude/hooks/hooks.json | Remove (deprecated) | P2 |

### Phase 3: Update GOAP_STATE.md

Document this task as current sprint work.

## Rationale

1. **Reduce wrong_approach friction**: Explicit guidance prevents repeated mistakes
2. **Improve tool efficiency**: Better tool selection reduces errors and improves search capability
3. **Atomic commits**: Easier code review, better history, simpler rollbacks
4. **Hooks consolidation**: Single configuration file is easier to maintain

## Consequences

### Positive

- Reduced session friction by addressing top issues
- Better developer experience through clear guidance
- Consistent code quality through enforced checks
- Single source of truth for hooks configuration

### Negative

- Documentation maintenance burden
- May need periodic updates as patterns emerge

## Acceptance Criteria

1. AGENTS.md contains three new sections
2. agent_docs/common_friction_points.md exists with detailed patterns
3. Hooks consolidated into settings.json
4. GOAP_STATE.md updated with this task
5. All existing hooks continue to work

## References

- Session analysis: 34 sessions, 234 messages, 97 commits
- ADR-022: GOAP Agent System
- ADR-038: Local CI Parity for Clippy Tests
- agent_docs/github_actions_patterns.md