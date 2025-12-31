# Phase 3 Integration Plan (Spatiotemporal Memory Organization)

**Document Version**: 1.0
**Created**: 2025-12-26
**Phase**: Phase 3 - Spatiotemporal Memory Organization (Days 21-30)
**Dependencies**: Phase 1 (PREMem) ✅, Phase 2 (GENESIS) ✅

---

## Executive Summary

This document provides a comprehensive integration plan for Phase 3 (Spatiotemporal Memory Organization). The plan details how to implement hierarchical spatiotemporal indexing for episodic retrieval, achieving +34% accuracy improvement through multi-level clustering, coarse-to-fine search, diversity maximization, and context-aware embeddings.

**Key Objectives**:
1. Implement hierarchical time-domain clustering (domain → task_type → temporal)
2. Build coarse-to-fine retrieval strategy
3. Add MMR-based diversity maximization (≥0.7 diversity score)
4. Integrate context-aware embeddings with contrastive learning
5. Achieve +34% retrieval accuracy improvement
6. Maintain query latency ≤100ms

---

## Context

### Research Foundation

**Paper**: "Hierarchical Spatiotemporal Memory Organization for Efficient Episodic Retrieval" (arXiv Nov 2025)

**Key Innovations**:
1. **Multi-level Hierarchical Indexing**: Episodes organized by domain → task_type → time
2. **Coarse-to-Fine Retrieval**: Start broad, narrow down progressively
3. **MMR Diversity**: Maximal Marginal Relevance for diverse, relevant results
4. **Contrastive Learning**: Task-specific embedding adaptation

### Completed Phases

**Phase 1 (PREMem)**: ✅ Complete
- Quality assessment before storage
- Salient feature extraction
- 89% quality assessment accuracy

**Phase 2 (GENESIS)**: ✅ Complete and Validated
- Capacity-constrained storage (10,000 episodes)
- Semantic summarization (100-200 words)
- 5.56× - 30.6× compression
- Relevance-weighted eviction

### Current Retrieval Architecture

**SelfLearningMemory::retrieve_relevant_context()**:
- Flat sequential search through all episodes
- Similarity-based retrieval using embeddings
- No hierarchical structure
- No diversity maximization
- Limited to top-k by similarity alone

**Limitations**:
- Inefficient for large episode collections
- May return redundant similar episodes
- No temporal awareness
- No domain/task-type filtering
- Linear time complexity O(n)

---

## Integration Goals

### Goal 1: Hierarchical Spatiotemporal Indexing
Organize episodes in multi-level hierarchy for efficient retrieval.

**Success Criteria**:
- Three-level hierarchy: domain → task_type → temporal clusters
- Episodes automatically indexed on storage
- Efficient lookup by domain, task_type, time range
- Temporal clusters sized for optimal retrieval (100-500 episodes)

### Goal 2: Coarse-to-Fine Retrieval
Multi-level search strategy for accuracy and speed.

**Success Criteria**:
- Level 1: Domain filtering (if specified)
- Level 2: Task-type filtering (if specified)
- Level 3: Temporal cluster selection (recent bias)
- Level 4: Fine-grained similarity within clusters
- Query latency ≤100ms

### Goal 3: Diversity Maximization
MMR algorithm to avoid redundant results.

**Success Criteria**:
- MMR implementation with λ parameter (default: 0.7)
- Diversity score ≥0.7 for result sets
- Balance relevance vs diversity
- Configurable diversity weight

### Goal 4: Context-Aware Embeddings
Improve semantic similarity through task-specific adaptation.

**Success Criteria**:
- Contrastive learning for embedding refinement
- Task-type specific embedding spaces
- Improved retrieval accuracy (+34% vs baseline)
- Backward compatibility with existing embeddings

### Goal 5: Performance
Maintain fast, scalable retrieval.

**Success Criteria**:
- Query latency ≤100ms (hierarchical search)
- Scales sub-linearly with episode count
- Efficient cluster management
- Minimal memory overhead

---

## Task Decomposition

### Component 1: SpatiotemporalIndex Module (4-6 hours)

#### Task 1.1: Design Hierarchical Index Structure
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- Episode struct with domain, task_type, timestamp
- Hierarchical indexing requirements

**Actions**:
1. Design three-level index structure
2. Define temporal clustering algorithm (time-based buckets)
3. Plan index update operations (insert, remove, rebalance)
4. Design query interface for hierarchical lookup

