# WG-135: Federated HDC for Multi-Agent Memory Sharing — Evaluation

**Date**: 2026-05-01
**Paper**: arXiv:2603.20037 — "Federated Hyperdimensional Computing for Resource-Constrained Industrial IoT"
**Authors**: Nikita Zeulin, O. Galinina, R. Balakrishnan, N. Himayat, S. Andreev
**Status**: ✅ Evaluated — Recommended for P3 backlog

---

## Paper Summary

The paper proposes integrating Hyperdimensional Computing (HDC) into a Federated
Learning (FL) framework for resource-constrained edge devices. The core innovation
is exchanging compact **prototype vectors** instead of full embeddings or raw data,
significantly reducing communication overhead while enabling collaborative learning.

Key properties of HDC leveraged:
- **10,240-bit hypervectors** — robust to noise, hardware-friendly
- **Holistic operations** — binding, bundling, permutation are O(n) and SIMD-friendly
- **One-shot learning** — no iterative backpropagation needed

---

## How Federated HDC Works

### Prototype Exchange Instead of Full Sync

Traditional federated learning synchronizes full model weights across devices.
Federated HDC instead exchanges only **prototype vectors**:

1. Each agent/device computes a prototype hypervector for each learned concept/class
   by bundling (summing) the hypervectors of all examples in that class
2. Prototypes are compact (~10KB for a 10,240-bit vector) vs. full embeddings
   (often 100KB-1MB+)
3. The central aggregator combines prototypes via bundling, producing a global
   prototype that represents the consensus across all agents
4. Agents download the aggregated prototype, enabling them to benefit from
   other agents' experiences without ever sharing raw data

### Communication Efficiency

| Approach | Payload per concept | Privacy |
|----------|-------------------|---------|
| Raw data sharing | 1-100MB+ | ❌ None |
| Full embedding sync | 100KB-1MB | ⚠️ Partial |
| Federated HDC prototype | ~10KB | ✅ Strong |

The prototype is a **distilled summary** of local data distributions — it cannot
be inverted to reconstruct individual training examples.

---

## Relevance to do-memory

### Existing HDC Foundation (CSM Integration)

The project already has HDC capabilities via the `chaotic_semantic_memory` (CSM)
crate behind the `csm` feature flag:

- **`HVec10240`** — 10,240-bit HDC vectors (matching the paper's dimensionality)
- **`HdcEncoder`** — text-to-hypervector encoding
- **`Bm25Index`** — keyword search (Tier 1 of cascade)
- **HDC similarity** — cosine similarity for Tier 2 retrieval

This means the foundational HDC operations (encoding, similarity, bundling) are
already available. Adding federated prototype exchange would reuse these primitives.

### Multi-Agent Use Cases

| Use Case | Current State | Federated HDC Benefit |
|----------|--------------|----------------------|
| Multi-instance agent teams | Each instance has isolated memory | Share distilled patterns without raw episode data |
| Cross-project learning | No transfer between projects | Aggregate prototypes across codebases |
| Edge/CI agent collaboration | Isolated CI runs | Merge episodic insights from parallel CI jobs |
| Privacy-preserving memory | Full episode sharing required | Prototypes preserve privacy |

---

## Architecture Considerations

### Prototype Bundling for Episodic Memory

In do-memory's context, a "prototype" could represent:
- **Domain patterns** — aggregated hypervectors for "web-api", "cli", "data-science"
- **Heuristic categories** — bug-fix patterns, refactoring patterns, etc.
- **Agent behavior signatures** — typical tool sequences, error patterns

### Integration Points

```
Agent A                         Aggregator                      Agent B
   │                                │                              │
   ├─ Encode episodes → HVec ──►   │                              │
   ├─ Bundle → prototype     ──►   │   ◄── Bundle → prototype ──┤
   │                                │                              │
   │   ◄── Global prototype ────┤   ├── Global prototype ──────►   │
   │                                │                              │
   ├─ Unbundle → enrich local ─┘   │   └─ Unbundle → enrich ───┘
```

### Required New Components

| Component | Description | Est. LOC |
|-----------|-------------|----------|
| `PrototypeBundler` | Bundle HVecs into class prototypes | ~100 |
| `FederatedAggregator` | Combine prototypes from multiple agents | ~150 |
| `PrototypeExchange` | Serialize/deserialize prototypes for network transfer | ~80 |
| `MultiAgentMemory` | Orchestrate federated memory across agent instances | ~200 |
| Tests | Integration + property tests | ~200 |
| **Total** | | **~730** |

---

## Technical Alignment: HDC for Trajectory Distillation

### Encoding Agent Trajectories (MemCollab Alignment)

MemCollab (arXiv:2603.23234) proposes contrastive trajectory distillation for
sharing cross-agent experiences. Federated HDC (arXiv:2603.20037) provides a
highly efficient mechanism to implement this:

1. **Step Encoding**: Each tool-call or execution step is encoded as an HVec
2. **Trajectory Binding**: A sequence of steps $S_1, S_2, \dots, S_n$ is encoded
   using the permutation operator ($\Pi$) to preserve temporal order:
   $H_{traj} = S_1 + \Pi(S_2) + \Pi^2(S_3) + \dots + \Pi^{n-1}(S_n)$
3. **Prototype Distillation**: Multiple trajectories representing a successful
   pattern (e.g., "fix-rust-clippy-lint") are bundled into a single prototype:
   $P_{pattern} = \text{Bundle}(H_{traj\_1}, H_{traj\_2}, \dots, H_{traj\_k})$

This allows agents to share **actionable sequences** rather than just static
semantic embeddings.

### Bandwidth & Resource Comparison

| Metric | Full Distillation (MemCollab) | Federated HDC Prototype | Savings |
|--------|-----------------------------|-------------------------|---------|
| Payload Size | 100-500 KB (Weights/Embeds) | 1.25 KB (10,240 bits) | **~98%** |
| Aggregation | Gradient averaging (heavy) | Bitwise bundling (SIMD) | **~99%** |
| Local Compute | Backprop required | One-shot (XOR/Sum) | **High** |

---

## Proposed Data Structures

### MemoryPrototype

```rust
/// A distilled memory representation for a specific concept or pattern.
pub struct MemoryPrototype {
    /// Unique identifier for the concept (e.g., "rust-refactoring-v1")
    pub concept_id: String,
    /// The 10,240-bit hypervector representing the distilled experience
    pub vector: HVec10240,
    /// Number of episodes bundled into this prototype
    pub sample_count: u32,
    /// Average reward/confidence for this prototype
    pub confidence: f32,
}
```

### PrototypePayload

```rust
/// The network exchange format for federated HDC.
pub struct PrototypePayload {
    /// Agent identifier
    pub agent_id: String,
    /// Collection of prototypes being shared
    pub prototypes: Vec<MemoryPrototype>,
    /// Timestamp of generation
    pub timestamp: u64,
}
```

---

## Feasibility Assessment

### Strengths

- **Reuses existing HDC primitives** — `HVec10240`, `HdcEncoder`, bundling operations
  already available via CSM
- **Low communication overhead** — ~10KB per prototype vs. MBs for full sync
- **Privacy-preserving** — prototypes cannot be inverted to raw data
- **Hardware-friendly** — HDC operations are SIMD-acceleratable, no GPU needed
- **Aligns with CSM cascade** — could serve as Tier 0 (cross-agent) before Tier 1 (BM25)

### Challenges

- **Semantic consistency** — ensuring prototypes from different agents represent
  compatible concepts (requires shared ontology — ConceptGraph from WG-131 helps)
- **Prototype drift** — concepts may evolve differently across agents; needs
  versioning and staleness detection
- **Cold start** — new agents need bootstrap prototypes; could use the embedded
  ontology as seed
- **Aggregation strategy** — simple bundling may lose information; weighted
  bundling by agent reliability could help
- **Network layer** — requires a prototype exchange protocol (gRPC, HTTP, or
  message queue); not currently in scope

### Risk Level: Medium

The core HDC math is well-understood and already implemented. The primary risk is
in the distributed coordination layer (aggregation, serialization, network protocol).

---

## Comparison with Other Approaches

| Approach | Bandwidth | Privacy | Complexity | Already in do-memory |
|----------|-----------|---------|------------|---------------------|
| Full embedding sync | High | Low | Low | ✅ (API-based) |
| Differential privacy | Medium | Medium | High | ❌ |
| Federated HDC (this) | Low | High | Medium | ⚠️ (HDC primitives exist) |
| Homomorphic encryption | High | Very High | Very High | ❌ |

---

## Recommendation

**Recommended for P3 backlog.** The approach has strong technical alignment with
existing infrastructure (CSM HDC primitives) and addresses a real need for
multi-agent memory sharing. However, the distributed coordination layer requires
substantial design work and should be deferred until:

1. The CSM cascade (WG-128 through WG-131) is proven in production
2. Multi-agent deployment patterns are better understood
3. A prototype exchange protocol is designed

**Estimated effort**: 5-8 days for a working prototype, 10-15 days for production-quality.

### Deferred Dependencies

- WG-131 (CascadeRetriever) — must be stable before adding cross-agent tier
- Network transport layer — needs design (not in current scope)
- Multi-agent test infrastructure — needs CI support

---

## References

- Paper: <https://arxiv.org/abs/2603.20037>
- CSM Integration: `agent_docs/csm_integration.md`
- WG-128/129/130/131: CSM cascade pipeline (BM25 → HDC → ConceptGraph → API)
- ADR-037: CSM workflow adoption
- ADR-053: v0.1.31 Comprehensive Analysis

## Cross-References

- GOAP_STATE.md: Phase 3 (WG-135, Federated HDC)
- GOALS.md: Goal 25 (WG-135)
- ACTIONS.md: ACT-126 (WG-135 evaluation)
- WG-131: CascadeRetriever implementation
