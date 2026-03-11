# Phase 2 Storage Integration Plan (GENESIS)

**Document Version**: 1.0
**Created**: 2025-12-25
**Phase**: Phase 2.3 - Storage Backend Integration (Day 15)
**Dependencies**: Phase 2.1 (CapacityManager), Phase 2.2 (SemanticSummarizer)

---

## Executive Summary

This document provides a comprehensive integration plan for Phase 2 (GENESIS - Capacity-Constrained Episodic Storage). The plan details how to integrate CapacityManager and SemanticSummarizer modules into the Turso and redb storage backends, enabling capacity-constrained storage with semantic summarization.

**Key Objectives**:
1. Add capacity enforcement to storage backends (evict before insert)
2. Store semantic summaries alongside episodes
3. Integrate with SelfLearningMemory workflow
4. Configure capacity limits and eviction policies
5. Ensure performance overhead ≤ 10ms

---

## Context

### Completed Modules

**CapacityManager** (`memory-core/src/episodic/capacity.rs`):
- Relevance-weighted eviction: `relevance_score = (quality * 0.7) + (recency * 0.3)`
- Eviction policies: LRU, RelevanceWeighted
- 19/19 tests passing
- 617 LOC total

**SemanticSummarizer** (`memory-core/src/semantic/summary.rs`):
- Episode compression: 100-200 word summaries
- Extracts 10-20 key concepts, 3-5 critical steps
- Optional embeddings for semantic search
- 18/18 tests passing
- 716 LOC total

### Current Storage Architecture

**Turso (memory-storage-turso)**:
- SQL database (libSQL/SQLite)
- Primary durable storage
- Tables: episodes, patterns, heuristics, embeddings, metadata

**redb (memory-storage-redb)**:
- Embedded key-value store
- Fast cache layer
- Tables: EPISODES_TABLE, PATTERNS_TABLE, HEURISTICS_TABLE, EMBEDDINGS_TABLE, METADATA_TABLE

**Episode Storage**:
- Episodes serialized with postcard (Phase 1 migration)
- Episode struct includes `salient_features: Option<SalientFeatures>` (Phase 1)
- No current capacity limits or eviction

---

## Integration Goals

### Goal 1: Capacity Enforcement API
Add capacity-limited episode storage to both backends with configurable eviction.

**Success Criteria**:
- `store_episode_with_capacity()` method in both backends
- Atomic eviction before insertion
- Configurable max_episodes per backend
- Eviction policy selection (LRU vs RelevanceWeighted)

### Goal 2: Semantic Summary Storage
Store and retrieve episode summaries efficiently.

**Success Criteria**:
- `store_episode_summary()` method in both backends
- `get_episode_summary()` retrieval
- Efficient storage (compressed summaries)
- Optional embedding storage

### Goal 3: SelfLearningMemory Integration
Update complete_episode workflow to use capacity-constrained storage.

**Success Criteria**:
- Capacity enforcement during episode completion
- Semantic summarization before storage
- Backward compatibility with existing episodes
- Configuration for capacity limits

### Goal 4: Performance
Maintain low overhead for capacity operations.

**Success Criteria**:
- Capacity check overhead ≤ 5ms
- Eviction overhead ≤ 10ms (when needed)
- Summary generation ≤ 20ms
- Total overhead ≤ 10ms average

---

## Task Decomposition

### Component 1: Database Schema Design (2-3 hours)

#### Task 1.1: Design Turso Schema for Episode Summaries
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Medium

**Input**:
- EpisodeSummary struct definition
- Existing Turso schema (episodes, patterns, etc.)

**Actions**:
1. Design `episode_summaries` table schema
2. Add indexes for efficient retrieval (episode_id, created_at)
3. Plan migration script for schema update
4. Document schema design

**Output**:
- SQL CREATE TABLE statement
- Index definitions
- Migration plan

**Success Criteria**:
- Schema supports all EpisodeSummary fields
- Efficient queries by episode_id
- Embedding storage (BLOB for Vec<f32>)
- Foreign key to episodes table

