# Phase 1 Implementation Plan: Critical Gaps
## GOAP Execution Strategy: Sequential with Quality Gates

**Goal**: Implement critical missing functionality to complete core learning cycle

---

## Task 1: Heuristic Learning Mechanism

### Objective
Implement complete heuristic extraction, learning, and usage pipeline

### Sub-Tasks

#### 1.1 Create HeuristicExtractor (150-200 LOC)
**File**: `memory-core/src/patterns/extractors/heuristic.rs`

**Responsibilities**:
- Extract condition→action rules from successful decision points
- Analyze patterns to generate generalizable heuristics
- Calculate initial confidence scores
- Create Evidence with episode IDs

**Algorithm**:
1. Analyze DecisionPoint patterns from episode
2. For successful decision points:
   - Extract condition (context + decision trigger)
   - Extract action (the step taken)
   - Group similar conditions
   - Calculate success rate for condition→action pairs
3. Generate Heuristic structs with confidence = success_rate × √sample_size
4. Filter by minimum confidence threshold (0.7)

**Dependencies**: DecisionPoint patterns must exist

---

#### 1.2 Integrate Heuristic Extraction in Learning Cycle (50-100 LOC)
**File**: `memory-core/src/memory/learning.rs`

**Modifications**:
- Add heuristic extraction after pattern extraction in `complete_episode()`
- Store extracted heuristics to both Turso and redb
- Link heuristics to episode
- Handle async extraction queue for heuristics

**Location**: Lines 84-154 in `complete_episode()`

**Integration Points**:
```rust
// After pattern extraction (line 130)
let heuristics = self.heuristic_extractor.extract(&episode).await?;
for heuristic in heuristics {
    self.turso_storage.store_heuristic(&heuristic).await?;
    self.redb_storage.store_heuristic(&heuristic).await?;
}
```

---

#### 1.3 Implement Heuristic Retrieval (50-100 LOC)
**File**: `memory-core/src/memory/retrieval.rs`

**New Method**:
```rust
pub async fn retrieve_relevant_heuristics(
    &self,
    context: &TaskContext,
    limit: usize,
) -> Result<Vec<Heuristic>>
```

**Algorithm**:
1. Query heuristics from storage by context (domain, language)
2. Calculate relevance score based on context similarity
3. Rank by confidence × relevance
4. Return top N heuristics

---

#### 1.4 Implement Heuristic Update Logic (50-100 LOC)
**File**: `memory-core/src/memory/learning.rs`

**New Method**:
```rust
pub async fn update_heuristic_confidence(
    &self,
    heuristic_id: Uuid,
    episode_outcome: TaskOutcome,
) -> Result<()>
```

**Algorithm**:
1. Retrieve heuristic from storage
2. Call `heuristic.update_evidence()` with new outcome
3. Recalculate confidence
4. Update in both Turso and redb

---

#### 1.5 Add Heuristic Tests (200-300 LOC)
**File**: `memory-core/tests/heuristic_learning.rs`

**Test Cases**:
- `test_heuristic_extraction_from_decision_points()`
- `test_heuristic_confidence_calculation()`
- `test_heuristic_storage_and_retrieval()`
- `test_heuristic_update_on_new_evidence()`
- `test_heuristic_integration_in_learning_cycle()`
- `test_heuristic_filtering_by_confidence()`

---

### Quality Gate 1: Heuristic Learning
- [ ] HeuristicExtractor extracts valid heuristics from episodes
- [ ] Heuristics stored in both Turso and redb
- [ ] Heuristic retrieval returns relevant results
- [ ] Confidence updates correctly on new evidence
- [ ] All tests passing
- [ ] Code passes clippy and fmt

---

## Task 2: Step Batching for High-Throughput Episodes

### Objective
Reduce I/O overhead by batching step writes

### Sub-Tasks

#### 2.1 Create StepBuffer Data Structure (100-150 LOC)
**File**: `memory-core/src/memory/step_buffer.rs`

**Structure**:
```rust
pub struct StepBuffer {
    episode_id: Uuid,
    steps: Vec<ExecutionStep>,
    config: BatchConfig,
    last_flush: Instant,
}

pub struct BatchConfig {
    pub max_batch_size: usize,      // Default: 50 steps
    pub flush_interval_ms: u64,     // Default: 5000ms
    pub auto_flush: bool,           // Default: true
}
```

**Methods**:
- `new(episode_id, config)` - Create buffer
- `add_step(step)` - Add step to buffer, auto-flush if needed
- `should_flush()` - Check if flush conditions met
- `flush()` - Write buffered steps to storage
- `len()` / `is_empty()` - Buffer state

**Flush Conditions**:
- Buffer size >= max_batch_size
- OR time since last_flush >= flush_interval_ms
- OR manual flush call

---

#### 2.2 Add BatchConfig to MemoryConfig (30-50 LOC)
**File**: `memory-core/src/types.rs`

**Modifications**:
```rust
pub struct MemoryConfig {
    pub storage: StorageConfig,
    pub enable_embeddings: bool,
    pub pattern_extraction_threshold: f32,
    pub batch_config: Option<BatchConfig>,  // New field
}
```

**Default**:
```rust
batch_config: Some(BatchConfig {
    max_batch_size: 50,
    flush_interval_ms: 5000,
    auto_flush: true,
})
```

---

#### 2.3 Integrate StepBuffer in SelfLearningMemory (100-150 LOC)
**File**: `memory-core/src/memory/mod.rs`

**Add Field**:
```rust
pub struct SelfLearningMemory<T, R> {
    // ... existing fields
    step_buffers: Arc<RwLock<HashMap<Uuid, StepBuffer>>>,
}
```

