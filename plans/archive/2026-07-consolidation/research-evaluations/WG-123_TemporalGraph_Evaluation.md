# WG-123: Temporal Graph Edges in Episode Store — Evaluation

**Date**: 2026-05-17 (post-hoc — implementation merged May 2026 via PR #570)
**Paper**: arXiv:2602.13530 (ICLR 2026) — "REMem: Temporal Graph-Enhanced Episodic Memory for Language Agents"
**Status**: ✅ Complete — Merged in PR #570

---

## Paper Summary

REMem introduces a **two-phase episodic memory framework** that constructs a **hybrid temporal memory graph**:

1. **Offline Indexing Phase**: Converts raw agent experiences into gists and facts, organized into a graph with explicit temporal and situational edges
2. **Agentic Inference Phase**: Tool-augmented multi-hop graph traversal enables temporally-aware episodic retrieval and reasoning

Key findings:
- **3.4-13.4% improvement** over SOTA episodic memory systems on recollection and reasoning benchmarks
- Temporal graph edges explicitly encode event order, duration, and proximity
- Multi-hop graph traversal enables reasoning over event sequences beyond flat vector similarity
- Hybrid graph unifies concept-level (entities, facts) and context-level (event gists, temporal facts) information

---

## Implementation Summary (PR #570)

The implementation delivered the following in `memory-core/src/episode/graph_algorithms/`:

- **Weighted traversal queries**: Graph traversal with significance weights (0.0-1.0)
- **Episode-to-pattern relationships**: New relationship type connecting episodes to extracted patterns
- **Path-finding with weights**: Enhanced path-finding algorithm incorporating relationship weights
- **Schema updates**: Storage schema extended to persist weights (`episode_relationships.weight` column)
- **Tests**: Comprehensive tests for weighted and pattern relationships

Files changed: `graph_algorithms/mod.rs`, `graph_algorithms/tests.rs`, `graph_algorithms/traversal.rs`, `episode/relationships.rs`, `episode/relationship_manager_tests.rs`, CLI commands, and storage backends.

---

## Applicability to Agent Memory System

### Temporal Graph Use Case

The Rust memory system already stores episodes with timestamps and basic relationships (`episode_relationships` table). REMem's approach extends this to:

1. **Rich temporal edges**: `before`, `after`, `during`, `overlaps` — not just `DependsOn`
2. **Temporal weighting**: Edge weights decay with time, preserving recency while maintaining history
3. **Situational edges**: Links episodes sharing participants, domains, tools, or task types
4. **Graph traversal queries**: Multi-hop traversal for "what happened after X that involved Y?"

### Mapping to Existing Architecture

| REMem Component | Existing System Mapping | Enhancement Needed |
|----------------|------------------------|-------------------|
| Temporal edges | `episode_relationships` with `RelationshipType` | Add temporal edge types (`Before`, `After`, `Overlaps`, `During`) |
| Event gists | `episode_summaries` table | Add temporal context to gist generation |
| Temporal facts | Episode `start_time`/`end_time` fields | Extract temporal facts (durations, sequences) |
| Multi-hop traversal | `get_weighted_neighbors()` | Add temporal-filtered traversal + path queries |
| Graph index | Turso relationship indexes | Add temporal-range indexes for time-window queries |

### Proposed Architecture (Future Enhancements)

```rust
/// Temporal graph edge types (extends existing RelationshipType)
enum TemporalEdgeType {
    HappensBefore,     // Episode A occurred before Episode B
    HappensAfter,      // Episode A occurred after Episode B
    Overlaps,          // Episodes overlap in time
    During,            // Episode A occurred during Episode B
    SharesContext,     // Episodes share task context/domain
    SharesTool,        // Episodes used the same tools
    CausedBy,          // Episode A caused Episode B
}

struct TemporalGraphEdge {
    edge_id: Uuid,
    from_episode_id: Uuid,
    to_episode_id: Uuid,
    edge_type: TemporalEdgeType,
    temporal_weight: f32,      // Decays with time distance
    confidence: f32,            // Confidence in the edge relation
    metadata: TemporalMetadata,
}
```

### Benefits

| Benefit | Description |
|---------|-------------|
| **Temporal reasoning** | Answer queries like "what happened before/after/during X?" |
| **Contextual retrieval** | Retrieve episodes within temporal windows, not just by semantic similarity |
| **Sequence pattern discovery** | Identify common episode sequences (debug → fix → test) |
| **Decay-weighted retrieval** | Older relationships naturally decay, keeping recent context prioritized |
| **Multi-hop reasoning** | Traverse episode chains to reconstruct full task workflows |

### Challenges

| Challenge | Mitigation |
|-----------|------------|
| Edge explosion | N² potential edges per N episodes; use threshold-based edge creation |
| Schema migration | New `temporal_edges` table; backward-compatible with existing `episode_relationships` |
| Temporal weighting tuning | Exponential decay function needs calibration for the agent domain |
| Multi-hop performance | Limit traversal depth (max 3-4 hops); index temporal ranges |
| Cold start | Seed temporal edges from existing episode timestamps |

---

## Implementation Phases (for future extensions)

### Phase 1: Schema & Types (3-4 days)
- New types: `TemporalEdgeType`, `TemporalGraphEdge`, `TemporalMetadata`
- Turso schema: `temporal_edges` table with temporal-range indexes
- redb schema: matching cache table
- Migration from existing `episode_relationships` (extract temporal edges from timestamps)

### Phase 2: Edge Creation (3-4 days)
- Auto-extract temporal edges from episode timestamps
- Extract situational edges from shared context/domain/tools
- Temporal weighting with exponential decay
- Edge creation threshold to prevent explosion

### Phase 3: Graph Traversal (3-4 days)
- `TemporalGraphTraverser`: multi-hop constrained traversal
- Time-window queries ("episodes in last hour")
- Path queries ("episode chain from debug to fix")
- Integration with existing `get_weighted_neighbors()`

### Phase 4: Retrieval Integration (2-3 days)
- Temporal-filtered retrieval in CascadeRetriever
- Hybrid scoring: semantic similarity × temporal proximity
- Sequence pattern discovery from traversal paths
- Benchmark: retrieval precision improvement

### Phase 5: Testing & Tuning (2-3 days)
- Unit tests for edge extraction and traversal
- Integration tests for temporal queries
- Benchmark: temporal vs flat retrieval quality
- Edge creation threshold tuning

| **Total** | **13-18 days** | High effort, high impact |

---

## Comparison with Existing Relationships

| Aspect | Current `episode_relationships` | REMem Temporal Graph |
|--------|-------------------------------|---------------------|
| Edge types | `DependsOn`, `RelatedTo`, `Blocks`, `Follows`, `References` | + `Before`, `After`, `Overlaps`, `During`, `SharesContext`, `SharesTool`, `CausedBy` |
| Temporal awareness | Implicit via timestamps | Explicit temporal edges with weights |
| Traversal | Single-hop via `get_weighted_neighbors()` | Multi-hop with temporal constraints |
| Weighting | Static `weight` field | Time-decaying `temporal_weight` |
| Edge discovery | Manual via `add_relationship()` | Auto-extracted from timestamps + context |

---

## Cross-References

- WG-127: Semantic gist extraction (gists are inputs to temporal graph nodes)
- WG-126: MemCollab (temporal edges inform contrastive trajectory pairing)
- WG-131: CascadeRetriever (temporal graph as additional retrieval tier)
- WG-120: Reconstructive retrieval windows (temporal windows complement reconstructive windows)
- WG-134: DAG-based state management (temporal edges are a natural DAG extension)

## References

- Paper: <https://arxiv.org/abs/2602.13530>
- ICLR 2026: <https://openreview.net/forum?id=REMem2026>
- Temporal Graph Networks: <https://arxiv.org/abs/2006.10637>
