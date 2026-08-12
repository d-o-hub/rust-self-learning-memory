# GOAP State Snapshot

- **Last Updated**: 2026-08-12
- **Version**: workspace `0.1.40` · latest tag `v0.1.39`
- **Branch**: main @ `872949b8` (PR #947 merged 2026-08-12)
- **Open PRs**: docs-only tracker PR (Step 8); #947 merged
- **Open issues**: none — #913 closed 2026-08-02
- **Active plan**: none in flight — #947 merged 2026-08-12; ADR-079 stage 4 + ADR-080/081 lifecycle await maintainer
- **Note**: #947 merged 2026-08-12; evidence in plans/STATUS/VALIDATION_LATEST.md.
- **Archive**: `plans/archive/2026-07-consolidation/`  
- **Release**: ✅ `v0.1.39` tagged and shipped (workspace bumped to `0.1.40` post-release)

---

## Phase: Analyze / decide — CI trust + product truth + attribution

| Package | Status |
|---------|--------|
| ADR-079 fail-closed required-check control plane | Accepted (P0) — stage 3 live (ruleset `9591004` requires `CI / Required`); stage 5 cleanup merged in #947 (2026-08-12); stage 4 live fault-inject proof = external maintainer evidence |
| CIT-A1 required aggregate + staged ruleset migration | ✅ same-run fast gate + commitlint; `ci-required-evaluate.sh` accepts only `success`; ruleset requires the aggregate (merged in #947, 2026-08-12) |
| CIT-A2 cancellation/actor fail-closed behavior | ✅ waiters fail closed + commit-lint wait + downstream Dependabot/fork actor parity (2026-08-10) |
| CIT-A3 semantic gate-contract parity | ✅ validator + negative fixtures + actor-parity/ruleset-context fixtures (2026-08-10) |
| CIT-A4 release/publish trigger truth | ✅ Done (2026-08-06) |
| CIT-A5 durable informational evidence + deduplication | ✅ Done (fuzz evidence; dedup measurement is follow-up) |
| PTA-A1 non-`csm` cascade capability truth | ✅ #916 merged (2026-08-02) | PTA-A1 |
| PTA-A2 CLI storage metric truth | ✅ #916 merged (2026-08-02) | PTA-A2 |
| PTA-A3 threshold command cleanup | ✅ #916 merged (2026-08-02) | PTA-A3 |
| cargo-mutants CI sharding (reward/retrieval/retry/patterns) | ✅ #917 merged (2026-08-02) | CI |
| ADR-080 automatic recommendation attribution | Proposed (code evidence merged in #947, 2026-08-12; lifecycle awaits maintainer acceptance) |
| RAT-A1…A7 contract tests + implementation | ✅ code-side merged in #947 (2026-08-12) (episode validation, checked receipts, fallible playbooks, cold-restart/capability/postcard tests) |
| Feedback-to-ranking adaptation | Deferred to separate ADR |
| PR queue cleanup (GOAP swarm) | ✅ 5 PRs → 0 open (2026-07-27) |
| cargo-mutants workspace fix #901 | ✅ Merged (fixes #898) |
| Dependabot batch #902/#903/#904 | ✅ Merged |
| R-A1 ship v0.1.36 | ✅ Released |
| R-A2 post-bump 0.1.37 | ✅ #886 |
| R-E2 medium-risk skill evals | ✅ #883 |
| Docs integrity ship unblock | ✅ #885 |
| Recommendations #878 | ✅ |
| Plans progress refresh | ✅ #889 |
| R-F8 CLI relationship show polish | ✅ #893 |
| R-F9 HNSW persistence + hardening | ✅ #893 |
| 6 new domain skills (40 total, all routed) | ✅ #894 |
| ADR-077 runtime embedding activation (A1-A5) | ✅ main (`9ef4b742`, `e0f7f712`) |
| ADR-077 A6 validate / document / gate | ✅ #897 merged |
| R-F* remaining product epics | ✅ GO spike artifacts: R-F1…R-F7, R-F10 (2026-07-28) |

---

## Closed campaigns (pointer)

| Campaign | Result |
|----------|--------|
| PR queue cleanup (GOAP swarm orchestration) | ✅ 2026-07-27 |
| v0.1.36 ship + post-bump | ✅ 2026-07-22…23 |
| Recommendations #878 | ✅ |
| F4 remainder / missing tasks / harness | ✅ #873/#874/#870 family |
| v0.1.35 release | ✅ |

Details: `plans/archive/2026-07-consolidation/completed-sprints/`.

---

## Goal-state flags (2026-07-24)

```text
truth_reconciled                  = true  (full plans refresh 2026-07-24; no open PRs)
sandbox_capability_boundary       = true
retrieval_identity_complete       = true  (ADR-074 Accepted/Implemented)
storage_awaits_lock_free          = true
durable_eviction                  = true
embedding_health_truthful         = true
retry_backpressure_effective      = true
gates_match_policy                = true  (ADR-079 stage 3 — live ruleset now requires the `CI / Required` aggregate, 2026-08-10; same-run fast gate + fail-closed evaluator landed + merged in #947 2026-08-12)
required_ci_causal                = true  (ruleset 9591004 requires Codacy + `CI / Required`; aggregate is causally same-run and rejects skipped, 2026-08-11)
ci_cancellation_fail_closed       = true  (five waiters fail closed + commit-lint wait; 2026-08-06)
automation_actor_parity           = true  (CIT-A2 — Dependabot/fork run same substantive assertions; validator fixture 2026-08-10)
fuzz_nightly_green                = true  (#934 — nightly toolchain + LTO-off; fuzz workflow success 2026-08-09)
release_dispatch_truthful         = true  (release.yml has no manual dispatch; publish --locked + bounded polling, ACT-338)
informational_ci_evidence_durable = true  (fuzz artifacts always() upload + visible non-green signal, ACT-339)
skill_evals_executable            = true
skill_routes_complete             = true
skill_evals_medium_depth          = true
docs_match_code                   = true
plan_registry_unique              ≈ true  (ADR 025/054 aliased)
feature_pilots_have_baselines     = true
release_current                   = true  (v0.1.39)
version_advanced_after_tag        = true  (workspace 0.1.40)
adr074_provenance_envelope        = true  (RetrievalProvenance + CacheKey all fields)
adr075_durable_complete           = true  (completion.rs hard-errors on backend failure)
adr076_pattern_ux                 = true  (empty diagnostics + sync messaging + pattern extract)
cosine_perf_merged                = true  (#888 merged — 8-way unrolled accumulators)
pattern_extract_command           = true  (ADR-076 §5 — G-P1-12, #891)
r_f8_relationship_show_polish     = true  (#893 — box-drawing panel + unit tests)
r_f9_hnsw_persistence             = true  (#893 — file_dump/load + capacity eviction)
skill_count_40_all_routed         = true  (checkpoint-handoff, embedding-ops, episode-relationships, episode-tags, playbook-ops, recommendation-feedback)
runtime_embedding_activation      = true  (ADR-077 Implemented A1-A6 — configure_embeddings activates exact provider; A6 docs + concurrency/redaction regression tests #897 merged)
cascade_capability_truthful       = true  (PTA-A1 — non-csm `retrieve` returns `Err(CascadeError::CapabilityUnavailable)`)
storage_metrics_truthful          = true  (PTA-A2 — `MetricValue` provenance: measured/estimated/unavailable)
unsupported_threshold_hidden      = true  (PTA-A3 — `eval set-threshold` removed from Clap + docs)
automatic_attribution_capture     = true  (ADR-080 merged in #927 + #947: episode validation, checked manual receipts, fallible playbooks, merged 2026-08-12)
attribution_capability_truth      = true  (ADR-081 §2 — StorageBackend capability advertisement + capability-gated persist_session_checked, 2026-08-10; concrete-backend capability tests in #947, merged 2026-08-12)
feedback_integrity_checked        = true  (RAT-A4 receipt-matrix tests #930 + #947 checked manual receipt matrix + cold-restart tests, merged 2026-08-12)
feedback_updates_ranking          = false (follow-up ADR required; nothing in #947 changes ranking)
r_f_spikes_go                     = true  (R-F1…R-F7 + R-F10 GO spike artifacts written + validated 2026-07-28)
r_f10_oidc_publishing             = true  (ACT-325 — publish-crates.yml OIDC id-token + exchange)
r_f4_simd_cosine                  = true  (ACT-326 — cosine_similarity_simd + simd bench variant)
```