**SQL Schema**:
```sql
CREATE TABLE IF NOT EXISTS episode_summaries (
    episode_id TEXT PRIMARY KEY,
    summary_text TEXT NOT NULL,
    key_concepts TEXT NOT NULL,  -- JSON array
    key_steps TEXT NOT NULL,     -- JSON array
    summary_embedding BLOB,      -- Optional Vec<f32>
    created_at INTEGER NOT NULL,
    FOREIGN KEY (episode_id) REFERENCES episodes(episode_id) ON DELETE CASCADE
);

CREATE INDEX idx_summaries_created_at ON episode_summaries(created_at);
```

**Dependencies**: None
**Estimated Time**: 1 hour

---

#### Task 1.2: Design redb Table for Summaries
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Low

**Input**:
- EpisodeSummary struct
- Existing redb table definitions

**Actions**:
1. Define SUMMARIES_TABLE constant
2. Plan table initialization in RedbStorage::new()
3. Document storage format (postcard serialization)

**Output**:
```rust
const SUMMARIES_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("summaries");
```

**Success Criteria**:
- Table definition added to lib.rs
- Compatible with existing redb architecture
- Postcard serialization for EpisodeSummary

**Dependencies**: None
**Estimated Time**: 30 minutes

---

#### Task 1.3: Design Capacity Metadata Storage
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Medium

**Input**:
- CapacityManager requirements
- Storage backend requirements

**Actions**:
1. Design metadata for tracking episode count
2. Plan storage of eviction policy configuration
3. Design metadata for oldest/newest episode tracking

**Output**:
- Metadata schema design
- Cache invalidation strategy

**Turso Metadata**:
```sql
-- Add to metadata table
INSERT INTO metadata (key, value) VALUES
    ('max_episodes', '10000'),
    ('eviction_policy', 'RelevanceWeighted'),
    ('current_episode_count', '0');
```

**redb Metadata**:
- Use existing METADATA_TABLE
- Store capacity config as JSON

**Success Criteria**:
- Efficient count tracking (no full table scans)
- Atomic count updates with episode operations
- Configuration persistence

**Dependencies**: Tasks 1.1, 1.2
**Estimated Time**: 1 hour

---

### Component 2: Turso Storage Implementation (4-6 hours)

#### Task 2.1: Implement Episode Summary Storage (Turso)
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Medium

**Input**:
- EpisodeSummary struct
- Turso schema from Task 1.1
- Existing TursoStorage implementation

**Actions**:
1. Implement `store_episode_summary()` method
2. Implement `get_episode_summary()` method
3. Handle embedding serialization (Vec<f32> to BLOB)
4. Add error handling for summary operations

**Output**:
```rust
impl TursoStorage {
    pub async fn store_episode_summary(&self, summary: &EpisodeSummary) -> Result<()> {
        // Implementation
    }

    pub async fn get_episode_summary(&self, episode_id: Uuid) -> Result<Option<EpisodeSummary>> {
        // Implementation
    }
}
```

**Success Criteria**:
- Summaries stored atomically with episodes
- Efficient retrieval by episode_id
- Embedding vector serialization correct
- Proper error handling

**Dependencies**: Task 1.1
**Estimated Time**: 2 hours

---

#### Task 2.2: Implement Capacity Enforcement (Turso)
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- CapacityManager module
- Turso storage implementation
- Episode and summary storage methods

**Actions**:
1. Implement `store_episode_with_capacity()` method
2. Add capacity check before insert
3. Implement eviction logic (call CapacityManager)
4. Ensure atomic evict-then-insert transaction
5. Update episode count metadata

**Output**:
```rust
impl TursoStorage {
    pub async fn store_episode_with_capacity(
        &self,
        episode: &Episode,
        summary: Option<&EpisodeSummary>,
        capacity_manager: &CapacityManager,
    ) -> Result<Option<Vec<Uuid>>> {
        // 1. Get current count from metadata
        // 2. If at capacity, select episodes to evict
        // 3. Begin transaction
        // 4. Delete evicted episodes + summaries
        // 5. Insert new episode + summary
        // 6. Update count metadata
        // 7. Commit transaction
        // 8. Return evicted episode IDs
    }
}
```

