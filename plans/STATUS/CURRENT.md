# Project Status — Self-Learning Memory System

**Last Updated**: 2026-08-07
**Released Version**: v0.1.38 (latest tag)
**Workspace Version**: 0.1.38 (next release)
**Edition**: Rust 2024  
**Active plan**: `plans/GOAP_PR_REVIEW_CI_FIX_WAVE_2026-08-07.md` on top of `plans/GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md` + `plans/GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`
**Branch**: `main` @ `fe623250`

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | None — #927/#928/#930 merged; v0.1.38 release-docs bump in flight |
| Open issues | #913 Nix CI evaluation |

## Recent completed (2026-08-07 — PR review & CI fix wave)

| Wave | Result |
|------|--------|
| #928 commit messages | ✅ 5 long-body commits rewrapped (≤100 chars), 2 no-op commits dropped, commitlint 6/6 clean |
| #927 release drift | ✅ `commit_limit` deadlock broken via `release-preparation` label (v0.1.38 ship remains TODO) |
| #927 Codecov patch | ✅ receipt matrix + MCP envelope tests + CLI render dedup (`attribution_output`) |
| Main cancelled runs | ✅ Skill Evals + Performance Benchmarks re-run |
| Memory CLI validation | ✅ 4 episodes learned; `pattern recommend --episode-id` receipt `Persisted` e2e |
| #930 receipt-matrix extension | ✅ `failed_backends` ordering + no-op re-persist tests merged |

## Snapshot

| Area | State |
|------|--------|
| Release **v0.1.37** | ✅ Tagged and shipped |
| Post-release workspace **0.1.38** | ✅ `92db07bf` |
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
| First-party merge gate | **Absent** — ruleset requires Codacy + CodeQL policy only; `CI / Required` context emitted (2026-08-06) but not yet required |
| CI wait semantics | ✅ **Fail-closed** — five waiters reject cancelled/skipped/missing; commit lint waited on |
| P0 plan gaps | **1 open** — live ruleset required aggregate (ADR-079 acceptance); PTA + CIT-A1/A2/A3 workflow side + CIT-A4/A5 done |
| ADR-079 CI control plane | Proposed; workflow aggregate + fail-closed waiters + semantic validator implemented 2026-08-06; ruleset unchanged |
| ADR-080 automatic attribution | ✅ #927 merged + #930 test extension |

## Immediate priorities

| Priority | Item | ID | Status |
|----------|------|-----|--------|
| P0 | Add causal same-run aggregate, then stage into ruleset with approval | ADR-079 / CIT-A1 | 🔄 aggregate done; ruleset stage pending (maintainer) |
| P0 | Fail closed and restore Dependabot/fork assertion parity | CIT-A2 | 🔄 waiters fail closed; downstream actor parity pending |
| P0 | Return typed unavailable/absent API for non-`csm` cascade | PTA-A1 | ✅ Implemented |
| P0 | Remove or label fabricated CLI storage telemetry | PTA-A2 | ✅ Implemented |
| P1 | Reconcile gate contract | CIT-A3 | ✅ semantic validator + negative fixtures (2026-08-06) |
| P1 | Repair release/publish/fuzz truth | CIT-A4/A5 | ✅ Implemented (2026-08-06) |
| P1 | Automatic episode-bound recommendation attribution | ADR-080 / RAT-A1…A7 | ✅ #927 merged + #930 test extension |
| P1 | Hide unsupported `eval set-threshold` command | PTA-A3 | ✅ Implemented |
| P2 | Research/product spikes (R-F1…R-F7, R-F10) | R-F* | ⏸ DEFER |
| P2 | Transitive Dependabot advisories | G-P1-9 | Monitor / upstream |

## Recent completed (2026-08-06)

| Wave | Result |
|------|--------|
| CIT-A4 release/publish trigger truth (ACT-338) | ✅ release `workflow_dispatch` removed; publish `--locked`, bounded polling, dependency closure |
| CIT-A5 durable fuzz evidence (ACT-339) | ✅ fuzz crash artifacts always uploaded + non-green signal; mutants already durable |
| R-F10 OIDC / R-F4 SIMD plan truth | ✅ ACT-325/326 confirmed shipped; trackers refreshed |

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
