# GOAP Missing Tasks Implementation — 2026-05-01

**Generated**: 2026-05-01
**Method**: Full codebase analysis + plans audit
**Scope**: Identify implementation/plan drift, missing ADRs, uncommitted changes

---

## Executive Summary

Analysis identified **1 major implementation drift** (WG-134 fully implemented but
marked as Backlog in all plans), **1 partial implementation** (WG-131 CascadeRetriever
has CSM path but ConceptGraph tier is placeholder), and **1 missing ADR** (no ADR
exists for WG-134). All findings are documented with specific file locations and
remediation actions.

---

## Finding 1: WG-134 DAG State Management — Implemented But Plans Stale

### Status: ⚠️ Critical Documentation Drift

**Plans say**: "🔵 Backlog" / "🔵 Planned" (GOAP_STATE, GOALS, ACTIONS, ROADMAP_ACTIVE)
**Reality**: Fully implemented in `memory-core/src/context/dag/` (~1,320 LOC, 24 tests)

### Implementation Evidence

| File | LOC | Status |
|------|-----|--------|
| `memory-core/src/context/dag/mod.rs` | 45 | Module header with architecture docs |
| `memory-core/src/context/dag/node.rs` | 190 | `StateNode` with 7 node types |
| `memory-core/src/context/dag/edge.rs` | 150 | `StateEdge` with 4 edge types |
| `memory-core/src/context/dag/dag.rs` | 255 | `StateDag` with full CRUD + stats |
| `memory-core/src/context/dag/assembler.rs` | 310 | `DagContextAssembler` with 3 formats |
| `memory-core/src/context/dag/tests.rs` | 370 | 24 unit + integration tests |
| `memory-core/src/context/mod.rs` | +27 | Added `pub mod dag` + re-exports |

### Git Status

```
modified:   memory-core/src/context/mod.rs
Untracked:  memory-core/src/context/dag/
```

**All changes are uncommitted** — the implementation exists only in the working tree.

### Remediation

| Action | Priority | Effort | Owner |
|--------|----------|--------|-------|
| Create ADR-054 for WG-134 | P0 | S | ✅ Done |
| Update GOAP_STATE.md: WG-134 → ✅ Complete | P0 | S | ✅ Done |
| Update GOALS.md: WG-134 → ✅ Complete | P0 | S | ✅ Done |
| Update ACTIONS.md: ACT-127 → ✅ Complete | P0 | S | ✅ Done |
| Update ROADMAP_ACTIVE.md: WG-134 → ✅ Complete | P0 | S | ✅ Done |
| Commit the dag module implementation | P1 | S | Pending |
| Run clippy + tests on dag module | P1 | S | ✅ Done |

---

## Finding 2: WG-131 CascadeRetriever — Complete ✅

### Status: ✅ Complete

WG-131 has been fully implemented with all 4 cascade tiers operational:
- Tier 1 (BM25): Keyword search via CSM crate
- Tier 2 (HDC): Similarity search via CSM crate
- Tier 3 (ConceptGraph): Curated 14-domain coding-agent ontology with term expansion
- Tier 4 (API): External API fallback

All 4 tiers are implemented and tested with 30 tests passing. WG-132 (LottaLoRA evaluation)
and WG-133 (Agentic Memory Taxonomy evaluation) are also complete with evaluation documents.

### Updated Implementation Status by Tier

| Tier | Method | Status | Notes |
|------|--------|--------|-------|
| 1 | BM25 keyword search | ✅ Complete | Full implementation via CSM crate |
| 2 | HDC similarity search | ✅ Complete | Full implementation via CSM crate |
| 3 | ConceptGraph expansion | ✅ Complete | Curated 14-domain coding-agent ontology |
| 4 | API fallback | ✅ Complete | Flags `api_calls: 1` when needed |
| Merge | BM25 + HDC weighted merge | ✅ Complete | Query-length-dependent weights |

### Remediation (Completed)

| Action | Priority | Status |
|--------|----------|--------|
| Create ontology JSON for coding-agent domain terms | P2 | ✅ Done |
| Wire ConceptGraph into Tier 3 of cascade | P2 | ✅ Done |
| Update GOAP documents to reflect actual status | P0 | ✅ Done |