**Success Criteria**:
- Atomic operation (transaction-based)
- Eviction before insertion
- Correct episode count maintained
- Evicted episode IDs returned
- Both episode and summary evicted together

**Dependencies**: Tasks 1.1, 1.3, 2.1
**Estimated Time**: 3 hours

---

#### Task 2.3: Implement Batch Eviction (Turso)
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: Medium

**Input**:
- CapacityManager eviction logic
- Turso database connection

**Actions**:
1. Implement efficient batch DELETE for episodes
2. Cascade delete summaries (via foreign key)
3. Update metadata in same transaction

**Output**:
```rust
async fn batch_evict_episodes(&self, episode_ids: &[Uuid]) -> Result<()> {
    // DELETE FROM episodes WHERE episode_id IN (...)
    // DELETE FROM episode_summaries WHERE episode_id IN (...)
    // (or rely on CASCADE)
}
```

**Success Criteria**:
- Efficient batch deletion (single query)
- Summaries deleted via CASCADE
- Metadata updated atomically

**Dependencies**: Task 2.2
**Estimated Time**: 1 hour

---

### Component 3: redb Storage Implementation (3-4 hours)

#### Task 3.1: Implement Episode Summary Storage (redb)
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Medium

**Input**:
- EpisodeSummary struct
- redb table definition from Task 1.2
- Existing RedbStorage implementation

**Actions**:
1. Implement `store_episode_summary()` using spawn_blocking
2. Implement `get_episode_summary()` using spawn_blocking
3. Use postcard serialization for summaries
4. Add to RedbStorage struct

**Output**:
```rust
impl RedbStorage {
    pub async fn store_episode_summary(&self, summary: &EpisodeSummary) -> Result<()> {
        // spawn_blocking for redb write
    }

    pub async fn get_episode_summary(&self, episode_id: Uuid) -> Result<Option<EpisodeSummary>> {
        // spawn_blocking for redb read
    }
}
```

**Success Criteria**:
- Async-safe (spawn_blocking for all redb operations)
- Postcard serialization working
- Error handling for serialization failures

**Dependencies**: Task 1.2
**Estimated Time**: 1.5 hours

---

#### Task 3.2: Implement Capacity Enforcement (redb)
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- CapacityManager module
- redb storage implementation
- Episode and summary storage methods

**Actions**:
1. Implement `store_episode_with_capacity()` method
2. Add capacity check (count episodes in table)
3. Implement eviction logic
4. Ensure single write transaction for evict+insert
5. Update metadata count

**Output**:
```rust
impl RedbStorage {
    pub async fn store_episode_with_capacity(
        &self,
        episode: &Episode,
        summary: Option<&EpisodeSummary>,
        capacity_manager: &CapacityManager,
    ) -> Result<Option<Vec<Uuid>>> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || {
            let write_txn = db.begin_write()?;
            // 1. Count episodes
            // 2. If at capacity, select episodes to evict
            // 3. Delete evicted episodes + summaries
            // 4. Insert new episode + summary
            // 5. Update metadata
            // 6. Commit transaction
            Ok(evicted_ids)
        }).await??
    }
}
```

**Success Criteria**:
- Single write transaction (atomic)
- Eviction before insertion
- Correct count maintained in metadata
- spawn_blocking for all redb operations

**Dependencies**: Tasks 1.2, 1.3, 3.1
**Estimated Time**: 2.5 hours

---

### Component 4: SelfLearningMemory Integration (3-4 hours)

#### Task 4.1: Add CapacityManager to SelfLearningMemory
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Medium

**Input**:
- SelfLearningMemory struct
- CapacityManager module
- MemoryConfig