**Output**:
```rust
pub struct SpatiotemporalIndex {
    // Level 1: Domain index
    domains: HashMap<String, DomainIndex>,
}

pub struct DomainIndex {
    domain: String,
    // Level 2: Task type index
    task_types: HashMap<TaskType, TaskTypeIndex>,
}

pub struct TaskTypeIndex {
    task_type: TaskType,
    // Level 3: Temporal clusters
    temporal_clusters: Vec<TemporalCluster>,
}

pub struct TemporalCluster {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    episode_ids: Vec<Uuid>,
    cluster_size: usize,
}
```

**Success Criteria**:
- Three-level hierarchy designed
- Temporal clustering strategy defined (e.g., monthly, weekly buckets)
- Efficient lookup methods specified
- Rebalancing strategy planned

**Dependencies**: None
**Estimated Time**: 1.5 hours

---

#### Task 1.2: Implement SpatiotemporalIndex Core
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- Index structure design from Task 1.1
- Episode data for indexing

**Actions**:
1. Implement `SpatiotemporalIndex::new()`
2. Implement `insert_episode()` - Add to hierarchy
3. Implement `remove_episode()` - Remove from hierarchy
4. Implement temporal clustering logic
5. Implement `query()` method for hierarchical lookup

**Output**:
```rust
impl SpatiotemporalIndex {
    pub fn new() -> Self { }

    pub fn insert_episode(&mut self, episode: &Episode) -> Result<()> {
        // Insert into domain → task_type → temporal hierarchy
    }

    pub fn remove_episode(&mut self, episode_id: Uuid) -> Result<()> {
        // Remove from all levels
    }

    pub fn query(
        &self,
        domain: Option<&str>,
        task_type: Option<TaskType>,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Vec<Uuid> {
        // Return episode IDs matching criteria
    }

    pub fn get_temporal_clusters(
        &self,
        domain: Option<&str>,
        task_type: Option<TaskType>,
    ) -> &[TemporalCluster] {
        // Return temporal clusters for given domain/task_type
    }
}
```

**Success Criteria**:
- All core methods implemented
- Episodes inserted into correct hierarchy
- Temporal clusters auto-created (e.g., monthly buckets)
- Query returns correct episode IDs
- O(log n) lookup time

**Dependencies**: Task 1.1
**Estimated Time**: 2.5 hours

---

#### Task 1.3: Add SpatiotemporalIndex Tests
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Medium

**Actions**:
1. Test insertion into hierarchy
2. Test removal from hierarchy
3. Test temporal clustering
4. Test hierarchical queries (all levels)
5. Test edge cases (empty index, single episode, etc.)

**Output**: 12+ unit tests

**Success Criteria**:
- 12+ tests passing
- All public methods tested
- Edge cases covered
- Performance validated (O(log n))

**Dependencies**: Task 1.2
**Estimated Time**: 1 hour

---

### Component 2: HierarchicalRetriever Module (4-6 hours)

#### Task 2.1: Design Coarse-to-Fine Retrieval Strategy
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- SpatiotemporalIndex from Component 1
- Current retrieval workflow
- Relevance scoring requirements

**Actions**:
1. Design 4-level retrieval strategy
2. Define relevance scoring across levels
3. Plan temporal bias (recent episodes weighted higher)
4. Design cluster selection algorithm

**Output**:
```rust
pub struct HierarchicalRetriever {
    index: SpatiotemporalIndex,
    temporal_bias_weight: f32,  // Default: 0.3
    max_clusters_to_search: usize,  // Default: 5
}

pub struct RetrievalQuery {
    query_text: String,
    query_embedding: Option<Vec<f32>>,
    domain: Option<String>,
    task_type: Option<TaskType>,
    limit: usize,
}

pub struct ScoredEpisode {
    episode_id: Uuid,
    relevance_score: f32,
    level_1_score: f32,  // Domain match
    level_2_score: f32,  // Task type match
    level_3_score: f32,  // Temporal proximity
    level_4_score: f32,  // Embedding similarity
}
```

**Success Criteria**:
- 4-level strategy designed
- Scoring formula defined per level
- Temporal bias incorporated
- Cluster selection algorithm specified

**Dependencies**: Task 1.2
**Estimated Time**: 1.5 hours

---

#### Task 2.2: Implement Hierarchical Retrieval
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- Retrieval strategy from Task 2.1
- SpatiotemporalIndex

