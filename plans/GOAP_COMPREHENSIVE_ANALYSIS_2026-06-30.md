# GOAP Comprehensive Codebase Analysis — 2026-06-30

**Orchestrator**: `goap-agent`
**Phase**: ANALYZE → DECOMPOSE → STRATEGIZE
**Branch**: `main`
**Workspace Version**: `0.1.33` (no git tag — 100 commits since `v0.1.32`)
**Last Green Push CI**: `a9fb3d73` (2026-06-30)
**Previous Analysis**: `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-28.md`

---

## Executive Summary

Since the 2026-06-28 analysis, 6 PRs have been merged completing WG-176..182 (CI health, code quality, architecture). All Phase 2-4 goals from the prior plan are resolved. The codebase is now in its healthiest CI state in months:

- **clippy `--all-features`**: ✅ Clean (0 findings)
- **Production files >500 LOC**: ✅ 0 violations (wrapper.rs split done)
- **cargo deny check**: ✅ Clean (stale advisories removed)
- **GitHub Actions**: ✅ Updated to Node 24 (13 actions bumped)
- **Push CI**: ✅ Green

**Remaining work** falls into three categories:
1. **Release drift** (P1) — WG-175 still pending; now 100 commits since v0.1.32
2. **Proactive code health** (P2) — files at 500 LOC boundary, dependency duplication, unwrap hygiene
3. **DevX backlog** (P3) — issues #652, #653 + ghost ADR-058

---

## Findings

### A. Release Drift (Critical)

| # | Issue | Commits Behind | Status |
|---|-------|----------------|--------|
| #674 | Release drift since v0.1.32 | 100 | 🔴 Overdue |

The workspace `Cargo.toml` is at `0.1.33` but no tag/release exists. This is the #1 priority.

### B. File Size Gate — Boundary Risk

Two files sit at **exactly** 500 LOC (the gate limit). Any future addition will trigger a quality gate failure:

| File | LOC | Risk |
|------|-----|------|
| `memory-core/src/retrieval/cascade/mod.rs` | 500 | 🟡 Boundary |
| `memory-cli/src/commands/tag/core.rs` | 500 | 🟡 Boundary |

Additionally, 8 files are in the 490-499 range (imminent breach):

| File | LOC |
|------|-----|
| `memory-core/src/embeddings/local.rs` | 497 |
| `memory-storage-turso/src/resilient.rs` | 494 |
| `memory-core/src/storage/mod.rs` | 494 |
| `memory-core/src/security/audit/mod.rs` | 493 |
| `memory-storage-turso/src/pool/adaptive.rs` | 491 |
| `memory-core/src/indexing/spatiotemporal/mod.rs` | 491 |
| `memory-storage-turso/src/storage/batch/query_batch.rs` | 490 |
| `memory-cli/src/commands/episode/relationships/types.rs` | 489 |

### C. Dependency Duplication

| Crate | Versions | Cause | Impact |
|-------|----------|-------|--------|
| `thiserror` | v1.0.69 + v2.0.18 | `agentfs-sdk` and `argmin` use v1 | Binary size, compile time |
| `zerocopy` | v0.7.35 + v0.8.48 | Transitive deps | Minor |
| `webpki-roots` | v0.26.11 + v1.0.7 | Transitive deps | Minor |

The workspace uses `thiserror = "2.0.18"` but transitive deps from `agentfs-sdk` and `argmin` still pull in v1. This is expected (not directly fixable without upstream changes) but should be monitored for future deduplication opportunities.

### D. Unwrap Hygiene

**472 `.unwrap()` calls** across production source files (excluding test modules). Breakdown:

- ~300 in inline test blocks (`#[cfg(test)]` modules within source files)
- ~80 in documentation examples (doc comments)
- ~90 in actual production paths (monitoring, config, embedding serialization)

The most concerning production unwraps are in:
- `memory-core/src/monitoring/core.rs` — 12+ unwraps in production metrics code
- `memory-core/src/embeddings/config/*/` — serialization unwraps (safe but brittle)
- `memory-storage-turso/src/` — scattered lock/parse unwraps

### E. Ghost ADR-058

GOAP_STATE.md references **ADR-058** ("Accepted — Scheduled gitleaks false positives, release drift") but the file was never created. The original 2026-06-14 analysis mentions creating `plans/adr/ADR-058-CI-Health-Gitleaks-Release-Drift-2026-06-14.md` but it was never committed.

### F. Open Issues (3)

| # | Title | Priority | Status |
|---|-------|----------|--------|
| #674 | Release drift: 100 unreleased commits since v0.1.32 | P1 | WG-175 pending |
| #652 | Automate llms.txt and full LLM context generation | P3 | WG-183 pending |
| #653 | Evaluate VERSION file adoption | P3 | WG-184 pending |