**Actions**:
1. Add `capacity_manager: Option<CapacityManager>` field
2. Initialize in constructors (new, with_storage, default)
3. Add configuration options (max_episodes, eviction_policy)

**Output**:
```rust
pub struct SelfLearningMemory {
    // ... existing fields ...
    capacity_manager: Option<CapacityManager>,
    semantic_summarizer: Option<SemanticSummarizer>,
}
```

**Success Criteria**:
- Backward compatibility (Option types)
- Configuration from MemoryConfig
- Default: no capacity limits (None)

**Dependencies**: None
**Estimated Time**: 1 hour

---

#### Task 4.2: Add SemanticSummarizer to SelfLearningMemory
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Low

**Input**:
- SelfLearningMemory struct
- SemanticSummarizer module

**Actions**:
1. Add `semantic_summarizer: Option<SemanticSummarizer>` field
2. Initialize in constructors
3. Configure summary length parameters

**Success Criteria**:
- Summarizer available in complete_episode
- Configurable summary parameters
- Default: summarization enabled

**Dependencies**: None
**Estimated Time**: 30 minutes

---

#### Task 4.3: Update complete_episode Workflow
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: High

**Input**:
- Existing complete_episode implementation
- CapacityManager, SemanticSummarizer
- Updated storage backend APIs

**Actions**:
1. After quality assessment, generate semantic summary
2. Before storage, check capacity and evict if needed
3. Store episode with summary using new API
4. Log evicted episode IDs
5. Ensure backward compatibility (capacity optional)

**Output**:
```rust
impl SelfLearningMemory {
    pub async fn complete_episode(
        &self,
        episode_id: Uuid,
        outcome: TaskOutcome,
    ) -> Result<()> {
        // ... existing code (quality assessment) ...

        // Generate semantic summary (Phase 2)
        let summary = if let Some(ref summarizer) = self.semantic_summarizer {
            Some(summarizer.summarize(&episode))
        } else {
            None
        };

        // Store with capacity enforcement (Phase 2)
        if let Some(ref capacity_mgr) = self.capacity_manager {
            let evicted = storage.store_episode_with_capacity(
                &episode,
                summary.as_ref(),
                capacity_mgr,
            ).await?;

            if let Some(evicted_ids) = evicted {
                info!("Evicted {} episodes due to capacity", evicted_ids.len());
            }
        } else {
            // Fallback: store without capacity enforcement
            storage.store_episode(&episode).await?;
            if let Some(summary) = summary {
                storage.store_episode_summary(&summary).await?;
            }
        }

        // ... rest of existing code ...
    }
}
```

**Success Criteria**:
- Summarization integrated seamlessly
- Capacity enforcement when configured
- Backward compatibility without capacity
- Eviction logging
- Performance overhead ≤ 10ms average

**Dependencies**: Tasks 2.2, 3.2, 4.1, 4.2
**Estimated Time**: 2 hours

---

### Component 5: Configuration (1-2 hours)

#### Task 5.1: Update MemoryConfig
**Priority**: P0 (Critical)
**Agent**: feature-implementer
**Complexity**: Low

**Input**:
- Existing MemoryConfig struct
- Capacity and summary requirements

**Actions**:
1. Add `max_episodes: Option<usize>` field
2. Add `eviction_policy: Option<EvictionPolicy>` field
3. Add `enable_summarization: bool` field (default: true)
4. Add `summary_min_length: usize` field (default: 100)
5. Add `summary_max_length: usize` field (default: 200)

**Output**:
```rust
pub struct MemoryConfig {
    // ... existing fields ...

    // Capacity management (Phase 2)
    pub max_episodes: Option<usize>,
    pub eviction_policy: Option<EvictionPolicy>,

    // Semantic summarization (Phase 2)
    pub enable_summarization: bool,
    pub summary_min_length: usize,
    pub summary_max_length: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            max_episodes: None,  // No limit by default
            eviction_policy: Some(EvictionPolicy::RelevanceWeighted),
            enable_summarization: true,
            summary_min_length: 100,
            summary_max_length: 200,
        }
    }
}
```

