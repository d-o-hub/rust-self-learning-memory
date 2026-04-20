# GOAP Goals Index

- **Last Updated**: 2026-04-20 (v0.1.31 reprioritized)
- **Source ADR**: ADR-037, ADR-052, ADR-053 (Accepted)
- **Status**: Active

---

## v0.1.31 Sprint Goals (Planning 🔵)

### Source: Release/package verification + efficiency analysis (2026-04-20)

`v0.1.30` is already released, and publishable workspace crates remain at `0.1.30`. The refreshed sprint goal is to lower CPU cost and prompt/token cost before the next version bump.

### GOAP Execution Model

- **Coordinator skills**: `goap-agent`, `agent-coordination`, `task-decomposition`
- **Implementation skills**: `feature-implement`, `performance`, `agents-update`
- **Validation skills**: `code-quality`, `test-runner`, `architecture-validation`
- **Learning/retention skills**: `memory-context`, `learn`

### Phase 0: Release & Package Truth

1. **WG-111**: Verify `v0.1.30` release and package parity
   - Priority: P0
   - Owner: github-release-best-practices
   - GOAP skills: `goap-agent`, `github-release-best-practices`, `agents-update`
   - Target: Confirm latest GitHub release + publishable crate versions are all `0.1.30`
   - Dependencies: None

2. **WG-112**: Version bump to `0.1.31`
   - Priority: P0
   - Owner: feature-implement
   - GOAP skills: `goap-agent`, `feature-implement`, `code-quality`, `test-runner`
   - Target: Bump workspace + publishable crates, update CHANGELOG after efficiency work lands
   - Dependencies: WG-111

3. **WG-113**: Refresh stale status/roadmap/GOAP truth sources
   - Priority: P0
   - Owner: agents-update
   - GOAP skills: `goap-agent`, `agents-update`
   - Target: Align release/version/package statements across `plans/`
   - Dependencies: None

### Phase 1: CPU Efficiency

4. **WG-114**: Reduce QueryCache contention
   - Priority: P1
   - Owner: performance
   - GOAP skills: `goap-agent`, `performance`, `test-runner`
   - Target: Validate `parking_lot::RwLock` and benchmark lock contention on hot retrieval paths

5. **WG-115**: Wire real cached retrieval into Turso integration
   - Priority: P1
   - Owner: feature-implement
   - GOAP skills: `goap-agent`, `feature-implement`, `performance`, `test-runner`
   - Target: Replace placeholder cached episode/pattern queries with storage-backed implementations

6. **WG-116**: Tune compression/cache CPU budget
   - Priority: P1
   - Owner: performance
   - GOAP skills: `goap-agent`, `performance`, `code-quality`
   - Target: Measure compression thresholds and zero-copy cache tradeoffs to avoid wasted CPU cycles

### Phase 2: Token Efficiency

7. **WG-117**: Implement `BundleAccumulator` sliding window
   - Priority: P1
   - Owner: feature-implement
   - GOAP skills: `goap-agent`, `feature-implement`, `memory-context`, `test-runner`
   - Target: Bound retrieved context by recency-weighted window instead of flat accumulation

8. **WG-118**: Add hierarchical/gist reranking
   - Priority: P1
   - Owner: feature-implement
   - GOAP skills: `goap-agent`, `feature-implement`, `memory-context`, `test-runner`
   - Target: Return fewer, denser context items to reduce prompt tokens without hurting retrieval quality

9. **WG-119**: Compact high-frequency skills/docs
     - Priority: P2
     - Owner: agents-update
     - GOAP skills: `goap-agent`, `agents-update`, `learn`
     - Target: Reduce prompt token load from large high-frequency agent docs and skills

### Phase 3: Research-Inspired Retrieval Upgrades

10. **WG-120**: Add reconstructive retrieval windows
     - Priority: P2
     - Owner: feature-implement
     - GOAP skills: `goap-agent`, `feature-implement`, `memory-context`
     - Target: Expand top-k hits into bounded local windows to preserve useful context with fewer irrelevant tokens
    - Paper: E-mem (arXiv:2601.21714)

11. **WG-121**: Add execution-signature retrieval
     - Priority: P2
     - Owner: feature-implement
     - GOAP skills: `goap-agent`, `feature-implement`, `performance`
     - Target: Rank traces by tools/errors/step-shape in addition to embeddings to reduce noisy retrieval
    - Paper: APEX-EM (arXiv:2603.29093)

12. **WG-122**: Add scope-before-search shard routing
     - Priority: P2
     - Owner: feature-implement
     - GOAP skills: `goap-agent`, `feature-implement`, `performance`
     - Target: Reduce candidate set size before vector search to lower CPU and token waste
    - Paper: ShardMemo (arXiv:2601.21545)

### Backlog (Future)

13. **WG-123**: Temporal graph edges in episode store
    - Priority: P3
    - Owner: feature-implement
    - Paper: REMem (ICLR 2026, arXiv:2602.13530)

14. **WG-124**: Procedural memory type
    - Priority: P3
    - Owner: feature-implement
    - Paper: ParamAgent (2026) — three-tier memory architecture

15. **WG-125**: Routing-Free MoE evaluation
    - Priority: P3
    - Owner: code-reviewer
    - Paper: arXiv:2604.00801 (Apr 2026) — eliminates routing drift, better scalability

16. **WG-126**: Cross-agent memory collaboration (MemCollab)
    - Priority: P3
    - Owner: feature-implement
    - Paper: arXiv:2603.23234 — contrastive trajectory distillation for agent-agnostic memory

17. **WG-127**: Semantic gist extraction + CogniRank (CogitoRAG)
    - Priority: P3
    - Owner: feature-implement
    - Paper: arXiv:2602.15895 — gist-based retrieval outperforms flat RAG

---

## v0.1.30 Sprint Goals (Complete ✅)

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
