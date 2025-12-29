# Turso AI Enhancement Phase 0: Test Infrastructure Plan

**Date**: 2025-12-29
**Agent**: testing-qa
**Status**: DRAFT
**Purpose**: Comprehensive test infrastructure design for Turso AI enhancements

## Overview

This document outlines the test infrastructure required to support the Turso AI enhancement phases (1-4). The goal is to ensure that all new functionality is thoroughly tested, performance regressions are detected, and quality gates are maintained (>90% coverage).

## 1. Test Scaffolding for Multi-Dimension Storage

### Current State Analysis
- **Schema**: Multi-dimension tables already defined in `schema.rs` (embeddings_384, embeddings_1024, embeddings_1536, embeddings_3072, embeddings_other)
- **Implementation**: Not yet used in storage layer (`store_embedding_backend` only uses single embeddings table)
- **Routing Logic**: No dimension-based routing exists
- **Migration**: No migration from old embeddings table to dimension-specific tables

### Test Requirements

#### 1.1 Unit Tests for Each Dimension Table
- **Objective**: Verify each dimension table schema works correctly
- **Test Cases**:
  - `test_embeddings_384_table_creation`: Create table, insert embedding, verify vector operations
  - `test_embeddings_1536_table_creation`: Same for 1536-dim
  - `test_embeddings_3072_table_creation`: Same for 3072-dim
  - `test_embeddings_other_table_creation`: For non-standard dimensions
- **Location**: `memory-storage-turso/tests/multi_dimension_unit_tests.rs`

#### 1.2 Integration Tests for Routing Logic
- **Objective**: Verify dimension-based routing selects correct table
- **Test Cases**:
  - `test_routing_384_dimension`: Store 384-dim embedding, verify goes to embeddings_384
  - `test_routing_1536_dimension`: Store 1536-dim embedding, verify goes to embeddings_1536
  - `test_routing_unsupported_dimension`: Store 500-dim embedding, verify goes to embeddings_other
  - `test_routing_edge_cases`: Zero-length, max-length, negative dimension handling
- **Location**: `memory-storage-turso/tests/multi_dimension_integration_tests.rs`

#### 1.3 Migration Test Suite
- **Objective**: Ensure migration from old embeddings table works correctly
- **Test Cases**:
  - `test_migration_empty_database`: Empty old table → no data migration
  - `test_migration_mixed_dimensions`: Mixed 384 and 1536 dim embeddings in old table → split correctly
  - `test_migration_idempotent`: Running migration twice doesn't duplicate data
  - `test_migration_data_integrity`: Verify all data preserved, embeddings retrievable
- **Location**: `memory-storage-turso/tests/migration_tests.rs`

#### 1.4 Backward Compatibility Tests
- **Objective**: Ensure existing code continues to work with new schema
- **Test Cases**:
  - `test_backward_compatibility_episode_embeddings`: Episode embedding storage/retrieval still works
  - `test_backward_compatibility_pattern_embeddings`: Pattern embedding storage/retrieval still works
  - `test_backward_compatibility_vector_search`: Vector similarity search still works
  - `test_backward_compatibility_api_contracts`: All public APIs maintain same signatures/behavior
- **Location**: `memory-storage-turso/tests/backward_compatibility_tests.rs`

### Test Scaffolding Code
Create test utilities in `test-utils/src/multi_dimension.rs`:
```rust
pub struct MultiDimensionTestHarness {
    storage: TursoStorage,
    temp_dir: TempDir,
}

impl MultiDimensionTestHarness {
    pub async fn new() -> Self { /* ... */ }
    pub async fn store_embedding(&self, dimension: usize, embedding: Vec<f32>) -> Uuid { /* ... */ }
    pub async fn verify_table_used(&self, embedding_id: Uuid, expected_table: &str) -> bool { /* ... */ }
    pub async fn get_table_for_dimension(&self, dimension: usize) -> &'static str { /* ... */ }
}
```

