# Phase 3 Action Plan: Implementation Roadmap

**Date**: 2025-12-26
**Based On**: PHASE3_ANALYSIS_REPORT_2025-12-26.md
**Branch**: `feature/fix-bincode-postcard-migration`
**Target**: Merge-ready with critical blockers resolved

---

## Overview

This action plan provides a step-by-step roadmap to address critical issues identified in Phase 3 analysis. The plan is divided into three phases:

- **Phase 1: Immediate Merge Requirements** (5-7 days) - MUST complete before merge
- **Phase 2: Follow-Up Improvements** (2-4 weeks) - High priority post-merge
- **Phase 3: Production Hardening** (1-3 months) - Production deployment preparation

**Critical Path**: Integrate spatiotemporal index → Add query validation → Expand tests → Merge

**Updated Decision**: Based on 2025 research analysis, SpatiotemporalIndex should be **INTEGRATED** (not removed). See `SPATIOTEMPORAL_INDEX_ANALYSIS.md` for detailed rationale.

**Performance Impact**: 7.5-180× faster retrieval at scale (>1K episodes)

---

## Phase 1: Immediate Merge Requirements (Days 1-5)

**Status**: ❌ NOT STARTED
**Priority**: CRITICAL - Blockers for merge
**Estimated Time**: 3-5 days

### Task 1.1: Integrate SpatiotemporalIndex into Retrieval Pipeline

**Status**: ❌ NOT STARTED
**Priority**: CRITICAL (Performance & Scalability)
**Estimated Time**: 3-5 hours
**Assignee**: feature-implementer
**Analysis**: See `SPATIOTEMPORAL_INDEX_ANALYSIS.md` for detailed research-backed rationale

#### Problem

The `SpatiotemporalIndex` module is fully implemented and tested (1043 lines, 13 tests passing) but is never called during retrieval. Without integration, retrieval has O(n) complexity and scales poorly (>1,000 episodes).

#### Research Justification

Based on 2025 research analysis:
- **REMem (OpenReview 2025)**: Hybrid memory graph with time-aware indexing improves retrieval
- **LiCoMemory (arXiv 2025)**: Hierarchical temporal-aware search shows efficiency gains
- **Nature NPJ Sci Learn (2025)**: Conceptual boundaries (domains/task types) significantly improve temporal order memory
- **Multiple papers**: Nested timescales (weekly/monthly/quarterly) align with human cognition
- **All research**: Hierarchical organization is essential for scalable episodic memory systems

#### Solution: INTEGRATE Index

**Performance Benefits**:
- 100 episodes: Same latency (~0.5ms), more memory-efficient
- 1,000 episodes: **7.5× faster** (0.6ms vs 4.5ms)
- 10,000 episodes: **45× faster** (1.0ms vs 45ms)
- 100,000 episodes: **180× faster** (2.5ms vs 450ms)

#### Implementation Steps

**Step 1: Update `complete_episode()` to Insert Episodes**

```rust
// memory-core/src/memory/mod.rs
impl SelfLearningMemory {
    pub async fn complete_episode(
        &self,
        episode_id: Uuid,
        outcome: TaskOutcome,
    ) -> Result<()> {
        // ... existing code (quality assessment, summarization, storage) ...

        // Phase 3: Update spatiotemporal index
        if let Some(ref mut index) = self.spatiotemporal_index {
            let episode = self.get_episode(&episode_id).await?;
            if let Some(episode) = episode {
                index.write().await.insert_episode(&episode)?;
            }
        }

        Ok(())
    }
}
```

**Step 2: Update `retrieve_relevant_context()` to Query Index**

```rust
// memory-core/src/memory/retrieval.rs
pub async fn retrieve_relevant_context(
    &self,
    task_description: String,
    context: TaskContext,
    limit: usize,
) -> Result<Vec<Episode>> {
    let start_time = std::time::Instant::now();

    // Phase 3: Use spatiotemporal index for efficient candidate selection
    let candidate_ids = if let Some(ref index) = self.spatiotemporal_index {
        let index_read = index.read().await;
        // O(log n) lookup - only return matching episode IDs
        index_read.query(
            Some(&context.domain),
            Some(context.task_type),
            None, // time_range (optional, can add later)
        )
    } else {
        // Fallback: load all episodes (current behavior)
        return self.legacy_retrieve(task_description, context, limit).await;
    };

    // Load ONLY candidate episodes (not all episodes)
    let mut candidate_episodes = Vec::new();
    for episode_id in &candidate_ids {
        if let Some(episode) = self.episodes.get(episode_id) {
            if episode.is_complete() {
                candidate_episodes.push(episode.clone());
            }
        }
    }

    // Use hierarchical retriever on candidates (not all episodes)
    let query = RetrievalQuery {
        query_text: task_description.clone(),
        query_embedding: None,
        domain: Some(context.domain.clone()),
        task_type: Some(context.task_type),
        limit: limit * 2,
    };

    let scored_episodes = match self.hierarchical_retriever.retrieve(&query, &candidate_episodes).await {
        Ok(scored) => scored,
        Err(e) => {
            debug!("Hierarchical retrieval failed: {}, falling back to legacy method", e);
            return self.legacy_retrieve(task_description, context, limit).await;
        }
    };

    // ... rest of existing MMR diversity code ...
}
```