**Success Criteria**:
- Configuration options available
- Sensible defaults
- Backward compatibility (None = no limits)

**Dependencies**: None
**Estimated Time**: 30 minutes

---

#### Task 5.2: Add Environment Variable Support
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: Low

**Input**:
- MemoryConfig
- Existing environment variable handling

**Actions**:
1. Add MEMORY_MAX_EPISODES environment variable
2. Add MEMORY_EVICTION_POLICY environment variable
3. Add MEMORY_ENABLE_SUMMARIZATION environment variable
4. Update MemoryConfig::from_env()

**Output**:
```bash
# Environment variables
MEMORY_MAX_EPISODES=10000
MEMORY_EVICTION_POLICY=RelevanceWeighted  # or LRU
MEMORY_ENABLE_SUMMARIZATION=true
```

**Success Criteria**:
- Environment variables override config
- Proper parsing and validation
- Error messages for invalid values

**Dependencies**: Task 5.1
**Estimated Time**: 1 hour

---

### Component 6: Integration Testing (4-6 hours)

#### Task 6.1: Capacity Enforcement Integration Tests (Turso)
**Priority**: P0 (Critical)
**Agent**: test-runner
**Complexity**: High

**Input**:
- Implemented capacity enforcement
- Test utilities

**Actions**:
1. Test: Store episodes up to capacity (no eviction)
2. Test: Eviction triggered at capacity
3. Test: Correct episodes evicted (LRU vs RelevanceWeighted)
4. Test: Episode count accurate after eviction
5. Test: Summaries evicted with episodes

**Output**: `memory-storage-turso/tests/capacity_enforcement_test.rs`

**Tests**:
```rust
#[tokio::test]
async fn test_capacity_enforcement_lru() {
    // Fill to capacity with LRU policy
    // Insert new episode
    // Verify oldest episode evicted
}

#[tokio::test]
async fn test_capacity_enforcement_relevance_weighted() {
    // Fill to capacity with RelevanceWeighted policy
    // Insert high-quality episode
    // Verify low-quality/old episode evicted
}

#[tokio::test]
async fn test_summary_cascades_with_episode() {
    // Store episode with summary
    // Evict episode
    // Verify summary also deleted
}

#[tokio::test]
async fn test_capacity_count_accuracy() {
    // Perform multiple insert/evict cycles
    // Verify episode count always correct
}

#[tokio::test]
async fn test_batch_eviction() {
    // Fill to capacity
    // Insert 10 new episodes
    // Verify batch eviction of 10 old episodes
}
```

**Success Criteria**:
- 5+ integration tests
- All eviction scenarios covered
- Both eviction policies tested
- Summary deletion verified

**Dependencies**: Task 2.2
**Estimated Time**: 2 hours

---

#### Task 6.2: Capacity Enforcement Integration Tests (redb)
**Priority**: P0 (Critical)
**Agent**: test-runner
**Complexity**: High

**Input**:
- Implemented capacity enforcement
- Test utilities

**Actions**:
1. Test: Store episodes up to capacity (no eviction)
2. Test: Eviction triggered at capacity
3. Test: Atomic transaction (evict + insert)
4. Test: Episode count accurate
5. Test: Summaries stored and retrieved

**Output**: `memory-storage-redb/tests/capacity_enforcement_test.rs`

**Tests** (same as Task 6.1 but for redb):
```rust
#[tokio::test]
async fn test_capacity_enforcement_redb_lru() { }

#[tokio::test]
async fn test_capacity_enforcement_redb_relevance() { }

#[tokio::test]
async fn test_redb_transaction_atomicity() { }

#[tokio::test]
async fn test_redb_summary_storage() { }
```

**Success Criteria**:
- 5+ integration tests
- Atomic transaction verified
- spawn_blocking working correctly

**Dependencies**: Task 3.2
**Estimated Time**: 2 hours

---

#### Task 6.3: End-to-End SelfLearningMemory Tests
**Priority**: P0 (Critical)
**Agent**: test-runner
**Complexity**: High

