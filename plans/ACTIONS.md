# GOAP Actions Backlog

- **Last Updated**: 2026-08-11
- **Active plan**: closure PR from branch `fix/ci-attribution-truth-closure` (PR number / head SHA recorded by the controller after creation). Prior waves (`GOAP_PR_REVIEW_CI_FIX_WAVE_2026-08-07.md`, `GOAP_CIT_A1_A2_A3_WORKFLOW_WAVE_2026-08-06.md`, `GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md`, `GOAP_ADR081_CAPABILITY_TRUTH_2026-08-10.md`) are historical completed slices. Upstream: `plans/GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`.
- **Archived plans**: `plans/archive/2026-07-consolidation/`

## Completed actions (2026-08-11 — closure PR)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-334 | Accept ADR-079 and freeze the `CI / Required` aggregate contract | CIT-A1 | ✅ Accepted — ruleset `9591004` requires `[Codacy Static Code Analysis, CI / Required]` (verified live) |
| ACT-335 | Implement and fault-inject same-run required aggregation | CIT-A1 | ✅ closure PR — same-run `commitlint` + `fast-gate`; `ci-required-evaluate.sh` accepts only `success`, rejects `skipped`/`cancelled`/`timed_out`/`failure`/missing/unknown; `--required-aggregate` fixtures |
| ACT-340 | Require the verified aggregate in ruleset `9591004` and validate blocking | CIT-A1/PTA-A9 | ✅ ruleset already requires `CI / Required` (stage 3); deliberate live fault-injection merge-block proof (stage 4) remains external maintainer evidence |
| ACT-348 | Validate episode existence; unify malformed-ID rejection across core/MCP/CLI | RAT-B4 | ✅ evidence-backed / landed in closure PR |
| ACT-349 | Add fallible playbook retrieval; no session on generation failure | RAT-B5 | ✅ evidence-backed / landed in closure PR |
| ACT-350 | Add `persist_feedback_checked`; give manual MCP/CLI commands receipt semantics | RAT-B6 | ✅ evidence-backed / landed in closure PR |
| ACT-351 | Declare `episode_id` in both MCP registries + registry-agreement test | RAT-B7 | ✅ evidence-backed / landed in closure PR |
| ACT-352 | Deduplicate CLI rendering; replace the `too_many_arguments` suppression with a request struct | RAT-B8 | ✅ evidence-backed / landed in closure PR |
| ACT-353 | Restart-safety, receipt-matrix, MCP snapshot, and CLI e2e tests to ≥ 90% | RAT-B9 | ✅ evidence-backed / landed in closure PR (coverage % measured by the controller's final validation) |
| ACT-354 | Docs + plans-registry + repo hygiene (`API_REFERENCE`, ADR-058 duplicate, `.gitignore`) | RAT-B10 | ✅ evidence-backed / landed in closure PR |

## Completed actions (2026-08-09 — fuzz nightly + LTO-off wave)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| T1 | Build all 3 fuzz targets locally with LTO off (`CARGO_PROFILE_RELEASE_LTO=false`) | G1 | ✅ Finished, no link error |
| T2 | Independent LTO root-cause review | G1 | ✅ nightly toolchain + `lto="fat"` config vs `-Zsanitizer` |
| T4 | Add `CARGO_PROFILE_RELEASE_LTO: false` to fuzz.yml job env | G1 | ✅ merged in #934 |
| T6 | Re-trigger fuzz workflow on branch | G1 | ✅ `success` (2026-08-09 17:20Z) |
| T9 | Merge #934 | G2 | ✅ merged 2026-08-09 |

## Active actions (2026-08-07 — PR review & CI fix wave)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-355 | Review/roast open PRs #928 + #927; fix all failing CI incl. pre-existing | GOAP | ✅ 2026-08-07 (see wave plan) |
| ACT-356 | Repair #928 commit messages (rewrap bodies ≤100, drop no-op commits) | commitlint | ✅ pushed; CI green |
| ACT-357 | Break #927 drift deadlock via `release-preparation` label | drift | ✅ Release Drift Check green |
| ACT-358 | Raise #927 Codecov patch coverage (receipt matrix + MCP + CLI dedup) | RAT-B8/B9 | ✅ pushed `68457631`→`52276c50` |
| ACT-359 | Ship v0.1.38 via release-guard to clear repo-wide drift for all PRs | R-A3 | ✅ shipped 2026-08-08 (workspace bumped to 0.1.39) |

## Active actions (2026-08-06)

| ID | Action | Rec | Status |
|----|--------|-----|--------|
| ACT-334 | Accept ADR-079 and freeze the `CI / Required` aggregate contract | CIT-A1 | ✅ Accepted (ruleset `9591004` requires `CI / Required`) — see 2026-08-11 closure table |
| ACT-335 | Implement and fault-inject same-run required aggregation | CIT-A1 | ✅ closure PR — same-run gates + fail-closed evaluator — see 2026-08-11 closure table |
| ACT-336 | Fail closed on cancellation/missing/commitlint and restore Dependabot/fork assertion parity | CIT-A2 | ✅ waiters fail closed + commit-lint wait + downstream actor parity (2026-08-10) |
| ACT-337 | Reconcile test/Clippy/quality scopes and add semantic gate-contract fixtures | CIT-A3 | ✅ semantic validator + negative fixtures + actor-parity/ruleset-context fixtures (2026-08-10) |
| ACT-338 | Remove broken release dispatch and make publish selection/dependency planning truthful | CIT-A4 | ✅ Done (2026-08-06) |
| ACT-339 | Preserve fuzz/mutation evidence, then measure and remove duplicate CI work | CIT-A5 | ✅ Done (fuzz half; mutants already durable — 2026-08-06) |
| ACT-340 | With approval, require the verified aggregate in ruleset `9591004` and validate blocking | CIT-A1/PTA-A9 | ✅ ruleset already requires `CI / Required` (stage 3); stage 4 live fault-inject proof = external maintainer evidence |
| ACT-302 | `./scripts/release-manager.sh ship --execute` for `v0.1.36` | R-A1 | ✅ Done |
| ACT-303 | Post-release workspace bump to 0.1.37 | R-A2 | ✅ #886 |
| ACT-315 | Plans progress truth (open PRs, post-ship) | R-G* | ✅ #889 |
| ACT-316 | Land #887 changelog hygiene | docs | ✅ #887 merged |
| ACT-317 | Review/merge #888 cosine perf | perf | ✅ #888 merged |
| ACT-318 | Mark ADR-074 as Accepted / Implemented | docs | ✅ Done (#891) |
| ACT-319 | Gap analysis tasks: pattern extract + ADR-074 docs | G-P1-12/ACT-317/318 | ✅ #891 merged |
| ACT-320 | R-F8 CLI relationship show polish | R-F8 | ✅ #893 merged |
| ACT-321 | R-F9 HNSW persistence + capacity eviction | R-F9 | ✅ #893 merged |
| ACT-322 | Add 6 domain skills (40 total, all routed) | skills | ✅ #894 |
| ACT-323 | ADR-077 A1-A5 runtime embedding activation | ADR-077 | ✅ main (`9ef4b742`, `e0f7f712`) |
| ACT-324 | ADR-077 A6 validate/document/gate (docs + concurrency + zero-unsafe redaction tests) | ADR-077 | ✅ #897 merged |
| ACT-312 | R-F* GO spike artifacts written + validated (2026-07-28) | R-F* | ✅ Done |
| ACT-325 | Implement R-F10 OIDC trusted publishing in publish-crates.yml | R-F10 | ✅ Done (`id-token: write` + OIDC exchange; plans refreshed) |
| ACT-326 | Implement R-F4 SIMD cosine acceleration + benchmark variants | R-F4 | ✅ Done (`cosine_similarity_simd` + simd bench variant) |
| ACT-341 | Make non-`csm` cascade retrieval capability-truthful | PTA-A1 | ✅ Implemented |
| ACT-342 | Make CLI storage metrics measured/estimated/unavailable explicitly | PTA-A2 | ✅ Implemented |
| ACT-343 | Hide unsupported `eval set-threshold` command | PTA-A3 | ✅ Implemented |
| ACT-328 | Accept ADR-080 and freeze attributed contracts | RAT-A1 | Proposed |
| ACT-329 | Add capability-aware checked attribution persistence | RAT-A2 | Blocked by ADR acceptance |
| ACT-330 | Add core attributed pattern/playbook operations | RAT-A3 | Blocked by ACT-329 |
| ACT-331 | Enforce session and feedback integrity | RAT-A4 | Blocked by ACT-330 |
| ACT-332 | Wire optional attribution through MCP and CLI | RAT-A5/A6 | Blocked by ACT-331 |
| ACT-333 | End-to-end validation, docs, and authority update | RAT-A7/PTA-A9 | Blocked by ACT-341…343 + RAT chain |

All ACT-300…ACT-324 items are complete. ACT-341…ACT-343 (PTA-A1/A2/A3) are
implemented 2026-08-01. ACT-325/326 (R-F10/R-F4) and ACT-338/339 (CIT-A4/A5)
are implemented 2026-08-06. ADR-080/081 attribution merged in #927 (receipt-matrix
tests #930); the ADR-081 §2 capability-truth gap closed 2026-08-10 via
`supports_recommendation_attribution` + capability-gated `persist_session_checked`.
The closure PR from branch `fix/ci-attribution-truth-closure` lands the same-run
fast gate, the fail-closed evaluator, the waiter/anchor removal (ADR-079 stage 5),
and the ADR-080/081 acceptance evidence (ACT-348…354, ACT-334/335/340). Remaining
open items are maintainer-external: ADR-079 stage 4 live fault-injection
merge-block proof, and ADR-080/081 lifecycle acceptance (both stay `Proposed`).

## Completed actions (summary)

All ACT-190…ACT-279 series and 2026-07 recommendation waves are **complete**.  
Full tables: `plans/archive/2026-07-consolidation/completed-sprints/`

### Prevention permanently (do not regress)

- Never `#[serde(tag=)]` on postcard types  
- StorageBackend new methods → all backends  
- CLI path flags → set `redb_path`  
- Cross-process storage features → e2e CLI test  
- No manual `gh release create`; use release-manager + `release.yml`  
- No soft-pass on cargo deny / required cancelled checks  
- Required status must be a causal same-run aggregate; an echo anchor is not a gate
- Dependabot/fork trust changes permissions and secret access, not test assertions
- Fail-closed `execute_agent_code` unless approved capability backend  
- sha2 digests: use portable hex encode (not `format!("{:x}", finalize())` on 0.11+)  
- Docs integrity: do not re-check `plans/archive/**` link rot as a ship blocker  
- After tag `vX.Y.Z`, immediately bump workspace to next patch before more feat/fix commits  
- Commit bodies must stay ≤ 100 chars; repair long bodies mechanically with `git filter-branch --msg-filter 'fold -s -w 100'` and verify with `npx commitlint --from <base> --to HEAD --verbose`
- No-op `chore(ci): re-trigger workflow runs` commits are lint-noise — drop them via rebase, never push them
- Pre-existing repo-wide release drift blocks every PR: fix the root cause (ship the release), use the `release-preparation` label only as a documented deadlock breaker
- Codecov patch coverage: dedupe duplicated rendering (removes uncovered lines from the denominator) AND add targeted tests for new core paths
