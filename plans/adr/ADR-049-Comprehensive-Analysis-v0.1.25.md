# ADR-049: Comprehensive Codebase Analysis & v0.1.25 Sprint Plan

- **Status**: Proposed
- **Date**: 2026-03-31
- **Deciders**: Project maintainers
- **Related**: ADR-044, ADR-048, ROADMAP_V030_VISION.md
- **Workspace Version**: 0.1.23

## Context

A full audit was performed on 2026-03-31 covering the entire codebase, all plans/ADRs, AGENTS.md, agent_docs/, `.agents/skills/`, and the v0.30 vision roadmap. The audit identified:

1. **Missing/partial implementations** that block user adoption
2. **New features** with high impact-to-effort ratio
3. **Noise** in documentation, skills, and plans
4. **Slow test bottleneck** degrading CI/DX

---

## Section A: Noise Reduction (RECOMMENDED FIRST)

### A1: Plans Folder Bloat (305 markdown files, 4.3MB)

| Issue | Details | Recommendation |
|-------|---------|----------------|
| 5 GOAP execution plans | `GOAP_EXECUTION_PLAN_v0.1.20` through `v0.1.24` — only latest is relevant | Archive v0.1.20–v0.1.23 to `plans/archive/` |
| 251 archived files | `plans/archive/` contains 251 files (most of the bloat) | Already archived — no action needed |
| GOAP_STATE.md is 770+ lines | Contains full historical sprint data back to v0.1.17 | Truncate to v0.1.24 sprint only; move history to `plans/archive/GOAP_STATE_HISTORY.md` |
| GOALS.md has 67 WGs | Mix of completed (60+), pending (2), partial (2) — hard to parse | Move completed WGs to `plans/archive/GOALS_COMPLETED.md`; keep only active/pending WGs |
| ACTIONS.md has 88+ ACTs | Same problem — completed actions bury the few pending ones | Archive completed ACTs; keep only active |

### A2: Skills (47 skills — KEEP AS-IS)

Skills are intentionally left unchanged. While overlaps exist, each serves a distinct trigger context and consolidation risks breaking established agent workflows.

### A3: Agent Docs Noise

| Issue | Details | Recommendation |
|-------|---------|----------------|
| `session_state_preservation.md` | Not referenced from AGENTS.md cross-references table | Add to cross-references or archive |
| 15 agent_docs files (2,541 lines) | Good density, but some duplicate guidance with AGENTS.md | Deduplicate — AGENTS.md should be index only, not repeat content |

### A4: ROADMAP_V030_VISION.md Staleness

- Title says "v0.1.9+ Vision" but project is at v0.1.23
- Code examples use `DateTime<Utc>` but project uses `chrono::DateTime<chrono::Utc>` (minor)
- Still references features as "Months 6-9" from an old baseline
- **Recommendation**: Update title/dates to realistic Q3-Q4 2026 targets

---

---

## Section A.5: Research-Driven Improvements (Feb/Mar 2026 Literature)

### R1: xMemory — Hierarchical Memory Retrieval (arXiv:2602.02007, Feb 2026)

**Paper**: "Beyond RAG for Agent Memory: Retrieval by Decoupling and Aggregation"

**Relevance**: Directly applies to this codebase's `retrieve_relevant_context()`. Standard top-k embedding retrieval returns redundant spans from correlated episodic memory. xMemory proposes:
- 4-level hierarchy (messages → episodes → semantics → themes)
- Stage I: kNN representative selection for coverage + diversity
- Stage II: Uncertainty-gated adaptive inclusion (only expand when it reduces reader uncertainty)

**Actionable for v0.1.25**: Add diversity-aware retrieval to `do-memory-core/src/memory/retrieval/context.rs`. Currently returns top-k by cosine similarity — add coverage-diversity reranking using existing pattern categories as "themes". Low effort, high retrieval quality impact.

### R2: OpenSpace — Self-Evolving Skill Engine (HKUDS, Mar 2026)

**Key insight**: 46% token reduction via three evolution modes (FIX, DERIVED, CAPTURED) for reusing task patterns. Directly validates this project's pattern extraction → playbook pipeline.