**Actions**:
1. Implement Level 1: Domain filtering
2. Implement Level 2: Task-type filtering
3. Implement Level 3: Temporal cluster selection (top-k recent)
4. Implement Level 4: Embedding similarity within clusters
5. Combine scores across levels

**Output**:
```rust
impl HierarchicalRetriever {
    pub fn new(index: SpatiotemporalIndex) -> Self { }

    pub async fn retrieve(
        &self,
        query: &RetrievalQuery,
        storage: &dyn StorageBackend,
    ) -> Result<Vec<ScoredEpisode>> {
        // Level 1: Filter by domain
        let domain_candidates = self.filter_by_domain(query)?;

        // Level 2: Filter by task type
        let task_type_candidates = self.filter_by_task_type(domain_candidates, query)?;

        // Level 3: Select temporal clusters (recent bias)
        let clusters = self.select_temporal_clusters(task_type_candidates, query)?;

        // Level 4: Fine-grained similarity
        let scored = self.score_episodes_in_clusters(clusters, query, storage).await?;

        // Combine and rank
        let ranked = self.rank_by_combined_score(scored)?;

        Ok(ranked)
    }
}
```

**Success Criteria**:
- All 4 levels implemented
- Scores combined correctly
- Temporal bias working (recent episodes scored higher)
- Query latency ≤100ms

**Dependencies**: Task 2.1
**Estimated Time**: 3 hours

---

#### Task 2.3: Add HierarchicalRetriever Tests
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Medium

**Actions**:
1. Test each retrieval level individually
2. Test combined retrieval workflow
3. Test temporal bias (verify recent episodes ranked higher)
4. Test with various query types (domain-specific, task-specific, general)
5. Benchmark query latency

**Output**: 15+ unit tests

**Success Criteria**:
- 15+ tests passing
- All levels tested
- Latency ≤100ms validated
- Temporal bias verified

**Dependencies**: Task 2.2
**Estimated Time**: 1.5 hours

---

### Component 3: DiversityMaximizer Module (3-4 hours)

#### Task 3.1: Implement MMR Algorithm
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Medium

**Input**:
- Scored episodes from HierarchicalRetriever
- MMR algorithm specification (λ parameter)

**Actions**:
1. Implement MMR (Maximal Marginal Relevance) algorithm
2. Add similarity calculation between episodes
3. Implement iterative selection (relevance vs diversity trade-off)
4. Make λ parameter configurable

**Output**:
```rust
pub struct DiversityMaximizer {
    lambda: f32,  // Default: 0.7 (70% relevance, 30% diversity)
}

impl DiversityMaximizer {
    pub fn new(lambda: f32) -> Self { }

    pub fn maximize_diversity(
        &self,
        candidates: Vec<ScoredEpisode>,
        limit: usize,
    ) -> Vec<ScoredEpisode> {
        // MMR algorithm:
        // Score(e) = λ * Relevance(e) - (1-λ) * max(Similarity(e, selected))
        // Iteratively select episodes with highest MMR score
    }

    fn calculate_similarity(
        &self,
        episode1: &ScoredEpisode,
        episode2: &ScoredEpisode,
    ) -> f32 {
        // Use embedding similarity or feature overlap
    }

    pub fn calculate_diversity_score(
        &self,
        selected: &[ScoredEpisode],
    ) -> f32 {
        // Average pairwise dissimilarity
    }
}
```

**Success Criteria**:
- MMR algorithm implemented correctly
- λ parameter adjustable (0.0-1.0)
- Diversity score calculation working
- Result sets have ≥0.7 diversity

**Dependencies**: None (independent module)
**Estimated Time**: 2 hours

---

#### Task 3.2: Add DiversityMaximizer Tests
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Low

**Actions**:
1. Test MMR with various λ values
2. Test diversity score calculation
3. Test edge cases (single result, identical episodes)
4. Verify ≥0.7 diversity for typical queries

**Output**: 10+ unit tests

**Success Criteria**:
- 10+ tests passing
- MMR working correctly
- Diversity score ≥0.7 validated
- Edge cases handled

**Dependencies**: Task 3.1
**Estimated Time**: 1 hour

---

### Component 4: ContextAwareEmbeddings Module (4-5 hours)

#### Task 4.1: Design Contrastive Learning Strategy
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- Existing embedding infrastructure
- Contrastive learning requirements
- Task-type specific adaptation needs

