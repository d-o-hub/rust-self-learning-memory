# Self-Learning Memory - v0.1.8 Planning

**Target Date**: Q1 2026 (January - February)
**Status**: PLANNING
**Priority**: HIGH
**Focus**: Research-based enhancements (PREMem, GENESIS, Spatiotemporal)

---

## Overview

v0.1.8 is a research-driven release focusing on integrating three groundbreaking academic papers into the memory system. This sprint implements proven techniques from EMNLP 2025 and arXiv 2025 to dramatically improve memory quality, storage efficiency, and retrieval accuracy.

**Research-Based Implementation**:
- **PREMem**: Pre-storage reasoning for memory quality enhancement
- **GENESIS**: Capacity-constrained episodic encoding with semantic summarization
- **Spatiotemporal Memory**: Enhanced RAG retrieval through spatiotemporal organization

**Expected Overall Impact**: 40-50% improvement across all key metrics

---

## 🎯 Phase 1: PREMem Implementation (Weeks 1-2)

**Paper**: "PREMem: Pre-Storage Reasoning for Episodic Memory Enhancement", EMNLP 2025

**Effort**: 60-75 hours
**Expected Impact**: +23% memory quality, 42% noise reduction, 2-3x storage efficiency

### Architecture

```rust
// Pre-storage quality assessment
memory-core/src/pre_storage/
├── mod.rs           // Module coordination
├── quality.rs       // QualityAssessor
└── extractor.rs     // SalientExtractor
```

### Components

#### 1. QualityAssessor

**Location**: `memory-core/src/pre_storage/quality.rs`
**Effort**: 15-20 hours

**Responsibility**: Evaluate episode quality before storage

**Quality Features**:
- Task complexity analysis (depth, branching, multiple tool types)
- Step diversity (unique tools, varied approaches)
- Error rate and recovery patterns
- Reflection depth (insight quality)
- Pattern novelty (new vs. recurring)

**Quality Scoring**:
```rust
pub struct QualityScore {
    pub total: f32,        // 0.0 - 1.0
    pub task_complexity: f32,
    pub step_diversity: f32,
    pub error_recovery: f32,
    pub reflection_depth: f32,
    pub pattern_novelty: f32,
}

impl QualityScore {
    pub fn overall(&self) -> f32 {
        // Weighted combination of features
        0.3 * self.task_complexity +
        0.2 * self.step_diversity +
        0.15 * self.error_recovery +
        0.15 * self.reflection_depth +
        0.2 * self.pattern_novelty
    }
}
```

**Default Quality Threshold**: 0.7 (configurable)

#### 2. SalientExtractor

**Location**: `memory-core/src/pre_storage/extractor.rs`
**Effort**: 15-20 hours

**Responsibility**: Extract key information for semantic summarization

**Extracted Features**:
- Critical decision points (branch conditions, tool choices)
- Tool combinations (sequential and parallel usage patterns)
- Error recovery strategies (successful error handling approaches)
- Key insights (high-value reflections)
- Task-specific context (domain, language, patterns)

**Extraction Strategy**:
```rust
pub struct SalientFeatures {
    pub decisions: Vec<DecisionPoint>,
    pub tool_combinations: Vec<ToolCombination>,
    pub error_recoveries: Vec<ErrorRecovery>,
    pub key_insights: Vec<String>,
    pub task_context: TaskContext,
}
```

#### 3. Storage Decision Integration

**Location**: Modify `memory-core/src/memory/mod.rs`
**Effort**: 10-15 hours

**Integration Point**: `SelfLearningMemory::complete_episode()`

**Decision Logic**:
```rust
impl SelfLearningMemory {
    pub async fn complete_episode(&self, episode_id: Uuid, outcome: TaskOutcome, reward: RewardScore) -> Result<()> {
        // ... existing reflection generation ...

        // NEW: Pre-storage quality assessment
        let quality_score = self.quality_assessor.evaluate(&episode, &outcome, &reflection)?;

        if quality_score.overall() < self.config.quality_threshold {
            tracing::warn!("Episode quality below threshold ({:.2}): {}", quality_score.overall(), quality_score);
            return Ok(()); // Reject low-quality episodes
        }

        // Extract salient features for GENESIS summarization
        let salient = self.salient_extractor.extract(&episode, &outcome)?;

        // Store with quality score and salient features
        self.storage.store_episode_with_quality(episode, quality_score, salient).await?;

        Ok(())
    }
}
```

