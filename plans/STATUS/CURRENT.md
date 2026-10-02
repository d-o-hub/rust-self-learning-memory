# Project Status — Self-Learning Memory System

**Last Updated**: 2026-10-02
**Released Version**: v0.1.44 (release pending tag)
**Workspace Version**: 0.1.44 (matches release)
**Edition**: Rust 2024  
**Active plan**: v0.1.44 prepared 2026-10-02 — checked completion receipts (#1080 → #1107), CLI drain-and-verify (#1081 → #1112), pattern-search input bounds (#1113), clippy 1.99 migration (#1108), architecture/status refresh (#1094 → #1110), coverage-floor reconciliation (#1090 → #1117); drift tracked by the open release-drift issue and the release-pipeline proposal (#1109) is open; ADR-080/081/082 lifecycle acceptance remains an external-maintainer item
**Branch**: main @ `1fc97118` (PR #1117 merged 2026-10-02)

## Open tracker (live)

| Kind | Items |
|------|--------|
| Open PRs | run `gh pr list --state open` (counts deliberately not pinned; `validate-plans.sh --tracker-drift` guards this header) |
| Open issues | run `gh issue list --state open` — drift is tracked by the open release-drift issue and the pipeline proposal by its own issue |

## Recent completed (2026-10-02 — v0.1.44 prepared)

| Wave | Result |
|------|--------|
| Pattern-search input bounds (#1113) | ✅ `search_patterns`/`recommend_patterns` clamp `limit`/`min_relevance` and truncate oversized `query`/`task_description`/`domain`/`tags` (CWE-770) |
| Architecture refresh (#1094 → #1110) | ✅ architecture/serialization/status evidence aligned with v0.1.44; bincode references corrected to postcard |
| Coverage floor (#1090 → #1117) | ✅ 70% is the blocking floor, 90% the aspirational target, across AGENTS.md, GATE_CONTRACT, skills and docs, with comparator unit tests |
| v0.1.44 release | ⏳ prepared in this PR — changelog `[0.1.44]` + version docs; ship via release-guard after main CI is green |

## Recent completed (2026-10-02 — durability receipts + toolchain hygiene)

| Wave | Result |
|------|--------|
| Checked completion receipts (#1080 → #1107) | ✅ `complete_episode_checked` returns `Local`/`Committed`/`Queued` sourced from live queue stats; legacy `complete_episode` unchanged (ADR-075 D2); `flush` errors name the permanently failed episode IDs; patch coverage raised 76.8% → 94.8% with embedding happy-path and provider-failure tests |
| CLI durable drain (#1081 → #1112) | ✅ `episode complete|fail` drain a `Queued` write with bounded `--durable-timeout-secs` (default 30) before any success output, exit non-zero naming the episode on timeout/permanent failure, and report the final `durability` (`committed`/`local`) in human/JSON/YAML |
| Clippy 1.99 migration (#1108) | ✅ the floating `stable` bump added `clippy::assert_is_empty`; ~100 pre-existing sites migrated (`assert_eq!`/`assert_ne!` on `.len()`), LESSON-029 records the trap |
| Release-pipeline proposal (#1109) | 📋 issue opened: OIDC trusted publishing without the token fallback, artifact + SBOM attestations, draft-first immutable releases, Pages boundary documented, publish-job de-duplication |

## Recent completed (2026-10-01 — v0.1.43 shipped)

| Wave | Result |
|------|--------|
| v0.1.43 release | ✅ tag `v0.1.43` on `a0078d0a`; GitHub Release with dist artifacts for five targets; drift issue #1098 auto-closed by the tag-triggered check; workspace bumped to 0.1.44 |
| LOC ceiling regression (#1103) | ✅ the oversized `server_impl` test modules split into sibling `*_tests.rs` files (source files back under the ceiling) and `check-loc.sh` now runs in the File Structure Validation workflow, so PR CI enforces what previously only the pre-commit sensor and the release gate caught |

## Recent completed (2026-09-30 — v0.1.43 prepared)

| Wave | Result |
|------|--------|
| MCP OAuth 2.1 enforcement (#1082) + trusted rate-limit identity (#1084) | ✅ #1101 — tokens verified (signature/issuer/audience/expiry/scope) before method dispatch, fail-closed startup without a secret, buckets keyed by the validated subject or the process identity, bounded identity cardinality |
| redb fail-closed schema handling (#1069) | ✅ #1096 — a mismatched non-empty database is preserved and fails closed with `Error::SchemaMigrationRequired`; `reset_all_tables` is the only clearing path |
| Embedding activation identity (#1072) | ✅ #1097 — serialized activation revisions, provider-identity-bound ANN indexes, blank identities rejected |
| MCP `tools/list` full registry (#1083) | ✅ #1100 — default listing enumerates every registered tool with schemas; lazy stubs keep `tools/describe` identical |
| Storage pool ownership (#1060-#1062) | ✅ #1095 — adaptive/caching/keep-alive guards own their state; no raw-pointer ownership or unnecessary `unsafe` |
| MCP fail-closed docs reconciliation | ✅ #1099 — security/architecture docs, skills and doc-integrity checks aligned with the shipped fail-closed behavior |

## Recent completed (2026-09-27 — v0.1.42 shipped)

| Wave | Result |
|------|--------|
| v0.1.42 release | ✅ tag `v0.1.42` on `47a07a0b`; GitHub Release with dist artifacts for five targets; drift issue #1048 auto-closed by the tag-triggered check; workspace bumped to 0.1.43 |
| Evidence classification confidence gating (#1053) | ✅ each evidence rule gated on its own dimension's confidence; shipped in v0.1.42 (4 new regressions; 47/47 with `--features csm`) |
| Changelog automation (#1054) | closed — the curated 0.1.42 notes (#1052/#1053) take precedence over the git-cliff rewrite |

## Recent completed (2026-09-24 — retrieval judgment + rerank + merge tooling)

| Wave | Result |
|------|--------|
| Typed semantic judgment (#1030 → PR #1041) | ✅ `RetrievalJudge` interface, typed atomic judgments, strict ID/score validation with ID-join alignment, optional `CascadeRetriever::with_judge`, bounded `JudgmentOutcome` telemetry; merged `d4d57a63`-era, 6 commits |
| Semantic shortlist rerank (#1031 → PR #1042) | ✅ opt-in `SemanticRerankConfig` + deterministic min-max/relevance fusion, single `finish_ranked` path over all local-success branches, `RerankStatus` telemetry, offline `--rerank` eval comparison (judge calls/query 0.73, candidates/query 5.93, regression PASSED); merged `1461d61d` |
| Merge/coverage tooling (PR #1046) | ✅ `merge-pr.sh` gated merge path, `validate-plans.sh --tracker-drift`, `coverage-waivers` skill (8/8 evals) |
| Harness issues | 8 filed upstream in `d-o-hub/do-harness` (#238–#245) from this wave's friction |

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