**Actions**:
1. Design contrastive learning approach (positive/negative pairs)
2. Plan task-type specific embedding spaces
3. Design fine-tuning strategy for embeddings
4. Plan backward compatibility

**Output**:
```rust
pub struct ContextAwareEmbeddings {
    base_embeddings: Arc<dyn EmbeddingProvider>,
    task_adapters: HashMap<TaskType, TaskAdapter>,
}

pub struct TaskAdapter {
    task_type: TaskType,
    adaptation_matrix: Vec<Vec<f32>>,  // Linear transformation
    trained_on_count: usize,
}

pub struct ContrastivePair {
    anchor: Episode,
    positive: Episode,  // Similar task, successful
    negative: Episode,  // Different task or failed
}
```

**Success Criteria**:
- Contrastive learning strategy designed
- Task-type adapters planned
- Fine-tuning approach specified
- Backward compatibility ensured

**Dependencies**: None
**Estimated Time**: 1.5 hours

---

#### Task 4.2: Implement Context-Aware Embedding Adaptation
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- Design from Task 4.1
- Episode embeddings
- Task-type information

**Actions**:
1. Implement `ContextAwareEmbeddings::new()`
2. Implement task adapter training (contrastive loss)
3. Implement adapted embedding generation
4. Add fallback to base embeddings if no adapter

**Output**:
```rust
impl ContextAwareEmbeddings {
    pub fn new(base_embeddings: Arc<dyn EmbeddingProvider>) -> Self { }

    pub async fn get_adapted_embedding(
        &self,
        text: &str,
        task_type: Option<TaskType>,
    ) -> Result<Vec<f32>> {
        // Get base embedding
        let base = self.base_embeddings.embed(text).await?;

        // Apply task-specific adaptation if available
        if let Some(task) = task_type {
            if let Some(adapter) = self.task_adapters.get(&task) {
                return Ok(adapter.adapt(base));
            }
        }

        // Fallback to base embedding
        Ok(base)
    }

    pub fn train_adapter(
        &mut self,
        task_type: TaskType,
        contrastive_pairs: &[ContrastivePair],
    ) -> Result<()> {
        // Train task-specific adapter using contrastive learning
        // Minimize distance to positives, maximize to negatives
    }
}
```

**Success Criteria**:
- Adapted embeddings generated
- Contrastive learning working
- Task-specific adapters trainable
- Backward compatibility maintained

**Dependencies**: Task 4.1
**Estimated Time**: 2.5 hours

---

#### Task 4.3: Add ContextAwareEmbeddings Tests
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: Medium

**Actions**:
1. Test base embedding fallback
2. Test task adapter training
3. Test adapted embedding generation
4. Verify improved similarity for same-task episodes

**Output**: 8+ unit tests

**Success Criteria**:
- 8+ tests passing
- Adapters improving similarity
- Backward compatibility working
- Contrastive learning validated

**Dependencies**: Task 4.2
**Estimated Time**: 1 hour

---

### Component 5: SelfLearningMemory Integration (3-4 hours)

#### Task 5.1: Update retrieve_relevant_context Method
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- Current retrieve_relevant_context implementation
- HierarchicalRetriever from Component 2
- DiversityMaximizer from Component 3
- ContextAwareEmbeddings from Component 4

**Actions**:
1. Add SpatiotemporalIndex to SelfLearningMemory
2. Add HierarchicalRetriever to SelfLearningMemory
3. Add DiversityMaximizer to SelfLearningMemory
4. Update retrieve_relevant_context to use hierarchical retrieval
5. Apply diversity maximization to results
6. Use context-aware embeddings if available

**Output**:
```rust
pub struct SelfLearningMemory {
    // ... existing fields ...

    // Phase 3 (Spatiotemporal)
    spatiotemporal_index: Option<SpatiotemporalIndex>,
    hierarchical_retriever: Option<HierarchicalRetriever>,
    diversity_maximizer: Option<DiversityMaximizer>,
    context_aware_embeddings: Option<ContextAwareEmbeddings>,
}

impl SelfLearningMemory {
    pub async fn retrieve_relevant_context(
        &self,
        query: String,
        context: TaskContext,
        limit: usize,
    ) -> Result<Vec<Episode>> {
        // Build retrieval query
        let retrieval_query = RetrievalQuery {
            query_text: query,
            query_embedding: None,  // Generated inside retriever
            domain: Some(context.domain.clone()),
            task_type: Some(context.task_type),
            limit: limit * 2,  // Get more for diversity filtering
        };

        // Use hierarchical retrieval if available
        let candidates = if let Some(ref retriever) = self.hierarchical_retriever {
            retriever.retrieve(&retrieval_query, self.storage.as_ref()).await?
        } else {
            // Fallback to flat retrieval
            self.flat_retrieve(&query, limit).await?
        };

        // Apply diversity maximization if available
        let diverse = if let Some(ref maximizer) = self.diversity_maximizer {
            maximizer.maximize_diversity(candidates, limit)
        } else {
            candidates.into_iter().take(limit).collect()
        };

        // Load full episodes
        let episodes = self.load_episodes(diverse).await?;

        Ok(episodes)
    }
}
```

