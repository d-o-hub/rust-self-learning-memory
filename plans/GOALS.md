# GOAP Goals Index

- **Last Updated**: 2026-07-18 (missing-tasks master: S1.2/S1.4b/S1.1b/K3.2/W2)
- **Status**: Active — PR #873 open (workspace 0.1.36, tag v0.1.35)
- **Plan**: `plans/GOAP_MISSING_TASKS_MASTER_2026-07-18.md`
- **Backlog**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md`

## 2026-07-18c Goals (Missing tasks Wave 1–3 — PR #873)

| Goal | Plan ref | Status |
|------|----------|--------|
| Retrieval cache identity + provenance remainder | S1.2 / ADR-074 | ✅ |
| Eviction reconciliation for partial failures | S1.4b | ✅ |
| Sandbox-dev quarantine + source reachability | S1.1b | ✅ |
| High-risk skill behavioral evals | K3.2 | ✅ |
| Skill-rules expansion + route validation | K3.3 | ✅ partial |
| Workflow / release / cancelled guards | W2.2b / W2.4 | ✅ |
| quality_gates subprocess success | W2.3b | ✅ |
| Benchmark signal + nightly upload/ratchet | W2.5 | ✅ |
| Close harness issues #862–#869 | — | ✅ |
| Plans hygiene (D3.3 / V5.1) | D3.3 | ✅ this update |
| PR + all CI green | — | 🟡 PR #873 |
| Feature pilots | F4 | 🔵 deferred only |

## 2026-07-18 Goals (S1.7 + K3.1b + W2.1b) ✅ MERGED (#860)

| Goal | Plan ref | Status |
|------|----------|--------|
| Recursive audit redaction + rotation size init | S1.7a | ✅ |
| Non-blocking audit writer + drop metrics | S1.7b | ✅ |
| Skill eval fixtures + changed skills in CI | K3.1b | ✅ |
| Gate contract CI parity enforced | W2.1b | ✅ |
| PR + all CI green | C7 | ✅ #860 |

## 2026-07-17 Goals (Open Issues → Code)

| Goal | Issues | ADR | Status |
|------|--------|-----|--------|
| G1 Cut v0.1.35 via release.yml | #849 | ADR-058 / #843 plan | ✅ |
| G2 Durable complete + operator fail path | #847 | ADR-075 | ✅ |
| G3 Pattern empty-result UX + docs | #845 residual | ADR-076 | ✅ |
| G4 Close config discoverability after release | #846 | (done #829) | ✅ |
| G5 Verify #845/#846 against released binary | #845, #846 | — | ⏳ optional post-release |

## 2026-07-16b Goals (S1.3–S1.6 + W2.2)

| Goal | Plan ref | Status |
|------|----------|--------|
| No write lock held across backend await on step paths | S1.3 | ✅ |
| Capacity eviction deletes durable backend data | S1.4 | ✅ |
| Embedding health truthful (mock ≠ available) | S1.5 | ✅ |
| Retry queue timeout + first attempt free of permits | S1.6 | ✅ |
| Advisory audit cannot soft-pass | W2.2 | ✅ |
| Open PR + CI green + review | B8 | 🟡 |

## 2026-07-16 Missing Tasks Goals (PR #840 ✅)

| Goal | Plan ref | Status |
|------|----------|--------|
| Public fuzzy_match rustdoc restored | #837 | ✅ |
| Retrieval cache identity complete for TaskContext | S1.2 / ADR-074 | ✅ partial (mode/provider/index generation deferred) |
| build-rust accepts do-memory-* packages | W2.3 | ✅ |
| Zero production sources >500 LOC | W2.6 | ✅ |
| Docs match fail-closed code execution | S1.1a / D3.2 | ✅ |
| Open PR + review | A7 | ✅ PR #840 |

## v0.1.35 Goals (CLI UX Patch — on main)

| Goal | Issues | Status |
|------|--------|--------|
| Patterns durable + listable across CLI processes | #831 | ✅ |
| Project-local DB via --db-path / MEMORY_DB_PATH | #830 | ✅ |
| Discoverable config (init, template, partial TOML) | #829 | ✅ |
| Clear storage_mode story | #832 | ✅ |
| Align version to 0.1.35 and cut release | #828 / #838 | 🟡 tag via release.yml after PR merges |

---

## v0.1.33 Sprint Goals (Release Drift Resolution)

### Source: Issue #674 — ~100 unreleased commits since v0.1.32

`v0.1.32` is released (2026-05-24). The workspace has accumulated ~100 unreleased commits including telemetry stub implementations, CI hardening, fuzz harness, MCP input bounds, action pinning, edit distance optimization, agent eval workflows, and now CI/quality fixes + dependency updates. No v0.1.33 tag exists despite the workspace bump.

### GOAP Execution Model

- **Coordinator skills**: `goap-agent`, `agent-coordination`
- **Implementation skills**: `feature-implement`, `code-quality`, `ci-fix`
- **Validation skills**: `code-quality`, `test-runner`, `architecture-validation`
- **Release skills**: `github-release-best-practices`, `release-guard`

### Phase 1: Release (P1 — Closes #674)

| WG | Step | Status |
|----|------|--------|
| WG-175 | Tag + release v0.1.33 (CHANGELOG + `gh release create`) | 🟡 Queued |

### Phase 2: CI Health (P2) — ✅ COMPLETE

| WG | Step | Status |
|----|------|--------|
| WG-176 | Add 3 missing gitleaks fingerprints to `.gitleaksignore` | ✅ PR #675 |
| WG-177 | Add disk cleanup step to `nightly-tests.yml` | ✅ PR #675 |
| WG-178 | Scope mutation testing to `memory-core` + reduce timeout to 2h | ✅ PR #675 |
| WG-179 | Bump `actions/upload-artifact` to Node 24-compatible SHA | ✅ PR #675 + #681 |

### Phase 3: Code Quality (P2) — ✅ COMPLETE

| WG | Step | Status |
|----|------|--------|
| WG-180 | Fix 5 clippy lints in `mistral/client.rs` (--all-features) | ✅ PR #675 |
| WG-181 | Split `cache/wrapper.rs` below 500 LOC gate | ✅ PR #675 |

### Phase 4: Architecture (P3) — ✅ COMPLETE

| WG | Step | Status |
|----|------|--------|
| WG-182 | Add `tracing::warn!` to non-CSM cascade fallback | ✅ PR #675 |

### Phase 5: DevX Backlog (P3)

| WG | Step | Status |
|----|------|--------|
| WG-183 | Implement llms.txt generator script (closes #652) | 🟡 Queued |
| WG-184 | Write ADR for VERSION file decision (closes #653) | 🟡 Queued |

### Dependency Maintenance (2026-06-30) — ✅ COMPLETE

| PR | Description | Status |
|----|-------------|--------|
| #682 | Remove 6 stale advisory ignores + update anyhow (RUSTSEC-2026-0190) | ✅ Merged |
| #681 | Bump 13 GitHub Actions to latest (Node 24 compatible) | ✅ Merged |
| #684 | Bump rust-patch-minor group (2 updates) | ✅ Merged |
| #678 | Bump sysinfo 0.38.4 → 0.39.5 (major) | ✅ Auto-merge enabled |

---

## v0.1.32 Sprint Goals (Complete ✅)

### Cross-Repo Impact Analysis Source

Impact analysis of `d-o-hub/github-template-ai-agents` and `d-o-hub/chaotic_semantic_memory` identified unadopted runtime patterns and skill gaps. All P1/P2 goals achieved.

### P1: Runtime Patterns (All Complete)

1. **WG-103**: `MemoryEvent` broadcast channel ✅
   - Priority: P1
   - Owner: feature-implement
   - Target: Add `tokio::broadcast`-based event channel for episode lifecycle
   - Result: `types/event.rs` + `subscribe()` method on SelfLearningMemory

2. **WG-104**: `select_nth_unstable_by` for top-k retrieval ✅
   - Priority: P1
   - Owner: feature-implement
   - Target: Replace O(n log n) sort with O(n) partial sort
   - Result: `search/top_k.rs` module with `select_top_k()` utilities

3. **WG-105**: Idempotent cargo publish ✅
   - Priority: P1
   - Owner: ci-fix
   - Target: Add crates.io version check before `cargo publish`
   - Result: Already exists in `publish-crates.yml` (version check step)

### P2: Agent Harness Skills (All Complete)

4. **WG-106**: Add `memory-context` skill ✅
   - Priority: P2
   - Owner: skill-creator
   - Target: Skill for episode retrieval via do-memory-cli
   - Result: `.agents/skills/memory-context/SKILL.md`

5. **WG-107**: Add `learn` skill (dual-write learning) ✅
   - Priority: P2
   - Owner: skill-creator
   - Target: Post-task learning pattern
   - Result: `.agents/skills/learn/SKILL.md`

### P3: Future Backlog

6. **WG-108**: Version-retained persistence
   - Priority: P3
   - Owner: feature-implement
   - Target: Track concept drift across episode versions
   - Status: 🔵 Backlog

7. **WG-109**: `BundleAccumulator` sliding window
   - Priority: P3
   - Owner: feature-implement
   - Target: Recency-weighted context for pattern retrieval
   - Status: 🔵 Backlog

8. **WG-110**: SIMD-accelerated similarity
   - Priority: P3
   - Owner: feature-implement
   - Target: SIMD cosine similarity — defer until benchmarks justify
   - Status: 🔵 Backlog

---

## Completed Sprint Summary

| Sprint | WGs | Status | Key Deliverables |
|--------|-----|--------|------------------|
| v0.1.30 | WG-103-107 | ✅ All Complete | MemoryEvent broadcast, top-k optimization, memory-context skill, learn skill |
| v0.1.29 | WG-094-102 | ✅ All Complete | WASM removal (-6,982 LOC), Turso native vector search, file splitting, dead code audit |
| v0.1.28 | WG-089-093 | ✅ All Complete | DyMoE routing-drift, dual reward scoring, AI spam detector, CodeQL fix |
| v0.1.27 | WG-073,075,077-079,084-085 | ✅ All Complete | Bayesian ranking, Episode GC, MMR diversity, MCP Server Card, spawn_blocking audit, GH Pages, llms.txt |
| v0.1.26 | WG-086-088 | ✅ All Complete | Crate renaming do-memory-*, crates.io publish, GitHub Release |
| v0.1.24 | WG-059-067,080-083 | ✅ All Complete | Test stability, dependency updates, CHANGELOG backfill, tag+release |
| v0.1.23 | WG-051-058 | ✅ All Complete | Durable attribution/checkpoints, MCP contract, docs refresh, CI coverage, disk hygiene |
| v0.1.22 | WG-040-050 | ✅ All Complete | Doctests, file splits, dead_code, snapshots, property tests, MCP parity, git-cliff |
| v0.1.20 | WG-022-024,026-027,030 | ✅ All Complete | redb compilation, ignored test fixes, coverage improvement, codecov config |
| v0.1.19 | WG-012-021 | ✅ All Complete | Nightly filter, changelog workflow, dead_code audit, stale TODOs |
| v0.1.18 | WG-008-011 | ✅ All Complete | Ignored test triage, batch MCP tools, error handling, dep dedup |
| v0.1.17 | WG-001-007 | ✅ All Complete | Docs integrity, release wrapper, GOAP index, Dependabot merges |

---

## Partially Complete / Backlog

1. **WG-025**: Un-ignore fixable tests
   - Status: 🟡 Partial — 119→118 (pattern CLI e2e un-ignored); 6 sandbox/WASM tests still pending

2. **WG-028**: Property test expansion
   - Status: 🟡 Partial — ACT-030 (serialization) and ACT-031 (calculator) complete; ACT-032 (fuzz) pending

3. **WG-029**: Integration coverage
   - Status: 🟠 Pending — ACT-033, ACT-034, ACT-035 not started
