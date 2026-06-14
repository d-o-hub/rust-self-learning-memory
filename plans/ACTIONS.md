# GOAP Actions Backlog

- **Last Updated**: 2026-05-19 (WG-124 completed; all active WGs complete)
- **Archived Plans**: `plans/archive/2026-03-consolidation/`

## Completed Actions Summary

All actions from v0.1.17 through v0.1.27 sprints are complete. See archived execution plans in `plans/archive/2026-03-consolidation/` for full details.

| Sprint | Actions | Count | Status |
|--------|---------|-------|--------|
| v0.1.27 | Bayesian, GC, Pages, llms.txt, semver fix | 7 | ✅ All Complete |
| v0.1.24 | ACT-089 through ACT-093 | 5 | ✅ All Complete |
| v0.1.23 | ACT-080 through ACT-088 | 9 | ✅ All Complete |
| v0.1.22 | ACT-053 through ACT-075 | 23 | ✅ All Complete |
| v0.1.21 | ACT-038 through ACT-052 | 15 | ✅ All Complete |
| v0.1.20 | ACT-020 through ACT-037 | 18 | ✅ All Complete |
| v0.1.17-19 | ACT-001 through ACT-019 | 19 | ✅ All Complete |

## Learning Delta (2026-03)

### redb 3.x Breaking Changes
- `begin_read()` moved to `ReadableDatabase` trait (must import it)
- `begin_write()` remains on `Database` struct (no change)

### rand 0.10 Breaking Changes
- `thread_rng()` → `rand::rng()` (function rename)
- `Rng::gen()` → `RngExt::random()` (method rename)
- `Rng::gen_range()` → `RngExt::random_range()` (method rename)
- Import `RngExt` for user-level RNG methods
- Keep `rand` and `rand_chacha` versions aligned

## Active Actions (v0.1.33 Sprint — Release Drift Resolution)

### GOAP Skills in Use

- **Coordinator**: `goap-agent`
- **Validation**: `code-quality`, `test-runner`, `architecture-validation`
- **Release**: `github-release-best-practices`, `release-guard`

### Phase 4: Validation & Release (v0.1.33)

- **ACT-170**: Version bump workspace to `0.1.33`
   - Goal: WG-169
   - Skills: `goap-agent`, `feature-implement`, `code-quality`
   - Action: Update `Cargo.toml` workspace version, publishable crate versions, regenerate `Cargo.lock`, update CHANGELOG
   - Dependencies: None
   - Status: 🔧 In progress

- **ACT-171**: Cut v0.1.33 release per #623
   - Goal: WG-170
   - Skills: `goap-agent`, `github-release-best-practices`, `release-guard`
   - Action: `./scripts/release-manager.sh validate` → `verify-release-state.sh --check-tag --check-unreleased` → `release-manager.sh full --execute`
   - Dependencies: ACT-170
   - Status: 🟡 Queued

### Phase 5: Benchmark Stabilization

- **ACT-172**: Mark performance benchmark tests as `#[ignore]`
   - Goal: CI stability
   - Skills: `code-quality`
   - Action: Add `#[ignore]` to all benchmark tests in `performance_benchmarks.rs` to prevent flaky CI failures
   - Dependencies: None
   - Status: ✅ Done