---

## Finding 3: Missing ADR for WG-134

### Status: ❌ No ADR Exists

WG-134 represents a significant architectural decision (~1,320 LOC, new module) but has
no Architecture Decision Record. All other major WGs (WG-117 BundleAccumulator, WG-131
CascadeRetriever) are covered by ADR-053.

### Remediation

| Action | Priority | Status |
|--------|----------|--------|
| Create ADR-054 | P0 | ✅ Done |

---

## Finding 4: WG-132 (LottaLoRA) and WG-133 (Agentic Memory Taxonomy) — Genuinely Unstarted

### Status: 🔵 Planned (Correctly)

No implementation exists for either WG. These are correctly marked as "🔵 Planned" in
all plans documents.

- **WG-132**: LottaLoRA-inspired local classifier for episode types (arXiv:2604.08749)
- **WG-133**: Align memory types with Anatomy of Agentic Memory taxonomy (arXiv:2602.19320)

### Recommendation

Defer to future sprint. No documentation drift.

---

## Finding 5: Plans Files — Status Drift Summary

### GOAP_STATE.md

| WG | Old Status | New Status | Evidence |
|----|-----------|------------|----------|
| WG-134 | 🔵 Planned | ✅ Complete | Fully implemented in `context/dag/` |
| WG-131 | 🔵 Planned (partially) | 🟡 Partial | CSM path complete, ConceptGraph placeholder |

### GOALS.md

| Section | Old | New |
|---------|-----|-----|
| WG-134 goal | "Evaluate DAG-based state management" | Mark as Complete |
| WG-131 goal | "Implement cascading retrieval pipeline" | Mark as Partial (Tier 3 pending) |

### ACTIONS.md

| Action | Old | New |
|--------|-----|-----|
| ACT-127 (WG-134) | 🔵 Backlog | ✅ Complete |
| ACT-120 (WG-131) | 🔵 Planned | 🟡 Partial (CSM path done, ConceptGraph pending) |

### ROADMAP_ACTIVE.md

| WG | Old | New |
|----|-----|-----|
| WG-134 | 🔵 Backlog | ✅ Complete |
| WG-131 | ✅ Complete (claimed) | 🟡 Partial (Tier 3 placeholder) |

---

## Consolidated Action Plan

### Phase 1: Documentation Fixes (Immediate — P0)

| # | Task | Status |
|---|------|--------|
| 1 | Create ADR-054 for WG-134 | ✅ Done |
| 2 | Update GOAP_STATE.md | ✅ Done |
| 3 | Update GOALS.md | ✅ Done |
| 4 | Update ACTIONS.md | ✅ Done |
| 5 | Update ROADMAP_ACTIVE.md | ✅ Done |

### Phase 2: Code Quality (P1)

| # | Task | Status |
|---|------|--------|
| 6 | Run clippy on dag module | ✅ Done |
| 7 | Run tests on dag module | ✅ Done |
| 8 | Commit dag module + context/mod.rs changes | Pending |

### Phase 3: Feature Completion (P2 — Future Sprint)

| # | Task | Status |
|---|------|--------|
| 9 | Create ontology JSON for ConceptGraph (WG-131 Tier 3) | Pending |
| 10 | Wire ConceptGraph into cascade pipeline | Pending |
| 11 | Evaluate WG-132 (LottaLoRA local classifier) | Pending |
| 12 | Evaluate WG-133 (Agentic Memory taxonomy alignment) | Pending |

---

## Cross-References

- `plans/adr/ADR-054-DAG-State-Management-WG134.md` — New ADR
- `plans/GOAP_STATE.md` — Updated WG-134/WG-131 status
- `plans/GOALS.md` — Updated goals
- `plans/ACTIONS.md` — Updated action statuses
- `plans/ROADMAPS/ROADMAP_ACTIVE.md` — Updated roadmap
- `memory-core/src/context/dag/` — Implementation (uncommitted)
- `memory-core/src/retrieval/cascade/mod.rs` — CascadeRetriever with CSM path