**Actionable**: The existing playbook system already captures patterns but doesn't track **token savings** from pattern reuse. Add a `tokens_saved` metric to `RecommendationSession` — trivial to implement, powerful for demonstrating ROI.

### R3: MCP Extensions & Apps (Official MCP Roadmap, Mar 2026)

MCP spec now supports **Extensions** (modular opt-in capabilities) and **MCP Apps** (interactive UI inside clients). Key developments:
- `ext-auth`: OAuth enterprise auth extension
- `ext-apps`: Rich UI rendering in MCP clients
- **Tasks primitive** (SEP-1686): Durable request tracking with polling and deferred results
- **Server Cards**: `.well-known` metadata discovery

**Actionable**: Implement MCP Server Card (`.well-known/mcp.json`) for this memory server. Zero risk, high discoverability impact. Also evaluate MCP Tasks for long-running episode operations.

### R4: Turso Concurrent Writes & Vector Search (Turso, Mar 2026)

Turso now supports **concurrent writes** (zero conflicts, no locking) and **native vector search** without extensions. Also ships browser WASM support.

**Actionable**: Evaluate Turso's native vector search to potentially replace the custom embedding storage layer. Could simplify the storage architecture significantly. Investigate concurrent writes for multi-agent episode creation.

### R5: Tokio Async Patterns (JetBrains/Carl Lerche, Feb 2026)

Key takeaway from Tokio creator: "Tasks only yield at `.await` — long CPU work stalls the runtime." This project's DBSCAN clustering and pattern extraction are CPU-heavy operations that should use `spawn_blocking`.

**Actionable**: Audit `do-memory-core/src/extraction/` and `do-memory-core/src/memory/pattern_search/` for CPU-heavy operations not wrapped in `spawn_blocking`. Already a core invariant in AGENTS.md but may not be fully enforced.

---

## Section B: Missing Implementation (Gaps)

### B1: Unwired Subsystems (Documented but not integrated)

| Subsystem | Status | Impact |
|-----------|--------|--------|
| `AdaptiveCache` | Exists in `do-memory-storage-redb/src/cache/adaptive/` but not wired as default storage path | Medium — redb uses `AdaptiveCacheAdapter` for metadata-only; value caching requires explicit opt-in |
| `CompressedTransport` | Exists as standalone utility, not wired into `TursoStorage` transport | Low — compression is used at embedding level, transport compression is optimization |
| `execute_agent_code` MCP tool | Fully implemented but **permanently disabled** due to WASM sandbox compilation issues | High — code execution is a flagship feature; 35+ references across codebase are dead weight |

### B2: Pending/Partial Work Items

| Item | Status | Details |
|------|--------|---------|
| WG-025 (Un-ignore fixable tests) | 🟡 Partial | 119→118 done; 6 sandbox/WASM tests still pending |
| WG-028 (Property test expansion) | 🟡 Partial | Serialization + calculator done; ACT-032 (fuzz tests) pending |
| WG-029 (Integration coverage) | Pending | CLI integration, MCP tool tests, storage error paths — ACT-033/034/035 |
| WG-035 (Pre-existing issue fixes) | ⏳ Pending | From v0.1.21 sprint |

### B3: Slow Test Bottleneck (NEW)

| Test | Duration | Root Cause |
|------|----------|------------|
| `cli_update_test` (5 tests) | **134s each** (670s total!) | Each test recompiles the CLI binary via `assert_cmd::Command::cargo_bin` — no binary caching |
| `benchmark_memory_usage` | 51s | DBSCAN on 100K points — acceptable but near ceiling |

**Impact**: `cli_update_test` alone adds **11 minutes** to test suite. The full suite takes 136s but would be ~30s without these.

---

## Section C: High-Impact New Features (v0.1.25)

Prioritized by: **user-facing value × feasibility × builds on existing code**

### C1: Bayesian/Wilson Pattern Ranking (P0 — HIGH IMPACT)

**Problem**: Pattern ranking uses hand-tuned weights (recency 30%, relevance 25%, effectiveness 25%, specificity 20%). Attribution data from WG-051/052 now provides real usage signals, but the ranking algorithm doesn't consume them.

**Impact**: Makes the self-learning loop genuinely data-driven. This is the *core value proposition* of the system.

