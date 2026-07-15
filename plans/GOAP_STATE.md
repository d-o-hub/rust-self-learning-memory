# GOAP State Snapshot

- **Last Updated**: 2026-07-15 (CLI UX patch #829-#832 on branch `fix/0.1.35-patch-issues`)
- **Version**: `0.1.35` (workspace; tag v0.1.34 latest released)
- **Branch**: `fix/0.1.35-patch-issues`
- **Plan**: `plans/GOAP_CLI_UX_PATCH_0.1.35_2026-07-15.md`
- **Validation**: CLI skill smoke + targeted nextest (see plan)

---

## Sprint 2026-07-15: CLI UX Patch v0.1.35 ✅ CODE COMPLETE

**Open issues closed in code** (verify after merge/release):

| Issue | Fix | Verified via CLI |
|-------|-----|------------------|
| #831 Pattern list empty after complete | Postcard-compatible Pattern + get_all_patterns lazy load | create→complete→list=1, search finds pattern |
| #830 --db-path ignored | Always set redb_path; default local when no Turso | Opens custom.redb / env.redb |
| #829 Undocumented config | config init/show-template + serde default + example.toml | partial.toml loads |
| #832 storage_mode placement | [database] canonical; [storage] alias normalized | config show → Storage Mode: local |
| #828 Release drift | Workspace 0.1.35 (not 0.2.0) | Version in Cargo.toml |

**Prevention tasks landed**: postcard Pattern test, redb trait list test, e2e pattern list assert, loader partial/alias tests, example config, CHANGELOG.

**Next**: commit → PR → CI green → merge → `release.yml` for v0.1.35 (never manual release).

---


## Sprint 2026-07-08: PR Batch Merge & Mutation Testing

**Task**: Merge 5 open PRs, implement mutation testing (#747), update skills canonical path

**PRs Merged** (auto-merge enabled):
| PR | Title | Fixes |
|----|-------|-------|
| #787 | fix(cli): persist episodes in local storage mode | #773 |
| #788 | docs(readme): add build dependencies section | #771 |
| #789 | ci(publish): improve crates.io publish pipeline | #772 |
| #790 | docs(plans): update status for v0.1.34 sprint | - |
| #791 | docs(agents): add lessons #013-#014 | - |

**Issues Implemented**:
- #747: Mutation testing workflow added (non-blocking, weekly schedule)

**Remaining Open Issues**:
| # | Title | Priority | Notes |
|---|-------|----------|-------|
| #786 | Release drift | High | Cut release after merges |
| #770 | crates.io availability | Medium | Partially by #789 |
| #753 | Retry budgets | Backlog | Large feature |
| #749 | Turso connection pooling | Backlog | Large feature |
| #746 | WASM build path | Backlog | Large feature |
| #743 | Storage refactor | Backlog | Large refactor |

---

## PR Remediation — 2026-07-08

**Task**: Resolve open PRs #769, #774, #775.

**Current Status**:
| PR | Title | Status | Action |
|----|-------|--------|--------|
| **#775** | feat(retrieval): add ANN-backed semantic episode retrieval | ✅ MERGEABLE, CI green | Ready to merge (macOS pending) |
| **#774** | Add ANN-backed semantic episode retrieval components | ❌ CLOSED | Superseded by #775 |
| **#769** | Add Turso + redb hybrid storage integration tests | 🔧 FIXED | Merge conflict resolved, CI workflow fixed, review comments addressed |

### PR #769 Changes
- Resolved merge conflict in `memory-core/src/types/structs.rs` (RewardScore doc comment conflict from `feat(reward): implement statistical normalization and temporal decay`)
- Fixed CI workflow: replaced `--all-features` with explicit `--features turso,redb` in hybrid storage-matrix mode
- Review feedback: typo already fixed, `assert_episode_parity` already compares values, TempDir warning documented, reconciliation confirmed
- Local verification: `cargo check` ✅, `cargo fmt` ✅, `cargo clippy` ✅, 22 hybrid tests pass

### PR #774
- Closed as superseded by #775 (same feature: ANN-backed semantic retrieval)
- #774 had 305 additions, CI failing (Quick PR Check, Semver Check)
- #775 has 1503 additions, CI green (all checks passing)

### Next Actions
- Merge #775 once macOS + benchmarks complete
- Merge #769 once CI re-runs green
- WG-175: Cut v0.1.33 release (urgent — additional commits accumulated)

---

## PR Remediation Campaign (2026-07-02) ✅ COMPLETE

**Task**: Resolve all open PRs from `plans/OPEN_PR_REMEDIATION_2026-07-02.md`.

**PRs Merged** (in recommended order):
1. **PR #719** — `fix(deps): unify sysinfo to 0.39` → Merged (sysinfo workspace dedup)
2. **PR #711** — `docs: align documentation with architecture` → Merged (rebase only)
3. **PR #715** — `refactor(mcp): remove deprecated handle_shutdown (ADR-060)` → Merged (add/add conflict resolved)
4. **PR #718** — `feat(patterns): AbstentionPattern extractor + abstention_score` → Merged (9 missing fields fixed)
5. **PR #709** — `Resolve flat file conflicts / consolidate modules` → Merged (3 rebase conflicts, dangling mods, 6 import renames)
6. **PR #727** — `docs: fix RewardScore doctest` → Merged (follow-up fix)
7. **PR #729** — `fix: sandbox test + wire up dead types_tests` → Merged (JS runtime error type + dead code)

**Key outcomes**:
- 0 open PRs, 3,453 tests pass (0 failures), clippy + fmt clean
- 0 dependency duplicates (down from 127)
- 23 stale local branches cleaned, 8 worktrees pruned
- `target/` reduced from 43G to 13G (30G freed)
- All quality gates pass

**Plan document**: `plans/OPEN_PR_REMEDIATION_2026-07-02.md`

---

## Dependency & CI Maintenance (2026-06-30)

**Task**: Merge all open PRs (4 Dependabot + 1 CI/quality) in correct dependency order.

**PRs Merged** (in order):
1. **PR #675** — `fix(ci,quality): WG-176..182 CI health + code quality` → Merged 2026-06-30T09:43:27Z
   - Fixed gitleaks fingerprints, clippy --all-features lints, cache/wrapper.rs split, cascade tracing
   - Workflow fixes: disk cleanup, mutation scoping, upload-artifact bump
   - **Completes**: WG-176, WG-177, WG-178, WG-179, WG-180, WG-181, WG-182
2. **PR #682** — `fix(deps): remove stale advisory ignores and update anyhow` → Merged 2026-06-30T10:10:58Z
   - Removed 6 stale RUSTSEC advisory ignores from deny.toml
   - Updated anyhow 1.0.102 → 1.0.103 (fixes RUSTSEC-2026-0190)
   - `cargo deny check` now passes cleanly
3. **PR #681** — `ci(deps): bump the actions-all group across 1 directory with 13 updates` → Merged 2026-06-30T11:10:33Z
   - Bumped actions/checkout, actions/github-script, peter-evans/create-pull-request, etc.
   - Resolves Node.js 20 deprecation warnings
4. **PR #684** — `chore(deps): bump the rust-patch-minor group across 1 directory with 2 updates` → Merged 2026-06-30T11:14:54Z
5. **PR #678** — `chore(deps): bump sysinfo from 0.38.4 to 0.39.5 in the rust-major group` → Auto-merge enabled, pending CI

**Key outcomes**:
- All GitHub Actions checks pass (Codacy, Quick PR Check, Dependency Audit, CodeQL, etc.)
- `cargo deny check` passes cleanly (6 stale advisories removed, anyhow advisory fixed)
- GitHub Actions updated to latest versions (Node 24 compatible)
- Dependency audit clean: no outstanding advisories require ignoring (except existing rustls-webpki transitive)

**Follow-up tasks**:
- WG-175: Cut v0.1.33 release (now more urgent — additional commits since analysis)
- ✅ WG-183: llms.txt generator (issue #652) — DONE (`scripts/generate-llms-txt.sh`)
- WG-184: ADR for VERSION file decision (issue #653)
- Monitor PR #678 auto-merge completion

---

## WG-185 + WG-183 Execution (2026-07-01)

**Branch**: `feat/wg185-loc-splits-llms-generator`
**Plan**: `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-30.md`

**WG-185 — LOC boundary splits** (both files were at exactly 500 LOC, the gate limit):
- `memory-core/src/retrieval/cascade/mod.rs`: 500 → 420 LOC. Extracted
  `CascadeConfig`/`TierResult`/`CascadeResult` into new `cascade/types.rs` (re-exported).
- `memory-cli/src/commands/tag/core.rs`: 500 → 382 LOC. Extracted `search_by_tags`
  into new `core/search_ops.rs`.
- Fixed pre-existing unused-variable warning in `cascade/weights.rs` tests (`cg` → `_cg`).

**WG-183 — llms.txt generator (closes #652)**:
- Added `scripts/generate-llms-txt.sh` (compact `llms.txt` tracked + regenerated;
  `--full` emits git-ignored `llms-full.txt` artifact; `--check` for CI freshness).
- Version sourced from `Cargo.toml`; crate map from `cargo metadata`.
- Regenerated `llms.txt` (was stale at v0.1.30 → now v0.1.33).
- Documented in `agent_docs/token_efficiency.md`; `.gitignore` excludes `llms-full.txt`.

**Verification**: clippy clean (core+cli default; core `--features csm`), fmt clean,
2009 tests pass, 165 doctests pass, `cargo doc` clean, docs-integrity unchanged
(only pre-existing archive-only broken links remain).

---

## Comprehensive Analysis (2026-06-28)

**Task**: Analyze codebase for improvements; document in `plans/` (GOAP orchestrated).

**Findings**:
- **Open Issues**: 3 — **#674** (release drift: 94 unreleased commits since v0.1.32), **#652** (llms.txt automation), **#653** (VERSION file evaluation).
- **Open PRs**: 0.
- **Push CI (main)**: ✅ Green (latest: `850bf69d` 2026-06-26).
- **Scheduled Security**: ❌ — Gitleaks still detects 3 false positives (`.gitleaksignore` has 81 entries but misses these 3). Same root cause as ADR-058.
- **Nightly Full Tests**: ❌ — Disk exit-95 (runner infra). Node.js 20 deprecation warnings on checkout/upload-artifact actions.
- **Mutation Testing**: ❌ Cancelled — 6h ceiling always hit (scope too broad).
- **Clippy `--all-features`**: ❌ 5 findings in `memory-core/src/embeddings/mistral/client.rs` (4 excessive_nesting + 1 unnecessary_wraps).
- **File size gate**: 1 violation — `cache/wrapper.rs` at 537 LOC.
- **Architecture**: Non-CSM cascade fallback returns empty result silently.
- **Security audit**: 2 allowed git2 warnings (transitive via agentfs-sdk).

**Recent improvements since last analysis (2026-06-14 → 2026-06-28)**:
- Fuzz harness added (`14d756bd`) — parser/serialization boundaries
- Agent harness map + eval/sensor workflow (`1884f300`)
- Edit distance space complexity optimized (`14173bbd`)
- GitHub Actions pinned to stable SHAs (`1b234d11`, `d828cc90`)
- MCP input bounds hardened (multiple commits)
- actions/checkout bumped from v6→v7 (`a6c20fc4`)
- Codacy script injection in mutants.yml fixed (`19a17257`)
- Root one-off scripts cleaned (`850bf69d`)

**Plan Document**: `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-28.md` — 10 work groups (WG-175..WG-184) across 5 phases.

**Next actions**: WG-175 cut v0.1.33 release → WG-176 gitleaks fingerprints → WG-177-179 CI fixes → WG-180-181 clippy/LOC → WG-182-184 arch/DevX.

---

## Comprehensive Analysis (2026-06-14)

**Task**: Analyze codebase for improvements + check open GitHub issues; document in `plans/` (GOAP + ADR).

**Findings**:
- **Open Issues**: 1 — **#619** (release drift: 54 unreleased commits since v0.1.32; 2 feat, 8 fix). Real/actionable.
- **Open PRs**: 0. PR #616 (cosine-similarity perf) **merged** 2026-06-12 with the ADR-057 A1/A2 fix (`tokio::sync::Mutex` + plans restore); push CI green.
- **Scheduled Security**: ❌ — Gitleaks "Leaks detected" on the **full-history schedule scan**. 3 findings, all **confirmed false positives** (doc/example/historical placeholder keys), none matched by `.gitleaksignore` (stale fingerprints after `.claude`→`.agents` move + commit/rule drift).
- **Nightly Full Tests**: ❌ — `should_scale_processing_with_different_worker_counts` slow-test still times out (ADR-057 **A3 never applied**); plus disk exit-95 + mutation 2h ceiling (infra).
- **Backlog**: WG-158/160/161/162 implemented (`6a43deae`); `query_hits: 0` now only in test fixtures. WG-156/157 residuals to re-audit.

**Plan Documents**:
- `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-14.md` — GOAP action plan (B1–B5)
- `plans/adr/ADR-058-CI-Health-Gitleaks-Release-Drift-2026-06-14.md` — decision record

**Next actions**: B1 add 3 gitleaks fingerprints → B2 cut release (#619) → B3 bound slow test (ADR-057 A3) → B4 WG-156/157 re-audit → B5 nightly infra watch.

---

## CI Health Sweep (2026-06-11)

**Task**: Analyze failing CI, open PRs/issues, and missing implementation; document GOAP + ADR.

**Findings**:
- **Open PRs**: 1 — PR #616 (`perf(encoder): optimize cosine similarity`, Jules bot). ❌ CI red.
  - Root cause: `clippy::await_holding_lock` — PR holds a `parking_lot::Mutex` guard across `.await` in `memory-mcp/src/bin/server_impl/storage.rs` tests. Cascades to CI/Coverage/Security/Perf via `wait-on-check`. `main` is clean.
  - PR is cut from a stale base and deletes recent `plans/*` files (would regress docs).
- **Open Issues**: none.
- **Nightly Full Tests (main)**: ❌ — `should_scale_processing_with_different_worker_counts` TIMEOUT (120s, `#[ignore]` slow test); regular-tests exit-95 = runner disk-space (infra); mutation testing = 2h ceiling.
- **Missing impl**: WG-156–162 stubs still present (e.g. `query_hits: 0 // Not yet implemented`); tracked in `GOAP_PRE_EXISTING_ISSUES_FOLLOWUP_2026-06-09.md`.

**Plan Documents**:
- `plans/GOAP_CI_ANALYSIS_2026-06-11.md` — GOAP action plan (A1–A5)
- `plans/adr/ADR-057-CI-Health-PR616-Nightly-Timeout.md` — decision record
- `plans/CODE_CHANGES_CI_REMEDIATION_2026-06-11.md` — detailed before/after code changes

---

## Remote Repository Analysis (2026-06-10)

**Task**: Analyze remote repository (d-o-hub/rust-self-learning-memory) for workflow impacts

**Strategy**: Parallel Swarm (3 agents)

**Status**: ✅ COMPLETE — No adaptations required

**Key Findings**:
- Remote repository is IDENTICAL to local codebase (same project)
- Workflow configurations match exactly
- No feature gaps identified
- Build check passed successfully

**Conclusion**: The local codebase is a complete working copy of the remote repository. Continue development using existing workflow patterns.

**Plan Documents**:
- `plans/remote-repo-analysis-2026-06-10.md` - Initial analysis plan
- `plans/remote-repo-synthesis-2026-06-10.md` - Synthesis findings
- `plans/GOAP_REMOTE_ANALYSIS_2026-06-10.md` - Execution summary

---

## Optional Maintenance Complete (2026-06-10)

**Task**: Perform all optional maintenance from remote repository analysis

**Strategy**: Parallel Swarm (3 agents)

**Status**: ✅ **ALL MAINTENANCE COMPLETE**

### Maintenance Results

| Task | Agent | Status | Key Finding |
|------|-------|--------|-------------|
| Documentation sync | documentation | ✅ Complete | All docs identical to remote |
| Release monitoring | github-release-best-practices | ✅ Complete | v0.1.32 latest, 33 unreleased commits |
| Feature verification | explore | ✅ Complete | Full feature parity confirmed |

### Summary
- **Documentation**: Zero drift detected, all files current
- **Releases**: v0.1.32 is latest release, 33 commits unreleased
- **Features**: Full parity across core, MCP, and CLI modules
- **Local Codebase**: Complete working copy of remote repository

**Plan Document**: `plans/GOAP_MAINTENANCE_2026-06-10.md`

---

## v0.1.33 Sprint — Release Drift Resolution (In Flight)

- **Issue**: [#623](https://github.com/d-o-hub/rust-self-learning-memory/issues/623) — 60 unreleased commits since v0.1.32
- **ADR**: ADR-058 (CI Health), ADR-055 (Missing Implementation Remediation)
- **Strategy**: Sequential — gitleaks fix (B1 ✅) → slow test fix (B3 ✅) → WG-156/162 re-audit (B4 ✅) → release v0.1.33

### Progress

| Action | Description | Status |
|--------|-------------|--------|
| B1 | Add 3 gitleaks false-positive fingerprints | ✅ Done (PR #620) |
| B3 | Bound slow test; stop worker pools between iterations | ✅ Done (PR #620) |
| B4 | Re-audit WG-156–162 stubs | ✅ Done (PR #620) |
| B2 | Cut v0.1.33 release (close #623) | 🔧 In progress |

---

## v0.1.32 Sprint — COMPLETE ✅ (Released 2026-05-24)

### v0.1.32 Feature Addition — PR #611 (2026-06-06)

**Issue**: [#610](https://github.com/d-o-hub/rust-self-learning-memory/issues/610) — feat(turso): expose local/offline mode  
**PR**: [#611](https://github.com/d-o-hub/rust-self-learning-memory/pull/611) — Expose local/offline mode as a first-class config path  
**ADR**: ADR-056 — Local Storage No Connection Pooling  
**Branch**: `feat/turso-local-mode-12832947082971821257`  
**CI Status (2026-06-08)**: 🔴 4 checks failing

| Check | Status | Root Cause |
|-------|--------|------------|
| Tests | ❌ FAILURE | SIGSEGV in turso relationship tests with `keepalive-pool` feature enabled |
| Multi-Platform (ubuntu) | ❌ FAILURE | Same SIGSEGV |
| Multi-Platform (macos) | ❌ FAILURE | Same SIGSEGV |
| Code Coverage Analysis | ❌ FAILURE | CLI `test_cli_help_output` snapshot mismatch |

**Root cause** (documented in `GOAP_PR611_CI_FIX_2026-06-09.md`):
- `new_local`/`new_in_memory` routed through `TursoConfig::default()` which has `enable_pooling=true` + `enable_keepalive=true`
- Background task holding libsql connections drops outside `#[tokio::test]` runtime → SIGSEGV
- Fix: `local_config()` with `enable_pooling=false`, `enable_keepalive=false`
- Snapshot mismatch: clap trailing whitespace stripped from hand-edited `.snap` file; fix via `cargo insta accept`

**Fix status**: Documented (2026-06-09), not yet applied to branch.

**Next action**: Apply fixes from `plans/GOAP_PR611_CI_FIX_2026-06-09.md` to `feat/turso-local-mode-12832947082971821257` and push.

### Phase 1 — User Contract (P1)

| WG | Gap | Owner Skill | Status | Evidence |
|----|-----|-------------|--------|----------|
| WG-150 | `relationship show <id>` (CLI bails) | `feature-implement` | ✅ Complete | `get_relationship_by_id` in storage trait + Turso/redb + CLI (`memory-cli/src/commands/relationships/core.rs:286`) |
| WG-151 | Global cycle validation (CLI bails) | `feature-implement` | ✅ Complete | DFS helper at `memory-cli/src/commands/relationships/core.rs:450` + `all_relationships` in core |
| WG-152 | `eval --custom-thresholds` no-op | `feature-implement` | ✅ Resolved-by-typed-error | `eval.rs:412` returns explicit `anyhow::bail!` instead of silent no-op (ADR-055 accepted resolution) |
| WG-153 | Cohere silent fallback to Local | `analysis-swarm` → `feature-implement` | ✅ Resolved-by-typed-error | `embeddings/tool/execute/configure.rs:30` returns "not implemented" rather than substituting Local |
| WG-154 | Mistral binary dequantization bails | `feature-implement` | ✅ Complete | Bit-unpacking implemented at `embeddings/mistral/client.rs:158` + unit tests |
| WG-155 | AgentFS test_connection always stub | `external-signal-provider` | ✅ Complete | Multiple commits (55fc1869, 4831a6dc, abe53ced) — real SDK + config-derived status |

### Phase 2 — Telemetry Truthfulness (P2)

| WG | Gap | Owner Skill | Status | Evidence |
|----|-----|-------------|--------|----------|
| WG-156 | `pattern_match_score` hard-coded 0.8 | `feature-implement` | ✅ Complete | `time_series.rs` computes from `applied_patterns` success ratio + pattern density fallback |
| WG-157 | `memory_usage_mb` hard-coded 50.0 | `feature-implement` | ✅ Complete | `time_series.rs` uses `sysinfo` for actual process memory; 50.0 is fallback on sysinfo failure |
| WG-158 | `episode_success_rate` hard-coded 99.0 | `feature-implement` | ✅ Complete | `monitoring/types.rs` `record_episode_creation` computes from `total_episodes_created` / `total_episode_failures` |
| WG-159 | `uptime_seconds` returns `process::id()` | `feature-implement` | ✅ Complete | `OnceLock<Instant>` at `memory-cli/src/commands/health.rs:10`; captured in `main.rs:207` |
| WG-160 | Turso cache query_hits/evictions = 0 | `feature-implement` | ✅ Complete | `query_hits: 0` now only in test fixtures (commit `6a43deae`) |

### Phase 3 — Internal Debt (P3)

| WG | Gap | Owner Skill | Status | Evidence |
|----|-----|-------------|--------|----------|
| WG-161 | Cascade `analyze_query` stub | `feature-implement` | ✅ Complete | `retrieval/cascade/mod.rs` `estimate_api_call_probability` uses real heuristics (length, keyword density, code tokens) |
| WG-162 | `generate_simple_embedding` prod placeholder | `code-quality` | ✅ Complete | `memory/retrieval/helpers.rs` implements 10-dim feature hashing (domain, task type, complexity, etc.) |
| WG-163 | WG-149 `emit_event` not wired to lifecycle | `feature-implement` | ✅ Complete | `memory/episode.rs:120` + `memory/completion.rs:400` invoke `emit_event_with_cloud` |
| WG-164 | Stale "extraction not implemented" comment | `code-quality` | ✅ Complete | `extraction/tests.rs` assertion is real (no stale comment) |

### Phase 4 — Validation & Release

| WG | Step | Status |
|----|------|--------|
| WG-165 | `cargo nextest run --all` | 🟡 Queued |
| WG-166 | `cargo test --doc` | 🟡 Queued |
| WG-167 | `./scripts/quality-gates.sh` (≥90%) | 🟡 Queued |
| WG-168 | Sprint-exit `rg` audit (0 matches) | 🟡 Queued |
| WG-169 | Bump workspace to `0.1.32` + CHANGELOG | 🟡 Queued |
| WG-170 | `gh release create v0.1.32` (release-guard) | 🟡 Queued |

---

## v0.1.31 Sprint (Released ✅)

### GOAP Analysis (2026-04-20)

**Primary Goal**: Reduce CPU usage and prompt/token usage via CPU-local retrieval tiers (CSM integration), cascading query pipeline, and skills consolidation — while keeping release/package truth sources accurate ahead of the `0.1.31` version bump.

**Constraints**:
- Time: Normal
- Resources: All agents available
- Dependencies: Release/package truth must stay aligned before the `0.1.31` bump

**Complexity Level**: Complex (4+ agents, mixed execution)

**Strategy**: Hybrid (Phase 0 sequential → CPU/token work parallelized → research follow-up deferred)

### GOAP Skill Stack

- **Planning/coordination**: `goap-agent`, `agent-coordination`
- **CPU work**: `performance`, `feature-implement`, `debug-troubleshoot`
- **CSM integration**: `feature-implement`, `performance`, `test-runner`
- **Token/docs work**: `agents-update`, `memory-context`, `learn`
- **Validation**: `code-quality`, `test-runner`, `architecture-validation`

### Execution Pattern

- **Analyze**: verify release/package truth, cache hot paths, token-heavy context assembly
- **Decompose**: split work into release/package, CPU efficiency, token efficiency, and deferred research upgrades
- **Coordinate**: run CPU and token tasks in parallel after truth-source alignment
- **Validate**: require benchmarks or measurable budget reductions before version bump

### Phase 0: Release & Package Truth (Sequential)

| Task | WG | Status | Owner |
|------|----|--------|-------|
| Verify v0.1.30 release/package parity | WG-111 | ✅ Complete | github-release-best-practices |
| Bump to 0.1.31 | WG-112 | ✅ Complete | feature-implement |
| Refresh stale truth sources | WG-113 | ✅ Complete | agents-update |

### Phase 1: CPU Efficiency (Parallel)

| Task | WG | Status | Owner | Notes |
|------|----|--------|-------|-------|
| Reduce QueryCache contention | WG-114 | ✅ Complete | performance | `parking_lot::RwLock` already implemented in `memory-core/src/retrieval/cache/lru.rs` |
| Replace placeholder cached retrieval | WG-115 | ✅ Complete | feature-implement | Verified: QueryCache fully implemented (273 LOC LRU+TTL+metrics), no placeholders |
| Tune compression/cache CPU budget | WG-116 | ✅ Complete | performance | Verified: Constants in `memory-core/src/constants.rs` (CACHE_SIZE=1000, TTL=3600s, MAX_EPISODES=10000, SIMILARITY_THRESHOLD=0.7) |

### Phase 1.5: CSM Integration ✅ Complete (crate dependency)

**Implementation**: Added `chaotic_semantic_memory = "0.3.2"` as optional dependency with `csm` feature flag. Re-exports in `memory-core/src/retrieval/mod.rs`.

| Task | WG | Status | Owner |
|------|----|--------|-------|
| BM25 keyword index from CSM | WG-128 | ✅ Complete | crate dependency |
| HDC local embedding fallback | WG-129 | ✅ Complete | crate dependency |
| ConceptGraph ontology expansion | WG-130 | ✅ Complete | crate dependency |
| Cascading retrieval pipeline | WG-131 | ✅ Complete | feature-implement | 732 LOC, 20+ tests, full 4-tier cascade implementation |

### Phase 2: Token Efficiency (Parallel with Phase 1)

| Task | WG | Status | Owner | Notes |
|------|----|--------|-------|-------|
| Implement BundleAccumulator window | WG-117 | ✅ Complete | feature-implement | Fully implemented in `memory-core/src/context/accumulator.rs` with 20+ tests |
| Add hierarchical/gist reranking | WG-118 | ✅ Complete | feature-implement | |
| Compact high-frequency skills/docs | WG-119 | ✅ Complete | agents-update | 4 skills compacted: web-doc-resolver (187→84), test-patterns (161→86), build-rust (143→84), code-quality (137→74) |

### Phase 3: Research-Inspired Retrieval Upgrades (P2 Priority)

| Task | WG | Status | Owner | Paper |
|------|----|--------|-------|-------|
| Reconstructive retrieval windows | WG-120 | ✅ Complete | feature-implement | E-mem (arXiv:2601.21714) - 462 LOC, 30+ tests |
| Execution-signature retrieval | WG-121 | ✅ Complete | feature-implement | APEX-EM (arXiv:2603.29093) - 593 LOC, 30+ tests |
| Scope-before-search shard routing | WG-122 | ✅ Complete | feature-implement | ShardMemo (arXiv:2601.21545) - 635 LOC, 27 tests |
| Procedural memory type | WG-124 | ✅ Complete (PR #569) | feature-implement | ParamAgent |
| LottaLoRA local classifier | WG-132 | ✅ Complete (evaluation doc) | feature-implement | LottaLoRA |
| Agentic memory taxonomy alignment | WG-133 | ✅ Complete (evaluation doc) | agents-update | Anatomy of Agentic Memory |
| DAG-based state management | WG-134 | ✅ Complete | feature-implement | arXiv:2602.22398 — ~1,320 LOC in `context/dag/`, 24 tests, ~86% token reduction |
| Temporal graph edges | WG-123 | ✅ Complete (PR #570) | feature-implement | REMem (ICLR 2026, arXiv:2602.13530) — weighted traversal, pattern edges, significance weights |
| MemCollab cross-agent memory | WG-126 | ✅ Complete (PR #572) | feature-implement | MemCollab (arXiv:2603.23234) — trajectory distillation, contrastive adapter, collaborative prototypes |
| Federated HDC multi-agent memory | WG-135 | 🔵 Evaluated (evaluation doc) | feature-implement | arXiv:2603.20037 |
| CloudEvents EventEmitter | WG-149 | ✅ Complete | feature-implement | ADR-054 — CloudEvents 1.0 spec

### Phase 5: CI Optimization (2026-04-28)

| Task | WG | Status | Owner | Notes |
|------|----|--------|-------|-------|
| Update benchmarks.yml paths trigger | WG-150 | ✅ Complete | feature-implement | PRs use `paths` only, push uses `paths-ignore` |
| Add skip-benchmarks label support | WG-151 | ✅ Complete | feature-implement | Label check in benchmark job condition |
| Make benchmark informational (not required) | WG-152 | ✅ Complete | feature-implement | `regression-check` uses `continue-on-error` |
| Update AGENTS.md with CI guidelines | WG-153 | ✅ Complete | agents-update | CI optimization section in AGENTS.md |
| Create ci-optimization skill | WG-154 | 🔵 Deferred (optional) | skill-creator | Not needed; covered by ci-fix + github-workflows skills |

**CI Optimization Result**: PR CI time reduced from ~50+ min to ~15-18 min for non-perf PRs. Benchmarks (~54 min) only run when perf-critical paths change or `skip-benchmarks` label is absent. See `plans/FIX_CI_AND_RELEASE_STATE.md` for 2026-05-21 analysis.

### WG-131 CascadeRetriever Status (Updated 2026-05-01)

The CascadeRetriever has a full CSM implementation behind the `csm` feature flag
(BM25 Tier 1, HDC Tier 2, ConceptGraph Tier 3 with curated ontology, API fallback Tier 4).
All 4 tiers are now implemented and tested.

**Status**: ✅ Complete — CSM path complete with ConceptGraph ontology, 30 tests passing.

### Phase 4: Housekeeping (Parallel)

| Task | WG | Status | Owner |
|------|----|--------|-------|
| Create `performance` skill | WG-136 | ✅ Complete | skill-creator |
| Prune skills 40 → ≤35 | WG-137 | ✅ Complete | agents-update |
| Fix CURRENT.md contradictions | WG-138 | ✅ Complete | agents-update |
| Refresh CODEBASE_ANALYSIS_LATEST.md | WG-139 | ✅ Complete | agents-update |

### Quality Gates
- **Gate 1** (after Phase 0): release/package/version truth sources all agree
- **Gate 1.5** (after Phase 1.5): BM25+HDC retrieval tested, cascading pipeline passes integration tests, API call count reduced
- **Gate 2** (after Phase 1-2): CPU hot paths benchmarked, token budget reduced, all tests pass
- **Gate 3** (after Phase 3): retrieval upgrades validated without coverage regressions

### Recommended Skill Invocation Order

1. `goap-agent`
2. `agent-coordination`
3. `performance` or `feature-implement` depending on work item
4. `agents-update` for high-frequency doc/skill compaction
5. `code-quality` and `test-runner` before closing a work group

## v0.1.30 Sprint (Complete ✅)

### Cross-Repo Impact Analysis (2026-04-09)

Impact analysis of `d-o-hub/github-template-ai-agents` and `d-o-hub/chaotic_semantic_memory` identified unadopted patterns and integration opportunities. All P1/P2 items adopted.

### P1: Runtime Patterns from `chaotic_semantic_memory`

| Task | WG | Status | Details |
|------|----|--------|---------|
| `MemoryEvent` broadcast channel | WG-103 | ✅ Complete | `tokio::broadcast` channel + subscribe() method + emit_event() helper |
| `select_nth_unstable_by` for top-k | WG-104 | ✅ Complete | `search::top_k` module with O(n) partial sort utilities |
| Idempotent cargo publish | WG-105 | ✅ Already exists | Version check step in `publish-crates.yml` |

### P2: Agent Harness Patterns from `github-template-ai-agents`

| Task | WG | Status | Details |
|------|----|--------|---------|
| Add `memory-context` skill | WG-106 | ✅ Complete | `.agents/skills/memory-context/SKILL.md` using do-memory-cli |
| Add `learn` skill (dual-write learning) | WG-107 | ✅ Complete | `.agents/skills/learn/SKILL.md` with dual-write pattern |

### P3: Future Backlog from CSM

| Task | WG | Status | Details |
|------|----|--------|---------|
| Version-retained persistence | WG-108 | 🔵 Backlog | Track concept drift across episode versions |
| `BundleAccumulator` sliding window | WG-109 | ✅ Complete (WG-117) | Recency-weighted context for pattern retrieval |
| SIMD-accelerated similarity | WG-110 | 🔵 Backlog | Marginal perf gain — defer until benchmarks justify |

## v0.1.29 Sprint (Complete ✅)

| Task | WG | Status | Details |
|------|----|--------|---------|
| Version bump to 0.1.29 | WG-094 | ✅ Complete | Workspace + inter-crate deps updated |
| Archive stale GOAP plans | WG-095 | ✅ Complete | Already archived in v0.1.28 sprint |
| Remove WASM sandbox | WG-096 | ✅ Complete | -6,982 LOC, 11 files removed |
| Remove wasmtime/rquickjs deps | WG-097 | ✅ Complete | Cargo.toml cleanup |
| Implement vector_top_k search | WG-098 | ✅ Complete | Native DiskANN queries |
| Embedding format migration | WG-099 | ✅ Complete | JSON → F32_BLOB |
| Integration tests | WG-100 | ✅ Complete | Vector search tests |
| Split >500 LOC files | WG-101 | ✅ Complete | 6 files split |
| Dead code audit | WG-102 | ✅ Complete | 31 → target ≤25 |

## v0.1.28 Sprint (Complete ✅)

| Task | WG | Status |
|------|----|--------|
| DyMoE routing-drift protection | WG-089 | ✅ Affinity gating |
| Dual reward scoring | WG-090 | ✅ Stability + novelty signals |
| Merge AI spam detector PR #406 | WG-091 | ✅ Merged |
| Dependabot alerts | WG-092 | ✅ Tracked (transitive) |
| CodeQL cleartext logging | WG-093 | ✅ Fixed |
| Plans consolidation | ACT-096 | ✅ 87% noise reduction |

## v0.1.27 Sprint (Complete ✅)

| Task | WG | Status |
|------|----|--------|
| Bayesian ranking | WG-073 | ✅ Wilson score from attribution data |
| Diversity retrieval | WG-077 | ✅ MMR reranking |
| Episode GC/TTL | WG-075 | ✅ Retention policy |
| MCP Server Card | WG-078 | ✅ `.well-known/mcp.json` |
| spawn_blocking audit | WG-079 | ✅ CPU-heavy async paths |
| GitHub Pages | WG-084 | ✅ mdBook + cargo doc |
| llms.txt | WG-085 | ✅ LLM context file |

## v0.1.26 Release (Complete ✅)

| Task | Status |
|------|--------|
| Crate renaming (`memory-*` → `do-memory-*`) | ✅ |
| Version bump `0.1.25` → `0.1.26` | ✅ |
| crates.io publish (all 4 crates) | ✅ |
| Binary names (`do-memory-mcp-server`, `do-memory-cli`) | ✅ |
| GitHub Release (tag v0.1.26, multi-platform) | ✅ |

---

## Release Process (MANDATORY)

**NEVER manually create GitHub releases.** Always use the automated workflow.

### How releases work
1. Bump version in `Cargo.toml` (workspace)
2. Update `CHANGELOG.md` / `ROADMAP_ACTIVE.md` / `STATUS/CURRENT.md`
3. Commit + push to `main`
4. Push a git tag: `git tag v<VERSION> && git push origin v<VERSION>`
5. The `release.yml` workflow triggers automatically on tag push:
   - Preflight: verifies tag matches `Cargo.toml` version
   - Builds multi-platform artifacts via `cargo-dist`
   - Creates GitHub Release with changelog + artifacts
6. The `release-drift.yml` monitors unreleased commits and auto-creates drift issues

### What NOT to do
- Do NOT use `gh release create` manually
- Do NOT create releases from non-main branches
- Do NOT skip the preflight tag-version check
- Do NOT bypass the workflow for "quick releases"

### Release readiness checklist
- [ ] All required CI checks green on `main`
- [ ] `cargo nextest run --all` passes
- [ ] `cargo test --doc` passes
- [ ] `./scripts/quality-gates.sh` passes (≥90% coverage)
- [ ] `./scripts/code-quality.sh clippy --workspace` clean
- [ ] Version bumped in `Cargo.toml`
- [ ] Changelog updated
- [ ] Tag pushed (triggers workflow)

---

## Key Metrics

| Metric | Value | Target |
|--------|-------|--------|
| Workspace version | 0.1.33 | — |
| Latest GitHub release | v0.1.32 | verified 2026-07-02 |
| Publishable workspace crates | 6 | all at 0.1.31 |
| Total tests | 3,282 | — |
| Ignored tests | 164 skipped | ceiling ≤165 |
| `allow(dead_code)` (prod) | 0 | ≤25 — ✅ Met (all 38 dead_code warnings eliminated, verified 2026-05-16) |
| Clippy | Clean | 0 warnings |
| Doctests | 0 failures | 0 |
| Skills count | 31 | ✅ target ≤35 met |
| Skills LOC | re-audit | minimize high-frequency prompt load |
| Clippy suppressions (lib.rs) | 64 | ≤20 |
| Files >500 LOC | 0 | 0 |
| Cargo audit | 3 unmaintained warnings | transitive |
| Dependabot alerts | 3 open | all transitive, tracked |
| CodeQL alerts | 0 open | ✅ fixed |
| CSM integration | Not started | BM25+HDC+ConceptGraph cascade |

---

## Active Issues / Blockers

| Issue | Status | Notes |
|-------|--------|-------|
| CLI_TURSO_SEGFAULT | Known | libsql upstream memory corruption; 71 Turso tests `#[ignore]` (ADR-027) |
| Dependabot alerts (3) | Tracked | All transitive deps, documented in `audit.toml` / `deny.toml` |
| CodeQL alert #60 | ✅ Fixed | PR #420 merged; CodeQL reports `fixed` |
| Issue #401 | Pending | Auto-closes when PR #406 merges |

---

## DyMoE Feature Impact Analysis (Issue #419)

### Architecture Touchpoints

| Component | File | Current Behavior | DyMoE Change |
|-----------|------|------------------|-------------|
| Pattern extraction | `memory/learning.rs` | Extracts & stores all patterns unconditionally | Add affinity gate before `store_pattern` |
| Effectiveness tracker | `patterns/effectiveness/calculator.rs` | Uniform decay by min_effectiveness (0.3) | Add `affinity_clarity` as second gating dimension |
| Reward scoring | `reward/mod.rs` | Single composite `RewardScore` (total/base/efficiency/etc.) | Add `stability_score` + `novelty_score` fields |
| Pattern types | `pattern/types.rs` | `PatternEffectiveness` tracks success_rate + avg_reward_delta | No schema change needed |
| Hybrid extractor | `patterns/extractors/hybrid.rs` | Runs 4 extractors, clusters, deduplicates | Insert affinity classifier before clustering |
| Cosine similarity | `embeddings/similarity.rs` | `cosine_similarity(a, b) -> f32` | Reuse existing — no new dependency |
| Recommendation | `memory/pattern_search/recommendation.rs` | Multi-signal scoring (semantic + recency + success) | Wire `EpisodeAssignmentGuard` into scoring |

### Impact Assessment

| Change | LOC Estimate | Risk | Files Modified | New Files |
|--------|-------------|------|---------------|-----------|
| `EpisodeAssignmentGuard` (WG-089 part 2) | ~50 | Low | `effectiveness/calculator.rs` | 0 |
| `PatternAffinityClassifier` (WG-089 part 1) | ~80 | Medium | `learning.rs`, `hybrid.rs` | `patterns/affinity.rs` |
| `DualRewardScore` (WG-090) | ~60 | Low | `reward/mod.rs` | 0 |
| DB schema migration | ~20 | Low | turso schema | 0 |
| Tests | ~200 | None | — | `tests/dymoe_*.rs` |
| **Total** | **~410** | **Medium** | **4–5** | **1–2** |

### Recommended Execution Order (from issue #419)
1. `EpisodeAssignmentGuard` — smallest diff, highest leverage
2. `PatternAffinityClassifier` — depends on cosine_similarity infra (already exists)
3. `DualRewardScore` — extends existing `RewardScore`, backward-compatible

### Key Finding
The existing `cosine_similarity` function in `embeddings/similarity.rs` can be reused directly for `compute_drel()`. No new embedding infrastructure needed. The `PatternEffectiveness` already tracks `success_rate` which maps to the `success_rate` dimension of the `EpisodeAssignmentGuard`.

---

## Self-Learnings (Consolidated)

Patterns extracted from v0.1.17–v0.1.27 sprint history (34 sessions, 234 msgs, 97 commits):

### API / Dependency Changes
- **redb 3.x**: `begin_read()` is on the `ReadableDatabase` trait — import accordingly
- **rand 0.10**: `thread_rng()` → `rand::rng()`, `gen()` → `random()`, `gen_range()` → `random_range()`

### Development Workflow
- **Turso local dev**: Use `turso dev` — no auth token needed
- **Doctest quality**: New features must have tested doctests before merge
- **File size invariant**: Templates extraction prevents LOC growth (≤500 LOC per file)
- **Integration tests**: Feature implementation without integration tests leaves gaps
- **Documentation sync**: ROADMAP / CURRENT must stay in sync with releases
- **Plan document drift**: Always verify metrics by running actual commands, not trusting stale docs

### CI / GitHub
- **PR supersession**: Track supersession chains (e.g., #388 → #389 → #391) to avoid referencing stale PRs
- **codecov/patch**: Not a required CI check — configure thresholds in `codecov.yml`, don't block merges
- **Jules PRs**: Reconcile external agent PRs before planning more work — they may implement "pending" items

### Quality Gates
- **Ignored test ceiling**: Monitor the 125 ceiling metric; actual count can creep close
- **Tool selection**: Target Bash:Grep ratio of 2:1 (historical 17:1 indicates over-reliance on shell)
- **Atomic commits**: One change per commit — 5 excessive_changes instances in early sprints
- **Wrong approach**: Read 3+ source files before implementing — 8 instances of proceeding without patterns

---

## Cross-References

| Document | Location |
|----------|----------|
| Archived Execution Plans | `plans/archive/2026-03-consolidation/` |
| Active Roadmap | `plans/ROADMAPS/ROADMAP_ACTIVE.md` |
| Latest Validation | `plans/STATUS/VALIDATION_LATEST.md` |
| Latest Gap Analysis | `plans/STATUS/GAP_ANALYSIS_LATEST.md` |
| ADR Index | `plans/adr/` |
