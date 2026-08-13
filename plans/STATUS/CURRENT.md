# Project Status — Self-Learning Memory System

**Last Updated**: 2026-08-12
**Released Version**: v0.1.39 (latest tag)
**Workspace Version**: 0.1.40 (post-v0.1.39 bump)
**Edition**: Rust 2024  
**Active plan**: merged #952 (2026-08-13) — ADR-082 + ADR-025/054 canonicalization landed; no in-flight code plan; ADR-080/081/082 lifecycle acceptance remains an external-maintainer item
**Branch**: main @ `9c8bfa79` (PR #952 merged 2026-08-13)

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | none (docs-only tracker PR #949 transient) |
| Open issues | None — #913 (Nix CI eval) closed 2026-08-02 |

## Recent completed (2026-08-12 — feedback-to-ranking adaptation + ADR registry)

| Wave | Result |
|------|--------|
| 2026-08-13 merge (#952) | ✅ squash-merged `9c8bfa79` by the controller; trackers re-pointed; ADR-082 stays `Proposed` pending maintainer acceptance |
| Feedback-to-ranking (ADR-082, Proposed) | ✅ derived per-pattern Wilson weight; capability-gated `list_recommendation_*` (Turso+redb); recommend re-rank (overfetch→boost→truncate); tracker-authoritative merge (stale durable rows don't shadow fresh feedback); e2e `ranking_adaptation_e2e.rs` 7/7 |
| Backend contracts | ✅ redb/turso `capability_attribution_test.rs` extended: `supports_ranking_adaptation` true + list round-trip |
| ADR 025/054 canonicalization (G-P1-8) | ✅ aliases moved to `plans/adr/_aliases/`; `validate-plans.sh --identifiers` now 51 unique, no duplicate warning |
| Trackers | ✅ GOALS/ACTIONS/GOAP_STATE/ROADMAP_ACTIVE/GAP_ANALYSIS updated; ADR-082 recorded |

## Recent completed (2026-08-11 — same-run fast gate + attribution truth)

| Wave | Result |
|------|--------|
| Same-run CI fast gate | ✅ `commitlint` + `fast-gate` run inside `ci.yml`; `test`/`mcp-build`/`multi-platform` depend on them; `ci-required-evaluate.sh` accepts only `success` and rejects `skipped` |
| Waiter/anchor removal (ADR-079 stage 5) | ✅ `quick-check.yml` + `pr-check-anchor.yml` deleted; cross-workflow waiters removed from coverage/security/benchmarks/file-structure |
| ADR-080/081 attribution closure | ✅ episode-existence validation, checked manual receipts, fallible playbook retrieval, split tracker modules, cold-restart + capability + postcard-safety tests, MCP/CLI truthful receipts |
| Docs/plan truth | ✅ `API_REFERENCE`/`PLAYBOOKS_AND_CHECKPOINTS`/`attribution::mod` ranking claims corrected; ADR-079/081 code-evidence + status updated; wave files marked historical |
| 2026-08-12 merge | ✅ PR #947 merged (squash `872949b8`) by the controller; trackers re-pointed to post-closure main |

## Recent completed (2026-08-11 — capability truth + dependabot parity)

| Wave | Result |
|------|--------|
| PR #940 ADR-081 capability truth | ✅ Merged 2026-08-11; non-advertising backends now yield `MemoryOnly`, never `Persisted` |
| PR #938 CIT-A2 dependabot parity | ✅ Dependabot + `CI / Required` gate enforced |

## Recent completed (2026-08-09 — fuzz nightly + LTO-off wave)

| Wave | Result |
|------|--------|
| PR #934 fuzz nightly toolchain + gitleaksignore + v0.1.39 bump | ✅ Merged 2026-08-09; fuzz workflow green on branch dispatch (`success`, no `__sancov_gen_` link errors) |

## Recent completed (2026-08-07 — PR review & CI fix wave)

| Wave | Result |
|------|--------|
| #928 commit messages | ✅ 5 long-body commits rewrapped (≤100 chars), 2 no-op commits dropped, commitlint 6/6 clean |
| #927 release drift | ✅ `commit_limit` deadlock broken via `release-preparation` label; v0.1.38 shipped 2026-08-08, v0.1.39 released (current tag) |
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
| First-party merge gate | ✅ **Live** — ruleset `9591004` requires `Codacy Static Code Analysis` + `CI / Required` (strict policy); the required aggregate is causally same-run (merged #947 2026-08-11→12) |
| CI fast-gate topology | ✅ **Same-run** — `commitlint` + `fast-gate` inside `ci.yml`; `ci-required-evaluate.sh` accepts only `success`, rejects `skipped`/`cancelled`/`timed_out`; waiter/anchor topology deleted (ADR-079 stage 5) |
| P0 plan gaps | **0 open code-side** — live ruleset required aggregate in place; remaining P0 evidence (ADR-079 stage 4 live fault-inject proof) is maintainer-external |
| ADR-079 CI control plane | **Accepted** — stage 3 live (ruleset requires `CI / Required`); stage 5 cleanup merged in #947 (2026-08-12); stage 4 fault-injection merge-block proof remains external maintainer evidence |
| ADR-080 automatic attribution | ✅ #927 merged + #930 test extension + #947 evidence (episode validation, checked receipts, cold-restart tests) |
| ADR-081 §2 capability truth | ✅ capability advertisement + capability-gated receipts (2026-08-10); #947 adds capability tests for all concrete backends |

## Immediate priorities

| Priority | Item | ID | Status |
|----------|------|-----|--------|
| P0 | Same-run required aggregate + skip-hardening (ADR-079 stage 5) | ADR-079 / CIT-A1 | ✅ #947 — fast gate + commitlint same-run; evaluator rejects skipped; waiter/anchor removed |
| P0 | Deliver live fault-injection merge-block proof | ADR-079 stage 4 | ⏸ external maintainer evidence — deliberately NOT performed in #947 |
| P0 | Fail closed and restore Dependabot/fork assertion parity | CIT-A2 | ✅ waiters fail closed + downstream actor parity (2026-08-10) |
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
