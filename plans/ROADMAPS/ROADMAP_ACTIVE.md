# Active Development Roadmap

**Last Updated**: 2026-10-02
**Released Version**: v0.1.43 (latest tag)
**Workspace Version**: 0.1.44 (post-v0.1.43 bump)
**Active Sprint**: post-v0.1.43 wave merged 2026-10-01/02 — checked completion receipts (#1080 → #1107), CLI drain-and-verify (#1081 → #1112), clippy 1.99 `assert_is_empty` migration (#1108), LOC-gate/LOC split (#1103); the release-pipeline best-practice proposal (#1109) is open and the automated PRs #1110/#1111/#1113 are in review
**Plan**: #1080 via #1107, #1081 via #1112, the 1.99 lint migration via #1108, LOC gate via #1103; v0.1.43 contents via #1101/#1096/#1097/#1100/#1095/#1099 (#1102/#1106 release tooling); prior waves historical (`GOAP_PR_REVIEW_CI_FIX_WAVE_2026-08-07.md`, `GOAP_CIT_A1_A2_A3_WORKFLOW_WAVE_2026-08-06.md`, `GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md`, `GOAP_ADR081_CAPABILITY_TRUTH_2026-08-10.md`, merged #947, #952)
**Branch**: main @ `580c32db` (PR #1112 merged 2026-10-02)
**Open PRs**: run `gh pr list --state open` — counts are deliberately not pinned in this header; it rotted twice (`validate-plans.sh --tracker-drift` now guards it)
**Open issues**: run `gh issue list --state open` — the release-pipeline proposal tracks the next pipeline work

## Sprint 2026-10-02 — Durability receipts + toolchain hygiene

| Prio | Area | Item | Status |
|------|------|------|--------|
| P1 | Core | #1080 checked episode completion receipt (`Local`/`Committed`/`Queued`) + failed-episode IDs in `flush` errors | ✅ Merged (#1107) |
| P1 | CLI | #1081 `episode complete|fail` drain the durable queue with `--durable-timeout-secs` before success output; report `durability` | ✅ Merged (#1112) |
| P1 | Tooling | clippy 1.99 `assert_is_empty` migration across the workspace (floating `stable` bump) | ✅ Merged (#1108) |
| P2 | Tooling | Release-pipeline best-practice proposal (OIDC publish, attestations, immutable releases, Pages boundary) | 📋 Issue opened (#1109) |

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
| P2 | Release eng | Trusted Publishing (OIDC) for crates.io | ✅ ACT-325 (2026-08-06 confirmed) |
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