**Success Criteria**:
- Hierarchical retrieval integrated
- Diversity maximization applied
- Context-aware embeddings used
- Backward compatibility (fallback to flat retrieval)
- Performance ≤100ms

**Dependencies**: Tasks 2.2, 3.1, 4.2
**Estimated Time**: 2 hours

---

#### Task 5.2: Update Episode Storage to Update Index
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Medium

**Input**:
- Current complete_episode workflow
- SpatiotemporalIndex

**Actions**:
1. After storing episode, insert into spatiotemporal index
2. On episode eviction, remove from index
3. Ensure atomic updates (index + storage)

**Output**:
```rust
impl SelfLearningMemory {
    pub async fn complete_episode(
        &self,
        episode_id: Uuid,
        outcome: TaskOutcome,
    ) -> Result<()> {
        // ... existing code (quality assessment, summarization, storage) ...

        // Phase 3: Update spatiotemporal index
        if let Some(ref mut index) = self.spatiotemporal_index {
            index.insert_episode(&episode)?;
        }

        Ok(())
    }
}
```

**Success Criteria**:
- Episodes inserted into index on storage
- Episodes removed from index on eviction
- Index stays synchronized with storage

**Dependencies**: Task 1.2, Task 5.1
**Estimated Time**: 1 hour

---

#### Task 5.3: Add Configuration for Phase 3
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: Low

**Input**:
- MemoryConfig struct
- Phase 3 configuration requirements

**Actions**:
1. Add `enable_spatiotemporal_indexing: bool` (default: true)
2. Add `enable_diversity_maximization: bool` (default: true)
3. Add `diversity_lambda: f32` (default: 0.7)
4. Add `temporal_bias_weight: f32` (default: 0.3)
5. Add environment variable support

**Output**:
```rust
pub struct MemoryConfig {
    // ... existing fields ...

    // Phase 3 (Spatiotemporal)
    pub enable_spatiotemporal_indexing: bool,
    pub enable_diversity_maximization: bool,
    pub diversity_lambda: f32,
    pub temporal_bias_weight: f32,
}

// Environment variables:
// MEMORY_ENABLE_SPATIOTEMPORAL=true
// MEMORY_ENABLE_DIVERSITY=true
// MEMORY_DIVERSITY_LAMBDA=0.7
// MEMORY_TEMPORAL_BIAS=0.3
```

**Success Criteria**:
- Configuration fields added
- Environment variables supported
- Defaults set appropriately
- Backward compatibility maintained

**Dependencies**: None
**Estimated Time**: 1 hour

---

### Component 6: Integration Testing (4-5 hours)

#### Task 6.1: Create Hierarchical Retrieval Integration Tests
**Priority**: P0 (Critical)
**Agent**: test-runner
**Complexity**: High

**Actions**:
1. Test end-to-end hierarchical retrieval
2. Test domain filtering
3. Test task-type filtering
4. Test temporal clustering
5. Test query latency ≤100ms
6. Test with large episode collections (1000+)

**Output**: `memory-core/tests/spatiotemporal_integration_test.rs`

**Tests**:
```rust
#[tokio::test]
async fn test_hierarchical_retrieval_by_domain() {
    // Create episodes in different domains
    // Query for specific domain
    // Verify only episodes in that domain returned
}

#[tokio::test]
async fn test_hierarchical_retrieval_by_task_type() {
    // Create episodes of different task types
    // Query for specific task type
    // Verify only matching task type returned
}

#[tokio::test]
async fn test_temporal_bias_recent_episodes_ranked_higher() {
    // Create old and recent episodes
    // Query
    // Verify recent episodes scored higher
}

#[tokio::test]
async fn test_query_latency_under_100ms() {
    // Create 1000+ episodes
    // Measure query time
    // Verify ≤100ms
}
```