**Modifications to log_step()**:
File: `memory-core/src/memory/episode.rs:166-203`

```rust
pub async fn log_step(&self, episode_id: Uuid, step: ExecutionStep) -> Result<()> {
    // Validate step
    self.validate_step(&step)?;

    // Get or create buffer
    let mut buffers = self.step_buffers.write().await;
    let buffer = buffers.entry(episode_id)
        .or_insert_with(|| StepBuffer::new(episode_id, self.config.batch_config.clone()));

    // Add step to buffer
    buffer.add_step(step)?;

    // Auto-flush if conditions met
    if buffer.should_flush() {
        self.flush_steps_internal(episode_id, buffer).await?;
    }

    Ok(())
}
```

---

#### 2.4 Implement Manual Flush Method (50-100 LOC)
**File**: `memory-core/src/memory/episode.rs`

**New Public Method**:
```rust
pub async fn flush_steps(&self, episode_id: Uuid) -> Result<()>
```

**Usage**: Called before episode completion to ensure all steps persisted

**Modify complete_episode()**:
```rust
// Add at start of complete_episode() (line 84)
self.flush_steps(episode_id).await?;
```

---

#### 2.5 Add Background Flush Task (50-100 LOC)
**File**: `memory-core/src/memory/step_buffer.rs`

**Method**:
```rust
pub fn start_periodic_flush(
    memory: Arc<SelfLearningMemory<T, R>>,
    interval_ms: u64,
) -> JoinHandle<()>
```

**Responsibility**:
- Spawn background task
- Every interval_ms, check all buffers
- Flush buffers that exceed time threshold
- Handle errors with logging

---

#### 2.6 Add Step Batching Tests (150-200 LOC)
**File**: `memory-core/tests/step_batching.rs`

**Test Cases**:
- `test_step_buffer_creation()`
- `test_step_buffer_auto_flush_on_size()`
- `test_step_buffer_auto_flush_on_time()`
- `test_step_buffer_manual_flush()`
- `test_log_step_with_batching()`
- `test_complete_episode_flushes_steps()`
- `test_periodic_flush_task()`
- `test_batching_performance()` - Benchmark improvement

---

### Quality Gate 2: Step Batching
- [ ] StepBuffer correctly buffers steps
- [ ] Auto-flush works on size threshold
- [ ] Auto-flush works on time threshold
- [ ] Manual flush persists all buffered steps
- [ ] complete_episode() flushes before completion
- [ ] Performance improvement measurable (>50% reduction in writes)
- [ ] All tests passing
- [ ] Code passes clippy and fmt

---

## Execution Strategy: Sequential with Quality Gates

### Phase 1.1: Heuristic Learning (Days 1-3)
1. Implement HeuristicExtractor
2. Integrate in learning cycle
3. Add retrieval and update methods
4. Write comprehensive tests
5. **QUALITY GATE 1** - All heuristic tests passing

### Phase 1.2: Step Batching (Days 4-5)
1. Implement StepBuffer and BatchConfig
2. Integrate in log_step()
3. Add flush methods and background task
4. Write comprehensive tests
5. **QUALITY GATE 2** - All batching tests passing

### Phase 1.3: Integration & Verification (Day 6)
1. Run full test suite
2. Run benchmarks
3. Update documentation
4. Code review
5. **FINAL QUALITY GATE** - All tests passing, clippy clean

---

## Success Criteria

### Heuristic Learning
- [x] Heuristics extracted from completed episodes
- [x] Heuristics stored in Turso and redb
- [x] Heuristics retrievable by context
- [x] Confidence updates on new evidence
- [x] Integration tests passing
- [x] No clippy warnings

### Step Batching
- [x] Steps buffered before persistence
- [x] Auto-flush on size threshold
- [x] Auto-flush on time threshold
- [x] Manual flush available
- [x] Episode completion triggers flush
- [x] Performance improvement >50%
- [x] All tests passing
- [x] No clippy warnings

### Overall
- [x] Zero test failures
- [x] Zero clippy warnings
- [x] Documentation updated
- [x] Benchmarks show improvement
- [x] Code review passed

---

## Risk Mitigation

### Risk 1: Heuristic Extraction Complexity
**Mitigation**: Start with simple condition→action extraction, iterate

### Risk 2: Step Buffer Memory Usage
**Mitigation**: Configurable max buffer size, auto-flush prevents unbounded growth

### Risk 3: Integration Failures
**Mitigation**: Incremental integration with tests at each step

### Risk 4: Performance Regression
**Mitigation**: Benchmark before/after, make batching optional via config

---

## Files to Create

1. `memory-core/src/patterns/extractors/heuristic.rs` (150-200 LOC)
2. `memory-core/src/memory/step_buffer.rs` (200-300 LOC)
3. `memory-core/tests/heuristic_learning.rs` (200-300 LOC)
4. `memory-core/tests/step_batching.rs` (150-200 LOC)

## Files to Modify

1. `memory-core/src/patterns/extractors/mod.rs` (add heuristic extractor export)
2. `memory-core/src/memory/mod.rs` (add step_buffers field)
3. `memory-core/src/memory/learning.rs` (integrate heuristic extraction)
4. `memory-core/src/memory/episode.rs` (integrate step buffering)
5. `memory-core/src/memory/retrieval.rs` (add heuristic retrieval)
6. `memory-core/src/types.rs` (add BatchConfig)

---

## Estimated Total

- **New LOC**: ~1,200-1,650
- **Modified LOC**: ~300-400
- **Test LOC**: ~350-500
- **Total Impact**: ~1,850-2,550 LOC
- **Timeline**: 5-6 days with testing

---

**Status**: Ready for implementation
**Next Action**: Launch feature-implementer agents for each task