**Step 3: Update Episode Eviction to Remove from Index**

```rust
// memory-core/src/memory/capacity.rs (or wherever eviction happens)
impl SelfLearningMemory {
    async fn evict_episodes(&self, count: usize) -> Result<Vec<Uuid>> {
        // ... existing eviction logic (GENESIS capacity management) ...

        // Remove evicted episodes from spatiotemporal index
        if let Some(ref mut index) = self.spatiotemporal_index {
            let mut index_write = index.write().await;
            for episode_id in &evicted_ids {
                index_write.remove_episode(*episode_id)?;
            }
        }

        Ok(evicted_ids)
    }
}
```

**Step 4: Add Integration Tests**

```rust
// memory-core/tests/spatiotemporal_integration_test.rs
#[tokio::test]
async fn test_index_integration_with_retrieval() {
    // Create memory with spatiotemporal index enabled
    let config = MemoryConfig {
        enable_spatiotemporal_indexing: true,
        ..Default::default()
    };
    let memory = SelfLearningMemory::new(config).unwrap();

    // Create episodes across different domains and task types
    for i in 0..100 {
        let domain = match i % 3 {
            0 => "web-api",
            1 => "database",
            2 => "ml-training",
            _ => unreachable!(),
        };

        let task_type = match i % 2 {
            0 => TaskType::CodeGeneration,
            1 => TaskType::Testing,
            _ => unreachable!(),
        };

        let context = TaskContext {
            domain: domain.to_string(),
            task_type,
            ..Default::default()
        };

        let episode_id = memory
            .start_episode(format!("Episode {}", i), context)
            .await
            .unwrap();

        memory.complete_episode(
            episode_id,
            TaskOutcome::Success {
                result: "Result".to_string(),
                lessons_learned: vec![],
            },
        ).await.unwrap();
    }

    // Verify index was updated
    let index_read = memory.spatiotemporal_index.as_ref().unwrap().read().await;
    assert_eq!(index_read.total_episodes(), 100);

    // Query by domain - should be fast
    let query_context = TaskContext {
        domain: "web-api".to_string(),
        task_type: TaskType::CodeGeneration,
        ..Default::default()
    };

    let start = std::time::Instant::now();
    let retrieved = memory
        .retrieve_relevant_context("Query".to_string(), query_context, 5)
        .await
        .unwrap();
    let latency = start.elapsed().as_millis();

    // Assertions
    assert!(!retrieved.is_empty());
    assert!(retrieved.len() <= 5);
    assert!(latency < 10, "Query should be fast with index (<10ms)");

    // All retrieved episodes should be from "web-api" domain
    for episode in &retrieved {
        assert_eq!(episode.context.domain, "web-api");
    }
}
```

#### Validation

- [ ] Code compiles without errors
- [ ] All tests pass (380 tests)
- [ ] Integration tests pass
- [ ] Index updated on episode completion
- [ ] Index queried during retrieval
- [ ] Index updated on episode eviction
- [ ] Performance benchmark shows improvement (>1K episodes)
- [ ] Memory usage reduced (selective loading)

---

### Task 1.2: Add Query Text Validation

**Status**: ❌ NOT STARTED
**Priority**: CRITICAL (Security)
**Estimated Time**: 1 hour
**Assignee**: feature-implementer

#### Problem

No input validation on `RetrievalQuery.query_text` - potential DoS vector and query injection.

#### Solution

Add length limits and sanitization for query text.

**Implementation**:

