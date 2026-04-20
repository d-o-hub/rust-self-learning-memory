# GOAP Actions Backlog

- **Last Updated**: 2026-04-20 (v0.1.31 reprioritized)
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

## Active Actions (v0.1.31 Sprint — CPU + Token Efficiency)

### GOAP Skills in Use

- **Coordinator**: `goap-agent`
- **Parallelization/worker orchestration**: `agent-coordination`
- **CPU implementation/measurement**: `performance`, `feature-implement`, `debug-troubleshoot`
- **Token/documentation optimization**: `agents-update`, `memory-context`, `learn`
- **Validation**: `code-quality`, `test-runner`, `architecture-validation`

### Phase 0: Release & Package Truth (Sequential)

- **ACT-102**: Verify `v0.1.30` release/package parity
   - Goal: WG-111
   - Skills: `goap-agent`, `github-release-best-practices`, `agents-update`
   - Action: Confirm latest GitHub release is `v0.1.30` and publishable workspace crates remain at `0.1.30`
   - Dependencies: None
   - Status: ✅ Complete

- **ACT-103**: Bump workspace version to `0.1.31`
   - Goal: WG-112
   - Skills: `goap-agent`, `feature-implement`, `code-quality`, `test-runner`
   - Action: Update `Cargo.toml` workspace version, publishable crate versions, regenerate `Cargo.lock`, update CHANGELOG via git-cliff
   - Dependencies: ACT-102
   - Status: 🔵 Planned

- **ACT-104**: Refresh stale release/version truth sources in `plans/`
   - Goal: WG-113
   - Skills: `goap-agent`, `agents-update`
   - Action: Align `ROADMAP_ACTIVE.md`, `GOALS.md`, `ACTIONS.md`, `GOAP_STATE.md`, and `STATUS/CURRENT.md` with verified release/package state
   - Dependencies: None
   - Status: ✅ Complete

### Phase 1: CPU Efficiency (Parallel)

- **ACT-105**: Benchmark QueryCache contention
   - Goal: WG-114
   - Skills: `goap-agent`, `performance`, `test-runner`
   - Action: Measure hot-path contention and validate `parking_lot::RwLock` impact for retrieval/cache paths
   - Status: 🔵 Planned

- **ACT-106**: Replace placeholder cached retrieval code
   - Goal: WG-115
   - Skills: `goap-agent`, `feature-implement`, `performance`, `test-runner`
   - Action: Implement storage-backed `query_episodes_cached()` and `query_patterns_cached()` in Turso cache integration
   - Status: 🔵 Planned

- **ACT-107**: Measure compression/cache CPU tradeoffs
   - Goal: WG-116
   - Skills: `goap-agent`, `performance`, `code-quality`
   - Action: Benchmark compression thresholds and zero-copy cache reuse to avoid spending CPU where token savings are negligible
   - Status: 🔵 Planned

- **ACT-108**: Implement bounded context assembly
   - Goal: WG-117
   - Skills: `goap-agent`, `feature-implement`, `memory-context`, `test-runner`
   - Action: Build `BundleAccumulator` sliding window to cap retrieval context size by recency and salience
   - Status: 🔵 Planned

- **ACT-109**: Add hierarchical/gist reranking
   - Goal: WG-118
   - Skills: `goap-agent`, `feature-implement`, `memory-context`, `test-runner`
   - Action: Add a second-stage reranker so fewer context items are sent to downstream prompts
   - Status: 🔵 Planned

- **ACT-110**: Compact high-frequency skills/docs
   - Goal: WG-119
   - Skills: `goap-agent`, `agents-update`, `learn`
   - Action: Shorten the largest frequently loaded skills/docs first to reduce baseline prompt tokens per session
   - Status: 🔵 Planned

### Phase 2: Research-Inspired Retrieval Upgrades (Parallel with Phase 1)

- **ACT-111**: Add reconstructive retrieval windows
   - Goal: WG-120
   - Skills: `goap-agent`, `feature-implement`, `memory-context`
   - Action: Expand top hits into bounded local context windows inspired by E-mem
   - Status: 🔵 Planned

- **ACT-112**: Add execution-signature retrieval
   - Goal: WG-121
   - Skills: `goap-agent`, `feature-implement`, `performance`
   - Action: Rank traces by tools, error classes, and step structure alongside embeddings
   - Status: 🔵 Planned

- **ACT-113**: Add scope-before-search shard routing
   - Goal: WG-122
   - Skills: `goap-agent`, `feature-implement`, `performance`
   - Action: Route queries through cheap scope filters before vector search to reduce candidate-set CPU and token waste
   - Status: 🔵 Planned

### Phase 3: Backlog (Deferred until CPU/token wins are landed)

- **ACT-114**: Add temporal graph edges to episode store
   - Goal: WG-123
   - Action: Add Turso schema for episode→episode and episode→pattern edges with temporal weights; implement graph traversal queries
   - Paper: REMem (ICLR 2026, arXiv:2602.13530)
   - Status: 🔵 Backlog

- **ACT-115**: Add procedural memory type
   - Goal: WG-124
   - Action: New `ProceduralMemory` type in memory-core; storage traits in turso/redb; extends existing episodic+semantic with learned skill patterns
   - Paper: ParamAgent (2026) three-tier memory
   - Status: 🔵 Backlog

- **ACT-116**: Evaluate Routing-Free MoE for DyMoE
   - Goal: WG-125
   - Action: Read arXiv:2604.00801 + reference implementation; write evaluation ADR comparing to current DyMoE routing-drift protection
   - Paper: arXiv:2604.00801
   - Status: 🔵 Backlog

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
