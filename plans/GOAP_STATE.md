# GOAP State Snapshot

- **Last Updated**: 2026-04-23 (v0.1.31 post-release audit)
- **Version**: `0.1.31` (workspace, released)
- **Branch**: `main`
- **Validation**: `plans/STATUS/VALIDATION_LATEST.md`
- **Gap Analysis**: `plans/STATUS/GAP_ANALYSIS_LATEST.md`
- **Primary ADRs**: ADR-052 (v0.1.29), ADR-037 (CSM workflow adoption), ADR-053 (Accepted)

---

## v0.1.31 Sprint (Complete ✅)

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

| Task | WG | Status | Owner |
|------|----|--------|-------|
| Reduce QueryCache contention | WG-114 | ✅ Complete | PR #450 merged (parking_lot::RwLock) |
| Replace placeholder cached retrieval | WG-115 | 🟡 In Progress | cascade pipeline implementation |
| Tune compression/cache CPU budget | WG-116 | ✅ Complete | thresholds well-tuned (1KB threshold, Zstd preferred) |

### Phase 1.5: CSM Integration ✅ Complete (crate dependency)

**Implementation**: Added `chaotic_semantic_memory = "0.3.2"` as optional dependency with `csm` feature flag. Re-exports in `memory-core/src/retrieval/mod.rs`.

| Task | WG | Status | Owner |
|------|----|--------|-------|
| BM25 keyword index from CSM | WG-128 | ✅ Complete | crate dependency |
| HDC local embedding fallback | WG-129 | ✅ Complete | crate dependency |
| ConceptGraph ontology expansion | WG-130 | ✅ Complete | crate dependency |
| Cascading retrieval pipeline | WG-131 | 🟡 In Progress | cascade.rs placeholder being implemented |

### Phase 2: Token Efficiency (Parallel with Phase 1)

| Task | WG | Status | Owner |
|------|----|--------|-------|
| Implement BundleAccumulator window | WG-117 | ✅ Complete | feature-implement |
| Add hierarchical/gist reranking | WG-118 | ✅ Complete | feature-implement |
| Compact high-frequency skills/docs | WG-119 | ✅ Complete | skills at 2,660 LOC (under 4,000 target) |

### Phase 3: Research-Inspired Retrieval Upgrades (Deferred)

| Task | WG | Status | Owner | Paper |
|------|----|--------|-------|-------|
| Reconstructive retrieval windows | WG-120 | 🔵 Planned | feature-implement | E-mem |
| Execution-signature retrieval | WG-121 | 🔵 Planned | feature-implement | APEX-EM |
| Scope-before-search shard routing | WG-122 | 🔵 Planned | feature-implement | ShardMemo |
| LottaLoRA local classifier | WG-132 | 🔵 Planned | feature-implement | LottaLoRA |
| Agentic memory taxonomy alignment | WG-133 | 🔵 Planned | agents-update | Anatomy of Agentic Memory |
| DAG-based state management | WG-134 | 🔵 Planned | feature-implement | arXiv:2602.22398 |
| Federated HDC multi-agent memory | WG-135 | 🔵 Planned | feature-implement | arXiv:2603.20037 |

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
| `BundleAccumulator` sliding window | WG-109 | 🔵 Backlog | Recency-weighted context for pattern retrieval |
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

## Key Metrics

| Metric | Value | Target |
|--------|-------|--------|
| Workspace version | 0.1.31 | — |
| Latest GitHub release | v0.1.31 | verified 2026-04-22 |
| Publishable workspace crates | 6 | all at 0.1.31 |
| Total tests | 2,924 | — |
| Ignored tests | 123 skipped | ceiling ≤125 |
| `allow(dead_code)` (prod src) | 30 | ≤25 | ⚠️ slightly over (API reserves/future features) |
| Clippy | Clean | 0 warnings |
| Doctests | 0 failures | 0 |
| Skills count | 31 | ✅ target ≤35 met |
| Skills LOC | 2,660 | ✅ under 4,000 target |
| Clippy suppressions (lib.rs) | 64 | ≤20 |
| Files >500 LOC | 0 | 0 |
| Cargo audit | 3 unmaintained warnings | transitive |
| Dependabot alerts | 3 open | all transitive, tracked |
| CodeQL alerts | 0 open | ✅ fixed |
| CSM integration | ✅ Tiers complete, pipeline placeholder | BM25+HDC+ConceptGraph cascade |

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