```rust
// memory-core/src/types.rs (add constant)
pub const MAX_QUERY_LENGTH: usize = 1000;

// memory-core/src/spatiotemporal/retriever.rs
impl RetrievalQuery {
    pub fn new(
        query_text: String,
        query_embedding: Option<Vec<f32>>,
        domain: Option<String>,
        task_type: Option<TaskType>,
        limit: usize,
    ) -> Result<Self> {
        // Validate query text length
        if query_text.len() > MAX_QUERY_LENGTH {
            return Err(anyhow::anyhow!(
                "Query text exceeds maximum length of {} characters",
                MAX_QUERY_LENGTH
            ));
        }

        // Validate limit
        if limit == 0 || limit > 1000 {
            return Err(anyhow::anyhow!(
                "Limit must be between 1 and 1000, got {}",
                limit
            ));
        }

        Ok(Self {
            query_text,
            query_embedding,
            domain,
            task_type,
            limit,
        })
    }
}

// In memory/src/memory/retrieval.rs:
// Replace direct struct construction with RetrievalQuery::new()
let query = RetrievalQuery::new(
    task_description.clone(),
    None, // query_embedding
    Some(context.domain.clone()),
    None, // task_type
    limit * 2,
)?;
```

**Add Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_validation_rejects_long_queries() {
        let long_query = "x".repeat(1001);
        let result = RetrievalQuery::new(long_query, None, None, None, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_validation_rejects_zero_limit() {
        let result = RetrievalQuery::new("test".to_string(), None, None, None, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_validation_rejects_excessive_limit() {
        let result = RetrievalQuery::new("test".to_string(), None, None, None, 1001);
        assert!(result.is_err());
    }
}
```

#### Validation

- [ ] Query text length validation works
- [ ] Limit validation works
- [ ] Tests pass
- [ ] Error messages are clear
- [ ] No breaking changes to existing tests

---

### Task 1.3: Add Integration Test for Phase 3 Retrieval

**Status**: ❌ NOT STARTED
**Priority**: CRITICAL (Test Coverage)
**Estimated Time**: 2 hours
**Assignee**: feature-implementer

#### Problem

No integration test validates the complete Phase 3 retrieval flow (hierarchical retriever + diversity maximization).

#### Solution

Create comprehensive integration test in `memory-core/tests/phase3_retrieval_integration_test.rs`.

**Implementation**:

```rust
use memory_core::{SelfLearningMemory, MemoryConfig, TaskContext, TaskType, ComplexityLevel, TaskOutcome};
use tokio;

#[tokio::test]
async fn test_phase3_hierarchical_retrieval_workflow() {
    // Setup memory system
    let config = MemoryConfig::default();
    let memory = SelfLearningMemory::new(config).unwrap();

    // Create test episodes across different domains and task types
    let episodes = vec![
        ("web-api", TaskType::CodeGeneration, "Build REST endpoint"),
        ("web-api", TaskType::CodeGeneration, "Create API model"),
        ("database", TaskType::Analysis, "Optimize query performance"),
        ("database", TaskType::CodeGeneration, "Add database migration"),
        ("frontend", TaskType::Testing, "Write unit tests for components"),
    ];

    for (domain, task_type, description) in episodes {
        let context = TaskContext {
            domain: domain.to_string(),
            task_type,
            complexity: ComplexityLevel::Moderate,
            language: Some("rust".to_string()),
            framework: None,
            tags: vec!["test".to_string()],
        };

        let episode_id = memory
            .start_episode(description.to_string(), context.clone())
            .await
            .unwrap();

        // Complete episode
        let outcome = TaskOutcome::Success {
            result: description.to_string(),
            lessons_learned: vec![],
        };

        memory.complete_episode(episode_id, outcome).await.unwrap();
    }

    // Test hierarchical retrieval with domain filtering
    let query_context = TaskContext {
        domain: "web-api".to_string(),
        task_type: TaskType::CodeGeneration,
        complexity: ComplexityLevel::Moderate,
        language: Some("rust".to_string()),
        framework: None,
        tags: vec![],
    };

    let retrieved = memory
        .retrieve_relevant_context("Create API endpoint".to_string(), query_context.clone(), 5)
        .await
        .unwrap();

    // Assertions
    assert!(!retrieved.is_empty(), "Should retrieve relevant episodes");
    assert!(
        retrieved.len() <= 5,
        "Should not exceed requested limit"
    );

    // Verify domain filtering (should only return web-api episodes)
    for episode in &retrieved {
        assert_eq!(
            episode.context.domain, "web-api",
            "All retrieved episodes should be from web-api domain"
        );
    }

    println!("✅ Phase 3 hierarchical retrieval integration test passed");
    println!("Retrieved {} episodes", retrieved.len());
}

#[tokio::test]
async fn test_diversity_maximization_reduces_redundancy() {
    let config = MemoryConfig::default();
    let memory = SelfLearningMemory::new(config).unwrap();

    // Create similar episodes
    for i in 0..10 {
        let context = TaskContext {
            domain: "test".to_string(),
            task_type: TaskType::CodeGeneration,
            complexity: ComplexityLevel::Simple,
            language: Some("rust".to_string()),
            framework: None,
            tags: vec![],
        };

        let episode_id = memory
            .start_episode(format!("Episode {}", i), context.clone())
            .await
            .unwrap();

        let outcome = TaskOutcome::Success {
            result: "Result".to_string(),
            lessons_learned: vec![],
        };

        memory.complete_episode(episode_id, outcome).await.unwrap();
    }

    // Retrieve with diversity enabled
    let query_context = TaskContext {
        domain: "test".to_string(),
        task_type: TaskType::CodeGeneration,
        complexity: ComplexityLevel::Simple,
        language: Some("rust".to_string()),
        framework: None,
        tags: vec![],
    };

    let retrieved = memory
        .retrieve_relevant_context("Test query".to_string(), query_context, 5)
        .await
        .unwrap();

    // Verify diversity (episodes should be varied, not all similar)
    assert!(!retrieved.is_empty());

    // Check for variety in episode IDs (not all the same episode returned)
    let unique_ids: std::collections::HashSet<_> = retrieved
        .iter()
        .map(|e| e.episode_id)
        .collect();

    assert_eq!(
        unique_ids.len(),
        retrieved.len(),
        "All retrieved episodes should have unique IDs (diversity working)"
    );

    println!("✅ Diversity maximization test passed");
}
```

#### Validation

- [ ] Integration tests pass
- [ ] Tests cover hierarchical retrieval flow
- [ ] Tests verify diversity maximization
- [ ] Tests validate domain filtering
- [ ] No performance regressions

---

### Task 1.4: Add Metrics Collection

**Status**: ❌ NOT STARTED
**Priority**: HIGH (Observability)
**Estimated Time**: 2 hours
**Assignee**: feature-implementer

#### Problem

No metrics for fallback usage, diversity scores, or retrieval latency - difficult to detect issues in production.

#### Solution

Add structured logging and metrics tracking for key Phase 3 operations.

**Implementation**:

```rust
// memory-core/src/memory/retrieval.rs
use tracing::{debug, info, instrument, Span};

// Update retrieve_relevant_context with metrics
#[instrument(
    skip(self),
    fields(
        retrieval_method, // "hierarchical" | "legacy_fallback"
        query_latency_ms,
        result_count,
        diversity_score,
        fallback_triggered = false
    )
)]
pub async fn retrieve_relevant_context(
    &self,
    task_description: String,
    context: TaskContext,
    limit: usize,
) -> Result<Vec<Episode>> {
    let start_time = std::time::Instant::now();

    // ... existing code ...

    // Phase 3: Use hierarchical retriever
    let (retrieval_method, scored_episodes) = match self.hierarchical_retriever.retrieve(&query, &completed_episodes).await {
        Ok(scored) => {
            debug!("Hierarchical retrieval succeeded");
            ("hierarchical", scored)
        }
        Err(e) => {
            debug!(error = %e, "Hierarchical retrieval failed, falling back to legacy method");
            // Mark fallback in span
            Span::current().record("fallback_triggered", true);

            // Fallback to legacy retrieval
            let mut relevant: Vec<Episode> = completed_episodes
                .into_iter()
                .filter(|e| self.is_relevant_episode(e, &context, &task_description))
                .collect();

            relevant.sort_by(|a, b| {
                let a_score = self.calculate_relevance_score(a, &context, &task_description);
                let b_score = self.calculate_relevance_score(b, &context, &task_description);
                b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
            });

            relevant.truncate(limit);
            ("legacy_fallback", vec![]) // No scored episodes in legacy mode
        }
    };

    // Apply MMR diversity maximization
    let diverse_scored = if !scored_episodes.is_empty() && retrieval_method == "hierarchical" {
        let diversity_candidates: Vec<crate::spatiotemporal::diversity::ScoredEpisode> = scored_episodes
            .iter()
            .filter_map(|scored| {
                completed_episodes
                    .iter()
                    .find(|e| e.episode_id == scored.episode_id)
                    .map(|episode| {
                        let embedding = generate_simple_embedding(episode);
                        crate::spatiotemporal::diversity::ScoredEpisode::new(
                            episode.episode_id.to_string(),
                            scored.relevance_score,
                            embedding,
                        )
                    })
            })
            .collect();

        let diverse = self.diversity_maximizer.maximize_diversity(diversity_candidates, limit);
        let diversity_score = self.diversity_maximizer.calculate_diversity_score(&diverse);

        Span::current().record("diversity_score", diversity_score);
        info!(diversity_score = diversity_score, "Applied MMR diversity maximization");

        diverse
    } else {
        scored_episodes
            .iter()
            .map(|s| crate::spatiotemporal::diversity::ScoredEpisode::new(
                s.episode_id.to_string(),
                s.relevance_score,
                vec![],
            ))
            .collect()
    };

    // Calculate latency
    let latency_ms = start_time.elapsed().as_millis();
    Span::current().record("query_latency_ms", latency_ms);

    // Extract final episodes
    let result_episodes: Vec<Episode> = diverse_scored
        .iter()
        .filter_map(|scored| {
            let episode_id = uuid::Uuid::parse_str(scored.episode_id()).ok()?;
            completed_episodes
                .iter()
                .find(|e| e.episode_id == episode_id)
                .cloned()
        })
        .collect();

    Span::current().record("result_count", result_episodes.len());
    Span::current().record("retrieval_method", retrieval_method);

    info!(
        retrieval_method = retrieval_method,
        result_count = result_episodes.len(),
        query_latency_ms = latency_ms,
        "Retrieval completed"
    );

    Ok(result_episodes)
}
```

#### Validation

- [ ] Metrics logged for all retrievals
- [ ] Fallback usage tracked
- [ ] Diversity scores logged
- [ ] Query latency measured
- [ ] Spans properly instrumented
- [ ] Logs are queryable (structured JSON format)

---

### Task 1.5: Document Current Limitations

**Status**: ❌ NOT STARTED
**Priority**: HIGH (Transparency)
**Estimated Time**: 1 hour
**Assignee**: feature-implementer

#### Problem

Users and developers need clear documentation of what's implemented vs. what's placeholder.

#### Solution

Add "Current Limitations" section to main README and Phase 3 documentation.

**Documentation**:

```markdown
# Phase 3: Spatiotemporal Memory Organization

## Current Implementation Status

### ✅ Fully Implemented Features

- **Hierarchical Retrieval**: Domain → Task Type → Temporal filtering
- **MMR Diversity Maximization**: Configurable relevance/diversity balance (λ=0.7)
- **Temporal Bias**: Recent episodes weighted higher (0.3 weight)
- **Graceful Fallback**: Legacy retrieval available if hierarchical fails

### ⚠️ Known Limitations (Phase 3.1)

#### Embedding Generation

**Status**: Placeholder Implementation

Current implementation uses simple hash-based embeddings that encode episode metadata:
- Domain hash
- Task type encoding
- Complexity level
- Step count, reward, duration

**Impact**:
- Provides basic hierarchical filtering (domain, task type, temporal)
- Does NOT capture semantic meaning
- Cross-domain learning is limited (different domains get different hashes)

**Future Work** (Phase 3.2):
- Integrate real semantic embeddings (Candle/ONNX)
- Enable true semantic similarity search
- Improve cross-domain episode retrieval

#### Spatiotemporal Index

**Status**: Not Integrated

The `SpatiotemporalIndex` module was implemented but not integrated into the retrieval pipeline to reduce complexity. Current retrieval uses in-memory filtering instead.

**Impact**:
- Slightly higher memory usage (all episodes loaded for filtering)
- Performance adequate for <10k episodes
- Scales to ~100k episodes before optimization needed

**Future Work** (Phase 3.3):
- Integrate SpatiotemporalIndex for O(log n) lookups
- Add index persistence to storage backends
- Optimize for 100k+ episode collections

#### Context-Aware Embeddings

**Status**: Framework Only

The architecture supports task-specific embedding adaptation, but no adapters are currently trained.

**Impact**:
- All episodes use same embedding space
- No task-specific optimization
- Accuracy improvement target (+34%) not yet achieved

**Future Work** (Phase 3.4):
- Train task adapters using contrastive learning
- Implement task-specific embedding spaces
- Validate accuracy improvements

### Configuration Options

All Phase 3 features are configurable via environment variables:

```bash
# Enable/Disable Features
MEMORY_ENABLE_SPATIOTEMPORAL=true    # Default: true
MEMORY_ENABLE_DIVERSITY=true          # Default: true

# Tune Parameters
MEMORY_DIVERSITY_LAMBDA=0.7          # Range: 0.0-1.0 (default: 0.7)
MEMORY_TEMPORAL_BIAS=0.3             # Range: 0.0-1.0 (default: 0.3)
MEMORY_MAX_CLUSTERS=5                 # Default: 5
```

### Performance Characteristics

**Current Performance** (with simple embeddings):
- Query latency: 0.45ms (100 episodes)
- Scales sub-linearly to ~10k episodes
- Degrades linearly beyond 10k episodes

**Expected Performance** (after real embeddings):
- Query latency: ~10-50ms (semantic overhead)
- Improved cross-domain retrieval accuracy
- Scales to 100k+ episodes with index

### Migration Guide for Users

**For Users with <1000 Episodes**:
- Current implementation is sufficient
- Diversity maximization provides immediate value
- No action required

**For Users with >1000 Episodes**:
- Consider enabling simple embeddings (already enabled)
- Monitor query latency
- Plan for real embedding integration (Phase 3.2)

**For Users with >10k Episodes**:
- Current performance may degrade
- Consider disabling temporal bias for faster retrieval
- Plan for SpatiotemporalIndex integration (Phase 3.3)

### Next Steps

See `PHASE3_ACTION_PLAN.md` for detailed roadmap:
- Phase 1: Immediate merge requirements (Days 1-5)
- Phase 2: Follow-up improvements (Weeks 2-4)
- Phase 3: Production hardening (Months 2-3)
```

#### Validation

- [ ] Documentation added to README
- [ ] Phase 3 documentation updated
- [ ] User-facing examples provided
- [ ] Migration guide included
- [ ] Configuration examples clear

---

## Phase 1 Summary

**Total Estimated Time**: 11-13 hours (2-3 days)
**Critical Path**: Task 1.1 → Task 1.2 → Task 1.3 → Task 1.4 → Task 1.5

**Completion Criteria**:
- [ ] SpatiotemporalIndex integrated into retrieval pipeline
- [ ] Query validation added
- [ ] Integration tests passing (including index usage)
- [ ] Metrics collection implemented
- [ ] Documentation updated (including performance characteristics)
- [ ] All 380 tests passing
- [ ] Zero clippy warnings
- [ ] Performance benchmarked with vs without index

**Ready for Merge**: After Phase 1 completion

---

## Phase 2: Follow-Up Improvements (Weeks 2-4)

**Status**: ❌ NOT STARTED
**Priority**: HIGH (Post-merge)
**Estimated Time**: 2-4 weeks

### Task 2.1: Real Embedding Integration

**Status**: ❌ NOT STARTED
**Priority**: HIGH
**Estimated Time**: 2-3 weeks
**Assignee**: feature-implementer

#### Objective

Replace hash-based embeddings with real semantic embeddings using Candle/ONNX.

#### Implementation Plan

**Week 1: Model Selection & Integration**
- Evaluate Candle/ONNX for local embedding generation
- Select pre-trained model (e.g., all-MiniLM-L6-v2)
- Integrate embedding provider into codebase
- Add configuration for model selection

**Week 2: Embedding Generation Pipeline**
- Implement embedding generation for episodes
- Cache embeddings to avoid recomputation
- Update retrieval to use real embeddings
- Benchmark embedding generation latency

**Week 3: Validation & Tuning**
- Compare retrieval accuracy with vs. without embeddings
- Tune similarity thresholds
- Validate cross-domain retrieval improvements
- Document performance characteristics

#### Validation

- [ ] Real embeddings generated successfully
- [ ] Retrieval accuracy improves
- [ ] Cross-domain learning enabled
- [ ] Performance acceptable (<100ms queries)
- [ ] Embeddings cached efficiently

---

### Task 2.2: Add Rate Limiting

**Status**: ❌ NOT STARTED
**Priority**: HIGH
**Estimated Time**: 3-5 days
**Assignee**: feature-implementer

#### Objective

Add per-query rate limiting to prevent DoS and abuse.

#### Implementation

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RateLimiter {
    max_requests_per_minute: usize,
    request_timestamps: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(max_requests_per_minute: usize) -> Self {
        Self {
            max_requests_per_minute,
            request_timestamps: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, client_id: &str) -> Result<(), anyhow::Error> {
        let mut timestamps = self.request_timestamps.write().await;
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        let client_timestamps = timestamps.entry(client_id.to_string()).or_insert_with(Vec::new);

        // Remove timestamps older than 1 minute
        client_timestamps.retain(|&ts| ts > one_minute_ago);

        // Check if limit exceeded
        if client_timestamps.len() >= self.max_requests_per_minute {
            return Err(anyhow::anyhow!(
                "Rate limit exceeded: {} requests per minute",
                self.max_requests_per_minute
            ));
        }

        // Add current timestamp
        client_timestamps.push(now);

        Ok(())
    }
}
```

#### Validation

- [ ] Rate limiting prevents DoS
- [ ] Legitimate requests not blocked
- [ ] Metrics for rate limit hits
- [ ] Configurable limits via environment variables

---

### Task 2.3: Add LRU Cache for Retrieval Queries

**Status**: ❌ NOT STARTED
**Priority**: MEDIUM
**Estimated Time**: 2-3 days
**Assignee**: feature-implementer

#### Objective

Cache retrieval query results to avoid recomputation for repeated queries.

#### Implementation

```rust
use lru::LruCache;
use std::hash::Hash;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[derive(Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    query_text: String,
    domain: Option<String>,
    task_type: Option<TaskType>,
    limit: usize,
}

pub struct RetrievalCache {
    cache: LruCache<CacheKey, Vec<Episode>>,
}

impl RetrievalCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(std::num::NonZeroUsize::new(capacity).unwrap()),
        }
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<&Vec<Episode>> {
        self.cache.get(key)
    }

    pub fn put(&mut self, key: CacheKey, value: Vec<Episode>) {
        self.cache.put(key, value);
    }
}
```

#### Validation

- [ ] Cache hit rate > 20% for repeated queries
- [ ] Cache invalidation works correctly
- [ ] Memory usage acceptable
- [ ] Performance improvement measured

---

### Task 2.4: Parallelize Hierarchical Filtering

**Status**: ❌ NOT STARTED
**Priority**: MEDIUM
**Estimated Time**: 2-3 days
**Assignee**: feature-implementer

#### Objective

Parallelize domain, task type, and temporal filters for faster retrieval.

#### Implementation

```rust
use futures::future::join_all;

async fn parallel_filtering(
    episodes: &[Episode],
    domain_filter: Option<&str>,
    task_type_filter: Option<TaskType>,
    time_filter: Option<(DateTime<Utc>, DateTime<Utc>)>,
) -> Vec<Episode> {
    let tasks = vec![
        // Level 1: Domain filter
        async {
            if let Some(domain) = domain_filter {
                episodes.iter().filter(|e| e.context.domain == domain).cloned().collect()
            } else {
                episodes.to_vec()
            }
        },
        // Level 2: Task type filter
        async {
            if let Some(task_type) = task_type_filter {
                episodes.iter().filter(|e| e.task_type == task_type).cloned().collect()
            } else {
                episodes.to_vec()
            }
        },
        // Level 3: Temporal filter
        async {
            if let Some((start, end)) = time_filter {
                episodes.iter()
                    .filter(|e| e.start_time >= start && e.start_time <= end)
                    .cloned()
                    .collect()
            } else {
                episodes.to_vec()
            }
        },
    ];

    let results = join_all(tasks).await;

    // Intersect results (episodes must pass all filters)
    // Implementation omitted for brevity
    vec![]  // Placeholder
}
```

#### Validation

- [ ] 20-30% speedup for >1000 episodes
- [ ] Results identical to sequential filtering
- [ ] No race conditions
- [ ] Thread-safe operations

---

## Phase 3: Production Hardening (Months 2-3)

**Status**: ❌ NOT STARTED
**Priority**: MEDIUM (Production deployment)
**Estimated Time**: 1-3 months

### Task 3.1: SpatiotemporalIndex Integration

**Status**: ❌ NOT STARTED
**Priority**: HIGH
**Estimated Time**: 2-3 weeks
**Assignee**: feature-implementer

#### Objective

Integrate SpatiotemporalIndex for O(log n) retrieval at scale.

#### Implementation

1. Add index updates in `complete_episode()`
2. Use index in `hierarchical_retriever.retrieve()`
3. Add index persistence to storage backends
4. Implement index rebuild on startup

#### Validation

- [ ] Index automatically updates on episode storage
- [ ] Retrieval uses index efficiently
- [ ] Index persists and restores correctly
- [ ] Performance scales to 100k+ episodes

---

### Task 3.2: Add Circuit Breaker

**Status**: ❌ NOT STARTED
**Priority**: MEDIUM
**Estimated Time**: 1 week
**Assignee**: feature-implementer

#### Objective

Add circuit breaker pattern to prevent cascading failures.

#### Implementation

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct CircuitBreaker {
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
    failure_count: Arc<AtomicUsize>,
    success_count: Arc<AtomicUsize>,
    last_failure_time: Arc<std::sync::RwLock<Option<Instant>>>,
    state: Arc<std::sync::RwLock<CircuitState>>,
}

#[derive(Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        // Check circuit state
        let state = *self.state.read().unwrap();
        match state {
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = *self.last_failure_time.read().unwrap() {
                    if last_failure.elapsed() < self.timeout {
                        return Err(/* Circuit breaker error */);
                    }
                }
                // Move to half-open state
                *self.state.write().unwrap() = CircuitState::HalfOpen;
            }
            _ => {}
        }

        // Execute function
        match f.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(error)
            }
        }
    }

    fn on_success(&self) {
        self.success_count.fetch_add(1, Ordering::SeqCst);

        if self.state.read().unwrap() == &CircuitState::HalfOpen {
            if self.success_count.load(Ordering::SeqCst) >= self.success_threshold {
                *self.state.write().unwrap() = CircuitState::Closed;
                self.failure_count.store(0, Ordering::SeqCst);
                self.success_count.store(0, Ordering::SeqCst);
            }
        }
    }

    fn on_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::SeqCst);

        if self.failure_count.load(Ordering::SeqCst) >= self.failure_threshold {
            *self.state.write().unwrap() = CircuitState::Open;
            *self.last_failure_time.write().unwrap() = Some(Instant::now());
        }
    }
}
```

#### Validation

- [ ] Circuit opens after failure threshold
- [ ] Circuit closes after success threshold
- [ ] No cascading failures
- [ ] Metrics for circuit state

---

### Task 3.3: Add Production Monitoring

**Status**: ❌ NOT STARTED
**Priority**: HIGH
**Estimated Time**: 2-3 weeks
**Assignee**: feature-implementer

#### Objective

Add comprehensive monitoring and alerting for production deployment.

#### Implementation

**Metrics to Track**:
- Retrieval latency (p50, p95, p99)
- Retrieval accuracy (via user feedback)
- Diversity scores
- Fallback usage rate
- Cache hit rates
- Error rates
- Rate limit hits

**Alerting Rules**:
- Fallback usage > 10% → Investigate retriever bugs
- Diversity score < 0.6 → Check lambda parameter
- Query latency p99 > 500ms → Check database issues
- Error rate > 1% → System degradation

#### Validation

- [ ] All metrics collected
- [ ] Alerting rules configured
- [ ] Dashboard created
- [ ] Monitoring tested

---

## Execution Plan

### Week 1: Phase 1 (Critical Path)

**Day 1**:
- [ ] Task 1.1: Remove unused SpatiotemporalIndex
- [ ] Task 1.2: Add query validation

**Day 2**:
- [ ] Task 1.3: Add integration test for Phase 3 retrieval
- [ ] Task 1.4: Add metrics collection

**Day 3**:
- [ ] Task 1.5: Document current limitations
- [ ] Run full test suite (380 tests)

**Day 4-5**:
- [ ] Code review and fixes
- [ ] Final validation
- [ ] **Merge ready**

### Week 2-4: Phase 2 (Post-Merge)

**Week 2**:
- [ ] Task 2.1: Real embedding integration (Week 1)
- [ ] Task 2.2: Add rate limiting

**Week 3**:
- [ ] Task 2.1: Real embedding integration (Week 2-3)
- [ ] Task 2.3: Add LRU cache

**Week 4**:
- [ ] Task 2.4: Parallelize hierarchical filtering
- [ ] Validation and testing

### Months 2-3: Phase 3 (Production)

**Month 2**:
- [ ] Task 3.1: SpatiotemporalIndex integration
- [ ] Task 3.2: Add circuit breaker

**Month 3**:
- [ ] Task 3.3: Add production monitoring
- [ ] Production deployment

---

## Validation Checklist

### Before Merge

- [ ] Unused SpatiotemporalIndex removed
- [ ] Query validation added and tested
- [ ] Integration tests for Phase 3 retrieval
- [ ] Metrics collection implemented
- [ ] Documentation updated
- [ ] All 380 tests passing
- [ ] Zero clippy warnings
- [ ] Code reviewed and approved
- [ ] Performance validated
- [ ] Security review completed

### Post-Merge (Week 2-4)

- [ ] Real embeddings integrated
- [ ] Rate limiting added
- [ ] LRU cache implemented
- [ ] Parallel filtering added
- [ ] Performance benchmarks updated
- [ ] Documentation updated

### Production Ready (Month 3)

- [ ] SpatiotemporalIndex integrated
- [ ] Circuit breaker implemented
- [ ] Production monitoring deployed
- [ ] Alerting configured
- [ ] On-call runbook created
- [ ] Rollback plan tested

---

## Risk Mitigation

### High-Risk Items

**Risk**: Real embeddings don't improve accuracy
**Mitigation**: Benchmark early, compare with baseline
**Fallback**: Keep simple embeddings as option

**Risk**: SpatiotemporalIndex causes bugs
**Mitigation**: Thorough testing, gradual rollout
**Fallback**: Feature flag to disable index

**Risk**: Rate limiting blocks legitimate users
**Mitigation**: Configurable limits, allow-lists
**Fallback**: Monitor and adjust limits dynamically

### Rollback Plan

If any issue arises:

1. **Feature Flags**: Disable specific Phase 3 features via environment variables
2. **Fallback**: Legacy retrieval always available
3. **Gradual Rollout**: Enable for specific domains first
4. **Monitoring**: Watch metrics closely after deployment
5. **Quick Revert**: All Phase 3 changes can be disabled

---

**Action Plan Version**: 1.0
**Last Updated**: 2025-12-26
**Status**: Phase 1 NOT STARTED
**Next Action**: Begin Task 1.1 (Remove unused SpatiotemporalIndex)
