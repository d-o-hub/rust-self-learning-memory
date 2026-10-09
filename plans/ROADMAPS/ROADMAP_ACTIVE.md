# Active Development Roadmap

**Last Updated**: 2026-10-06
**Released Version**: v0.1.44 (latest tag)
**Workspace Version**: 0.1.45 (post-v0.1.44 bump)
**Active plan**: **audit-backlog wave** — `GOAP_AUDIT_BACKLOG_WAVE_2026-10-04.md` (W1–W6 + queued ACT-376…385). Status: **W1–W4 merged 2026-10-05** (#1131, #1135, #1134, #1139); **W5/W6 + Q02 in review** (#1138, #1140, #1130). Trigger: 22 open code issues (#1063–#1092) were filed at audit baseline `9f50c607` and had been absent from every tracker; re-validation at `74a44a15` confirmed none were fixed.
**Sprint 2026-10-02 (closed)**: v0.1.44 shipped 2026-10-02 (tag on `4f4f4ba8`) — checked completion receipts (#1080 → #1107), CLI drain-and-verify (#1081 → #1112), pattern-search input bounds (#1113), clippy 1.99 `assert_is_empty` migration (#1108), LOC gate (#1103), architecture/status docs refresh (#1110), coverage-floor reconciliation (#1090 → #1117); release-pipeline work in #1109 landed as C1/C2 (#1123), C3/C4 (#1121), C5–C7 (#1125)
**Plan**: #1080 via #1107, #1081 via #1112, #1113 direct, the 1.99 lint migration via #1108, LOC gate via #1103, docs refresh via #1110, coverage policy via #1117; prior waves historical (`GOAP_PR_REVIEW_CI_FIX_WAVE_2026-08-07.md`, `GOAP_CIT_A1_A2_A3_WORKFLOW_WAVE_2026-08-06.md`, `GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md`, `GOAP_ADR081_CAPABILITY_TRUTH_2026-08-10.md`, `GOAP_RELEASE_PIPELINE_C5_C7_2026-10-04.md`, merged #947, #952)
**Branch**: main @ `0485bb66` (v0.1.44 tagged on `4f4f4ba8`; audit-wave merges through 2026-10-05)
**Open PRs**: run `gh pr list --state open` — counts are deliberately not pinned in this header; it rotted twice (`validate-plans.sh --tracker-drift` now guards it)
**Open issues**: run `gh issue list --state open` — the open set is a registered audit backlog, not free work: per-issue verdicts and evidence live in `STATUS/GAP_ANALYSIS_LATEST.md`

## Sprint 2026-10-04 — audit-backlog wave W1–W6

| Prio | Area | Item | Status |
|------|------|------|--------|
| P1 | Core | #1077 (R06) ranking read guard held across `get_all_patterns().await` and the recommendation await — AGENTS.md invariant violation | ✅ Merged (#1131) |
| P2 | MCP | #1086 (M05) latency computed from `as_secs()` → every sub-second request records 0 ms | ✅ Merged (#1135) |
| P2 | Storage | #1088 (M07) `save_episode_tags` `BEGIN`/`COMMIT` with `?` early returns and no `ROLLBACK` | ✅ Merged (#1134) |
| P1 | Storage | #1066 (S07) redb episode→session index overwritten by write order, not by `(timestamp, session_id)` | ✅ Merged (#1139) |
| P1 | Retrieval | #1075 (R04) `calculate_keyword_similarity` returns the constant `0.5`, ignoring query and pattern | 🔄 In review (#1138) — roast-fixed |
| P1 | MCP | #1085 (M04) health infers connectivity from env-var existence, hard-codes cache metrics to zero, reports `uptime 0` — **and leaks `TURSO_DATABASE_URL` into the response** | 🔄 In review (#1140) — roast-fixed |
| P1 | Tests | #1091 (Q02) ignored-test inventory + un-ignored protocol tests + isolated fail-visible nightly Turso job (159 entries; ADR-027 refreshed) | ✅ Merged (#1130, 2026-10-06) |

## Sprint 2026-10-06 — wave A slices (in review)

| Prio | Area | Item | Status |
|------|------|------|--------|
| P1 | Storage | #1065 (S06) recommendation timestamp precision + deterministic tie-break | ✅ Merged (#1142) |
| P1 | Observability | #1079 (R08) execution-backed retrieval provenance | ✅ Merged (#1148) |
| P2 | Quality | #1092 (Q03) lint suppressions — slice 1 turso `unsafe_code` peel + ratchet | ✅ Merged (#1143; slice 2 #1153 in review) |
| P1 | Storage | #1067 (S08) modification watermark + bounded keyset pages (revision table, durable watermark, Turso backfill) | 🔄 In review (#1149) — 6 roast MAJORs fixed |
| P1 | Storage | #1064 (S05) scoped pool checkout retaining permits; roast BLOCKER (nested checkout) fixed | 🔄 In review (#1147) |
| P1 | Storage | #1070 (S11) capacity eviction: durable outbox + single transaction + partial outcome | 🔄 In review (#1150) |
| P1 | Pool | #1063 (S04) adaptive pool capacity model + monotonic cooldown (E4 test replaced) | 🔄 In review (#1146) |
| P1 | Storage | #1071 (S12) strict row decode + allowlisted query builder | 🔄 In review (#1145) |
| P1 | Embeddings | #1073 (R02) identity-scoped embedding storage adapter | 🔄 In review (#1151) |
| P2 | Ranking | #1078 (R07) incremental ranking index (100k: 222 ms → 0.84 µs) + benches | 🔄 In review (#1154) |
| P1 | Storage | #1087 (M06) capability truth: cleanup pair / procedural+relationships / inventory+lint | 🔄 In review (#1144, #1152, #1158) |
| P2 | CI | Nightly isolated-job libtest-JSON flag fix (follow-up to #1130) | 🔄 In review (#1159) |

**Next queue**: #1068 + #1089 (after #1149 merges), #1074 (in flight), #1076 (after #1074). Release `v0.1.45` follows the wave (#1137).

## Sprint 2026-10-02 — Durability receipts + toolchain hygiene

| Prio | Area | Item | Status |
|------|------|------|--------|
| P1 | Core | #1080 checked episode completion receipt (`Local`/`Committed`/`Queued`) + failed-episode IDs in `flush` errors | ✅ Merged (#1107) |
| P1 | CLI | #1081 `episode complete|fail` drain the durable queue with `--durable-timeout-secs` before success output; report `durability` | ✅ Merged (#1112) |
| P1 | MCP | #1113 `search_patterns`/`recommend_patterns` clamp `limit`/`min_relevance` and truncate oversized inputs (CWE-770) | ✅ Merged (#1113) |
| P1 | Tooling | clippy 1.99 `assert_is_empty` migration across the workspace (floating `stable` bump) | ✅ Merged (#1108) |
| P1 | Docs | Architecture/serialization/status evidence refreshed for v0.1.44 (#1094) | ✅ Merged (#1110) |
| P1 | Tooling | Coverage floor reconciled: 70% blocking, 90% aspirational (#1090) | ✅ Merged (#1117) |
| P0 | Release | v0.1.44 shipped — tag `v0.1.44` on `4f4f4ba8`, GitHub Release with dist artifacts, drift issue closed by the tag; workspace bumped to 0.1.45 | ✅ 2026-10-02 |
| P2 | Tooling | Release-pipeline best-practice proposal (OIDC publish, attestations, immutable releases, Pages boundary) | ✅ C3/C4 (#1121); C1/C2 (#1123); C5–C7 (#1125) — manual crates.io registrations still pending |

## Sprint 2026-09-30 — Security + durability fixes (v0.1.43)

| Prio | Area | Item | Status |
|------|------|------|--------|
| P1 | MCP security | #1082 OAuth 2.1 enforcement before method dispatch; fail-closed startup without `MCP_OAUTH_TOKEN_SECRET` | ✅ Merged (#1101) |
| P1 | MCP security | #1084 rate-limit identity from the validated token subject / process identity; bounded bucket cardinality | ✅ Merged (#1101) |
| P1 | Storage | #1069 redb fails closed on schema mismatch instead of clearing data | ✅ Merged (#1096) |
| P1 | Core | #1072 atomic, provider-identity-aware embedding activation | ✅ Merged (#1097) |
| P2 | MCP | #1083 `tools/list` enumerates the full registry by default | ✅ Merged (#1100) |
| P2 | Storage | #1060-#1062 adaptive/caching/keep-alive pool ownership without raw pointers or `unsafe` | ✅ Merged (#1095) |
| P2 | Docs | MCP fail-closed claims reconciled across docs, skills and doc-integrity checks | ✅ Merged (#1099) |
| P0 | Release | v0.1.43 shipped — tag `v0.1.43` on `a0078d0a`, GitHub Release with dist artifacts, drift issue closed by the tag-triggered check | ✅ 2026-10-01 |
| P1 | Tooling | LOC ceiling regression fixed: oversized `server_impl` test modules split; `check-loc.sh` now runs in File Structure Validation | ✅ Merged (#1103) |

## Sprint 2026-09-24 — Retrieval judgment + rerank

| Prio | Area | Item | Status |
|------|------|------|--------|
| P1 | Retrieval | #1030 typed semantic judgment interface (`RetrievalJudge`, validation, bounded telemetry) | ✅ Merged (#1041) |
| P1 | Retrieval | #1031 opt-in semantic shortlist rerank (deterministic fusion, single finalization path, `--rerank` eval arm) | ✅ Merged (#1042) |
| P2 | Tooling | Gated merge path (`merge-pr.sh`) + soft tracker-drift check + `coverage-waivers` skill | ✅ Merged (#1046) |
| P2 | Retrieval | #1032 evidence-aware passage classification (depends on #1030/#1031) | ✅ Merged (#1049); confidence gating fixed in #1053 |

## Sprint 2026-09-05 — Observability + dev-harness adoption

| Prio | Area | Item | Status |
|------|------|------|--------|
| P1 | Observability | #962 retrieval telemetry: bounded labels, MCP `get_metrics(retrieval)`, redaction tests | ✅ Merged (#1005, 2026-09-15) |
| P1 | Build | Fix `.gitignore` swallowing `metrics/` sources; tokio `net` feature; wasm gate | ✅ Done |
| P1 | Observability | Prometheus exposition validity (single TYPE per family, per-(op,tier) quantiles) | ✅ Done |
| P2 | Observability | Module split ≤500 LOC (labels/registry/exposition) | ✅ Done |
| P2 | Observability | CLI retrieval-metrics surface + docs dashboard example (#962 acceptance) | ✅ Done (`do-memory-cli monitor retrieval` + `docs/RETRIEVAL_OBSERVABILITY.md`) |
| P2 | Workflow | do-harness dev harness adopted (sensors fmt/check/clippy/test/deny/loc) | ✅ Done |
| P1 | Release | #976 drift (30 commits / 24 d): ship via release-guard once #962 lands | ✅ Shipped v0.1.40 (2026-09-06) and v0.1.41 (2026-09-20); workspace bumped to 0.1.42 (#1039) |

---


## Completed sprint 2026-07-22…25 — Ship + post-bump + gap analysis + R-F8/R-F9 + skills

| Priority | Item | Description | Status |
|----------|------|-------------|--------|
| 1 | Ship v0.1.36 | `release-manager.sh ship --execute` + release.yml | ✅ |
| 2 | Post-bump | Workspace → 0.1.37 (#886) | ✅ |
| 3 | R-E2 skill evals | Medium-risk behavioral fixtures (#883) | ✅ |
| 4 | Docs integrity | Unblock ship gate (#885) | ✅ |
| 5 | Plans truth (#889) | CURRENT / GOALS / ACTIONS / GOAP_STATE / GAP refresh | ✅ |
| 6 | Changelog hygiene (#887) | Update CHANGELOG.md for v0.1.36 | ✅ |
| 7 | Cosine unrolled (#888) | 8-way accumulator optimization | ✅ |
| 8 | Gap tasks (#891) | ADR-074 docs, pattern extract command (G-P1-12), coverage | ✅ |
| 9 | R-F8 + R-F9 (#893) | CLI relationship box-drawing panel + HNSW persistence/eviction | ✅ |
| 10 | 6 domain skills | checkpoint-handoff, embedding-ops, episode-relationships, episode-tags, playbook-ops, recommendation-feedback (40 total) | ✅ |
| 11 | ADR-077 A1-A5 | Runtime embedding activation: exact-provider factory, atomic runtime seam, MCP end-to-end (main `9ef4b742`, `e0f7f712`) | ✅ |
| 12 | ADR-077 A6 | Validate/document/gate: activation docs + concurrency + zero-unsafe credential-redaction regression tests | ✅ #897 merged |

---

## Active forward work

| Priority | Item | Description | Status |
|----------|------|-------------|--------|
| P0 | ADR-079 / CIT-A1 | Same-run `CI / Required` aggregate, fail-closed evaluator, staged ruleset migration | ✅ same-run fast gate + commitlint; `ci-required-evaluate.sh` rejects `skipped`; ruleset `9591004` requires `CI / Required` (stage 3 live); waiter/anchor removed (stage 5) |
| P0 | ADR-079 stage 4 | Deliberate live fault-injection merge-block proof | ⏸ external maintainer evidence — not performed in #947 |
| P0 | CIT-A2 | Fail closed on cancellation/missing/commitlint; Dependabot/fork parity | ✅ waiters fail closed + actor parity (2026-08-10) |
| P0 | PTA-A1 cascade truth | Non-`csm` retrieval returns typed `CapabilityUnavailable` instead of successful empty | ✅ #916 |
| P0 | PTA-A2 storage metric truth | Label measured/estimated/unavailable via `MetricValue` provenance; remove fabricated telemetry | ✅ #916 |
| P1 | CIT-A3 | Exact local/CI command scope and semantic gate-contract validation | ✅ semantic validator + negative fixtures + `--required-aggregate` (merged in #947, 2026-08-12) |
| P1 | CIT-A4/A5 | Truthful release/publish triggers and durable fuzz/mutation evidence | ✅ 2026-08-06 |
| P1 | PTA-A3 threshold CLI truth | Hide advertised `eval set-threshold` non-operation | ✅ #916 |
| P1 | ADR-080/081 / RAT-A1…A7 | Episode-bound automatic attribution with truthful persistence receipts | ✅ code-side closed in #947 (2026-08-12) (episode validation, checked receipts, cold-restart, capability, postcard safety); ADR lifecycle stays Proposed pending maintainer acceptance |
| P2 | Ranking adaptation | Idempotent feedback-to-ranking update; separate ADR | ✅ code-side in this PR — ADR-082: derived Wilson weight, capability-gated `list_recommendation_*`, recommend re-rank, e2e; lifecycle Proposed |

---

## Follow-on backlog (P2 — spike-gated)

| Priority | Theme | Items | Status |
|----------|-------|-------|--------|
| P2 | Research | WG-108 / WG-110 / WG-125 / WG-135 | ⏸ DEFER |
| P2 | Vision | Distributed sync, multi-tenancy, OTel | Future |
| P2 | Release eng | Trusted Publishing (OIDC) for crates.io + release-path hardening | ✅ ACT-325 + ACT-368 + ACT-369 (official action, no token fallback, truthful triggers; 2026-10-04) |
| P2 | CI cost | Reusable workflows/artifact handoff after required-gate correctness | Blocked by CIT-A1…A3 |
| P2 | Security | Transitive Dependabot advisories | Monitor |
| P2 | CLI | ADR-076 §5 `pattern extract` error-arm coverage | ✅ Done (#891) |
| P2 | CLI | R-F8 relationship info box-drawing panel | ✅ Done (#893) |
| P2 | Embeddings | R-F9 HNSW persistence + capacity eviction | ✅ Done (#893) |

---

## Standing product decisions (do not reopen casually)

| Topic | Decision |
|-------|----------|
| Agent code execution | **Fail-closed**; S1.1c Wasmtime/WASI **NO-GO** |
| Batch MCP tools | Explicitly deferred |
| Release creation | Automated only: `release-manager.sh ship` → tag → `release.yml` |
| Serialization | Postcard required |
| ADR-075 durable complete | All-or-nothing; backend failures are hard errors |
| ADR-076 pattern list | Empty diagnostics in human mode; JSON/YAML machine-stable |

---

## History pointer

Completed sprint tables live under `plans/archive/2026-07-consolidation/` and older archives.  
Do not re-expand completed WG tables (ADR-039).
