# GOAP State Snapshot

- **Last Updated**: 2026-08-10
- **Version**: workspace `0.1.39` · latest tag `v0.1.38`
- **Branch**: `main` @ `75f52a91`
- **Open PRs**: none — #927/#928/#930/#931/#932/#934 all merged (#934 fuzz/LTO wave 2026-08-09)
- **PR merge session**: #929 merged into this wave branch 2026-08-07 (CIT-A1/A2/A3); #916 + #917 merged 2026-08-02
- **Open issues**: none — #913 closed 2026-08-02
- **Active plan**: `plans/GOAP_PR_REVIEW_CI_FIX_WAVE_2026-08-07.md` + `plans/GOAP_CIT_A1_A2_A3_WORKFLOW_WAVE_2026-08-06.md` + `plans/GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md` + `plans/GOAP_ATTRIBUTION_COMPLETION_AND_CODEBASE_IMPROVEMENTS_2026-08-06.md`
- **Archive**: `plans/archive/2026-07-consolidation/`  
- **Release**: ✅ `v0.1.38` tagged and shipped 2026-08-08 (workspace bumped to `0.1.39` post-release)

---

## Phase: Analyze / decide — CI trust + product truth + attribution

| Package | Status |
|---------|--------|
| ADR-079 fail-closed required-check control plane | Proposed (P0) — workflow half implemented; ruleset half awaits maintainer acceptance |
| CIT-A1 required aggregate + staged ruleset migration | 🔄 workflow side done (`CI / Required`); ruleset stage pending |
| CIT-A2 cancellation/actor fail-closed behavior | ✅ waiters fail closed + commit-lint wait + downstream Dependabot/fork actor parity (2026-08-10) |
| CIT-A3 semantic gate-contract parity | ✅ validator + negative fixtures + actor-parity/ruleset-context fixtures (2026-08-10) |
| CIT-A4 release/publish trigger truth | ✅ Done (2026-08-06) |
| CIT-A5 durable informational evidence + deduplication | ✅ Done (fuzz evidence; dedup measurement is follow-up) |
| PTA-A1 non-`csm` cascade capability truth | ✅ #916 merged (2026-08-02) | PTA-A1 |
| PTA-A2 CLI storage metric truth | ✅ #916 merged (2026-08-02) | PTA-A2 |
| PTA-A3 threshold command cleanup | ✅ #916 merged (2026-08-02) | PTA-A3 |
| cargo-mutants CI sharding (reward/retrieval/retry/patterns) | ✅ #917 merged (2026-08-02) | CI |
| ADR-080 automatic recommendation attribution | Proposed |
| RAT-A1 contract tests | Blocked by ADR acceptance |
| RAT-A2…A7 implementation | Blocked by preceding RAT packages |
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
gates_match_policy                = true  (ADR-079 stage 3 — live ruleset now requires the `CI / Required` aggregate, 2026-08-10)
required_ci_causal                = true  (ruleset 9591004 requires Codacy + `CI / Required`; aggregate is causally enforced, 2026-08-10)
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
release_current                   = true  (v0.1.38)
version_advanced_after_tag        = true  (workspace 0.1.39)
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
automatic_attribution_capture     = true  (ADR-080 merged in #927)
attribution_capability_truth      = true  (ADR-081 §2 — StorageBackend capability advertisement + capability-gated persist_session_checked, 2026-08-10)
feedback_integrity_checked        = true  (RAT-A4 receipt-matrix tests #930)
feedback_updates_ranking          = false (follow-up ADR required)
r_f_spikes_go                     = true  (R-F1…R-F7 + R-F10 GO spike artifacts written + validated 2026-07-28)
r_f10_oidc_publishing             = true  (ACT-325 — publish-crates.yml OIDC id-token + exchange)
r_f4_simd_cosine                  = true  (ACT-326 — cosine_similarity_simd + simd bench variant)
```
