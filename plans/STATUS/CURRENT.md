# Project Status — Self-Learning Memory System

**Last Updated**: 2026-07-27  
**Released Version**: v0.1.36  
**Workspace Version**: 0.1.37 (release preparation in progress)  
**Edition**: Rust 2024  
**Active plan**: `plans/GOAP_RUNTIME_EMBEDDING_ACTIVATION_2026-07-26.md` (A1-A6 complete)  
**Branch**: `main` @ `45c6eee8`

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | *(none)* |
| Open issues | *(none)* |

## Snapshot

| Area | State |
|------|--------|
| Release **v0.1.36** | ✅ Shipped 2026-07-22 — [release](https://github.com/d-o-hub/rust-self-learning-memory/releases/tag/v0.1.36) |
| Post-release workspace **0.1.37** | ✅ #886 merged |
| Recommendations + F4 + skill contracts | ✅ #878 |
| Medium-risk skill evals (R-E2) | ✅ #883 |
| Docs integrity ship gate | ✅ #885 |
| Production LOC >500 (non-test `src`) | ✅ Clean |
| Skill evals / routes | 40/40 |
| R-F8 relationship info show polish | ✅ #893 |
| R-F9 HNSW persistence + eviction | ✅ #893 |
| 6 new domain skills added | ✅ #894 |
| ADR-077 runtime embedding activation (A1-A5) | ✅ main (`9ef4b742`, `e0f7f712`) |
| ADR-077 A6 validate / document / gate | ✅ #897 merged |
| Code execution | Fail-closed (S1.1c NO-GO) |
| MCP provenance (`with_provenance`) | ✅ |
| P0 plan gaps | **None open** |

## Immediate priorities

| Priority | Item | ID | Status |
|----------|------|-----|--------|
| P0 | Cut v0.1.37 release (31 commits ready) | release | 🚀 IN PROGRESS |
| P2 | Research/product spikes (R-F1…R-F7, R-F10) | R-F* | ⏸ DEFER |
| P2 | Transitive Dependabot advisories | G-P1-9 | Monitor / upstream |

## Recent completed (2026-07-26…27)

| Wave | Result |
|------|--------|
| PR queue cleanup (GOAP swarm orchestration) | ✅ 5 PRs → 0 open |
| cargo-mutants workspace path fix #901 | ✅ Merged (fixes #898) |
| Dependabot actions-all #902 | ✅ Merged |
| Dependabot rust-patch-minor (14 updates) #903 | ✅ Merged |
| Dependabot rust-major (serial_test, base64, jsonwebtoken) #904 | ✅ Merged |
| Duplicate PR #899 closed (superseded by #901) | ✅ Closed |
| Ship v0.1.36 + GitHub Release artifacts | ✅ |
| Post-bump 0.1.37 #886 | ✅ Merged |
| Docs integrity unblock #885 | ✅ Merged |
| R-E2 medium-risk skill evals #883 | ✅ Merged |
| Release docs #880 / rust-major #877 / tracker #881 | ✅ Merged |
| Recommendations #878 | ✅ Merged |
| R-F8 CLI relationship panel + R-F9 HNSW #893 | ✅ Merged |
| 6 new domain skills (40 total, all routed) | ✅ Merged |
| ADR-077 runtime embedding activation A1-A5 | ✅ Merged (main) |
| ADR-077 A6 validate / document / gate | ✅ #897 merged |

## Canonical companions

- Roadmap: `plans/ROADMAPS/ROADMAP_ACTIVE.md`
- Goals / actions / GOAP: `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`
- Gaps: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- Validation: `plans/STATUS/VALIDATION_LATEST.md`
- Archive: `plans/archive/2026-07-consolidation/`
