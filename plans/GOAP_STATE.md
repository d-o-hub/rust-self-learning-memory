# GOAP State Snapshot

- **Last Updated**: 2026-07-14 (codebase analysis, PR/issue review, CI verification)
- **Version**: `0.1.35` (target — workspace Cargo.toml needs version correction from 0.2.0 back to 0.1.35)
- **Branch**: `main`
- **Validation**: `plans/STATUS/VALIDATION_LATEST.md`
- **Gap Analysis**: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- **Primary ADRs**: ADR-072 (Accepted — v0.1.35 Comprehensive Analysis, this session), ADR-071 (Auto-Checkpoint on Abstained), ADR-060 (handle_shutdown deprecation), ADR-058 (CI Health), ADR-057 (CI Health PR616)

---

## Sprint 2026-07-14: Supply Chain Fix & Patch Release v0.1.35

**Task**: Fix yanked spin crate blocking CI, merge dependency PRs, correct version to 0.1.35, cut patch release

**World State**:
- Local quality: ✅ Build, Clippy, Fmt all pass
- Main branch CI: ⚠️ `Supply Chain Security` and `Security` workflows FAILING (yanked `spin 0.9.8`)
- PR #824 (fix/yanked-spin): ✅ **CLEAN — Ready to merge** (all checks pass)
- PR #820 (dependabot patch-minor): ⚠️ UNSTABLE (blocked by yanked spin, will fix after #824)
- PR #821 (dependabot sysinfo major): ⚠️ UNSTABLE (blocked by yanked spin, will fix after #824)
- Issue #823: Release drift — 7 unreleased commits since v0.1.34
- Version issue: Cargo.toml shows 0.2.0 (PR #822 over-bumped); must revert to 0.1.35 for patch release

**Immediate Actions (Priority Order)**:

| # | Action | Owner Skill | Priority | Status |
|---|--------|-------------|----------|--------|
| 1 | Merge PR #824 (fix yanked spin 0.9.8 → 0.9.9) | `pr-readiness` | 🔴 Critical | Ready |
| 2 | Fix workspace version: 0.2.0 → 0.1.35 | `feature-implement` | 🔴 Critical | TODO |
| 3 | Rebase/update PR #820 against main | `ci-fix` | High | Blocked on #1 |
| 4 | Rebase/update PR #821 against main | `ci-fix` | High | Blocked on #1 |
| 5 | Merge #820 and #821 after CI green | `pr-readiness` | High | Blocked on #3, #4 |
| 6 | Cut v0.1.35 release (close #823) | `release-guard` | High | Blocked on #2, #5 |

---

## Open Issues

| # | Title | Priority | Notes |
|---|-------|----------|-------|
| #823 | Release drift: 7 unreleased commits since v0.1.34 | 🔴 High | Auto-generated; resolve with v0.1.35 release |

**Backlog** (from previous sprints, still relevant):
| # | Title | Priority | Notes |
|---|-------|----------|-------|
| #753 | Retry budgets | Backlog | Large feature; not blocking |
| #749 | Turso connection pooling | Backlog | Large feature; not blocking |
| #746 | WASM build path | Backlog | Large feature; not blocking |
| #743 | Storage refactor | Backlog | Large refactor; not blocking |

---

## Open PRs Status

| PR | Title | Merge State | CI | Action |
|----|-------|-------------|----|----|
| **#824** | fix(deps): update yanked spin 0.9.8 → 0.9.9 | ✅ CLEAN | ✅ All pass | **Merge immediately** |
| #821 | chore(deps): bump sysinfo 0.38.4 → 0.39.6 | ⚠️ UNSTABLE | ❌ cargo-deny (spin) | Wait for #824 merge, then rebase |
| #820 | chore(deps): bump rust-patch-minor (3 updates) | ⚠️ UNSTABLE | ❌ cargo-deny (spin) | Wait for #824 merge, then rebase |

---

## CI Health (Main Branch)

| Workflow | Status | Notes |
|----------|--------|-------|
| Quick Check | ✅ Pass | Format + Clippy clean |
| CI (Tests, Semver, MCP, Multi-Platform) | ✅ Pass | All jobs green |
| Storage Matrix Tests | ✅ Pass | redb, turso, hybrid |
| Performance Benchmarks | ✅ Pass | No regressions |
| Coverage | ✅ Pass | |
| File Structure Validation | ✅ Pass | |
| Release Drift Check | ✅ Pass | (created #823) |
| **Supply Chain Security** | ❌ FAIL | `cargo-deny`: yanked spin 0.9.8 |
| **Security** | ❌ FAIL | `cargo-deny`: yanked spin 0.9.8 |

**Root Cause**: `spin` crate version 0.9.8 was yanked from crates.io. PR #824 updates to 0.9.9.

---

## Version Correction Required

PR #822 bumped workspace to `0.2.0` citing a "semver breaking change" in MCP. However, the project follows `0.1.x` patch releases. The correct action is:

1. Revert workspace version in `Cargo.toml` from `0.2.0` → `0.1.35`
2. If the MCP change was truly breaking, document it in CHANGELOG but keep on patch line (pre-1.0 semver allows breaking changes in minor/patch)
3. Tag release as `v0.1.35`

**Rationale**: Pre-1.0 semver (0.x.y) does not require major bumps for breaking changes. The project has consistently released as 0.1.x and should continue until a deliberate 1.0 decision is made.

---

## Key Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Workspace version (current) | 0.2.0 | 0.1.35 | ⚠️ Needs fix |
| Latest GitHub release | v0.1.34 | — | ⚠️ Drift |
| Open issues | 1 | 0 | Release drift |
| Open PRs | 3 | 0 | 1 ready, 2 blocked |
| Local build | ✅ Pass | Pass | ✅ |
| Local clippy | ✅ 0 warnings | 0 | ✅ |
| Local fmt | ✅ Clean | Clean | ✅ |
| Skills count | 33 | ≤35 | ✅ |
| Files >500 LOC | 0 | 0 | ✅ |
| ADRs | 42 → 43 | — | +ADR-072 |

---

## Recommended Next Sprint: v0.1.35 Patch Release

**Goal**: Fix CI, correct version, merge deps, cut v0.1.35 patch release.

**Work Groups**:

| WG | Task | Owner Skill | Effort | Priority |
|----|------|-------------|--------|----------|
| WG-186 | Merge PR #824 (spin fix) | `pr-readiness` | 5 min | 🔴 Critical |
| WG-187 | Revert version 0.2.0 → 0.1.35 in Cargo.toml | `feature-implement` | 10 min | 🔴 Critical |
| WG-188 | Rebase + merge PRs #820, #821 | `ci-fix` + `pr-readiness` | 30 min | High |
| WG-189 | Update CHANGELOG.md for v0.1.35 | `agents-update` | 15 min | High |
| WG-190 | Update STATUS/CURRENT.md | `agents-update` | 10 min | High |
| WG-191 | Tag + push v0.1.35 release | `release-guard` | 10 min | High |
| WG-192 | Verify release workflow completes | `ci-poll` | 20 min | High |
| WG-193 | Close issue #823 (auto-closes on release) | — | Auto | — |

**Quality Gates**:
- Gate 1 (after WG-186): Main branch `Supply Chain Security` + `Security` workflows pass
- Gate 2 (after WG-187): `cargo metadata` shows version 0.1.35
- Gate 3 (after WG-188): All open PRs merged, 0 open PRs
- Gate 4 (after WG-191): v0.1.35 release created via workflow, 0 open issues

---

## Future Backlog (Post-v0.1.35)

| Priority | Item | Notes |
|----------|------|-------|
| Medium | Trusted Publishing (OIDC) for crates.io | Eliminate CARGO_REGISTRY_TOKEN; see ADR-045 |
| Medium | Code coverage improvement (61% → 70%) | ADR-042 Phase 1 target |
| Backlog | Retry budgets (#753) | Large feature |
| Backlog | Turso connection pooling (#749) | Large feature |
| Backlog | WASM build path (#746) | Large feature |
| Backlog | Storage refactor (#743) | Large refactor |
| Backlog | Federated HDC multi-agent (WG-135) | Research feature |

---

## GOAP Skill Stack (This Sprint)

| Phase | Skills | Strategy |
|-------|--------|----------|
| Fix CI | `ci-fix`, `pr-readiness` | Sequential (merge #824 first) |
| Fix version | `feature-implement` | Sequential (Cargo.toml edit) |
| Merge deps | `pr-readiness`, `ci-poll` | Sequential (rebase, wait for CI) |
| Release | `release-guard`, `agents-update` | Sequential (docs → tag → verify) |
| Validate | `code-quality`, `test-runner` | Parallel (local checks) |

---

## Release Process (MANDATORY)

**NEVER manually create GitHub releases.** Always use the automated workflow:
1. Bump version in `Cargo.toml` to `0.1.35`
2. Update `CHANGELOG.md` / `ROADMAP_ACTIVE.md` / `STATUS/CURRENT.md`
3. Commit + push to `main`
4. Push git tag: `git tag v0.1.35 && git push origin v0.1.35`
5. The `release.yml` workflow triggers automatically on tag push
6. The `release-drift.yml` monitors unreleased commits and auto-creates drift issues

---

## Cross-References

| Document | Location |
|----------|----------|
| ADR-072 (this analysis) | `plans/adr/ADR-072-Comprehensive-Analysis-v0.1.35.md` |
| Active Roadmap | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| Latest Validation | `plans/STATUS/VALIDATION_LATEST.md` |
| Latest Gap Analysis | `plans/STATUS/GAP_ANALYSIS_LATEST.md` |
| ADR Index | `plans/adr/` |
| Skills | `.agents/skills/` (33 skills) |