**Success Criteria**:
- 10+ integration tests passing
- All retrieval levels validated
- Latency verified
- Large-scale performance tested

**Dependencies**: Task 5.1
**Estimated Time**: 2.5 hours

---

#### Task 6.2: Create Diversity Maximization Integration Tests
**Priority**: P0 (Critical)
**Agent**: test-runner
**Complexity**: Medium

**Actions**:
1. Test MMR diversity with various λ values
2. Test diversity score ≥0.7
3. Test balance between relevance and diversity
4. Compare with and without diversity maximization

**Output**: Integration tests in `spatiotemporal_integration_test.rs`

**Tests**:
```rust
#[tokio::test]
async fn test_diversity_maximization_reduces_redundancy() {
    // Create similar episodes (same task, slight variations)
    // Query without diversity
    // Query with diversity
    // Verify diversity version has higher diversity score
}

#[tokio::test]
async fn test_diversity_score_threshold() {
    // Query with diversity enabled
    // Calculate diversity score
    // Verify ≥0.7
}
```

**Success Criteria**:
- 5+ diversity tests passing
- ≥0.7 diversity validated
- MMR working correctly

**Dependencies**: Task 5.1
**Estimated Time**: 1.5 hours

---

#### Task 6.3: Create Retrieval Accuracy Benchmark
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: High

**Actions**:
1. Create benchmark comparing flat vs hierarchical retrieval
2. Create test set with ground truth relevant episodes
3. Measure precision, recall, F1 score
4. Validate +34% accuracy improvement

**Output**: `benches/spatiotemporal_benchmark.rs`

**Benchmarks**:
```rust
fn baseline_flat_retrieval(c: &mut Criterion) {
    // Measure accuracy of flat retrieval
}

fn hierarchical_retrieval_accuracy(c: &mut Criterion) {
    // Measure accuracy of hierarchical retrieval
}

fn diversity_impact_on_accuracy(c: &mut Criterion) {
    // Measure how diversity affects accuracy
}

fn query_latency_scaling(c: &mut Criterion) {
    // Measure latency with 100, 1000, 10000 episodes
}
```

**Success Criteria**:
- Accuracy improvement ≥34%
- Latency ≤100ms validated
- Diversity score ≥0.7 validated
- Benchmarks reproducible

**Dependencies**: Task 5.1
**Estimated Time**: 2 hours

---

### Component 7: Documentation (2-3 hours)

#### Task 7.1: API Documentation
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: Low

**Actions**:
1. Document all public APIs with examples
2. Add module-level documentation
3. Document configuration options
4. Add examples for common use cases

**Success Criteria**:
- All public APIs documented
- Examples for all major features
- cargo doc builds without warnings

**Dependencies**: All implementation tasks
**Estimated Time**: 1.5 hours

---

#### Task 7.2: Create User Guide
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: Low

**Actions**:
1. Create `docs/SPATIOTEMPORAL_RETRIEVAL.md`
2. Explain hierarchical indexing
3. Explain diversity maximization
4. Provide configuration examples
5. Show performance comparisons

**Output**: `docs/SPATIOTEMPORAL_RETRIEVAL.md`

**Success Criteria**:
- Clear explanation of features
- Configuration examples
- Performance guidance
- Migration guide from flat retrieval

**Dependencies**: All implementation tasks
**Estimated Time**: 1.5 hours

---

## Dependency Graph

```
Task 1.1 (Index Design) ──> Task 1.2 (Index Impl) ──> Task 1.3 (Index Tests)
                                     │
                                     ├──> Task 2.1 (Retriever Design) ──> Task 2.2 (Retriever Impl) ──> Task 2.3 (Retriever Tests)
                                     │                                              │
Task 3.1 (MMR Impl) ──> Task 3.2 (MMR Tests) ────────────────────────────────────┤
                                                                                   │
Task 4.1 (Embed Design) ──> Task 4.2 (Embed Impl) ──> Task 4.3 (Embed Tests) ────┤
                                                                                   │
                                                                                   ├──> Task 5.1 (Integration)
                                                                                   │           │
Task 5.3 (Config) ─────────────────────────────────────────────────────────────┘           │
                                                                                             │
Task 1.2 (Index Impl) ──────────────────────────────────> Task 5.2 (Storage Integration) ──┘
                                                                                             │
                                                                                             ├──> Task 6.1 (Integration Tests)
                                                                                             ├──> Task 6.2 (Diversity Tests)
                                                                                             ├──> Task 6.3 (Accuracy Benchmark)
                                                                                             │
                                                                                             └──> Task 7.1 (API Docs)
                                                                                                  Task 7.2 (User Guide)
```