#### 4. Quality Metrics

**Location**: Add to `memory-mcp/src/tools/analyze_patterns.rs`
**Effort**: 5-10 hours

**Metrics**:
- Memory quality score over time
- Noise reduction rate (episodes filtered / total)
- Quality score distribution
- Feature importance weights

**MCP Tool Enhancement**:
```json
{
  "name": "analyze_patterns",
  "new_parameters": {
    "include_quality_metrics": true,
    "quality_threshold": 0.7
  }
}
```

### Success Criteria

- [ ] Quality assessment accuracy ≥ 80% (known good/bad episodes)
- [ ] Noise reduction ≥ 30% (target: 42%)
- [ ] Memory quality score improvement ≥ 20% (target: +23%)
- [ ] All unit tests passing
- [ ] Integration tests validated
- [ ] Performance impact ≤ 10% (pre-storage reasoning overhead)

---

## 🎯 Phase 2: GENESIS Integration (Weeks 3-4)

**Paper**: "GENESIS: A Generative Model of Episodic-Semantic Interaction", arXiv Oct 2025

**Effort**: 60-80 hours
**Expected Impact**: 3.2x storage compression, 65% faster access, 95% reconstruction accuracy

### Architecture

```rust
// Capacity-constrained episodic encoding
memory-core/src/
├── episodic/
│   └── capacity.rs      // CapacityManager
└── semantic/
    └── summary.rs       // SemanticSummarizer

memory-storage-turso/src/capacity.rs
memory-storage-redb/src/capacity.rs
```

### Components

#### 1. CapacityManager

**Location**: `memory-core/src/episodic/capacity.rs`
**Effort**: 15-20 hours

**Responsibility**: Manage episode storage with capacity constraints

**Configuration**:
```rust
pub struct CapacityConfig {
    pub max_episodes: usize,              // Episode limit (e.g., 10,000)
    pub eviction_policy: EvictionPolicy,    // LRU or RelevanceWeighted
    pub storage_compression_ratio: f32,     // Target compression (3.2x)
}

pub enum EvictionPolicy {
    LRU,                          // Least Recently Used
    RelevanceWeighted,             // Weight by quality score + recency
}
```

**Eviction Logic** (RelevanceWeighted):
```rust
pub fn compute_relevance(&self, episode: &StoredEpisode) -> f32 {
    let quality = episode.quality_score;
    let age_days = (Utc::now() - episode.stored_at).num_days() as f32;
    let age_factor = 1.0 / (1.0 + age_days / 30.0); // Decay over months

    quality * age_factor
}
```

#### 2. SemanticSummarizer

**Location**: `memory-core/src/semantic/summary.rs`
**Effort**: 10-15 hours

**Responsibility**: Generate compressed semantic summaries

**Summarization Strategy**:
- Use existing embeddings for semantic understanding
- Extract key concepts using simple NLP (keyword extraction)
- Generate 100-200 word summaries
- Summarize decisions, tool combinations, insights

**Summary Structure**:
```rust
pub struct EpisodeSummary {
    pub summary: String,               // 100-200 words
    pub key_decisions: Vec<String>,    // Critical decision points
    pub tool_combinations: Vec<String>, // Patterns found
    pub key_insights: Vec<String>,     // High-value insights
    pub summary_embedding: Vec<f32>,   // Embedding of summary
}
```

#### 3. Storage Backend Capacity Enforcement

**Locations**:
- `memory-storage-turso/src/capacity.rs` (10-15 hours)
- `memory-storage-redb/src/capacity.rs` (10-15 hours)

**Method**: `enforce_capacity_limit()`

