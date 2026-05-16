# WG-133: Agentic Memory Taxonomy Alignment — Evaluation

**Date**: 2026-05-01
**Paper**: arXiv:2602.19320 — "Anatomy of Agentic Memory"
**Status**: 🔵 Evaluated — Recommended for architectural documentation

---

## Paper Summary

"Anatomy of Agentic Memory" proposes a structured taxonomy of memory structures for LLM agents, providing a diagnostic framework for understanding when and why specific memory types are effective. The paper identifies common failure modes in current agentic memory systems:
- Misaligned evaluation metrics
- Lack of scalable memory organization
- Underutilization of specific memory structures for different task demands

The taxonomy categorizes memory into **4 structures** and maps them to agent capabilities like long-horizon reasoning, personalization, and state persistence beyond fixed context windows.

---

## The 4 Memory Structures

Based on the paper's taxonomy, the four memory structures are:

### 1. Working Memory (Short-Term)
- **Function**: Active task context, immediate state
- **Lifetime**: Single task / interaction
- **Capacity**: Limited (similar to human working memory: 7±2 items)
- **Our mapping**: `StepBuffer`, current episode execution state

### 2. Episodic Memory (Experience)
- **Function**: Records of past task executions with outcomes
- **Lifetime**: Persistent across sessions
- **Capacity**: Large, but benefits from filtering/summarization
- **Our mapping**: `Episode` (core), `EpisodeSummary`, spatiotemporal index

### 3. Semantic Memory (Knowledge)
- **Function**: Generalized facts, patterns, and heuristics derived from episodes
- **Lifetime**: Persistent, evolves over time
- **Capacity**: Moderate, deduplicated
- **Our mapping**: `Pattern`, `Heuristic`, `DualRewardScore`, Bayesian ranking

### 4. Procedural Memory (Skills)
- **Function**: Learned action sequences, workflows, playbooks
- **Lifetime**: Persistent, refined through practice
- **Capacity**: Small-medium, high-value
- **Our mapping**: `Playbook`, `CheckpointMeta`/`HandoffPack`, `RecommendationTracker`

---

## Alignment with Current Architecture

### What's Well-Aligned

| Taxonomy Structure | Our Implementation | Alignment |
|--------------------|--------------------|-----------|
| Working Memory | `StepBuffer` (batch I/O), in-flight episode state | ✅ Strong |
| Episodic Memory | `Episode` with storage backends, `SpatiotemporalIndex`, `EpisodeSummary` | ✅ Strong |
| Semantic Memory | `Pattern` extraction, `Heuristic`, `EffectivenessTracker`, `DBSCANAnomalyDetector` | ✅ Strong |
| Procedural Memory | `PlaybookGenerator`, `CheckpointMeta`/`HandoffPack` | ✅ Partial (ADR-044 features exist but need deeper integration) |

### Gaps

| Gap | Description | Priority |
|-----|-------------|----------|
| Procedural memory training | Playbooks are generated but not refined through repeated use | P2 |
| Memory consolidation | No explicit process for moving items from working → episodic → semantic | P2 |
| Salience-based forgetting | Current TTL/capacity eviction doesn't use learned importance scores | P3 |
| Cross-structure retrieval | Retrieval queries episodically but doesn't cross-reference all 4 structures | P2 |

---

## Recommendations

### 1. Documentation: Taxonomy Alignment Map

Update architecture docs (`docs/`) to explicitly map our memory types to the taxonomy. This helps new contributors understand the system design and provides a framework for future design decisions.

**Files to update**:
- `docs/API_REFERENCE.md` — Add "Memory Taxonomy" section
- `agent_docs/database_schema.md` — Add taxonomy annotations
- `agent_docs/service_architecture.md` — Add alignment diagram

### 2. Procedural Memory Enhancement (WG-124, already in backlog)

Implement `ProceduralMemory` as a first-class type:
- Playbook execution tracking (success/failure rates per step)
- Playbook refinement based on execution feedback
- Checkpoint-to-playbook promotion (repeated checkpoint patterns → playbook)

### 3. Cross-Structure Retrieval

Enhance `retrieve_relevant_context()` to query across all 4 structures:
```rust
struct UnifiedRetrievalResult {
    episodic: Vec<Episode>,       // Similar past episodes
    semantic: Vec<Pattern>,       // Relevant patterns/heuristics
    procedural: Vec<Playbook>,   // Applicable playbooks
    working: Vec<ContextItem>,    // Current session context
}
```

---

## Effort Estimate

| Task | Effort | Priority |
|------|--------|----------|
| Documentation alignment | 1 day | P1 |
| ProceduralMemory type (WG-124) | 5-7 days | P2 |
| Cross-structure retrieval | 3-4 days | P2 |
| Memory consolidation | 3-5 days | P3 |
| **Total** | **12-17 days** | Mixed priority |

---

## Recommendation

**Recommended for architectural documentation update (P1)**. The alignment map provides immediate value for onboarding and design decisions with low effort (1 day). Full implementation of procedural memory and cross-structure retrieval is deferred to future sprints (P2/P3).

---

## Cross-References

- ADR-044: High-Impact Features (Playbooks, Attribution, Checkpoints)
- ADR-050: Spatiotemporal Memory Organization
- ADR-053: v0.1.31 Comprehensive Analysis
- WG-124: Procedural Memory Type (Backlog)
- WG-134: DAG-based State Management (Complete)

## References

- Paper: <https://arxiv.org/abs/2602.19320>
- Human Memory Taxonomy: <https://en.wikipedia.org/wiki/Memory#Types>