## 2. Performance Test Suite for Vector Search

### Current State Analysis
- **Existing Benchmarks**: `turso_vector_performance.rs` benchmarks 384-dim native search
- **Gaps**: No benchmarks for 1536-dim native search, hybrid search, extension performance
- **Regression Detection**: No automated regression detection in CI

### Test Requirements

#### 2.1 Benchmark Tests for Different Dimensions
- **Objective**: Measure performance across supported dimensions
- **Benchmark Cases**:
  - `benchmark_384_dim_native_search`: Native vector search for 384-dim (existing)
  - `benchmark_1536_dim_native_search`: Native vector search for 1536-dim (new)
  - `benchmark_3072_dim_native_search`: Native vector search for 3072-dim (new)
  - `benchmark_unsupported_dim_bruteforce`: Brute-force search for other dimensions
- **Location**: `benches/turso_vector_performance.rs` (extend existing)

#### 2.2 Comparison Tests: Native vs Brute-Force
- **Objective**: Validate native vector search provides speedup
- **Test Cases**:
  - `test_native_vs_bruteforce_384`: Compare accuracy and speed for 384-dim
  - `test_native_vs_bruteforce_1536`: Compare accuracy and speed for 1536-dim
  - `test_accuracy_validation`: Ensure native search results match brute-force within tolerance
- **Location**: `tests/performance/vector_search_comparison.rs`

#### 2.3 Scaling Tests with Dataset Size
- **Objective**: Measure performance scaling as dataset grows
- **Test Cases**:
  - `benchmark_scaling_384_dim`: 100, 1K, 10K, 100K embeddings
  - `benchmark_scaling_1536_dim`: Same for 1536-dim
  - `benchmark_memory_usage`: Track memory usage across dataset sizes
- **Location**: `benches/vector_search_scaling.rs`

#### 2.4 Performance Regression Detection
- **Objective**: Automatically detect performance regressions in CI
- **Implementation**:
  - Extend quality gates to include performance regression detection
  - Store baseline measurements in `benchmark_results/`
  - Compare current benchmarks against baseline, fail if >10% regression
  - Use `criterion` with baseline comparison feature
- **Script**: `scripts/check_performance_regression.sh` (already exists, needs enhancement)

### Performance Test Framework Design
Extend existing `benches/turso_vector_performance.rs` with:
- Parameterized benchmarks for all dimensions (384, 1024, 1536, 3072)
- Mixed workload benchmarks (reads/writes concurrent)
- Memory usage tracking via `memory_stats` crate
- Accuracy validation against brute-force ground truth

## 3. Integration Test Plan for Hybrid Search

### Current State Analysis
- **No FTS5 Implementation**: Currently no full-text search capability
- **Hybrid Search**: No combination of vector and keyword search
- **Ranking Algorithm**: No blending algorithm exists

### Test Requirements

#### 3.1 End-to-End Tests for FTS5 + Vector Search
- **Objective**: Verify FTS5 tables and triggers work correctly
- **Test Cases**:
  - `test_fts5_table_creation`: Create episodes_fts table, verify virtual table properties
  - `test_fts5_trigger_synchronization`: Insert episode, verify appears in FTS5 table
  - `test_fts5_search_basic`: Keyword search returns relevant episodes
  - `test_fts5_search_advanced`: Porter stemming, unicode61 tokenization works
- **Location**: `memory-storage-turso/tests/fts5_integration_tests.rs`

#### 3.2 Ranking Algorithm Validation Tests
- **Objective**: Verify hybrid ranking produces better results than single methods
- **Test Cases**:
  - `test_hybrid_ranking_vector_weight_1`: Weight = 1.0 (vector-only) matches vector search
  - `test_hybrid_ranking_fts_weight_1`: Weight = 0.0 (keyword-only) matches FTS5 search
  - `test_hybrid_ranking_blended`: Weight = 0.5 produces blended results
  - `test_hybrid_ranking_relevance`: Hybrid results more relevant than single methods (qualitative)