**Implementation**:
```rust
#[async_trait]
impl StorageBackend for TursoStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        // Before storing, check capacity
        self.enforce_capacity_limit().await?;

        // Store episode
        // ...

        Ok(())
    }

    async fn enforce_capacity_limit(&self) -> Result<()> {
        let current_count = self.count_episodes().await?;

        if current_count > self.config.max_episodes {
            // Query episodes by relevance (quality + recency)
            let candidates = self.query_episodes_by_relevance(
                current_count - self.config.max_episodes
            ).await?;

            // Delete low-relevance episodes
            for episode_id in candidates {
                self.delete_episode(&episode_id).await?;
                tracing::info!("Evicted episode {} due to capacity limit", episode_id);
            }
        }

        Ok(())
    }
}
```

#### 4. SelfLearningMemory Integration

**Location**: Modify `memory-core/src/memory/mod.rs`
**Effort**: 5-10 hours

**Integration**:
```rust
impl SelfLearningMemory {
    pub fn new(config: MemoryConfig) -> Result<Self> {
        // Initialize CapacityManager
        let capacity_manager = CapacityManager::new(config.capacity)?;

        Ok(Self {
            // ... existing fields ...
            capacity_manager,
        })
    }

    pub async fn complete_episode(&self, episode_id: Uuid, outcome: TaskOutcome, reward: RewardScore) -> Result<()> {
        // ... quality assessment and extraction ...

        // Generate semantic summary
        let summary = self.semantic_summarizer.summarize(&episode, &salient)?;

        // Store with summary
        let stored_episode = StoredEpisode {
            episode,
            quality_score,
            salient,
            summary,
        };

        self.storage.store_episode(&stored_episode).await?;

        // Capacity management handled by storage backend
        Ok(())
    }
}
```

### Success Criteria

- [ ] Storage compression ≥ 2x (target: 3.2x)
- [ ] Access speed improvement ≥ 50% (target: 65%)
- [ ] Reconstruction accuracy ≥ 90% (future enhancement)
- [ ] Capacity eviction policy correctness validated
- [ ] All unit tests passing
- [ ] Integration tests validated

---

## 🎯 Phase 3: Spatiotemporal Memory Organization (Weeks 5-6)

**Paper**: "Enhancing RAG with Episodic Memory and Generative Semantic Embeddings", arXiv Nov 2025

**Effort**: 55-65 hours
**Expected Impact**: +34% RAG retrieval accuracy, +25% semantic relevance, 43% faster retrieval

### Architecture

```rust
// Spatiotemporal retrieval
memory-core/src/retrieval/
├── spatiotemporal.rs     // SpatiotemporalIndex
├── hierarchical.rs       // HierarchicalRetriever
└── diversity.rs         // Maximal Marginal Relevance

memory-core/src/embeddings/
└── contextual.rs        // ContextualEmbeddingProvider
```

### Components

#### 1. SpatiotemporalIndex

**Location**: `memory-core/src/retrieval/spatiotemporal.rs`
**Effort**: 15-20 hours

**Responsibility**: Multi-dimensional indexing for efficient retrieval

**Index Structure**:
```rust
pub struct SpatiotemporalIndex {
    pub time_index: BTreeMap<i64, Vec<Uuid>>,      // Time → Episode IDs
    pub domain_index: HashMap<String, Vec<Uuid>>,    // Domain → Episode IDs
    pub task_type_index: HashMap<String, Vec<Uuid>>,  // Task Type → Episode IDs
    pub embedding_index: HashMap<Uuid, Vec<f32>>,     // Episode ID → Embedding
}
```

**Retrieval Logic**:
```rust
impl SpatiotemporalIndex {
    pub fn retrieve_candidates(
        &self,
        query_embedding: &[f32],
        time_range: Option<(i64, i64)>,
        domain: Option<&str>,
        task_type: Option<&str>
    ) -> Vec<Uuid> {
        let mut candidates = self.embedding_index.keys().cloned().collect::<Vec<_>>();

        // Filter by time
        if let Some((start, end)) = time_range {
            candidates.retain(|id| {
                if let Some(timestamps) = self.time_index.range(start..=end).next() {
                    timestamps.1.contains(id)
                } else {
                    false
                }
            });
        }

        // Filter by domain
        if let Some(d) = domain {
            candidates.retain(|id| {
                self.domain_index.get(d).map_or(false, |eps| eps.contains(id))
            });
        }

        // Filter by task type
        if let Some(tt) = task_type {
            candidates.retain(|id| {
                self.task_type_index.get(tt).map_or(false, |eps| eps.contains(id))
            });
        }

        // Re-rank by embedding similarity
        candidates.sort_by(|a, b| {
            let sim_a = cosine_similarity(query_embedding, self.embedding_index.get(a).unwrap());
            let sim_b = cosine_similarity(query_embedding, self.embedding_index.get(b).unwrap());
            sim_b.partial_cmp(&sim_a).unwrap()
        });

        candidates.truncate(100); // Top 100 candidates
        candidates
    }
}
```