- **ACT-173**: Fix doctest compilation and mark flaky Turso test
   - Goal: CI stability
   - Skills: `code-quality`
   - Action: Fix doctest errors, mark flaky Turso test
   - Dependencies: None
   - Status: ✅ Done (PR #622)

### Phase 4: Research Backlog (Deferred until CPU/token wins are landed)

- **ACT-114**: Add temporal graph edges to episode store
   - Goal: WG-123
   - Action: Add Turso schema for episode→episode and episode→pattern edges with temporal weights; implement graph traversal queries
   - Paper: REMem (ICLR 2026, arXiv:2602.13530)
   - Status: ✅ Complete (PR #570: weighted traversal, pattern edges, significance weights, storage schema)

- **ACT-115**: Add procedural memory type
   - Goal: WG-124
   - Action: New `ProceduralMemory` type in memory-core; storage traits in turso/redb; extends existing episodic+semantic with learned skill patterns
   - Paper: ParamAgent (2026) three-tier memory
   - Status: ✅ Complete (PR #569 merged via admin)

- **ACT-116**: Evaluate Routing-Free MoE for DyMoE
   - Goal: WG-125
   - Action: Read arXiv:2604.00801 + reference implementation; write evaluation ADR comparing to current DyMoE routing-drift protection
   - Paper: arXiv:2604.00801
   - Status: 🔵 Backlog

- **ACT-125**: Evaluate LottaLoRA-inspired local classifier
   - Goal: WG-132
   - Action: Read arXiv:2604.08749; prototype frozen-random-backbone + LoRA for episode-type classification (CPU-only, no API)
   - Paper: LottaLoRA (arXiv:2604.08749, Apr 2026)
   - Status: ✅ Complete — evaluation document at `plans/WG-132_LottaLoRA_Evaluation.md`

- **ACT-126**: Align memory architecture with agentic memory taxonomy
   - Goal: WG-133
   - Action: Map current episodic/semantic/pattern types to arXiv:2602.19320's 4-structure taxonomy; update architecture docs
   - Paper: Anatomy of Agentic Memory (arXiv:2602.19320)
   - Status: ✅ Complete — evaluation document at `plans/WG-133_AgenticMemoryTaxonomy_Evaluation.md`

- **ACT-127**: Evaluate DAG-based state management
   - Goal: WG-134
   - Action: Adapt arXiv:2602.22398 DAG-based conversation state approach for episode context assembly; target 20-86% token reduction
   - Paper: arXiv:2602.22398
   - Status: ✅ Complete — ~1,320 LOC in `memory-core/src/context/dag/`, 24 tests, ADR-054

- **ACT-129**: Implement CloudEvents EventEmitter
   - Goal: WG-149
   - Skills: `goap-agent`, `feature-implement`, `architecture-validation`, `test-runner`
   - Action: Add CloudEvent struct (1.0 spec), EventEmitter trait, MemoryEventMapping, LogEmitter, NoOpEmitter, HttpEmitter (http-emitter feature), EventEmitterMode enum, wired into SelfLearningMemory, Environment variable support (MEMORY_EVENT_EMITTER, MEMORY_EVENT_EMITTER_URL)
   - Dependencies: None
   - Status: ✅ Complete — 13 emitter tests pass, all 1050 core tests pass

- **ACT-128**: Evaluate federated HDC for multi-agent memory
   - Goal: WG-135
   - Action: Evaluate HDC prototype exchange (arXiv:2603.20037) as bandwidth-efficient alternative for WG-126 MemCollab
   - Paper: arXiv:2603.20037
   - Status: 🔵 Evaluated — evaluation document at `plans/WG-135_FederatedHDC_Evaluation.md`

## Completed Actions (v0.1.30 Sprint)

- **ACT-097**: Implement `MemoryEvent` broadcast channel — ✅ Complete (WG-103)
- **ACT-098**: Replace sorted retrieval with `select_nth_unstable_by` — ✅ Complete (WG-104)
- **ACT-099**: Idempotent cargo publish guard — ✅ Already exists (WG-105)
- **ACT-100**: Create `memory-context` skill — ✅ Complete (WG-106)
- **ACT-101**: Create `learn` skill — ✅ Complete (WG-107)

## Completed Actions (v0.1.28–v0.1.29)

- **ACT-094**: Merge PR #406 (ai-slop detector) — ✅ Merged (WG-091)
- **ACT-095**: Fix CodeQL cleartext logging — ✅ Complete, commit fc9c302c (WG-093)
- **ACT-096**: Archive completed plans/ noise — ✅ 87% noise reduction
