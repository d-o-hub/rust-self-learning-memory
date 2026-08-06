# Gap Analysis — 2026-08-06

**Generated**: 2026-08-06
**Audit commit**: `92db07bf` (`main`)
**Workspace**: `0.1.38` · **Tag**: `v0.1.37`
**Active plan**: [`../GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md`](../GOAP_CIT_A4_A5_AND_PLAN_TRUTH_2026-08-06.md) on top of [`../GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md`](../GOAP_CODEBASE_TRUTH_AND_ATTRIBUTION_2026-07-30.md)

## Method

- Read active planning authority and ADR-039/044/077 constraints.
- Traced recommendation generation through core, MCP, CLI, Turso, and redb.
- Inspected feature-disabled cascade behavior, storage-stat output, evaluation
  command dispatch, and embedding-provider activation boundaries.
- Audited checked-in workflows, local gate scripts, recent Actions runs, and the
  active GitHub repository ruleset through `gh`/REST evidence.
- Kept intentional fail-closed code execution and deferred batch tools as non-gaps.
- Ranked observable false-success behavior above additive provider work.
- Prior wave (2026-07-28): all P0 ship items closed; R-F8/R-F9 merged (#893);
  6 skills added (40 total); R-F1…R-F7 + R-F10 GO spike artifacts validated.

## Closed this wave (2026-08-06)

| Gap | Resolution |
|-----|------------|
| G-P1-15 release manual dispatch broken | ✅ CIT-A4 — `workflow_dispatch` removed from release.yml; publish uses `--locked`, bounded polling, and dependency-closure failure semantics |
| G-P1-15 fuzz evidence silent green | ✅ CIT-A5 — fuzz artifacts upload with `always()`, status report fails the informational job on crashes/timeouts/startup failures |
| G-P2-1/7 R-F10 OIDC (ACT-325) | ✅ Already shipped (`id-token: write` + OIDC exchange); trackers refreshed |
| G-P2-1/7 R-F4 SIMD cosine (ACT-326) | ✅ Already shipped (`cosine_similarity_simd` + simd bench variant); trackers refreshed |

## Closed this wave (prior)

| Gap | Resolution |
|-----|------------|
| G-P0-1 v0.1.36 unreleased | ✅ Tag + GitHub Release 2026-07-22 |
| G-P0-4 / G-P0-5 release docs / rust-major | ✅ #880 / #877 |
| G-P1-7 medium-risk eval depth | ✅ R-E2 #883 |
| Docs integrity ship blocker | ✅ #885 |
| Post-tag version lag | ✅ workspace `0.1.37` #886 |
| G-P1-10 open hygiene/perf PRs | ✅ #887, #888, #889, #891, #893 all merged |
| R-F8 relationship show polish (GO spike) | ✅ #893 — box-drawing panel + unit tests |
| R-F9 HNSW persistence + eviction (GO spike) | ✅ #893 — file_dump/load + capacity eviction |
| Skill count 34, 6 domain skills untracked | ✅ 40 skills, all routed (#894) |
| ADR-077 runtime embedding activation A1-A5 | ✅ main (`9ef4b742`, `e0f7f712`) — exact-provider factory + atomic runtime seam + MCP end-to-end |
| ADR-077 A6 validate / document / gate | ✅ #897 merged — activation docs + concurrency + zero-unsafe credential-redaction regression tests |
| G-P2-1…7 R-F* spike artifacts | ✅ GO artifacts for R-F1…R-F7 + R-F10 (plans/STATUS/spikes/, 2026-07-28) |

## Open gaps (current)

### P0

| ID | Gap | Evidence | Track |
|----|-----|----------|-------|
| G-P0-12 | `main-protection` requires no first-party build/test context; the echo anchor is unused and non-substantive | Ruleset `9591004`, `pr-check-anchor.yml` | ADR-079 / CIT-A1 |
| G-P0-13 | Five waiters permit cancelled/skipped/missing format/Clippy and ignore commit lint | waiter workflows; PR #914 | ADR-079 / CIT-A2 |
| G-P0-10 | Cascade retrieval without `csm` returns a successful empty result, indistinguishable from no matches | `memory-core/src/retrieval/cascade/mod.rs` | ✅ PTA-A1 closed — typed `CascadeError::CapabilityUnavailable` |
| G-P0-11 | CLI storage stats and connection status expose estimates/unknowns as measured values | `memory-cli/src/commands/storage/commands.rs`, `types.rs` | ✅ PTA-A2 closed — `MetricValue` provenance |

### P1

| ID | Gap | Evidence | Track |
|----|-----|----------|-------|
| G-P1-8 | Historical ADR number reuse on disk | Dual 025/054 filenames; aliases in `plans/adr/README.md` | residual docs |
| G-P1-9 | Transitive Dependabot advisories | Upstream chains (libsql/openssl/webpki) | security hygiene |
| G-P1-10 | `eval set-threshold` is advertised but always fails; suggested `eval show` command does not exist | `memory-cli/src/commands/eval.rs` | ✅ PTA-A3 closed — command removed |
| G-P1-11 | Pattern recommendations require manual session creation; playbooks record `Uuid::nil()` in memory only | core/MCP/CLI attribution paths | ADR-080 / RAT-A1…A7 |
| G-P1-12 | Feedback accepts integrity states that can corrupt attribution statistics | tracker/API/persistence paths | ADR-080 / RAT-A4 |
| G-P1-13 | Dependabot is excluded from most substantive code/test/security assertions | actor conditions across CI/coverage/security/file/benchmark workflows | ADR-079 / CIT-A2 |
| G-P1-14 | Gate contract claims parity while tests, Clippy, LOC, and quality-bundle semantics differ; validator is presence-only | `GATE_CONTRACT.md`, `ci.yml`, `quick-check.yml`, `validate-gate-contract.sh` | ADR-079 / CIT-A3 |
| G-P1-15 | Release manual dispatch is broken; publish selection and fuzz evidence have silent skip/green paths | release run `30301797956`, publish/fuzz workflow conditions | ✅ CIT-A4/A5 closed 2026-08-06 |

### P2 (product / research)

| ID | Gap | Notes | Track |
|----|-----|-------|--------|
| G-P2-1…7 | R-F1…R-F7, R-F10 epics | R-F4 (ACT-326) and R-F10 (ACT-325) implemented; R-F1…R-F3/R-F5…R-F7 deferred | R-F* |
| G-P2-8 | Feedback does not idempotently update later recommendation ranking | ADR-080 captures data only | Follow-up ADR |
| G-P2-9 | Azure/Custom/Cohere runtime embedding adapters absent | Honest rejection required by ADR-077 | Provider-specific ADR if prioritized |

## Explicit non-gaps

| Claim | Verdict |
|-------|---------|
| Working `execute_agent_code` backend | Intentional fail-closed |
| Batch MCP tools | Deferred product decision |
| Production LOC >500 | Closed |
| Medium-risk skill presence-only evals | Closed |
| Release lag / commit_limit on tag | Closed by v0.1.37 ship and 0.1.38 post-bump |
| R-F8 relationship show polish | ✅ #893 |
| R-F9 HNSW persistence | ✅ #893 |
| Unsupported embedding adapters | Honest ADR-077 rejection; additive P2 work |
| Automatic attribution changes ranking | **False** until a separate idempotent update design is implemented |
| `Required Check Anchor` protects merges | **False** — not in the live ruleset and only echoes |
| Green first-party workflow means merge-required | **False** — live ruleset does not require those contexts |

## Exit criteria for this register

- ADR-079 CIT-A1…A5 exits are implemented, including live aggregate protection.
- PTA-A1…A3 have code and feature-matrix tests. ✅ (2026-08-01)
- ADR-080 acceptance criteria and RAT-A1…A7 are implemented with evidence.
- Ranking adaptation remains explicitly open until separately decided and tested.
- G-P1-8 and G-P1-9 are monitor-only (no code action required).
- P2 GO spike gate cleared 2026-07-28; next gate is ADR draft + implementation PR per epic.
