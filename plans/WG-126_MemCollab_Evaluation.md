# WG-126: MemCollab Cross-Agent Memory Collaboration — Evaluation

**Date**: 2026-05-17 (post-hoc — implementation merged May 2026 via PR #572)
**Paper**: arXiv:2603.23234 — "MemCollab: Cross-Agent Memory Collaboration via Contrastive Trajectory Distillation"
**Status**: ✅ Complete — Merged in PR #572

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

## Implementation Summary (PR #572)

The implementation delivered the following in `memory-core/src/learning/distillation/` and `memory-core/src/memory/collaboration.rs`:

- **Trajectory distillation**: Episodes converted into compact semantic representations
- **Contrastive triplet adapter**: Trajectory adapter with contrastive learning for invariant extraction
- **Collaborative prototypes**: Manage, bundle, and share distilled prototypes across agents
- **Tier-0 collaboration check**: Optional collaboration-aware retrieval pre-check
- **Memory APIs**: `distill_prototypes()` and `sync_prototypes()` for cross-agent sharing
- **Tests**: Unit tests for distillation, adapter training, and prototype management

Files changed: `learning/distillation/adapter.rs`, `learning/distillation/mod.rs`, `learning/distillation/tests.rs`, `memory/collaboration.rs`, `memory/types.rs`, `retrieval/cascade/mod.rs`, and more.

---

## Applicability to Agent Memory System

### Cross-Agent Collaboration Use Case

The Rust memory system already stores episodic memory traces per agent session. MemCollab's approach enables:

1. **Trajectory Collection**: Existing episodic memory already captures agent trajectories (steps, tools, outcomes)
2. **Contrastive Pairing**: Compare successful vs. failed episodes for the same task type
3. **Invariant Extraction**: Distill "what always works" and "what never works" across agents
4. **Shared Memory Index**: Store distilled invariants as retrievable memory entries

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

## Implementation Phases (for future extensions)

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
