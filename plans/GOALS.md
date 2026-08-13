# GOAP Goals Index

- **Last Updated**: 2026-08-11
- **Status**: closure PR (`fix/ci-attribution-truth-closure`) implements the same-run CI fast gate + ADR-080/081 attribution closure; v0.1.39 shipped; workspace bumped to 0.1.40; ADR-080/081 remain `Proposed` pending maintainer acceptance; ADR-079 stage 4 live fault-inject proof is external maintainer evidence
- **Workspace**: `0.1.40` · **Tag**: `v0.1.39`
- **Plan**: closure PR from branch `fix/ci-attribution-truth-closure` (PR number / head SHA recorded by the controller after creation)
- **Archive**: `plans/archive/2026-07-consolidation/`

## Closed this wave (2026-08-09)

| Goal | Status |
|------|--------|
| G1 fuzz_workflow_green (nightly toolchain + LTO-off) | ✅ fuzz workflow `success` on branch dispatch (#934) |
| G2 pr_934_mergeable | ✅ #934 merged 2026-08-09 |

## Closed this wave (2026-08-07)

| Goal | Status |
|------|--------|
| Repair #928 commit messages (commitlint clean, no-op commits dropped) | ✅ pushed |
| Unblock #927 pre-existing release drift (`release-preparation` deadlock breaker) | ✅ drift check green |
| Raise #927 Codecov patch coverage (receipt matrix + MCP envelope tests + CLI dedup) | ✅ pushed; re-measuring |
| Main cancelled CI runs re-run + plans/learnings refreshed | ✅ |

## Active goals (2026-08-06)

| Goal | Rec IDs | Priority | Status |
|------|---------|----------|--------|
| Make first-party validation causally merge-required | CIT-A1 / ADR-079 | P0 | ✅ same-run fast gate + `commitlint` in `ci.yml`; `ci-required-evaluate.sh` accepts only `success`; ruleset `9591004` requires `CI / Required` (closure PR); stage 4 fault-inject proof = external maintainer evidence |
| Fail closed across cancellation, missing checks, commit lint, Dependabot, and forks | CIT-A2 | P0 | ✅ waiters fail closed + commit-lint wait + downstream actor parity (2026-08-10) |
| Reconcile local/CI gate scope and semantic drift validation | CIT-A3 | P1 | ✅ semantic validator + negative fixtures + ruleset-context/Actor-parity fixtures (2026-08-10) + `--required-aggregate` fixtures (closure PR) |
| Make release/publish/fuzz automation truthful and observable | CIT-A4/A5 | P1 | ✅ Implemented (2026-08-06) |
| Make disabled cascade capability truthful | PTA-A1 | P0 | ✅ Implemented |
| Make storage metrics provenance-truthful | PTA-A2 | P0 | ✅ Implemented |
| Remove unsupported threshold command from CLI help | PTA-A3 | P1 | ✅ Implemented |
| Capture episode-bound recommendation attribution automatically | RAT-A1…A7 / ADR-080 | P1 | ✅ code-side closed in closure PR (#927 + #930 + episode validation + checked receipts + cold-restart tests); ADR-080 stays Proposed pending maintainer acceptance |
| Advertise and enforce attribution persistence capability | ADR-081 §2 | P1 | ✅ `StorageBackend::supports_recommendation_attribution` + capability-gated receipts (2026-08-10) + concrete-backend capability tests (closure PR) |
| Design idempotent feedback-to-ranking updates | ADR-082 | P2 | ✅ code-side in this PR — derived Wilson weight, capability-gated `list_recommendation_*` read surface, recommend re-rank, e2e tests (ADR-082 Proposed; lifecycle = maintainer) |
| R-F10 OIDC trusted publishing (publish-crates.yml) | R-F10 | P2 | ✅ Implemented (ACT-325) |
| R-F4 SIMD cosine acceleration + benchmark variants | R-F4 | P2 | ✅ Implemented (ACT-326) |
| Optional research/product spikes (R-F1…R-F3, R-F5…R-F7) | R-F* | P3 | ⏸ DEFER |

The ranking-learning loop is now closed code-side (ADR-082, life cycle Proposed):
attribution feedback derives a durable per-pattern learned weight (Wilson lower
bound) and the recommendation path re-ranks its candidate pool by base relevance
plus that weight; generic search/discovery/retrieval are unchanged. First-party CI
is now causally merge-required (ruleset `9591004` requires `CI / Required`, and the
aggregate is same-run fail-closed); ADR-079 stage 4 (deliberate live
fault-injection merge-block proof) remains external maintainer evidence.
R-F10 (ACT-325) and R-F4 (ACT-326) are implemented; CIT-A4/A5 (ACT-338/339)
implemented 2026-08-06; PTA-A1/A2/A3 implemented.

## Closed this wave (2026-08-06)

| Goal | Status |
|------|--------|
| CIT-A4 release/publish trigger truth | ✅ (2026-08-06) |
| CIT-A5 durable fuzz evidence + non-green signal | ✅ (2026-08-06) |
| R-F10 OIDC trusted publishing (ACT-325) | ✅ (already shipped; plans refreshed) |
| R-F4 SIMD cosine + bench variants (ACT-326) | ✅ (already shipped; plans refreshed) |

## Closed this wave (2026-07-20…25)

| Goal | Status |
|------|--------|
| Ship v0.1.36 (R-A1) | ✅ |
| Post-release bump 0.1.37 (R-A2) | ✅ #886 |
| R-E2 medium-risk skill evals | ✅ #883 |
| Docs integrity ship gate | ✅ #885 |
| Recommendations R-B/C/D/E/G/H | ✅ #878 |
| Plans truth refresh | ✅ #889 |
| Changelog hygiene | ✅ #887 |
| Cosine perf (8-way unrolled) | ✅ #888 |
| Gap tasks (ADR-074 docs, G-P1-12 pattern extract) | ✅ #891 |
| R-F8 CLI relationship show polish + R-F9 HNSW persistence | ✅ #893 |
| 6 new domain skills (40 total, all routed) | ✅ #894 |
| ADR-077 runtime embedding activation A1-A5 | ✅ main (`9ef4b742`, `e0f7f712`) |
| ADR-077 A6 validate / document / gate | ✅ #897 merged |
| Full gap audit — 0 P0/P1 code gaps | ✅ 2026-07-25 |

## Completed goal series (pointer only)

| Series | Outcome | Archive |
|--------|---------|---------|
| Post-v0.1.36 sprint (all tasks) | ✅ Complete 2026-07-24 | — |
| Recommendations #878 | Merged | — |
| 2026-07-14 improvements S1/W2/K3/F4 | Implemented; S1.1c NO-GO | `archive/2026-07-consolidation/completed-sprints/` |
| v0.1.35 CLI UX + ADR-075/076 | Released | same |
| Harness + release-cadence-manager | Merged | same |
| v0.1.36 release campaign | Shipped 2026-07-22 | — |

Do not re-list completed WG tables here.