#### 2. HierarchicalRetriever

**Location**: `memory-core/src/retrieval/hierarchical.rs`
**Effort**: 10-15 hours

**Responsibility**: Three-level retrieval for efficient and accurate results

**Retrieval Levels**:
```rust
impl HierarchicalRetriever {
    pub async fn retrieve(
        &self,
        query: &str,
        context: &TaskContext,
        limit: usize
    ) -> Result<Vec<Episode>> {
        // Level 1: Semantic search (embeddings)
        let query_embedding = self.embedding_provider.embed(query).await?;
        let semantic_candidates = self.spatiotemporal.retrieve_candidates(
            &query_embedding,
            None,
            Some(&context.domain),
            Some(&context.task_type)
        );

        // Level 2: Spatiotemporal filter (time, domain, task)
        let time_range = context.time_range();
        let spatiotemporal_filtered = self.apply_time_filter(
            semantic_candidates,
            time_range
        );

        // Level 3: Detailed episode retrieval from storage
        let episodes = self.storage.retrieve_by_ids(&spatiotemporal_filtered).await?;

        // Apply diversity maximization
        self.diversity_maximizer.maximize_diversity(episodes, limit)
    }
}
```

#### 3. DiversityMaximization

**Location**: `memory-core/src/retrieval/diversity.rs`
**Effort**: 10-10 hours

**Responsibility**: Maximize relevance while minimizing similarity

**Algorithm**: Maximal Marginal Relevance (MMR)

```rust
pub struct DiversityMaximizer;

impl DiversityMaximizer {
    pub fn maximize_diversity(&self, episodes: Vec<Episode>, limit: usize) -> Vec<Episode> {
        if episodes.len() <= limit {
            return episodes;
        }

        let mut selected = Vec::new();
        let mut remaining = episodes;

        // Select most relevant first
        selected.push(remaining.remove(0));

        // Greedy selection for diversity
        while selected.len() < limit && !remaining.is_empty() {
            let mut best_idx = 0;
            let mut best_score = -f32::INFINITY;

            for (i, episode) in remaining.iter().enumerate() {
                let relevance = self.compute_relevance(episode);
                let diversity = self.compute_diversity(episode, &selected);

                // MMR score: λ * relevance - (1-λ) * similarity
                let mmr_score = 0.7 * relevance - 0.3 * diversity;

                if mmr_score > best_score {
                    best_score = mmr_score;
                    best_idx = i;
                }
            }

            selected.push(remaining.remove(best_idx));
        }

        selected
    }

    fn compute_diversity(&self, episode: &Episode, selected: &[Episode]) -> f32 {
        // Maximum similarity to already selected episodes
        selected.iter()
            .map(|ep| cosine_similarity(&episode.embedding, &ep.embedding))
            .fold(0.0f32, |acc, sim| acc.max(sim))
    }
}
```

#### 4. ContextualEmbeddingProvider

**Location**: `memory-core/src/embeddings/contextual.rs`
**Effort**: 10-15 hours

**Responsibility**: Domain and task-type-specific embedding shifts

**Contextual Projection**:
```rust
pub struct ContextualEmbeddingProvider {
    base_provider: Box<dyn EmbeddingProvider>,
    domain_shifts: HashMap<String, Vec<f32>>,      // Domain-specific shifts
    task_type_shifts: HashMap<String, Vec<f32>>,   // Task-type-specific shifts
}

impl ContextualEmbeddingProvider {
    pub async fn embed(&self, text: &str, context: &TaskContext) -> Result<Vec<f32>> {
        let base_embedding = self.base_provider.embed(text).await?;

        // Apply domain shift
        let domain_shift = self.domain_shifts
            .get(&context.domain)
            .unwrap_or(&vec![0.0; 384]);
        let mut contextual = add_vectors(&base_embedding, domain_shift);

        // Apply task-type shift
        let task_shift = self.task_type_shifts
            .get(&context.task_type)
            .unwrap_or(&vec![0.0; 384]);
        contextual = add_vectors(&contextual, task_shift);

        Ok(contextual)
    }
}
```

