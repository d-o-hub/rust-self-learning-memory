# ADR-029: GitHub Actions Workflow Modernization

**Status**: Accepted
**Date**: 2026-02-14
**Deciders**: Project maintainers
**Context**: Comprehensive audit of all 9 GitHub Actions workflows against 2025-2026 best practices

## Alternatives Considered

1. **Full SHA pinning for all actions**
   - Pros: Maximum security
   - Cons: High maintenance burden
   - **DEFERRED**: Dependabot auto-updates provide sufficient coverage

2. **Migrate to actions-rust-lang/setup-rust-toolchain**
   - Pros: Would replace both dtolnay/rust-toolchain and Swatinem/rust-cache, better problem matchers
   - Cons: Non-critical change, workflow churn
   - **DEFERRED**: Not justified as standalone migration

3. **Self-hosted runners**
   - Pros: Would eliminate disk space issues entirely
   - Cons: Cost/complexity not justified yet
   - **DEFERRED**: Revisit when CI minutes or disk constraints become blocking

4. **Phased Modernization (Chosen)**
   - Pros: Prioritized by severity, low risk per change, easy rollback
   - Cons: Multiple PRs, more review cycles
   - **ACCEPTED**: Best risk/reward balance with clear priority ordering

## Decision Details

A full audit of the `.github/` directory on 2026-02-14 identified 16 issues across 4 severity levels. The workflows are functional but have security gaps, outdated action versions, a correctness bug, and several missed optimization opportunities.

We will implement a phased modernization:

### Phase 1: Critical Fixes (immediate, ~1h)
1. **Pin supply-chain risky actions** - `easimon/maximize-build-space@master` and `jlumbroso/free-disk-space@main` must be pinned to SHA or release tag
2. **Fix quality-gates permissions** - `ci.yml` quality-gates job overrides workflow permissions, breaking checkout (missing `contents: read`)
3. **Remove double checkout** - `nightly-tests.yml` has checkout at line 58 and 77 (bug)
4. **Fix phantom artifact upload** - `ci.yml` test job installs nextest but runs cargo test; uploads nonexistent junit.xml

### Phase 2: Version Updates (1-2h)
5. **actions/checkout v4 → v6** - All workflows (improved credential security, Node.js 24)
6. **actions/upload-artifact v4 → v6** - Standardize across all workflows (currently inconsistent: v6 in ci.yml, v4 in coverage.yml)
7. **taiki-e/install-action@nextest → @v2 with tool parameter** - Avoid moving ref

### Phase 3: Security Hardening (1-2h)
8. **Split benchmark permissions** - PR benchmark runs should NOT have `contents: write`
9. **Tighten security.yml triggers** - Remove push on `**` (all branches); limit to main/develop + PRs + schedule
10. **Fix dependency-review continue-on-error** - Currently silently passes even when it should fail
11. **Pin yamllint version** - `pip install yamllint` is non-reproducible

### Phase 4: Optimization (optional, 1-2h)
12. **Improve concurrency groups** - Use `event.pull_request.number || github.ref` instead of `run_id`
13. **Add Dependabot grouping** - Group patch/minor updates for cargo; group GitHub Actions updates
14. **Remove single-value matrix** - `multi-platform` has `rust: [stable]` single-value matrix
15. **Consider actions-rust-lang/setup-rust-toolchain@v1** - Better problem matchers than dtolnay
16. **Add PR/Issue templates** - Not currently present

## Tradeoffs
- **Positive**: Eliminates 2 critical supply-chain vulnerabilities
- **Positive**: Fixes correctness bugs (double checkout, phantom artifact)
- **Positive**: Standardizes on latest action versions (Node.js 24 runtime)
- **Positive**: Reduces blast radius on PR-triggered workflows
- **Negative**: actions/checkout v6 requires Actions Runner v2.329.0+ (GitHub-hosted runners already updated)
- **Negative**: Minor workflow churn in PR history

## Consequences
- Supply-chain risk eliminated for unpinned `@master`/`@main` refs
- Correctness bugs fixed → CI results are trustworthy
- Action versions standardized → consistent Node.js runtime, reduced deprecation warnings
- PR-triggered workflows get minimal permissions → reduced blast radius
- Dependabot grouping reduces PR noise from dependency updates

## Implementation
See: `plans/GOAP_GITHUB_ACTIONS_2026-02-14.md` for task decomposition and execution order.

## References
- [actions/checkout v6 release notes](https://github.com/actions/checkout/releases/tag/v6.0.2)
- [actions/upload-artifact v6](https://github.com/actions/upload-artifact/releases/tag/v6.0.0)
- [GitHub Actions security hardening guide](https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions)
- **ADR-023**: CI/CD GitHub Actions Remediation (predecessor)
- **ADR-025**: Project Health Remediation

## Related
- **ADR-023**: CI/CD GitHub Actions Remediation (predecessor, now complete)
- **Plans**: `plans/GOAP_GITHUB_ACTIONS_2026-02-14.md` (execution plan)