**Effort**: 2-3 days. The data (recommendation sessions, feedback) already exists.

**Implementation**:
- Replace fixed effectiveness weight with Wilson score interval (lower bound)
- Add adoption-rate signal from attribution data
- Bayesian update on pattern confidence using feedback loop
- Falls back to current weights when insufficient data

### C2: Online/Streaming Pattern Learning (P1 — HIGH IMPACT)

**Problem**: Pattern extraction happens only at `complete_episode()`. The checkpoint system (WG-052) creates mid-task snapshots but no partial pattern signals are extracted.

**Impact**: Enables real-time learning without waiting for episode completion — critical for long-running agent tasks.

**Effort**: 3-4 days. Reuses existing `extractor.rs` on partial episodes.

### C3: Garbage Collection / TTL for Old Episodes (P1 — HIGH IMPACT)

**Problem**: Episodes accumulate forever. No retention policy, no garbage collection. As the system scales, retrieval degrades and storage grows unbounded.

**Impact**: Required for any production deployment. Simple TTL + soft-delete with configurable retention.

**Effort**: 2-3 days. Storage layer already supports delete operations.

### C4: WASM Sandbox Resolution (P1 — TECH DEBT, DEFERRED)

**Problem**: `execute_agent_code` is fully implemented (35+ references) but permanently disabled. Dead code adds maintenance burden.

**Decision deferred** to a future sprint — risk/complexity too high for v0.1.25.

### C5: CLI Binary Caching for Tests (P2 — DX)

**Problem**: `cli_update_test` recompiles CLI binary per test (670s overhead).

**Fix**: Use `#[ctor]` or `lazy_static` to build binary once, or restructure as unit tests that test the command parsing layer directly.

**Effort**: 1 day. Saves 10+ minutes per CI run.

---

## Section D: Decision Summary

### Recommended Sprint Structure (v0.1.25)

| Phase | Priority | WGs | Effort |
|-------|----------|-----|--------|
| Phase 0: Noise Reduction | P0 | WG-068–071 | 1 day |
| Phase 1: CLI Test Fix | P0 | WG-072 | 1 day |
| Phase 2: Bayesian Ranking | P0 | WG-073 | 2-3 days |
| Phase 3: WASM Decision | P1 | WG-074 | 2-3 days |
| Phase 4: Episode GC/TTL | P1 | WG-075 | 2-3 days |
| Phase 5: Online Learning | P2 | WG-076 | 3-4 days |

### Not Recommended for v0.1.25

| Feature | From | Reason |
|---------|------|--------|
| Distributed Memory (CRDT) | Vision Phase 1 | Premature — single-instance not production-ready yet |
| A/B Testing Framework | Vision Phase 1 | Needs Bayesian ranking first (C1) |
| Prometheus/OpenTelemetry | Vision Phase 2 | Nice-to-have; tracing already exists |
| Multi-Tenancy/RBAC | Vision Phase 3 | No multi-tenant demand yet |
| Custom Embedding Models | Vision Phase 5 | 5 providers already supported |

## Consequences

### Positive
- Noise reduction makes plans/ navigable and skills discoverable
- Bayesian ranking delivers on the "self-learning" promise using data that already exists
- CLI test fix saves 10+ minutes per CI run
- WASM resolution eliminates 35+ dead code references

### Negative
- Noise reduction requires file moves (git history preserved via `git mv`)
- Skills consolidation requires updating cross-references in docs
- Bayesian ranking needs A/B validation against current weights

### Risks
- Skills consolidation may break agent workflows that reference specific skill names
- Bayesian ranking may initially perform worse with insufficient attribution data (mitigated by fallback to current weights)

## Cross-References

- [ROADMAP_V030_VISION.md](../ROADMAPS/ROADMAP_V030_VISION.md) — features NOT recommended
- [ADR-044](ADR-044-High-Impact-Features-v0.1.20.md) — attribution/checkpoint features this builds on
- [ADR-048](ADR-048-v0.1.24-Stability-Sprint.md) — preceding stability sprint
- [GOAP_EXECUTION_PLAN_v0.1.25.md](../GOAP_EXECUTION_PLAN_v0.1.25.md) — execution plan
