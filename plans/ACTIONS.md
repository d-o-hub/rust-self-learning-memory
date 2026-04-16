# GOAP Actions Backlog

- **Last Updated**: 2026-04-16 (v0.1.31 sprint planning)
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

## Active Actions (v0.1.31 Sprint — Planning)

### Phase 0: Release & Hygiene (Sequential)

- **ACT-102**: Release v0.1.30 to crates.io + GitHub Release
   - Goal: WG-111
   - Action: Run release-guard checks, tag v0.1.30, `cargo publish`, create GitHub Release with multi-platform binaries
   - Dependencies: None
   - Status: 🔵 Planned

- **ACT-103**: Bump workspace version to 0.1.31
   - Goal: WG-112
   - Action: Update `Cargo.toml` workspace version, regenerate `Cargo.lock`, update CHANGELOG via git-cliff
   - Dependencies: ACT-102
   - Status: 🔵 Planned

- **ACT-104**: Audit clippy suppressions in `memory-core/src/lib.rs`
   - Goal: WG-113
   - Action: Remove blanket `clippy::restriction`, keep only justified suppressions, move remaining to `.clippy.toml` where possible
   - Dependencies: None
   - Status: 🔵 Planned

### Phase 1: Skills Consolidation (Parallel)

- **ACT-105**: Merge `build-compile` into `build-rust`
   - Goal: WG-114
   - Action: Combine content, delete `build-compile/`, update skill-rules.json
   - Status: 🔵 Planned

- **ACT-106**: Merge `perplexity-researcher-pro` + `perplexity-researcher-reasoning-pro` + `web-search-researcher` → `web-researcher`
   - Goal: WG-115
   - Action: Create unified skill with mode parameter (quick/deep/reasoning), delete old skills
   - Status: 🔵 Planned

- **ACT-107**: Merge `code-quality` + `rust-code-quality` + `clean-code-developer` → `code-quality`
   - Goal: WG-116
   - Action: Consolidate into single code-quality skill with Rust-specific section
   - Status: 🔵 Planned

- **ACT-108**: Merge `context-retrieval` + `context-compaction` + `memory-context` → `memory-context`
   - Goal: WG-117
   - Action: Combine retrieval + compaction + CLI into unified memory-context skill
   - Status: 🔵 Planned

- **ACT-109**: Merge `quality-unit-testing` + `episodic-memory-testing` + `rust-async-testing` → `test-patterns`
   - Goal: WG-118
   - Action: Create unified test-patterns skill with domain-specific sections
   - Status: 🔵 Planned

- **ACT-110**: Compact oversized skills
   - Goal: WG-119
   - Action: `git-worktree-manager` 549→≤150 LOC, `yaml-validator` 486→≤100, `general` 403→≤100
   - Status: 🔵 Planned

### Phase 2: Code Quality (Parallel with Phase 1)

- **ACT-111**: Split >500 LOC production files
   - Goal: WG-120
   - Action: Split retention.rs, affinity.rs, ranking.rs, handlers.rs into sub-modules
   - Status: 🔵 Planned

- **ACT-112**: Reduce `#[allow(dead_code)]` annotations
   - Goal: WG-121
   - Action: Audit 35 dead_code annotations, remove unused code or promote to public API
   - Status: 🔵 Planned

- **ACT-113**: Update stale documentation
   - Goal: WG-122
   - Action: Refresh STATUS/CURRENT.md metrics, remove frozen session counts from AGENTS.md, update skill descriptions referencing "2025"
   - Status: 🔵 Planned

### Phase 3: Research-Inspired Features (Sequential)

- **ACT-114**: Add temporal graph edges to episode store
   - Goal: WG-123
   - Action: Add Turso schema for episode→episode and episode→pattern edges with temporal weights; implement graph traversal queries
   - Paper: REMem (ICLR 2026, arXiv:2602.13530)
   - Status: 🔵 Planned

- **ACT-115**: Add procedural memory type
   - Goal: WG-124
   - Action: New `ProceduralMemory` type in memory-core; storage traits in turso/redb; extends existing episodic+semantic with learned skill patterns
   - Paper: ParamAgent (2026) three-tier memory
   - Status: 🔵 Planned

- **ACT-116**: Evaluate Routing-Free MoE for DyMoE
   - Goal: WG-125
   - Action: Read arXiv:2604.00801 + reference implementation; write evaluation ADR comparing to current DyMoE routing-drift protection
   - Paper: arXiv:2604.00801
   - Status: 🔵 Planned

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