### G. CI State (All Green)

| Workflow | Status | Notes |
|----------|--------|-------|
| Push CI (main) | ✅ Green | All checks pass |
| Security (scheduled) | ✅ Green | Gitleaks fingerprints added (WG-176) |
| Nightly Full Tests | ✅ Green | Disk cleanup added (WG-177) |
| Mutation Testing | ✅ Green | Scoped to memory-core (WG-178) |
| Fuzzing | ✅ Green | — |
| Dependency Audit | ✅ Green | Stale advisories removed |

---

## GOAP Action Plan

### Phase 1: Release (P1 — Critical)

| WG | Action | Skill | Deps |
|----|--------|-------|------|
| WG-175 | Cut v0.1.33 release — CHANGELOG + `gh release create` | `release-guard` | None |

### Phase 2: Code Health (P2 — Proactive)

| WG | Action | Skill | Deps |
|----|--------|-------|------|
| WG-185 | Split `cascade/mod.rs` and `tag/core.rs` below 500 LOC boundary | `code-quality` | None |
| WG-186 | Preemptive splits for files at 490-497 LOC (top 4 highest risk) | `code-quality` | WG-185 |
| WG-187 | Audit and reduce production unwrap() calls in monitoring + storage | `code-quality` | None |

### Phase 3: Documentation Hygiene (P2)

| WG | Action | Skill | Deps |
|----|--------|-------|------|
| WG-188 | Create ADR-058 file (gitleaks + release drift decision record) | `goap-agent` | None |
| WG-189 | Create ADR-059 (this analysis — boundary LOC + unwrap strategy) | `goap-agent` | None |

### Phase 4: DevX Backlog (P3)

| WG | Action | Skill | Deps |
|----|--------|-------|------|
| WG-183 | Implement llms.txt generator script (closes #652) | `feature-implement` | WG-175 |
| WG-184 | Write ADR for VERSION file decision (closes #653) | `goap-agent` | WG-175 |

---

## Execution Strategy

```
Phase 1 (WG-175) ─────────────────────────> Release v0.1.33 [CRITICAL PATH]
                                              │
Phase 2 (WG-185..187) ─── parallel ────────> LOC boundary + unwrap hygiene
Phase 3 (WG-188..189) ─── parallel ────────> ADR documentation debt
Phase 4 (WG-183..184) ─── sequential ─────> DevX issues closed
```

**Recommended order**: Phase 1 first (release drift is 100 commits and growing). Phases 2-3 can be parallelized. Phase 4 after release since both depend on stable version.

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| Unreleased commits | 100 | 0 (after v0.1.33 release) |
| Open issues | 3 (#674, #652, #653) | 0 |
| Files at exactly 500 LOC | 2 | 0 |
| Files in 490-499 LOC range | 8 | ≤4 |
| Production unwrap() calls (non-test) | ~90 | ≤50 |
| Ghost ADRs (referenced but missing) | 1 (ADR-058) | 0 |
| CI workflows green | 5/5 | 5/5 ✅ |
| Clippy `--all-features` | 0 findings | 0 ✅ |
| `cargo deny check` | Clean | Clean ✅ |

---

## Delta from Previous Analysis (2026-06-28)

| Item | 2026-06-28 | 2026-06-30 | Change |
|------|-----------|-----------|---------|
| Clippy `--all-features` | 5 findings | 0 | ✅ Resolved (PR #675) |
| Files >500 LOC | 1 (wrapper.rs) | 0 | ✅ Resolved (PR #675) |
| Scheduled Security | ❌ Failing | ✅ Green | ✅ Resolved (PR #675) |
| Nightly Tests | ❌ Failing | ✅ Green | ✅ Resolved (PR #675) |
| Mutation Testing | ❌ Cancelled | ✅ Completes | ✅ Resolved (PR #675) |
| Dependency Audit | Advisory active | ✅ Clean | ✅ Resolved (PR #682) |
| GitHub Actions | Node.js 20 warnings | ✅ Node 24 | ✅ Resolved (PR #681) |
| Cascade degradation | Silent empty | tracing::warn! | ✅ Resolved (PR #675) |
| Files at 500 LOC boundary | — | 2 (new finding) | 🟡 New |
| Production unwraps | — | ~90 (new finding) | 🟡 New |
| Ghost ADR-058 | — | Missing file | 🟡 New |

---

## References

- Previous analysis: `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-28.md`
- GOAP State: `plans/GOAP_STATE.md`
- ADR-057: CI Health (PR #616, nightly timeout)
- ADR-058: Referenced but never created (gitleaks + release drift)
- Issue #674: Release drift
- Issue #652: llms.txt automation
- Issue #653: VERSION file evaluation