**Input**:
- Complete SelfLearningMemory integration
- Both storage backends

**Actions**:
1. Test: Complete episode with summarization
2. Test: Complete episode with capacity enforcement
3. Test: Eviction during complete_episode
4. Test: Retrieve episode summary
5. Test: Performance overhead ≤ 10ms

**Output**: `memory-core/tests/genesis_integration_test.rs`

**Tests**:
```rust
#[tokio::test]
async fn test_complete_episode_with_summary() {
    // Complete episode with summarization enabled
    // Verify summary stored
    // Verify summary retrievable
}

#[tokio::test]
async fn test_complete_episode_with_capacity() {
    // Configure max_episodes = 10
    // Complete 15 episodes
    // Verify only 10 stored
    // Verify correct episodes evicted
}

#[tokio::test]
async fn test_eviction_during_completion() {
    // Fill to capacity
    // Complete new high-quality episode
    // Verify low-quality episode evicted
}

#[tokio::test]
async fn test_capacity_performance_overhead() {
    // Measure time with and without capacity enforcement
    // Verify overhead ≤ 10ms average
}

#[tokio::test]
async fn test_backward_compatibility_no_capacity() {
    // Create memory without capacity limits
    // Complete episodes
    // Verify no eviction occurs
}
```

**Success Criteria**:
- 5+ end-to-end tests
- Full workflow tested
- Performance validated
- Backward compatibility verified

**Dependencies**: Task 4.3
**Estimated Time**: 2 hours

---

### Component 7: Documentation (2-3 hours)

#### Task 7.1: API Documentation
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: Low

**Actions**:
1. Document all new storage methods
2. Add examples to rustdoc
3. Document configuration options

**Success Criteria**:
- All public APIs documented
- Examples for common use cases
- cargo doc builds without warnings

**Dependencies**: Tasks 2.2, 3.2, 4.3
**Estimated Time**: 1 hour

---

#### Task 7.2: User Guide for Capacity Management
**Priority**: P1 (Important)
**Agent**: feature-implementer
**Complexity**: Low

**Actions**:
1. Create docs/CAPACITY_MANAGEMENT.md
2. Explain eviction policies
3. Provide configuration examples
4. Document performance characteristics

**Output**: `docs/CAPACITY_MANAGEMENT.md`

**Success Criteria**:
- Clear explanation of capacity enforcement
- Configuration examples
- Performance guidance

**Dependencies**: Task 4.3
**Estimated Time**: 1.5 hours

---

## Dependency Graph

```
Task 1.1 (Turso Schema) ──┬──> Task 2.1 (Summary Storage Turso) ──> Task 2.2 (Capacity Turso)
Task 1.2 (redb Schema) ───┼──> Task 3.1 (Summary Storage redb) ───> Task 3.2 (Capacity redb)
Task 1.3 (Metadata) ──────┘

Task 2.2 ──> Task 2.3 (Batch Eviction Turso)

Task 4.1 (Add CapacityManager) ─┐
Task 4.2 (Add Summarizer) ──────┼──> Task 4.3 (Update complete_episode)
Task 2.2 (Capacity Turso) ──────┤
Task 3.2 (Capacity redb) ───────┘

Task 5.1 (MemoryConfig) ──> Task 5.2 (Environment Variables)

Task 2.2 ──> Task 6.1 (Turso Integration Tests)
Task 3.2 ──> Task 6.2 (redb Integration Tests)
Task 4.3 ──> Task 6.3 (End-to-End Tests)

Task 4.3 ──> Task 7.1 (API Documentation)
Task 4.3 ──> Task 7.2 (User Guide)
```

---

## Execution Strategy

### Phase 2.3: Integration Planning (Current - Day 15)
**Duration**: 1 day
**Strategy**: Sequential analysis and planning
- ✅ COMPLETE: This planning document

### Phase 2.4: Storage Backend Implementation (Days 16-17)
**Duration**: 2 days
**Strategy**: PARALLEL implementation across backends

