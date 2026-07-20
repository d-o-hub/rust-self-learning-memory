# GOAP Comprehensive Codebase Analysis — 2026-06-28

**Orchestrator**: `goap-agent`
**Phase**: ANALYZE → DECOMPOSE → STRATEGIZE
**Branch**: `main`
**Workspace Version**: `0.1.33` (no git tag — 94 commits since `v0.1.32`)
**Last Green Push CI**: `850bf69d` (2026-06-26)

---

## Executive Summary

The project is in good health on default features (clippy clean, build green, push CI passing). However, several improvement vectors remain:

1. **Release drift** — 94 unreleased commits (issue #674); workspace bumped to 0.1.33 but never tagged/released
2. **CI health** — Scheduled Security (gitleaks) and Nightly Full Tests still failing; Mutation cancelled at 6h
3. **Code quality** — 5 clippy lints under `--all-features`; 1 file over 500 LOC gate
4. **Architecture** — Non-CSM cascade fallback returns empty result (no graceful degradation)
5. **Testing** — Mutation testing never produces signal (ceiling too low or scope too broad)
6. **DevX backlog** — Issues #652 (llms.txt) and #653 (VERSION file) are low-hanging

---

## Findings

### A. Open Issues (3)

| # | Title | Priority | Action |
|---|-------|----------|--------|
| #674 | Release drift: 94 unreleased commits since v0.1.32 | P1 | Cut v0.1.33 release |
| #652 | Automate llms.txt and full LLM context generation | P3 | Implement generator script |
| #653 | Evaluate VERSION file adoption | P3 | Document decision (ADR) |

### B. CI Health

| Workflow | Status | Root Cause | Fix |
|----------|--------|------------|-----|
| Push CI (`main`) | ✅ Green | — | — |
| Security (scheduled) | ❌ Failing | Gitleaks: 3 false positives not in `.gitleaksignore` | Add fingerprints |
| Nightly Full Tests | ❌ Failing | Disk exit-95 (runner infra) | Runner disk cleanup step |
| Mutation Testing | ❌ Cancelled | 6h ceiling always hit | Scope to hot-path crates only |
| Fuzzing | ✅ Green | — | — |

**Node.js 20 Deprecation**: `actions/checkout@v4` (SHA `11bd719`) and `actions/upload-artifact@v4` (SHA `65c4c4a`) still target Node.js 20. GitHub is forcing them to run on Node.js 24 with deprecation warnings. Need to bump to `@v5` when available or accept the warning.

### C. Code Quality

| Finding | Location | Severity | Fix |
|---------|----------|----------|-----|
| 4× `excessive_nesting` | `memory-core/src/embeddings/mistral/client.rs:174-177` | Error (--all-features) | Extract inner match arms into helper functions |
| 1× `unnecessary_wraps` | `memory-core/src/embeddings/mistral/client.rs:158` | Warning | Remove `Result` wrapper from `dequantize_binary_embeddings` |
| File >500 LOC | `memory-storage-turso/src/cache/wrapper.rs` (537 LOC) | Gate violation | Split into `wrapper.rs` + `wrapper_ops.rs` |

### D. Architecture

| Finding | Location | Impact |
|---------|----------|--------|
| Non-CSM cascade returns empty result | `memory-core/src/retrieval/cascade/mod.rs:207` | Silent no-op when CSM feature disabled; callers get zero results with no indication why |

**Recommendation**: Add `compile_error!` or a meaningful log warning, or return a `CascadeResult` with a `degraded: true` flag so callers know the cascade is inactive.

### E. Security

| Advisory | Crate | Status | Impact |
|----------|-------|--------|--------|
| RUSTSEC-2026-0183 | git2 0.20.4 | Warning (allowed) | Transitive via `agentfs-sdk` |
| RUSTSEC-2026-0184 | git2 0.20.4 | Warning (allowed) | Transitive via `agentfs-sdk` |

No actionable vulnerability — both are in transitive deps from `turso_core` and have been accepted.

### F. Testing & Coverage

- **Test count**: Builds compile (7m23s for workspace test binaries)
- **Mutation testing**: Always cancelled at 6h ceiling — never produces signal
- **Fuzz harness**: Added in commit `14d756bd` — covers parser/serialization boundaries
- **Property tests**: 154 occurrences across 17 files (healthy)

---

## GOAP Action Plan

### Phase 1: Release (P1 — Closes #674)

| WG | Action | Skill | Deps |
|----|--------|-------|------|
| WG-175 | Tag and release v0.1.33 from current `main` | `release-guard`, `github-release-best-practices` | None |

**Preconditions**: Push CI is green; workspace already at 0.1.33. Need CHANGELOG entry for unreleased commits, then `gh release create v0.1.33`.

### Phase 2: CI Health (P2)

| WG | Action | Skill | Deps |
|----|--------|-------|------|
| WG-176 | Add missing gitleaks fingerprints for 3 false positives | `ci-fix` | None |
| WG-177 | Add disk cleanup step to nightly-tests.yml (before build) | `ci-fix` | None |
| WG-178 | Scope mutation testing to `memory-core` only + reduce timeout to 2h | `ci-fix` | None |
| WG-179 | Bump `actions/upload-artifact` to latest SHA (Node 24 target) | `ci-fix` | None |

### Phase 3: Code Quality (P2)

| WG | Action | Skill | Deps |
|----|--------|-------|------|
| WG-180 | Fix excessive_nesting + unnecessary_wraps in `mistral/client.rs` | `code-quality` | None |
| WG-181 | Split `cache/wrapper.rs` below 500 LOC gate | `code-quality` | None |

### Phase 4: Architecture (P3)

| WG | Action | Skill | Deps |
|----|--------|-------|------|
| WG-182 | Add `tracing::warn!` to non-CSM cascade fallback path | `feature-implement` | None |

### Phase 5: DevX Backlog (P3)

| WG | Action | Skill | Deps |
|----|--------|-------|------|
| WG-183 | Implement llms.txt generator script (closes #652) | `feature-implement` | None |
| WG-184 | Write ADR for VERSION file decision (closes #653) | `goap-agent` | None |

---

## Execution Strategy

**Pattern**: Parallel Swarm for independent phases; Sequential for release-blocking items.

```
Phase 1 (WG-175) ────────────────────────────────> Release v0.1.33
                                                      │
Phase 2 (WG-176..179) ──── parallel ──────────────> CI green across all scheduled workflows
Phase 3 (WG-180..181) ──── parallel ──────────────> Clippy --all-features clean; 500 LOC gate met
Phase 4 (WG-182) ────────────────────────────────> Graceful cascade degradation
Phase 5 (WG-183..184) ──── parallel ──────────────> DevX issues closed
```

**Recommended execution order**: Phase 1 first (release drift is an automated alert and visible regression); Phase 2-3 in parallel; Phase 4-5 last.

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| Unreleased commits | 94 | 0 (after v0.1.33 release) |
| Open issues | 3 (#674, #652, #653) | 0 |
| Clippy `--all-features` | 5 findings | 0 |
| Files >500 LOC (prod) | 1 | 0 |
| Scheduled CI green | 1/3 (Fuzzing only) | 3/3 |
| Mutation testing signal | None (always cancelled) | Completes in <2h with results |

---

## References

- Previous analysis: `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-14.md`
- ADR-058: CI Health (Gitleaks, Release Drift)
- ADR-057: CI Health (PR #616 clippy, nightly timeout)
- Issue #674: Release drift
- Issue #652: llms.txt automation
- Issue #653: VERSION file evaluation