#### 5. MCP Tool Updates

**Location**: Update `memory-mcp/src/tools/query_memory.rs`
**Effort**: 5-10 hours

**New Parameters**:
```json
{
  "name": "query_memory",
  "new_parameters": {
    "time_range": {
      "type": "object",
      "description": "Optional time range filter",
      "properties": {
        "start": { "type": "integer" },
        "end": { "type": "integer" }
      }
    },
    "domain": { "type": "string" },
    "task_type": { "type": "string" },
    "maximize_diversity": { "type": "boolean", "default": true }
  }
}
```

### Success Criteria

- [ ] Retrieval accuracy improvement ≥ 30% (target: +34%)
- [ ] Semantic relevance improvement ≥ 20% (target: +25%)
- [ ] Retrieval latency improvement ≥ 40% (target: 43%)
- [ ] Diversity score improvement ≥ 50%
- [ ] All unit tests passing
- [ ] Integration tests validated

---

## 📊 Phase 4: Benchmark Evaluation (Week 7)

**Effort**: 20-30 hours
**Expected Impact**: Comprehensive performance baseline and validation

### Benchmark Creation

**Files**:
- `benches/premem_benchmark.rs` (3-5 hours)
- `benches/genesis_benchmark.rs` (3-5 hours)
- `benches/spatiotemporal_benchmark.rs` (3-5 hours)

**Baselines**:
- Current system performance (control)
- Expected performance improvements (treatment)

### Performance Measurement

**Metrics** (5-10 hours):
- Memory quality improvement (baseline vs. PREMem)
- Storage efficiency (baseline vs. GENESIS)
- Retrieval accuracy (baseline vs. spatiotemporal)
- Latency improvements (access time, retrieval time)
- Comparison against paper expectations

### Research Integration Report (5-5 hours)

**Sections**:
1. Implementation summary (what was built)
2. Performance results (actual vs. expected)
3. Deviations and explanations (why results differ)
4. Recommendations (future improvements)
5. Code location and usage examples

### Documentation Updates (0-0 hours)

- Update `PERFORMANCE_BASELINES.md`
- Update architecture documentation
- Update user guides with new features
- Create research integration summary in plans/

### Success Criteria

- [ ] All benchmarks passing
- [ ] Performance baselines documented
- [ ] Research integration report generated
- [ ] Documentation updated with results
- [ ] All quality gates passing

---

## Implementation Timeline

| Week | Focus | Hours | Deliverables |
|-------|--------|--------|--------------|
| **Week 1** | PREMem - QualityAssessor | 15-20 | Quality assessment module |
| **Week 2** | PREMem - Integration | 45-55 | Full PREMem integration |
| **Week 3** | GENESIS - CapacityManager | 30-35 | Capacity management |
| **Week 4** | GENESIS - Integration | 30-45 | Full GENESIS integration |
| **Week 5** | Spatiotemporal - Index | 25-35 | Spatiotemporal indexing |
| **Week 6** | Spatiotemporal - Integration | 30-35 | Full spatiotemporal retrieval |
| **Week 7** | Benchmarks & Reporting | 20-30 | Validation complete |

**Total Effort**: 175-220 hours

---

## Cross-References

- **Version History**: See [ROADMAP_VERSION_HISTORY.md](ROADMAP_VERSION_HISTORY.md)
- **Current Status**: See [ROADMAP_V017_CURRENT.md](ROADMAP_V017_CURRENT.md)
- **Active Work**: See [ROADMAP_ACTIVE.md](ROADMAP_ACTIVE.md)
- **Research Paper**: See [plans/research/EPISODIC_MEMORY_RESEARCH_2025.md](plans/research/EPISODIC_MEMORY_RESEARCH_2025.md)
- **Implementation Plan**: See [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md)

---

*Last Updated: 2025-12-20*
*Status: Planning Phase Complete*
*Next: Implementation Start*
