# Project Status — Self-Learning Memory System

**Last Updated**: 2026-07-18 (PR swarm + permanent YAML wait fix)
**Released Version**: v0.1.35
**Workspace Version**: 0.1.36 (post-release development; unreleased)
**Active Sprint**: Merge open PR stack with all CI green
**Branch**: multi-PR swarm (#860 → #870 → #872)
**Edition**: Rust 2024

## Sprint 2026-07-18 — Open PR swarm (GOAP)

| PR | Title | Status |
|----|-------|--------|
| #860 | S1.7 + K3.1b/W2.1b | 🟡 CI re-run after permanent yaml-lint fix |
| #870 | Harness engineering sprint | 🟡 macos snapshot + HARNESS.md + main sync |
| #872 | release-cadence-manager | 🟡 rebased on #870; waits on stack |

**Merge order**: #860 → #870 → #872  
**Validation**: `plans/STATUS/VALIDATION_LATEST.md`  
**Lessons**: LESSON-021 (yaml wait cancel), LESSON-022 (f32 insta)

## Sprint 2026-07-18 — S1.7 + K3.1b + W2.1b

| Item | Status |
|------|--------|
| S1.7 recursive redaction + rotation size + bounded writer | ✅ |
| K3.1b `.github/workflows/skill-evals.yml` | ✅ |
| W2.1b `validate-gate-contract.sh --ci-parity` | ✅ |
| Plans + LESSONS-018/019 | ✅ |
| Permanent CI: yaml-lint ungated + 40m waits | ✅ code |
| PR + CI | 🟡 re-running |

**Plan**: `plans/GOAP_MISSING_TASKS_S17_K31B_W21B_2026-07-18.md`

## Release v0.1.35 — ✅ SHIPPED

| Item | Status |
|------|--------|
| PR #850 ADR-075/076 open-issue fixes | ✅ Merged |
| PR #851 K3.1 skill evals + W2.1 gate contract | ✅ Merged |
| Tag `v0.1.35` + `release.yml` | ✅ |

## Open Issues vs Codebase — 2026-07-17

| Issue | Verdict | Status |
|-------|---------|--------|
| #849 | Release cadence critical | 🟡 Tag v0.1.35 |
| #847 | ADR-075 durable complete + `episode fail` | ✅ Merged #850 |
| #845 | ADR-076 pattern empty diagnostics | ✅ Merged #850 |
| #846 | Config precedence docs | ✅ Merged #850 |

**Plan**: `plans/GOAP_OPEN_ISSUES_ANALYSIS_2026-07-17.md`  
**ADRs**: ADR-075, ADR-076

## Release Drift Prevention #843 — ✅ CODE COMPLETE

| Item | Status |
|------|--------|
| One canonical issue updated in place | ✅ Implemented |
| PR cadence warning/blocking thresholds | ✅ Implemented |
| 30-commit / 14-day hard limit | ✅ Implemented |
| Trusted release-preparation escape hatch | ✅ Implemented |
| Exact-tag-only release manager | ✅ Implemented |
| Shell regression suite | ✅ Passing |

**Plan**: `plans/GOAP_RELEASE_DRIFT_ISSUE_843_2026-07-17.md`

## Sprint 2026-07-16b — S1.3–S1.6 + W2.2 ✅ CODE COMPLETE

| Item | Description | Status |
|------|-------------|--------|
| S1.3 | Short write locks; no backend await under episodes write | ✅ Fixed |
| S1.4 | Capacity eviction deletes cache/Turso episodes + embeddings | ✅ Fixed |
| S1.5 | `EmbeddingHealth` Real/DegradedMock/Unavailable; mock fail-closed | ✅ Fixed |
| S1.6 | Retry queue timeout; first attempt free; reject zero concurrency | ✅ Fixed |
| W2.2 | cargo deny blocking; cargo audit no longer soft-passes | ✅ Fixed |
| #843 | Release drift (25 commits since v0.1.34) | 🟡 Prevention complete; v0.1.35 release still required |

**Plan**: `plans/GOAP_MISSING_TASKS_S13_S16_W22_2026-07-16.md`

## Sprint 2026-07-16 — Missing Tasks Swarm ✅ MERGED (PR #840)

| Item | Description | Status |
|------|-------------|--------|
| #837 | Public `fuzzy_match` rustdoc restored | ✅ Fixed |
| S1.2 | Retrieval cache identity includes TaskContext | ✅ Fixed |
| W2.3 | `build-rust.sh` accepts hyphenated packages | ✅ Fixed |
| W2.6 | Production source files ≤500 LOC | ✅ Fixed |
| S1.1a/D3.2 | Fail-closed execute_agent_code docs | ✅ Fixed |

**Plan**: `plans/GOAP_MISSING_TASKS_SWARM_2026-07-16.md`

## v0.1.35 Sprint — CLI UX Patch ✅ ON MAIN

| Issue | Description | Status |
|-------|-------------|--------|
| #831 | Pattern extraction not listable across processes | ✅ Fixed |
| #830 | `--db-path` / `MEMORY_DB_PATH` ignored for redb | ✅ Fixed |
| #829 | Config file format hard to discover | ✅ Fixed |
| #832 | `storage_mode` config placement unclear | ✅ Fixed |
| #828 | Release drift | 🟡 Ready to tag after merge |

**Plan**: `plans/GOAP_CLI_UX_PATCH_0.1.35_2026-07-15.md`

---

## v0.1.34 Sprint — COMPLETE ✅ (Released)

| Phase | Issue | Description | Status |
|-------|-------|-------------|--------|
| CI Fix | — | `test_security_bypass_attempts` assertion relaxed | ✅ Merged (PR #806) |
| CI Fix | — | Benchmark timeout increased to 55min | ✅ Merged (PR #806) |
| Docs | #770 | Pre-built binary installation section in README | ✅ Merged (PR #806) |
| Docs | #743 | Storage module pipeline documentation | ✅ Merged (PR #806) |
| Feature | #753 | Retry budgets and backpressure controls | ✅ Merged (PR #806) |
| Feature | #749 | Turso connection pool enhancements + ADR-056 | ✅ Merged (PR #806) |
| Feature | #746 | WASM compile-check CI job | ✅ Merged (PR #806) |
| Publish | #770 | CLI crates.io publish prep | ✅ Merged (PR #806) |

**All 6 open issues resolved.** Release tag v0.1.34 pending.

## v0.1.33 Sprint — COMPLETE ✅ (Released)

| Phase | WGs | Strategy | Status |
|-------|-----|----------|--------|
| P1 — Release | WG-175 | Sequential | ✅ Complete (v0.1.33 tag exists) |
| P2 — CI Health | WG-176..WG-179 | Parallel | ✅ Complete (PR #675, #681) |
| P3 — Code Quality | WG-180..WG-181 | Parallel | ✅ Complete (PR #675) |
| P4 — Architecture | WG-182 | Sequential | ✅ Complete (PR #675) |
| P5 — DevX Backlog | WG-183..WG-184 | Parallel | ✅ WG-183 done; WG-184 queued |
| P6 — Code Health | WG-185 | Parallel | ✅ WG-185 done (LOC boundary splits) |

**Plan**: `plans/GOAP_COMPREHENSIVE_ANALYSIS_2026-06-28.md`

---

## Key Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Workspace members | 9 | — | — |
| Workspace version | 0.1.33 | — | ✅ Released (tag v0.1.33 exists) |
| Latest GitHub release | v0.1.33 | — | ✅ Published |
| Publishable workspace crates | 6 | — | ✅ All at `0.1.33` in workspace  |
| Commits since v0.1.34 | 0 | 0 | ✅ Released |
| Clippy (default features) | Clean | Clean | ✅ |
| Clippy (--all-features) | Clean | 0 | ✅ Fixed in PR #675 |
| Production src files >500 LOC | 0 | 0 | ✅ Fixed in PR #675 (wrapper_backend.rs split) |
| Push CI (main) | Green | Green | ✅ |
| Scheduled Security | Fixed | Green | ✅ WG-176 done (PR #675) |
| Nightly Full Tests | Fixed | Green | ✅ WG-177 done (PR #675 disk cleanup) |
| Mutation Testing | Fixed | Completes | ✅ WG-178 done (scoped to memory-core) |
| Fuzzing | Green | Green | ✅ |
| Security audit (`cargo deny`) | Clean | Clean | ✅ Fixed in PR #682 |
| Open issues | 10 | 0 | 🟡 (#773, #772, #771, #784, #770, +5 others) |
| Open PRs | 3 | 0 | 🟡 PRs #787, #788, #789 |
| Fuzz harness | Present | Present | ✅ |
| Property test files | 17 | ≥13 | ✅ |
| MSRV | Rust 2024 / stable 1.95.0 | — | ✅ |
| GitHub Actions | Latest (Node 24) | Up to date | ✅ PR #681 |
| Dependencies | Patch current | Current | ✅ PRs #682, #684, #678 |

## v0.1.30 Sprint Highlights

- **MemoryEvent Broadcast**: `tokio::broadcast` channel for episode lifecycle events
- **Top-k Optimization**: O(n) `select_nth_unstable_by` for retrieval hot paths
- **Zero-copy Retrieval Caching**: Bolt optimization for episodic memory
- **Agent Skills**: Added `memory-context` and `learn` skills

## v0.1.31 Planning Focus

- **CSM integration**: Cascading retrieval pipeline (BM25 → HDC → ConceptGraph → API) to eliminate 50-70% of embedding API calls
- **CPU efficiency**: QueryCache contention, cached retrieval wiring, compression/cache thresholds
- **Token efficiency**: bounded context windows, hierarchical/gist reranking, compact high-frequency skills/docs
- **Housekeeping**: Create missing `performance` skill, prune skills 40→≤35, fix metric contradictions, refresh stale analysis docs
- **Release/package hygiene**: keep GitHub release, package versions, and planning docs aligned before the `0.1.31` bump

## v0.1.28 Release Highlights

- **DyMoE Routing-Drift Protection**: Affinity gating and dual reward scoring
- **CodeQL Fixed**: Cleartext logging alert resolved (WG-093)
- **Dependabot Analyzed**: All 3 alerts are transitive dependencies (accepted risk)

## v0.1.26 Release Highlights

- **Crate Renaming**: All crates renamed from `memory-*` to `do-memory-*` namespace
- **crates.io Publishing**: All 4 crates published successfully
- **Binary Names**: `do-memory-mcp-server`, `do-memory-cli`
- **GitHub Release**: v0.1.26 with multi-platform binaries

---

## Open Items (2026-06-28 Analysis)

### Open Issues

| # | Title | Labels | Status |
|---|-------|--------|--------|
| [#674](https://github.com/d-o-hub/rust-self-learning-memory/issues/674) | ⚠️ Release drift: 94 unreleased commits since v0.1.32 | release-drift | 🔴 Open — WG-175 |
| [#652](https://github.com/d-o-hub/rust-self-learning-memory/issues/652) | Automate llms.txt and full LLM context generation | — | 🟢 Resolved — WG-183 (`scripts/generate-llms-txt.sh`) |
| [#653](https://github.com/d-o-hub/rust-self-learning-memory/issues/653) | Evaluate VERSION file adoption | — | 🔴 Open — WG-184 |

### Open PRs

None.

### CI Health

| Workflow | Status | Root Cause |
|----------|--------|------------|
| Push CI (main) | ✅ Green | — |
| Security (scheduled) | ❌ Failing | 3 gitleaks false positives (WG-176) |
| Nightly Full Tests | ❌ Failing | Disk exit-95 infra (WG-177) |
| Mutation Testing | ❌ Cancelled | 6h ceiling exceeded (WG-178) |
| Fuzzing | ✅ Green | — |

### Code Quality

| Finding | Location | WG |
|---------|----------|-----|
| 4× excessive_nesting + 1× unnecessary_wraps | `mistral/client.rs` | WG-180 |
| File >500 LOC (537) | `cache/wrapper.rs` | WG-181 |
| Non-CSM cascade returns empty silently | `cascade/mod.rs:207` | WG-182 |
authored 2026-06-09 and needs to be pushed to the PR branch to trigger re-run.

### Recently Merged PRs

| # | Title | Status |
|---|-------|--------|
| 547 | chore(ci): resolve merge conflicts with main; use create-pull-request | ✅ Merged 2026-05-16 |
| 546 | fix(mcp): enforce input bounds clamping on all public tool parameters (CWE-770) | ✅ Merged 2026-05-16 |
| 548 | chore: YAML frontmatter validation + code quality | ✅ Merged 2026-05-16 |
| 545 | chore: YAML frontmatter validation and Dependabot fix plan | ✅ Merged 2026-05-16 |
| 544 | feat: CloudEvents EventEmitter, ConceptGraph, evaluations | ✅ Merged 2026-05-15 |
| 542 | code health: update coverage badge generation workflow | ✅ Merged 2026-05-14 |
| 454 | fix(persistence): SQL injection in metadata query | ✅ Merged 2026-04-18 (P0 security fix) |

### Security: Dependabot Alerts (Accepted Risk — Transitive)
| # | Dependency | Severity | Notes |
|---|-----------|----------|-------|
| 12 | rustls-webpki | Medium | CRL matching logic bug; transitive via libsql |
| 2 | lru | Low | IterMut Stacked Borrows violation; transitive |
| 1 | libsql-sqlite3-parser | Low | Crash on invalid UTF-8; transitive via libsql |

### Security: Code Scanning
| Status | Notes |
|--------|-------|
| ✅ No open alerts | CodeQL cleartext logging alert #60 resolved (WG-093) |

### Known Issues (P1)
| Issue | Status | Workaround |
|-------|--------|------------|
| CLI Turso segfault | Under investigation | Use redb-only or `turso dev` server |

## Completed Phases

All research/implementation phases are complete:

- ✅ **Phase 1 (PREMem)**: Quality assessment (89% accuracy)
- ✅ **Phase 2 (GENESIS)**: Capacity management (88–2307× above targets)
- ✅ **Phase 3 (Spatiotemporal)**: Retrieval accuracy (+150%, 4.4× target)
- ✅ **Phase 4 (Benchmarking)**: All research claims validated

## v0.1.22 Features (ADR-044 — Polished)

| Feature | Core | MCP | CLI | Tests | Doctests | Snapshots |
|---------|------|-----|-----|-------|---------|-----------|
| Actionable Playbooks | ✅ | ✅ | ✅ | 26 | ✅ Fixed | ✅ |
| Recommendation Attribution | ✅ | ✅ | ✅ | 8 | ✅ Fixed | ✅ |
| Episode Checkpoints/Handoff | ✅ | ✅ | ✅ | 6 | ✅ | ✅ |
| Recommendation Feedback | ✅ | ✅ | ✅ | 3 | ✅ | ✅ |

## Key Capabilities

- **Multi-provider embeddings**: 5 providers (OpenAI, Cohere, Ollama, Local, Custom)
- **MCP server**: Full tool registry with lazy loading (ADR-024)
- **Episode management**: Full lifecycle with relationships, tagging, patterns
- **Playbooks**: Template-driven actionable recommendations from patterns
- **Attribution**: Recommendation session tracking and feedback loops
- **Durable attribution storage**: Turso/redb persistence for sessions, feedback, and metrics (WG-051 validated via `tests/attribution_integration_test.rs`)
- **Durable checkpoint/handoff storage**: Turso episode checkpoint serialization + restart-safe handoff resume metadata persistence (WG-052 validated via `tests/checkpoint_integration_test.rs`)
- **Checkpoints**: Mid-task state snapshotting and agent handoff packs
- **Storage**: Turso/libSQL (persistent) + redb (cache) dual-layer
- **Security**: Path traversal protection, parameterized SQL (WASM removed in v0.1.29)
- **CI/CD**: 6 workflows all passing, cargo-nextest, mutation testing
- **Performance**: Exceeds all targets (17–2307×)

## CSM Cascading Retrieval (Completed ✅)

**Integration Method**: Crate dependency (`chaotic_semantic_memory = "0.3.2"`), not source code copy.
**Implementation**: 732 LOC, 20+ tests for full 4-tier cascade (`CascadeRetriever` behind `csm` feature flag).

| Tier | Method | Source | API Calls | Status |
|------|--------|--------|-----------|--------|
| 1 | BM25 keyword index | `chaotic_semantic_memory` crate | 0 | ✅ WG-128 Complete |
| 2 | HDC 10,240-bit encoding | `chaotic_semantic_memory` crate | 0 | ✅ WG-129 Complete |
| 3 | ConceptGraph expansion | `chaotic_semantic_memory` crate | 0 | ✅ WG-130 Complete |
| 4 | API embedding (fallback) | OpenAI/Cohere/Ollama | 1 | Existing |
| Pipeline | Cascade orchestrator | New `CascadeRetriever` | 0-1 | ✅ WG-131 Complete (732 LOC, 20+ tests) |

---

## CI Health (Post-Remediation 2026-07-02)

| Workflow | Status | Notes |
|----------|--------|-------|
| Push CI (main) | ✅ Green | — |
| Security (scheduled) | ✅ Fixed | WG-176 done (PR #675) |
| Nightly Full Tests | ✅ Fixed | WG-177 done (PR #675 disk cleanup) |
| Mutation Testing | ✅ Fixed | WG-178 done (scoped to memory-core) |
| Fuzzing | ✅ Green | — |

---

## Critical Issues for v0.1.22 Tag — ALL RESOLVED

| Issue | Priority | Status |
|-------|----------|--------|
| ~~2 failing doctests (attribution, playbook)~~ | P0 | ✅ Fixed |
| ~~1 test timeout (quality_gate_no_clippy_warnings)~~ | P0 | ✅ Fixed |
| ~~3 files >500 LOC~~ | P0 | ✅ Fixed |

## Quality Debt

| Item | Current | Target | Notes |
|------|---------|--------|-------|
| Ignored tests | 164 | ≤165 ceiling | 70 Turso (upstream libsql bug), rest by design |
| `#[allow(dead_code)]` (prod src) | 0 | ≤25 | ✅ Met (all 38 eliminated; removed unused params, cfg-gated test-only utils, verified 2026-05-17) |
| Skills count | 31 | ≤35 | ✅ Target met (5 skills merged/removed) |
| Broken markdown links | 0 active | ≤80 | ✅ 101 archived-only (acceptable) |
| Snapshot tests | 80 | ≥80 | ✅ Target met |
| Property test files | 17 | ≥13 | ✅ Exceeds target |

## Removed Features

| Feature | Version Removed | Reason |
|---------|-----------------|--------|
| WASM sandbox (wasmtime) | v0.1.29 (WG-096) | Maintenance burden, security concerns, 1,899 LOC removed |

## Infrastructure (Completed via PR #391)

| Item | Since | Status |
|------|-------|--------|
| Changelog automation (git-cliff) | v0.1.17 | ✅ `.github/workflows/changelog.yml` |
| libsql version monitor (T5.3) | v0.1.20 | ✅ `scripts/check-libsql-version.sh` |
| Structured tech-debt registry | v0.1.17 | ✅ `docs/TECH_DEBT.md` |

## Infrastructure Backlog

| Item | Since | Priority |
|------|-------|----------|
| Nightly trend tracking (T5.2) | v0.1.20 | P3 |
| CLI workflow parity generator | v0.1.17 | P3 |

## Cross-References

- **Gap analysis**: [GAP_ANALYSIS_LATEST.md](GAP_ANALYSIS_LATEST.md)
- **Execution plan**: [GOAP Execution Plans](../GOAP_STATE.md)
- **Active roadmap**: [ROADMAP_ACTIVE.md](../ROADMAPS/ROADMAP_ACTIVE.md)
- **ADRs**: [ADR Directory](../adr/)
- **Comprehensive analysis**: [COMPREHENSIVE_ANALYSIS_2026-04-21.md](COMPREHENSIVE_ANALYSIS_2026-04-21.md)
- **CSM repo**: <https://github.com/d-o-hub/chaotic_semantic_memory>