**Parallel Track 1 (Turso)**:
- Task 1.1: Turso schema design
- Task 2.1: Summary storage
- Task 2.2: Capacity enforcement
- Task 2.3: Batch eviction
- Task 6.1: Integration tests

**Parallel Track 2 (redb)**:
- Task 1.2: redb schema design
- Task 3.1: Summary storage
- Task 3.2: Capacity enforcement
- Task 6.2: Integration tests

**Parallel Track 3 (Configuration)**:
- Task 1.3: Metadata design
- Task 5.1: MemoryConfig updates
- Task 5.2: Environment variables

**Agent Assignment**:
- Agent A (feature-implementer): Turso implementation (Tasks 1.1, 2.1, 2.2, 2.3)
- Agent B (feature-implementer): redb implementation (Tasks 1.2, 3.1, 3.2)
- Agent C (feature-implementer): Configuration (Tasks 1.3, 5.1, 5.2)

**Quality Gate**: All storage backend tests passing (10+ tests)

### Phase 2.5: SelfLearningMemory Integration (Days 18-19)
**Duration**: 2 days
**Strategy**: SEQUENTIAL (depends on Phase 2.4 completion)

**Tasks**:
- Task 4.1: Add CapacityManager
- Task 4.2: Add SemanticSummarizer
- Task 4.3: Update complete_episode workflow
- Task 6.3: End-to-end integration tests

**Agent Assignment**:
- Agent A (feature-implementer): SelfLearningMemory updates
- Agent B (test-runner): End-to-end testing

**Quality Gate**: End-to-end tests passing, performance ≤ 10ms overhead

### Phase 2.6: Documentation and Validation (Day 20)
**Duration**: 1 day
**Strategy**: PARALLEL documentation + validation

**Parallel Track 1**:
- Task 7.1: API documentation
- Task 7.2: User guide

**Parallel Track 2**:
- Run full test suite
- Performance benchmarking
- Quality review

**Agent Assignment**:
- Agent A: Documentation
- Skill: rust-code-quality (code review)
- Skill: test-runner (full validation)

**Quality Gate**: All Phase 2 quality gates passed

---

## Success Criteria

### Functional Requirements
- [ ] Capacity enforcement working in both storage backends
- [ ] Semantic summaries stored and retrievable
- [ ] Eviction policies (LRU, RelevanceWeighted) working correctly
- [ ] Episode count accurate after all operations
- [ ] Summaries cascade-deleted with episodes

### Performance Requirements
- [ ] Capacity check overhead ≤ 5ms
- [ ] Eviction overhead ≤ 10ms (when triggered)
- [ ] Summary generation ≤ 20ms
- [ ] Total overhead ≤ 10ms average (across all episodes)

### Quality Requirements
- [ ] 20+ integration tests passing (100%)
- [ ] Zero clippy warnings
- [ ] Full API documentation
- [ ] User guide complete

### Integration Requirements
- [ ] Backward compatibility (no capacity = unlimited)
- [ ] Configuration via MemoryConfig and environment
- [ ] Logging for eviction events
- [ ] Error handling for all edge cases

---

## Risk Assessment

### High Risk
1. **Transaction Atomicity**: Eviction and insertion must be atomic
   - **Mitigation**: Use database transactions (Turso, redb)
   - **Validation**: Test concurrent operations

2. **Performance Overhead**: Capacity checks on every insert
   - **Mitigation**: Efficient counting (metadata, not table scans)
   - **Validation**: Performance benchmarks

### Medium Risk
3. **Eviction Policy Bugs**: Incorrect episode selection
   - **Mitigation**: Comprehensive test cases for both policies
   - **Validation**: Manual inspection of eviction decisions

4. **Summary Embedding Size**: Large embeddings increase storage
   - **Mitigation**: Make embeddings optional
   - **Validation**: Monitor storage usage

### Low Risk
5. **Configuration Complexity**: Many new config options
   - **Mitigation**: Sensible defaults, clear documentation
   - **Validation**: User guide with examples