---

## Execution Strategy

### Phase 3.1: Core Module Implementation (Days 21-24)
**Duration**: 4 days
**Strategy**: PARALLEL implementation of independent modules

**Parallel Track 1 (Indexing)**:
- Task 1.1: Index design
- Task 1.2: Index implementation
- Task 1.3: Index tests

**Parallel Track 2 (Retrieval)**:
- Task 2.1: Retriever design
- Task 2.2: Retriever implementation
- Task 2.3: Retriever tests

**Parallel Track 3 (Diversity)**:
- Task 3.1: MMR implementation
- Task 3.2: MMR tests

**Parallel Track 4 (Embeddings)**:
- Task 4.1: Embedding design
- Task 4.2: Embedding implementation
- Task 4.3: Embedding tests

**Agent Assignment**:
- Agent A (feature-implementer): Track 1 (Indexing)
- Agent B (feature-implementer): Track 2 (Retrieval)
- Agent C (feature-implementer): Track 3 (Diversity)
- Agent D (feature-implementer): Track 4 (Embeddings)

**Quality Gate**: All 4 modules implemented with tests passing

### Phase 3.2: Integration (Days 25-27)
**Duration**: 3 days
**Strategy**: SEQUENTIAL (depends on Phase 3.1 completion)

**Tasks**:
- Task 5.1: Update retrieve_relevant_context
- Task 5.2: Update storage to update index
- Task 5.3: Add configuration
- Task 6.1: Integration tests

**Agent Assignment**:
- Agent A (feature-implementer): Integration
- Agent B (test-runner): Integration tests

**Quality Gate**: End-to-end retrieval working, integration tests passing

### Phase 3.3: Benchmarking and Validation (Days 28-29)
**Duration**: 2 days
**Strategy**: SEQUENTIAL

**Tasks**:
- Task 6.2: Diversity tests
- Task 6.3: Accuracy benchmark

**Agent Assignment**:
- Agent A (feature-implementer): Benchmarks
- Skill: test-runner (validation)

**Quality Gate**: +34% accuracy validated, ≥0.7 diversity validated

### Phase 3.4: Documentation and Final Validation (Day 30)
**Duration**: 1 day
**Strategy**: PARALLEL

**Parallel Track 1**:
- Task 7.1: API documentation
- Task 7.2: User guide

**Parallel Track 2**:
- Skill: rust-code-quality (code review)
- Skill: test-runner (full test suite)

**Quality Gate**: All Phase 3 quality gates passed

---

## Success Criteria

### Functional Requirements
- [ ] Hierarchical indexing working (domain → task_type → temporal)
- [ ] Coarse-to-fine retrieval implemented
- [ ] MMR diversity maximization working (λ=0.7)
- [ ] Context-aware embeddings integrated
- [ ] Episodes automatically indexed on storage
- [ ] Index synchronized with storage (insert/evict)

### Performance Requirements
- [ ] Query latency ≤100ms
- [ ] Retrieval accuracy +34% vs baseline
- [ ] Diversity score ≥0.7
- [ ] Scales sub-linearly with episode count
- [ ] Minimal memory overhead (<10% of episode storage)

### Quality Requirements
- [ ] 40+ unit tests passing (100%)
- [ ] 20+ integration tests passing (100%)
- [ ] Accuracy benchmark validated
- [ ] Zero clippy warnings
- [ ] Full API documentation
- [ ] User guide complete

### Integration Requirements
- [ ] Backward compatibility (fallback to flat retrieval)
- [ ] Configuration via MemoryConfig and environment
- [ ] Logging for retrieval decisions
- [ ] Error handling for all edge cases

---

## Risk Assessment

### High Risk
1. **Retrieval Accuracy**: May not achieve +34% improvement
   - **Mitigation**: Benchmark early, tune parameters (λ, temporal bias)
   - **Validation**: Create test set with ground truth

2. **Query Latency**: Hierarchical search may exceed 100ms
   - **Mitigation**: Optimize cluster sizes, use caching
   - **Validation**: Benchmark with large episode collections

