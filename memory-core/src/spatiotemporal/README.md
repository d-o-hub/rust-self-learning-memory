# Spatiotemporal Memory Organization

Phase 3 implementation for hierarchical spatiotemporal indexing and diverse episodic retrieval.

## Current Status

### ✅ Completed: Diversity Maximization (Tasks 3.1, 3.2)

The **DiversityMaximizer** module implements the Maximal Marginal Relevance (MMR) algorithm for selecting diverse, non-redundant results.

#### Features

- **MMR Algorithm**: Balances relevance and diversity
- **Configurable Trade-off**: Lambda parameter (0.0-1.0)
- **Cosine Similarity**: Embedding-based similarity calculation
- **Diversity Scoring**: Measures result set diversity (target: ≥0.7)

#### Quick Start

```rust
use memory_core::spatiotemporal::{DiversityMaximizer, ScoredEpisode};

// Create episodes with embeddings
let candidates = vec![
    ScoredEpisode::new("ep1".to_string(), 0.9, vec![1.0, 0.0, 0.0]),
    ScoredEpisode::new("ep2".to_string(), 0.85, vec![0.9, 0.1, 0.0]),
    ScoredEpisode::new("ep3".to_string(), 0.8, vec![0.0, 1.0, 0.0]),
];

// Select diverse episodes
let maximizer = DiversityMaximizer::new(0.7); // 70% relevance, 30% diversity
let diverse = maximizer.maximize_diversity(candidates, 2);

// Check diversity
let diversity_score = maximizer.calculate_diversity_score(&diverse);
assert!(diversity_score >= 0.7);
```

#### Configuration

| Lambda | Behavior | Use Case |
|--------|----------|----------|
| 1.0 | Pure relevance | Maximum accuracy |
| 0.7 | Balanced (default) | Recommended |
| 0.5 | Equal weight | Exploratory |
| 0.0 | Pure diversity | Maximum variety |

#### Testing

```bash
# Run diversity tests
cargo test --package memory-core --lib spatiotemporal::diversity::tests

# Check implementation
cargo clippy --package memory-core --lib

# Build
cargo build --package memory-core --lib
```

**Test Results**: 22/22 tests passing ✅

---

## Roadmap (Phase 3)

### 🚧 In Progress

#### Component 1: Hierarchical Indexing
- [ ] `SpatiotemporalIndex` - Three-level hierarchy (domain → task_type → temporal)
- [ ] `TemporalCluster` - Adaptive time-based bucketing
- [ ] Index maintenance (insert/remove/rebalance)

#### Component 2: Hierarchical Retrieval
- [ ] `HierarchicalRetriever` - Coarse-to-fine search
- [ ] Level 1: Domain filtering
- [ ] Level 2: Task type filtering
- [ ] Level 3: Temporal cluster selection
- [ ] Level 4: Embedding similarity
- [x] Diversity maximization (DiversityMaximizer) ✅

#### Component 3: Context-Aware Embeddings
- [ ] `ContextAwareEmbeddings` - Task-specific adaptation
- [ ] Contrastive learning
- [ ] Task adapters

#### Component 4: Integration
- [ ] Wire into `SelfLearningMemory`
- [ ] Configuration (`MemoryConfig`)
- [ ] Integration tests
- [ ] Benchmarks (accuracy, latency, diversity)

---

## Architecture

```
SelfLearningMemory
    ↓
HierarchicalRetriever
    ↓
┌─────────────────────────────┐
│ Level 1: Domain Filtering   │ → Filter by domain
├─────────────────────────────┤
│ Level 2: Task Type Filter   │ → Filter by task type
├─────────────────────────────┤
│ Level 3: Temporal Clusters  │ → Select recent clusters
├─────────────────────────────┤
│ Level 4: Similarity Scoring │ → Fine-grained similarity
└─────────────────────────────┘
    ↓
DiversityMaximizer (✅ IMPLEMENTED)
    ↓
Diverse, Relevant Results
```

---

## Research Foundation

Based on: **"Hierarchical Spatiotemporal Memory Organization for Efficient Episodic Retrieval"** (arXiv Nov 2025)

**Key Innovations**:
- Multi-level hierarchical indexing (+34% accuracy)
- Coarse-to-fine retrieval (≤100ms latency)
- MMR diversity (≥0.7 diversity score)
- Contrastive learning (task-specific embeddings)

---

## API Reference

See inline documentation:
```bash
cargo doc --package memory-core --no-deps --open
```

Navigate to: `memory_core::spatiotemporal`

---

## Performance Targets (Phase 3)

| Metric | Target | Status |
|--------|--------|--------|
| Retrieval accuracy | +34% vs baseline | 🚧 Pending |
| Query latency | ≤100ms | 🚧 Pending |
| Diversity score | ≥0.7 | ✅ Achieved |
| Unit tests | 40+ | 🚧 In Progress (22/40) |
| Integration tests | 20+ | 🚧 Pending |

---

## Contributing

When adding new components:

1. **Follow AGENTS.md guidelines**
   - Keep files <500 LOC (split if needed)
   - Use `anyhow::Result` for errors
   - Document all public APIs
   - Write comprehensive tests

2. **Test thoroughly**
   - Unit tests (>80% coverage)
   - Integration tests
   - Edge cases
   - Performance benchmarks

3. **Verify quality**
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test --all
   cargo build --all
   ```

---

## License

Same as parent project.

---

*Last Updated: 2025-12-26*
*Phase: 3.1 - Core Module Implementation*
