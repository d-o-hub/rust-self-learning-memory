# ADR-054: DAG-Based State Management for Episode Context (WG-134)

- **Status**: Accepted (Implemented)
- **Date**: 2026-05-01
- **Deciders**: Project maintainers
- **Related**: WG-134, WG-117 (BundleAccumulator), ADR-053 (v0.1.31 Comprehensive Analysis)
- **Paper**: arXiv:2602.22398 — "DAG-based conversation state management for LLM agents"

## Context

The `BundleAccumulator` (WG-117) provides bounded context assembly, but each episode's
context attributes (language, domain, framework, tags, task type) are stored redundantly.
For N episodes sharing the same language/domain/task type, this wastes N × (context tokens).

arXiv:2602.22398 proposes a DAG-based state management approach where shared context
attributes are stored once as nodes, and episodes reference them via edges. This achieves
up to 86% token reduction by deduplicating shared context between episodes.

## Decision

**Implement a DAG-based state management system** (`StateDag` + `DagContextAssembler`)
in `memory-core/src/context/dag/` with these components:

1. **`StateNode`** — Represents a shared context attribute (language, domain, framework,
   task type, tag, complexity, or composite). Stores the value, reference count, and
   access metadata. Each node is referenced by ID instead of storing full context strings.

2. **`StateEdge`** — Connects episodes to state nodes with relationship types
   (`HasAttribute`, `InheritsFrom`, `DependsOn`, `SimilarTo`), strength scores, and
   metadata about the source field.

3. **`StateDag`** — Manages the node/edge graph with O(1) node lookup via
   `HashMap<(StateNodeType, String), NodeId>` index. Supports:
   - Episode registration (creates/updates nodes, adds edges)
   - Episode removal (cleans up unreferenced nodes)
   - Shared context extraction across episode sets
   - Token savings calculation and reduction percentage

4. **`DagContextAssembler`** — Traverses the DAG to assemble deduplicated context
   for prompts. Supports three output formats:
   - `Compact` — Node ID references only (minimal tokens)
   - `Full` — Human-readable with sections
   - `TokenOptimized` — SHARED:/EP: format (minimum token count)

### Architecture

```
Episodes ──► StateDag ──► DagContextAssembler ──► Deduplicated Prompt
   │           │              │
   └──► StateNode ──► Shared attribute stored once
        - language="rust"       (not per-episode)
        - domain="web-api"
        - task_type="Debugging"
```

### Token Reduction Model

For N episodes sharing the same language/domain/task_type:
- **Old**: N × (language + domain + task_type) tokens
- **New**: 1 × (language + domain + task_type) + N × node_id refs
- **Reduction**: ~86% when N > 5 and shared context is large

## Alternatives Considered

1. **Flat context storage (status quo)**
   - Pros: Simple, already implemented
   - Cons: Wastes tokens on repeated context attributes, no deduplication
   - **REJECTED**: Does not meet token efficiency goals

2. **Compression-based deduplication**
   - Pros: Transparent to application layer
   - Cons: Requires compression library, CPU overhead per assembly
   - **REJECTED**: Structural deduplication is more efficient

3. **Relational foreign-key approach**
   - Pros: Database-native, queryable
   - Cons: Requires storage schema changes, not in-memory efficient
   - **REJECTED**: Adds storage coupling to what should be an in-memory assembly concern

4. **DAG-based state management (chosen)**
   - Pros: ~86% token reduction, in-memory, format-flexible, 24 tests
   - Cons: Additional module (~900 LOC), requires episode registration step
   - **ACCEPTED**: Best balance of efficiency and integration simplicity

## Implementation

### Files Created

| File | LOC | Purpose |
|------|-----|---------|
| `memory-core/src/context/dag/mod.rs` | 45 | Module declarations, re-exports, architecture docs |
| `memory-core/src/context/dag/node.rs` | 190 | `StateNode`, `StateNodeType`, token savings |
| `memory-core/src/context/dag/edge.rs` | 150 | `StateEdge`, `EdgeType`, `EdgeMetadata` |
| `memory-core/src/context/dag/dag.rs` | 255 | `StateDag`, `DagStats`, CRUD operations |
| `memory-core/src/context/dag/assembler.rs` | 310 | `DagContextAssembler`, 3 format modes |
| `memory-core/src/context/dag/tests.rs` | 370 | 24 unit + integration tests |
| **Total** | **~1,320** | |

### Files Modified

| File | Change |
|------|--------|
| `memory-core/src/context/mod.rs` | Added `pub mod dag;` + re-exports of all DAG types + architecture docs |

### Public API Surface

```rust
// Re-exported from memory-core::context
pub use dag::{
    AssembledContext, DagAssemblyConfig, DagContextAssembler, DagStats,
    EdgeMetadata, EdgeType, NodeId, StateDag, StateEdge, StateNode,
    StateNodeType,
};
```

### Integration with BundleAccumulator

The `DagContextAssembler` integrates with `BundleAccumulator` (WG-117):
1. `BundleAccumulator` collects and prioritizes episodes
2. `DagContextAssembler` registers them in the DAG
3. Context assembly traverses DAG to produce deduplicated output
4. Integration test in `tests.rs` validates this pipeline

### Test Coverage

24 tests covering:
- StateNode: creation, refs, token savings, estimation
- StateEdge: creation, attributes, strength clamping
- StateDag: CRUD, shared context, removal cleanup, token reduction
- DagContextAssembler: registration, assembly, all 3 formats, reduction calculation
- Integration: DAG + BundleAccumulator pipeline, DAG clear

## Consequences

### Positive

- **86% token reduction** for episodes sharing context attributes
- **Structural deduplication** — shared context stored once, referenced many times
- **Format flexibility** — compact for token budgets, full for debugging, optimized for prompts
- **Clean integration** — works with existing `BundleAccumulator` pipeline
- **Zero new dependencies** — uses only existing `uuid`, `chrono`, `serde`, `tracing`
- **No storage coupling** — purely in-memory assembly

### Negative

- **+1,320 LOC** in `context/` module (stays within project conventions)
- **Registration step required** — episodes must be explicitly registered before assembly
- **ConceptGraph tier (WG-131)** still placeholder — full CSM cascade benefits not yet realized

### Neutral

- Module gated behind `context` module — no breaking API changes
- Existing `BundleAccumulator` behavior unchanged
- DAG is serializable (serde) for potential future persistence

## Validation

- [x] All 24 dag tests pass
- [x] Integration test with BundleAccumulator passes
- [x] Clippy clean
- [x] Format clean
- [x] Public types re-exported from `context/mod.rs`

## References

- Paper: arXiv:2602.22398 — DAG-based conversation state management
- WG-117: BundleAccumulator (bounded context assembly)
- WG-131: CascadeRetriever (CSM cascading pipeline)
- ADR-053: v0.1.31 Comprehensive Analysis