- **Location**: `memory-core/tests/hybrid_search_tests.rs`

#### 3.3 Trigger Synchronization Tests
- **Objective**: Ensure FTS5 tables stay synchronized with base tables
- **Test Cases**:
  - `test_trigger_on_insert`: New episode automatically added to FTS5
  - `test_trigger_on_update`: Updated episode description updates FTS5
  - `test_trigger_on_delete`: Deleted episode removed from FTS5
  - `test_trigger_performance`: Measure overhead of triggers on write operations
- **Location**: `memory-storage-turso/tests/fts5_trigger_tests.rs`

#### 3.4 Query Combination Tests
- **Objective**: Test various query combinations and edge cases
- **Test Cases**:
  - `test_hybrid_empty_vector`: Vector embedding missing, fallback to FTS5
  - `test_hybrid_empty_keyword`: Keyword empty, fallback to vector
  - `test_hybrid_both_empty`: Both empty, return empty results
  - `test_hybrid_domain_filtering`: Combine hybrid search with domain filtering
  - `test_hybrid_pagination`: Verify limit/offset works correctly
- **Location**: `memory-core/tests/hybrid_query_tests.rs`

### Integration Test Framework
Create `test-utils/src/hybrid_search.rs`:
```rust
pub struct HybridSearchTestHarness {
    storage: TursoStorage,
    fts5_enabled: bool,
}

impl HybridSearchTestHarness {
    pub async fn setup_fts5() -> Self { /* ... */ }
    pub async fn search_hybrid(&self, query: &str, embedding: Option<Vec<f32>>, weight: f32) -> Vec<Episode> { /* ... */ }
    pub fn assert_relevance_order(&self, results: &[Episode], expected_order: &[Uuid]) { /* ... */ }
}
```

## 4. Extension Compatibility Test Matrix

### Current State Analysis
- **SQLite Extensions**: Turso preloads JSON, Stats, Crypto, UUID extensions
- **Current Usage**: Minimal usage in codebase
- **Compatibility**: Need to verify extensions work across Turso versions

### Test Requirements

#### 4.1 JSON Functions Compatibility Tests
- **Objective**: Verify JSON functions work correctly
- **Test Cases**:
  - `test_json_extract_basic`: Extract scalar values from JSON
  - `test_json_extract_nested`: Extract nested objects/arrays
  - `test_json_query_performance`: Compare JSON query vs Rust deserialization speed
  - `test_json_error_handling`: Malformed JSON handled gracefully
- **Location**: `memory-storage-turso/tests/json_extension_tests.rs`

#### 4.2 SQLean Stats Extension Tests
- **Objective**: Verify statistical functions work
- **Test Cases**:
  - `test_stats_mean_median`: Basic statistical functions
  - `test_stats_stddev_variance`: Advanced statistics
  - `test_stats_percentile`: Percentile calculations
  - `test_stats_query_integration`: Use in analytical queries
- **Location**: `memory-storage-turso/tests/stats_extension_tests.rs`

#### 4.3 Crypto/UUID Extension Tests
- **Objective**: Verify cryptographic and UUID functions
- **Test Cases**:
  - `test_crypto_hash_sha256`: SHA256 hashing works
  - `test_crypto_hmac`: HMAC generation
  - `test_uuid_generation`: UUID generation functions
  - `test_uuid_validation`: UUID validation/parsing
- **Location**: `memory-storage-turso/tests/crypto_uuid_tests.rs`

#### 4.4 Feature Flag Tests
- **Objective**: Verify extensions can be disabled via feature flags
- **Test Cases**:
  - `test_feature_flag_json_disabled`: JSON functions unavailable when feature disabled
  - `test_feature_flag_stats_disabled`: Stats functions unavailable
  - `test_feature_flag_graceful_fallback`: Code falls back to Rust implementations
  - `test_feature_flag_compilation`: Verify all feature combinations compile