### Medium Risk
3. **Index Synchronization**: Index may diverge from storage
   - **Mitigation**: Atomic operations, periodic consistency checks
   - **Validation**: Integration tests for concurrent operations

4. **Memory Overhead**: Index may consume too much memory
   - **Mitigation**: Compress index, lazy loading
   - **Validation**: Memory profiling with large datasets

### Low Risk
5. **Backward Compatibility**: May break existing retrieval
   - **Mitigation**: Fallback to flat retrieval if index unavailable
   - **Validation**: Test existing retrieval still works

---

## Testing Plan

### Unit Tests (in module code)
- SpatiotemporalIndex: 12 tests
- HierarchicalRetriever: 15 tests
- DiversityMaximizer: 10 tests
- ContextAwareEmbeddings: 8 tests
- **Total**: 45 unit tests

### Integration Tests
- Hierarchical retrieval end-to-end: 10 tests
- Diversity maximization: 5 tests
- Index synchronization: 5 tests
- **Total**: 20 integration tests

### Benchmarks
- Retrieval accuracy (vs baseline)
- Query latency scaling
- Diversity score measurement
- **Total**: 3 benchmarks

---

## Rollback Plan

If Phase 3 fails or performance is unacceptable:

1. **Feature Flag**: Add `enable_spatiotemporal_indexing` flag
2. **Fallback**: Default to flat retrieval if hierarchical fails
3. **Gradual Rollout**: Enable for specific domains first
4. **Performance Tuning**: Adjust cluster sizes, temporal bias, λ parameter

---

## Next Steps

### Immediate (Day 21 - Today)
- [x] Create this integration plan
- [ ] Review and approve plan
- [ ] Create GitHub issue/branch for Phase 3

### Days 21-24 (Phase 3.1)
- [ ] Launch 4 parallel agents for module implementation
- [ ] Implement SpatiotemporalIndex
- [ ] Implement HierarchicalRetriever
- [ ] Implement DiversityMaximizer
- [ ] Implement ContextAwareEmbeddings

### Days 25-30 (Phases 3.2-3.4)
- [ ] Integrate with SelfLearningMemory
- [ ] Run integration tests
- [ ] Create accuracy benchmarks
- [ ] Validate +34% improvement
- [ ] Write documentation
- [ ] Phase 3 completion report

---

## Appendices

### Appendix A: Temporal Clustering Strategy

**Approach**: Time-based bucketing
- Recent episodes (<1 month): Weekly clusters
- Medium-age (1-6 months): Monthly clusters
- Old episodes (>6 months): Quarterly clusters

**Rationale**: More granularity for recent episodes (higher retrieval probability)

### Appendix B: MMR Formula

```
MMR Score(e) = λ * Relevance(e) - (1-λ) * max(Similarity(e, selected_i))

Where:
- λ = diversity weight (default: 0.7)
- Relevance(e) = similarity to query
- Similarity(e, selected_i) = similarity to already-selected episodes
- max(...) = maximum similarity to any selected episode
```

### Appendix C: Diversity Score Calculation

```
Diversity Score = (1/n²) * Σ(i,j) Dissimilarity(e_i, e_j)

Where:
- n = number of episodes in result set
- Dissimilarity(e_i, e_j) = 1 - Similarity(e_i, e_j)
- Target: ≥0.7
```

### Appendix D: Quality Gates

From RESEARCH_INTEGRATION_EXECUTION_PLAN.md Phase 3:

| Quality Gate | Target | Validation |
|--------------|--------|------------|
| Retrieval accuracy | +34% improvement | Accuracy benchmark |
| Diversity score | ≥0.7 (MMR) | Diversity tests |
| Query latency | ≤100ms | Performance benchmark |
| Unit tests | 40+ passing | cargo test |
| Integration tests | 20+ passing | cargo test --test |
| Zero clippy warnings | 0 | cargo clippy |
| Documentation | Complete | All APIs documented |

---

**Document Status**: ✅ COMPLETE
**Next Phase**: Phase 3.1 - Core Module Implementation (Days 21-24)
**Execution Mode**: PARALLEL (4 tracks: Indexing, Retrieval, Diversity, Embeddings)

---

*This integration plan provides a comprehensive roadmap for Phase 3 (Spatiotemporal Memory Organization) with detailed task decomposition, dependency mapping, and success criteria.*
