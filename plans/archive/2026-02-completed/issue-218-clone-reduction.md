# Issue #218: Reduce Clone Operations from 298 to <200

## Target
Reduce clone operations from current count to less than 200 across all production code files.

## Current State Analysis

| Metric | Value |
|--------|-------|
| Total Clone Operations | 509 |
| Target Clone Operations | <200 |
| Reduction Required | ~309 (61%) |

## Clone Distribution by Module

| Module | Current Clones | Target | Reduction Needed |
|--------|---------------|--------|------------------|
| memory-core | 213 | <85 | 128 |
| memory-mcp | 170 | <68 | 102 |
| memory-storage-turso | 102 | <41 | 61 |
| memory-cli | 53 | <21 | 32 |
| memory-storage-redb | 18 | <7 | 11 |

## Optimization Strategy

### Phase 1: High-Impact Arc Conversions (~120 clones)

#### 1.1 Episode Retrieval Returns Arc<Episode>
- **File**: `memory-core/src/memory/retrieval/context.rs`
- **Function**: `retrieve_relevant_context`
- **Current**: Returns `Vec<Episode>`
- **Target**: Returns `Vec<Arc<Episode>>`
- **Impact**: ~30 clones eliminated

#### 1.2 Conflict Resolution Arc-Based
- **File**: `memory-core/src/sync/conflict.rs`
- **Functions**: `resolve_episode_conflict`, `resolve_pattern_conflict`
- **Current**: Returns owned `Episode`/`Pattern`
- **Target**: Returns `Arc<Episode>`/`Arc<Pattern>`
- **Impact**: ~12 clones eliminated

### Phase 2: Pattern Storage Optimization (~50 clones)

#### 2.1 Avoid Clone Before Format
- **File**: `memory-storage-turso/src/storage/batch/pattern_batch.rs`
- **Pattern**: Clone before `format!()` or `join()`
- **Target**: Use iterator directly
- **Impact**: ~15 clones eliminated

#### 2.2 Use AsRef for String Parameters
- **File**: `memory-storage-turso/src/storage/patterns.rs`
- **Target**: Accept `&str` instead of `String` where possible
- **Impact**: ~25 clones eliminated

### Phase 3: Embedding Tool Optimization (~40 clones)

#### 3.1 Accept &str for Optional Fields
- **File**: `memory-mcp/src/mcp/tools/embeddings/tool/execute.rs`
- **Function**: `execute_query_semantic_memory`
- **Target**: Accept `&str` for `domain`, `query` parameters
- **Impact**: ~20 clones eliminated

#### 3.2 Reduce Arc Dereference Clones
- **File**: `memory-mcp/src/mcp/tools/embeddings/tool/execute.rs`
- **Pattern**: `(*arc_ep).clone()`
- **Target**: Use `Arc::clone()` or return `Arc`
- **Impact**: ~15 clones eliminated

### Phase 4: Caching Optimization (~30 clones)

#### 4.1 Cache Results as Arc
- **File**: `memory-mcp/src/cache.rs`
- **Pattern**: `result.clone()` on cache hit
- **Target**: Store as `Arc<Vec<Episode>>`
- **Impact**: ~6 clones eliminated

#### 4.2 Reduce Pattern Cache Clones
- **Target**: Similar optimization for pattern caching
- **Impact**: ~10 clones eliminated

### Phase 5: Context and TaskContext Optimization (~50 clones)

#### 5.1 Use Cow for Context Fields
- **File**: `memory-core/src/types/structs.rs`
- **Target**: Use `Cow<'_, str>` for domain, language, framework
- **Impact**: ~25 clones eliminated

#### 5.2 Accept References in Episode Methods
- **Target**: Accept `&Episode` instead of `Episode` in methods
- **Impact**: ~25 clones eliminated

## Implementation Plan

### Step 1: Setup and Baseline
- [ ] Run baseline clone count: `grep -r "\.clone()" --include="*.rs" | grep -v "tests/" | grep -v "benches/" | grep -v "_test.rs" | wc -l`
- [ ] Create branch: `git checkout -b fix/issue-218-reduce-clones`

### Step 2: Phase 1 - Arc Conversions
- [ ] Modify `retrieve_relevant_context` to return `Vec<Arc<Episode>>`
- [ ] Update all callers of retrieval functions
- [ ] Modify conflict resolution to use Arc
- [ ] Run tests: `cargo test --all`
- [ ] Run clippy: `cargo clippy --all`

### Step 3: Phase 2 - Pattern Storage
- [ ] Optimize pattern batch storage
- [ ] Optimize pattern storage
- [ ] Run tests and clippy

### Step 4: Phase 3 - Embedding Tools
- [ ] Optimize embedding tool inputs
- [ ] Reduce Arc dereference clones
- [ ] Run tests and clippy

### Step 5: Phase 4 - Caching
- [ ] Optimize cache storage
- [ ] Run tests and clippy

### Step 6: Phase 5 - Context Optimization
- [ ] Implement Cow for context fields
- [ ] Accept references in methods
- [ ] Run tests and clippy

### Step 7: Final Validation
- [ ] Run final clone count
- [ ] Run full test suite: `cargo test --all`
- [ ] Run quality gates: `./scripts/quality-gates.sh`
- [ ] Run clippy: `cargo clippy --all`
- [ ] Commit changes

## Success Criteria
- Clone count reduced from 509 to <200
- All tests passing (99.5% current)
- Zero clippy warnings
- Coverage maintained at >90%

## Risk Mitigation
- Backward compatibility: Ensure API changes are minimal
- Performance: Arc operations are cheap reference count increments
- Testing: Comprehensive test coverage already exists

## Notes
- Benchmark files are exempt from clone reduction (performance testing)
- Focus on production code only
- Maintain code readability while optimizing
