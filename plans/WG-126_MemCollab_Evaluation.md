# WG-126: MemCollab Cross-Agent Memory Collaboration — Evaluation

**Date**: 2026-05-17
**Paper**: arXiv:2603.23234 — "MemCollab: Cross-Agent Memory Collaboration via Contrastive Trajectory Distillation"
**Status**: 🔵 Evaluated — Recommended for future sprint

---

## Paper Summary

MemCollab introduces a **contrastive trajectory distillation mechanism** to enable cross-agent memory sharing without transferring agent-specific biases. Instead of directly copying memory traces between agents, MemCollab:

1. Collects **paired reasoning trajectories** from different agents solving the same task (one successful, one failed)
2. **Contrasts** these trajectories to distill **abstract reasoning invariants** (common success patterns) and **violation patterns** (common failure modes)
3. Produces a **shared memory** of transferable, high-level reasoning constraints — not raw agent-specific chains of thought

Key findings:
- Cross-agent memory sharing improves performance across heterogeneous agent models
- Contrastive distillation filters out agent-specific idiosyncrasies, retaining only transferable knowledge
- Memory acts as a **collaborative reasoning resource**: constraints to enforce + failures to avoid
- Works across model families (cross-architecture transfer)

---

## Applicability to Agent Memory System

### Cross-Agent Collaboration Use Case

The Rust memory system already stores episodic memory traces per agent session. MemCollab's approach enables:

1. **Trajectory Collection**: Existing episodic memory already captures agent trajectories (steps, tools, outcomes)
2. **Contrastive Pairing**: Compare successful vs. failed episodes for the same task type
3. **Invariant Extraction**: Distill "what always works" and "what never works" across agents
4. **Shared Memory Index**: Store distilled invariants as retrievable memory entries

### Mapping to Existing Architecture

| MemCollab Component | Existing System Mapping |
|---------------------|------------------------|
| Trajectory encoding | `Episode.steps` + `ExecutionStep` (already captures tool/action/result) |
| Contrastive pairs | Group episodes by `TaskContext` + `TaskType`, compare success vs failure |
| Reasoning invariants | New `MemoryInvariant` pattern type — extracted successful reasoning patterns |
| Failure patterns | New `FailurePattern` type — anti-patterns to avoid |
| Shared memory index | Turso DB `memory_invariants` table with embeddings for retrieval |
| Task-aware retrieval | Semantic search over invariants filtered by `TaskContext` |

### Proposed Architecture

```rust
struct MemCollabEngine {
    invariant_extractor: ContrastiveExtractor,
    invariant_store: InvariantStore,      // Turso-backed
    failure_pattern_store: FailureStore,  // Turso-backed
}

struct ContrastiveExtractor {
    // Encodes episode trajectories into feature vectors
    // Applies contrastive loss between success/failure pairs
    // Distills invariant reasoning steps
}

struct MemoryInvariant {
    id: Uuid,
    task_type: TaskType,
    invariant_steps: Vec<InvariantStep>,  // "What always works"
    confidence: f32,                       // From contrastive pairing
    supporting_episodes: Vec<Uuid>,        // Source episodes
    embedding: Vec<f32>,                   // For semantic retrieval
}
```

### Benefits

| Benefit | Description |
|---------|-------------|
| **Cross-agent knowledge transfer** | Agents learn from each other's successes and failures |
| **Bias reduction** | Contrastive distillation filters agent-specific quirks |
| **Incremental learning** | New invariants discovered as more episodes accumulate |
| **Failure prevention** | Explicit failure patterns warn agents away from known pitfalls |
| **Zero-API retrieval** | Invariants stored locally, retrieved via embedding similarity |

### Challenges

| Challenge | Mitigation |
|-----------|------------|
| Paired trajectory data | Need minimum N success + M failure episodes per task type |
| Contrastive loss implementation | Requires careful encoding of step sequences; could use sentence-transformers for trajectory embedding |
| Invariant validation | Need confidence threshold — minimum supporting evidence before promoting to invariant |
| Storage schema | New tables in Turso + redb; schema migration required |
| Cold start | Bootstrap from existing episode corpus (thousands of episodes available) |

---

## Implementation Phases

### Phase 1: Infrastructure (4-5 days)
- New types: `MemoryInvariant`, `FailurePattern`, `InvariantStep`
- Turso schema: `memory_invariants`, `failure_patterns` tables
- redb schema: matching cache tables
- Storage trait methods: `store_invariant`, `query_invariants`, `store_failure_pattern`

### Phase 2: Contrastive Extraction (3-4 days)
- `ContrastiveExtractor`: trajectory encoding + contrastive pairing
- Episode grouping by `TaskContext` + `TaskType`
- Success/failure labeling from episode outcomes
- Invariant candidate generation with confidence scoring

### Phase 3: Retrieval Integration (2-3 days)
- Embedding generation for invariants
- Semantic similarity search for task-aware invariant retrieval
- Integration with existing CascadeRetriever (as additional retrieval tier)
- Weighting: invariants boost confidence for known patterns

### Phase 4: Testing & Tuning (2-3 days)
- Unit tests for invariant extraction
- Integration tests for cross-agent invariant transfer
- Benchmark: retrieval quality improvement from invariants
- Confidence threshold tuning

| **Total** | **11-15 days** | High effort, high impact |

---

## Effort Estimate

| Phase | Effort | Description |
|-------|--------|-------------|
| Infrastructure | 4-5 days | Types, schemas, storage traits |
| Contrastive extraction | 3-4 days | Trajectory encoding, pairing, scoring |
| Retrieval integration | 2-3 days | Embeddings, search, CascadeRetriever integration |
| Testing & tuning | 2-3 days | Tests, benchmarks, thresholds |
| **Total** | **11-15 days** | High effort, high impact |

---

## Comparison with WG-135 Federated HDC

| Aspect | WG-126 MemCollab | WG-135 Federated HDC |
|--------|-----------------|---------------------|
| Sharing mechanism | Contrastive distillation of trajectories | HDC prototype exchange |
| Data transferred | Abstract reasoning invariants | Binary HDC vectors (compact) |
| Bandwidth | Medium (invariant descriptions) | Low (binary vectors only) |
| Quality | High (distilled knowledge) | Medium (prototype matching) |
| Implementation complexity | High (contrastive learning) | Medium (HDC operations) |
| Complementarity | WG-126 + WG-135 could work together: HDC for fast prototype matching, contrastive for deep invariant extraction |

---

## Recommendation

**Recommended for future sprint (P2 priority)**. MemCollab addresses a key gap — the current system has no mechanism for cross-agent knowledge sharing. While implementation is high-effort, the architectural alignment is strong (existing episodics, pattern extraction, embedding retrieval). Consider implementing Phase 1 (infrastructure/schema) first to unblock incremental development.

---

## Cross-References

- WG-127: Semantic gist extraction (complementary — gists can be input to contrastive extraction)
- WG-131: CascadeRetriever (invariants become a new retrieval tier)
- WG-135: Federated HDC (complementary — lightweight prototype sharing)
- WG-120: Reconstructive retrieval windows (invariants enhance window selection)
- ADR-054: CloudEvents EventEmitter (could emit invariant discovery events)

## References

- Paper: <https://arxiv.org/abs/2603.23234>
- Contrastive Learning: <https://arxiv.org/abs/2002.05709> (SimCLR)
- Trajectory Distillation: <https://arxiv.org/abs/2310.10798>