- **Location**: `memory-storage-turso/tests/feature_flag_tests.rs`

### Compatibility Matrix

| Extension | Function | Turso Version | Required | Fallback |
|-----------|----------|---------------|----------|----------|
| JSON | json_extract, json_group_array | All | Yes | Rust serde_json |
| Stats | mean, median, stddev | All | Optional | Calculate in Rust |
| Crypto | sha256, hmac | All | Optional | Rust ring/sha2 |
| UUID | uuid(), uuid_str() | All | Optional | Rust uuid crate |

Test matrix to verify each combination works.

## 5. Coverage Maintenance Strategy

### Current Coverage: 92.5% (line), >85% (branch)
**Goal**: Maintain >90% line coverage throughout changes

### Strategy
1. **Test-First Development**: Write tests before implementing features
2. **Coverage Monitoring**: Run `cargo llvm-cov` after each significant change
3. **Gap Analysis**: Identify uncovered code after each merge
4. **Incremental Coverage**: Require new code to have >90% coverage

### Implementation
- Add coverage check to pre-commit hook
- Extend CI to fail if coverage drops below 90%
- Add coverage badge to README
- Weekly coverage report generation

## 6. CI/CD Integration

### Enhanced GitHub Actions Workflow
Add new jobs to `.github/workflows/ci-enhanced.yml`:
1. **Multi-dimension tests**: Run new test suites
2. **Performance regression**: Compare benchmarks against baseline
3. **Extension compatibility**: Test with different feature flags
4. **Coverage enforcement**: Fail if coverage drops

### Quality Gate Updates
Update `tests/quality_gates.rs` to include:
- `quality_gate_multi_dimension_coverage`: Verify all dimension tables tested
- `quality_gate_hybrid_search_functional`: Verify hybrid search works
- `quality_gate_extension_compatibility`: Verify extensions work

## 7. Implementation Timeline

### Day 1 (2025-12-29)
- [ ] Create test scaffolding code (multi-dimension test utilities)
- [ ] Extend performance benchmarks (add 1536-dim, 3072-dim)
- [ ] Design hybrid search test harness
- [ ] Create extension compatibility test matrix

### Day 2 (2025-12-30)
- [ ] Write unit tests for multi-dimension routing
- [ ] Implement performance regression detection
- [ ] Write FTS5 integration tests
- [ ] Write JSON extension tests

## 8. Handoff Procedures

### To rust-specialist (Phase 1):
- Multi-dimension test utilities ready
- Unit test patterns for routing logic
- Migration test suite ready

### To feature-implementer (Phase 3):
- FTS5 test harness ready
- Hybrid ranking test cases defined
- Trigger synchronization test patterns

### To performance (Phase 2):
- Extended benchmark suite ready
- Performance regression detection script
- Scaling test framework

### To all agents:
- Test plan document
- Coverage maintenance strategy
- CI integration recommendations

## 9. Risk Mitigation

### Technical Risks
1. **Test Flakiness**: Use deterministic test data, isolate tests
2. **Performance Variability**: Use statistical validation, multiple runs
3. **Coverage Gaps**: Incremental coverage checks, gap analysis

### Coordination Risks
1. **Test-Dependency Deadlocks**: Provide test scaffolding before implementation
2. **Quality Gate Conflicts**: Communicate threshold changes early
3. **CI Pipeline Slowdown**: Parallel test execution, selective runs

## 10. Success Metrics

### Quantitative
- [ ] All new tests pass (100%)
- [ ] Coverage maintained >90%
- [ ] Performance benchmarks show improvement
- [ ] No regression in existing functionality

### Qualitative
- [ ] Test scaffolding easy for other agents to use
- [ ] Performance regression detection catches issues
- [ ] Extension compatibility verified
- [ ] Hybrid search relevance validated

---

*Turso AI Enhancement Test Infrastructure Plan v1.0*
*Created by testing-qa agent on 2025-12-29*