---

## Testing Plan

### Unit Tests (in module code)
- CapacityManager logic (already 19/19 passing)
- SemanticSummarizer logic (already 18/18 passing)
- New storage methods (embedded in implementation)

### Integration Tests (in tests/ directories)
- **Turso**: 5+ tests for capacity enforcement
- **redb**: 5+ tests for capacity enforcement
- **SelfLearningMemory**: 5+ end-to-end tests

### Performance Tests
- Benchmark capacity check time
- Benchmark eviction time
- Benchmark summary generation time
- Validate total overhead ≤ 10ms

### Edge Case Tests
- Capacity = 0 (reject all episodes)
- Capacity = 1 (constant eviction)
- Empty database eviction
- Concurrent operations
- Eviction during retrieval

---

## Rollback Plan

If integration fails or performance is unacceptable:

1. **Revert to Phase 1**: Disable capacity enforcement
2. **Feature Flag**: Add feature flag for GENESIS integration
3. **Gradual Rollout**: Enable capacity for specific domains first
4. **Performance Tuning**: Optimize eviction algorithm if needed

---

## Next Steps

### Immediate (Day 15 - Today)
- [x] Create this integration plan
- [ ] Review and approve plan
- [ ] Create GitHub issue/branch for Phase 2.4

### Tomorrow (Day 16)
- [ ] Launch 3 parallel agents for storage implementation
- [ ] Begin Turso schema updates
- [ ] Begin redb implementation
- [ ] Begin configuration updates

### Days 17-20
- [ ] Complete storage backend implementation
- [ ] Integrate with SelfLearningMemory
- [ ] Run comprehensive tests
- [ ] Write documentation
- [ ] Phase 2 completion report

---

## Appendices

### Appendix A: Storage Schema Summary

**Turso Tables**:
- `episodes` (existing, no schema changes needed)
- `episode_summaries` (NEW)
- `metadata` (existing, new entries for capacity config)

**redb Tables**:
- `EPISODES_TABLE` (existing)
- `SUMMARIES_TABLE` (NEW)
- `METADATA_TABLE` (existing, capacity config)

### Appendix B: Configuration Example

```rust
use memory_core::{SelfLearningMemory, MemoryConfig, EvictionPolicy};

let config = MemoryConfig {
    max_episodes: Some(10000),
    eviction_policy: Some(EvictionPolicy::RelevanceWeighted),
    enable_summarization: true,
    summary_min_length: 100,
    summary_max_length: 200,
    ..Default::default()
};

let memory = SelfLearningMemory::with_config(config);
```

### Appendix C: Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Capacity check | ≤ 5ms | Metadata query time |
| Eviction (single) | ≤ 10ms | DELETE query time |
| Eviction (batch 10) | ≤ 15ms | Batch DELETE time |
| Summary generation | ≤ 20ms | SemanticSummarizer.summarize() |
| **Total overhead** | **≤ 10ms** | **Average across all episodes** |

### Appendix D: Quality Gates

From RESEARCH_INTEGRATION_EXECUTION_PLAN.md Phase 2:

| Quality Gate | Target | Validation |
|--------------|--------|------------|
| Capacity enforcement | 100% accurate | Integration tests |
| Eviction accuracy | Correct episodes evicted | Test both policies |
| Storage compression | 3.2x vs raw | Measure summary size |
| Retrieval speed | +65% faster | Benchmark queries |
| Unit tests | 20+ passing | cargo test |
| Integration tests | 100% pass | End-to-end tests |
| Zero clippy warnings | 0 | cargo clippy |
| Documentation | Complete | All APIs documented |

---

**Document Status**: ✅ COMPLETE
**Next Phase**: Phase 2.4 - Storage Backend Implementation (Days 16-17)
**Execution Mode**: PARALLEL (3 tracks: Turso, redb, config)

---

*This integration plan provides a comprehensive roadmap for Phase 2 (GENESIS) storage integration with detailed task decomposition, dependency mapping, and success criteria.*
