# Episode Relationships - Testing Strategy

**Last Updated**: 2026-01-31  
**Feature Version**: v0.1.14  
**Status**: Phase 1 Tests Complete ✅ (11/11 passing)

---

## Overview

This document outlines the comprehensive testing strategy for the Episode Relationships feature across all 6 implementation phases. The goal is to maintain >90% test coverage while ensuring correctness, performance, and reliability.

---

## Testing Pyramid

```
                    ┌─────────────────┐
                    │  E2E Tests (10) │
                    └────────┬────────┘
                 ┌───────────┴───────────┐
                 │ Integration Tests (25)│
                 └──────────┬────────────┘
            ┌───────────────┴───────────────┐
            │     Unit Tests (80+)          │
            └───────────────────────────────┘
```

**Total Tests**: 116+ across all phases  
**Current**: 11 tests (Phase 1 only)  
**Remaining**: 105+ tests

---

## Phase 1: Storage Layer Tests ✅ COMPLETE

### Current Test Coverage

| Module | Tests | Status | Coverage |
|--------|-------|--------|----------|
| `memory-core/relationships.rs` | Data structures only | N/A | 100% |
| `memory-storage-turso/relationships.rs` | 5 | ✅ All passing | 100% |
| `memory-storage-redb/relationships.rs` | 6 | ✅ All passing | 100% |
| **Phase 1 Total** | **11** | **✅** | **100%** |

### Test Details

#### Turso Storage Tests (5 tests)

1. **`test_add_relationship`**
   - Creates two episodes
   - Adds relationship between them
   - Verifies relationship stored correctly
   - Checks all fields match

2. **`test_get_relationships`**
   - Creates 3 episodes with relationships
   - Tests outgoing direction
   - Tests incoming direction
   - Tests both directions

3. **`test_remove_relationship`**
   - Adds relationship
   - Removes it
   - Verifies it no longer exists

4. **`test_relationship_exists`**
   - Adds relationship
   - Checks exists() returns true
   - Checks non-existent returns false

5. **`test_get_dependencies`**
   - Creates chain: A → B → C (DependsOn)
   - Verifies get_dependencies() returns correct IDs
   - Verifies get_dependent_episodes() works

#### Redb Cache Tests (6 tests)

1. **`test_cache_relationship`**
   - Caches a relationship
   - Retrieves it
   - Verifies data integrity

2. **`test_get_cached_relationship`**
   - Tests cache hit
   - Tests cache miss (returns None)

3. **`test_remove_cached_relationship`**
   - Caches relationship
   - Removes it
   - Verifies removal

4. **`test_get_cached_relationships`**
   - Caches multiple relationships
   - Tests direction filtering
   - Verifies correct results

5. **`test_clear_relationships_cache`**
   - Caches multiple relationships
   - Clears cache
   - Verifies all removed

6. **`test_count_cached_relationships`**
   - Tests count after adding relationships
   - Verifies count increments correctly

### Running Phase 1 Tests

```bash
# Run all Turso tests
cargo test -p memory-storage-turso relationships -- --nocapture

# Run all Redb tests
cargo test -p memory-storage-redb relationships -- --nocapture

# Run specific test
cargo test -p memory-storage-turso test_add_relationship -- --exact

# With debug logging
RUST_LOG=debug cargo test -p memory-storage-turso relationships
```

---

## Phase 2: Business Logic Tests (20+ tests)

### RelationshipManager Tests (12 tests)

#### Validation Tests (6 tests)

1. **`test_add_valid_relationship`**
   - Add simple valid relationship
   - Verify success

2. **`test_prevent_self_relationship`**
   - Try to add A → A
   - Expect ValidationError::SelfRelationship

3. **`test_prevent_duplicate_relationship`**
   - Add A → B (DependsOn)
   - Try to add A → B (DependsOn) again
   - Expect ValidationError::DuplicateRelationship

4. **`test_detect_simple_cycle`**
   - Add A → B → C
   - Try to add C → A
   - Expect ValidationError::CycleDetected

5. **`test_detect_complex_cycle`**
   - Create diamond: A → B, A → C, B → D, C → D
   - Try to add D → A
   - Expect cycle detection

6. **`test_invalid_priority`**
   - Try priority = 0 (expect error)
   - Try priority = 11 (expect error)
   - Try priority = 5 (expect success)

#### Query Tests (3 tests)

7. **`test_get_outgoing_relationships`**
   - Add multiple outgoing from A
   - Verify all returned

8. **`test_get_incoming_relationships`**
   - Add multiple incoming to A
   - Verify all returned

9. **`test_get_by_type`**
   - Add relationships of different types
   - Filter by type
   - Verify correct subset returned

#### Removal Tests (2 tests)

10. **`test_remove_existing_relationship`**
    - Add relationship
    - Remove it
    - Verify gone

11. **`test_remove_nonexistent_relationship`**
    - Try to remove non-existent ID
    - Expect RemovalError::NotFound

#### Loading Tests (1 test)

12. **`test_load_relationships`**
    - Create relationships in storage
    - Load into manager
    - Verify graph structure correct

### Graph Algorithm Tests (10 tests)

#### Path Finding Tests (3 tests)

1. **`test_has_path_simple`**
   - A → B → C
   - Verify has_path(A, C) = true
   - Verify has_path(C, A) = false

2. **`test_has_path_complex`**
   - Create complex graph
   - Test various path queries

3. **`test_find_path`**
   - Create A → B → C → D
   - Find path A to D
   - Verify returns [A, B, C, D]

#### Cycle Detection Tests (3 tests)

4. **`test_has_cycle_simple`**
   - A → B → A
   - Expect has_cycle() = true

5. **`test_has_cycle_complex`**
   - Large graph with cycle
   - Verify detection

6. **`test_no_cycle_dag`**
   - Create valid DAG
   - Verify has_cycle() = false

#### Topological Sort Tests (2 tests)

7. **`test_topological_sort_simple`**
   - A → B → C
   - Verify order: [A, B, C]

8. **`test_topological_sort_fails_on_cycle`**
   - Create cycle
   - Expect error from topological_sort()

#### Transitive Closure Tests (2 tests)

9. **`test_transitive_closure_simple`**
   - A → B → C → D
   - get_transitive_closure(A) = {B, C, D}

10. **`test_transitive_closure_complex`**
    - Complex graph
    - Verify all reachable nodes

---

## Phase 3: Integration Tests (15+ tests)

### MemoryManager Integration Tests (10 tests)

1. **`test_add_relationship_end_to_end`**
   - Create episodes via MemoryManager
   - Add relationship
   - Query and verify

2. **`test_add_relationship_validates_episode_exists`**
   - Try to add relationship with non-existent episode
   - Expect error

3. **`test_cycle_detection_integration`**
   - Create chain via MemoryManager
   - Try to add cycle
   - Verify rejection

4. **`test_remove_relationship_integration`**
   - Add relationship via MemoryManager
   - Remove it
   - Verify removed from storage and cache

5. **`test_get_relationships_cache_hit`**
   - Add relationship
   - Query (should cache)
   - Query again (should hit cache)
   - Verify cache metrics

6. **`test_get_relationships_cache_miss`**
   - Query non-cached relationship
   - Verify falls back to storage
   - Verify cache warmed

7. **`test_find_related_episodes_with_filter`**
   - Add relationships of different types
   - Use RelationshipFilter
   - Verify correct subset

8. **`test_find_related_episodes_with_limit`**
   - Add 20 relationships
   - Query with limit=5
   - Verify only 5 returned

9. **`test_get_relationship_graph`**
   - Create complex graph
   - Export with depth=2
   - Verify structure correct

10. **`test_relationship_cascade_delete`**
    - Add relationships
    - Delete episode
    - Verify relationships auto-removed

### Cache Consistency Tests (5 tests)

11. **`test_cache_invalidation_on_remove`**
    - Add and cache relationship
    - Remove relationship
    - Verify cache invalidated

12. **`test_cache_warming_on_query`**
    - Query relationship (not cached)
    - Verify cache warmed after query

13. **`test_cache_consistency_across_operations`**
    - Add multiple relationships
    - Mix of cache hits and misses
    - Verify consistency

14. **`test_cache_handles_storage_errors`**
    - Simulate storage failure
    - Verify cache returns stale data (if configured)

15. **`test_concurrent_cache_access`**
    - Multiple threads accessing cache
    - Verify thread safety

---

## Phase 4: MCP Tool Tests (16 tests)

### Tool Handler Tests (2 tests per tool = 16 tests)

For each of 8 tools, test:
1. **Success case** - Valid input, expect success
2. **Error case** - Invalid input, expect error

#### Example: add_episode_relationship

1. **`test_mcp_add_relationship_success`**
   ```rust
   let params = json!({
       "from_episode_id": "abc-123",
       "to_episode_id": "def-456",
       "relationship_type": "depends_on"
   });
   let result = handle_add_episode_relationship(&manager, params).await;
   assert!(result.is_ok());
   ```

2. **`test_mcp_add_relationship_invalid_uuid`**
   ```rust
   let params = json!({
       "from_episode_id": "not-a-uuid",
       "to_episode_id": "def-456",
       "relationship_type": "depends_on"
   });
   let result = handle_add_episode_relationship(&manager, params).await;
   assert!(result.is_err());
   ```

### JSON-RPC Integration Tests (2 tests)

1. **`test_mcp_jsonrpc_protocol`**
   - Send JSON-RPC 2.0 request
   - Verify response format

2. **`test_mcp_error_responses`**
   - Send invalid requests
   - Verify error format matches JSON-RPC spec

---

## Phase 5: CLI Tests (14 tests)

### CLI Command Tests (2 tests per command = 14 tests)

For each of 7 commands, test:
1. **Success case** - Valid args, expect success
2. **Error case** - Invalid args, expect error

#### Example: episode add-relationship

1. **`test_cli_add_relationship_success`**
   ```rust
   let output = Command::new("memory-cli")
       .args(&["episode", "add-relationship", "abc-123", 
               "--to", "def-456", "--type", "depends_on"])
       .output()
       .expect("Failed to execute");
   assert!(output.status.success());
   ```

2. **`test_cli_add_relationship_missing_arg`**
   ```rust
   let output = Command::new("memory-cli")
       .args(&["episode", "add-relationship", "abc-123"])
       .output()
       .expect("Failed to execute");
   assert!(!output.status.success());
   ```

### Output Format Tests (3 tests)

1. **`test_cli_table_format`**
   - List relationships with --format table
   - Verify table structure

2. **`test_cli_json_format`**
   - List relationships with --format json
   - Verify valid JSON

3. **`test_cli_dot_format`**
   - Export graph with --format dot
   - Verify valid DOT syntax

---

## Phase 6: E2E & Performance Tests (25+ tests)

### End-to-End Workflow Tests (10 tests)

1. **`test_e2e_dependency_workflow`**
   - Create episodes via CLI
   - Add dependencies via CLI
   - Query via MCP tool
   - Verify consistency

2. **`test_e2e_hierarchical_workflow`**
   - Create epic → story → subtask hierarchy
   - Verify relationships
   - Test cascade delete

3. **`test_e2e_cycle_prevention`**
   - Attempt to create cycle via MCP
   - Verify rejection
   - Check via CLI

4. **`test_e2e_graph_export`**
   - Create complex graph
   - Export via CLI (DOT format)
   - Import into visualization tool
   - Verify structure

5. **`test_e2e_cache_performance`**
   - Add 1000 relationships
   - Query repeatedly
   - Measure cache hit rate (expect >80%)

6. **`test_e2e_concurrent_operations`**
   - Multiple clients adding relationships
   - Verify no race conditions
   - Check data consistency

7. **`test_e2e_large_graph`**
   - Create graph with 1000+ episodes
   - 5000+ relationships
   - Verify query performance

8. **`test_e2e_relationship_metadata`**
   - Add relationships with rich metadata
   - Query and filter by metadata
   - Verify preservation

9. **`test_e2e_mixed_relationship_types`**
   - Create relationships of all 7 types
   - Test queries by type
   - Verify correct behavior

10. **`test_e2e_storage_recovery`**
    - Add relationships
    - Restart server
    - Verify relationships persisted

### Performance Benchmarks (15 benchmarks)

#### Storage Layer Benchmarks (5)

1. **`bench_add_relationship`**
   - Target: <10ms (P95)
   - Measure: Single relationship insertion

2. **`bench_get_relationships`**
   - Target: <20ms (P95)
   - Measure: Query 100 relationships

3. **`bench_remove_relationship`**
   - Target: <10ms (P95)
   - Measure: Single deletion

4. **`bench_relationship_exists`**
   - Target: <5ms (P95)
   - Measure: Existence check

5. **`bench_get_dependencies`**
   - Target: <50ms (P95)
   - Measure: Transitive query (depth 3)

#### Graph Algorithm Benchmarks (5)

6. **`bench_has_cycle_small`**
   - Graph: 100 nodes, 200 edges
   - Target: <10ms (P95)

7. **`bench_has_cycle_large`**
   - Graph: 1000 nodes, 5000 edges
   - Target: <100ms (P95)

8. **`bench_topological_sort`**
   - Graph: 500 nodes (DAG)
   - Target: <50ms (P95)

9. **`bench_find_path`**
   - Graph: 1000 nodes
   - Target: <20ms (P95)

10. **`bench_transitive_closure`**
    - Graph: 500 nodes
    - Target: <100ms (P95)

#### End-to-End Benchmarks (5)

11. **`bench_e2e_add_via_mcp`**
    - Full MCP tool call
    - Target: <100ms (P95)

12. **`bench_e2e_query_via_cli`**
    - Full CLI command execution
    - Target: <200ms (P95)

13. **`bench_cache_hit_rate`**
    - 1000 queries (mixed)
    - Target: >80% hit rate

14. **`bench_concurrent_writes`**
    - 10 concurrent clients
    - 100 writes each
    - Target: No deadlocks, <500ms avg

15. **`bench_graph_export`**
    - 500 nodes to DOT format
    - Target: <500ms (P95)

---

## Test Execution Strategy

### Local Development

```bash
# Run all tests
cargo test --workspace

# Run specific phase tests
cargo test -p memory-core relationships
cargo test -p memory-storage-turso relationships
cargo test -p memory-mcp relationships

# Run with coverage
cargo tarpaulin --workspace --out Html --output-dir coverage

# Run benchmarks
cargo bench --bench relationships
```

### CI/CD Pipeline

```yaml
# .github/workflows/test-relationships.yml
name: Relationship Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run unit tests
        run: cargo test --workspace
      
      - name: Check coverage
        run: |
          cargo tarpaulin --workspace --out Xml
          if [ $(grep -oP 'line-rate="\K[0-9.]+' cobertura.xml) < 0.90 ]; then
            echo "Coverage below 90%"
            exit 1
          fi
      
      - name: Run benchmarks
        run: cargo bench --bench relationships -- --test
```

---

## Coverage Requirements

| Phase | Unit Tests | Integration Tests | E2E Tests | Min Coverage |
|-------|-----------|-------------------|-----------|--------------|
| 1 | 11 | 0 | 0 | 100% ✅ |
| 2 | 20 | 0 | 0 | >90% |
| 3 | 10 | 15 | 0 | >90% |
| 4 | 16 | 2 | 0 | >90% |
| 5 | 14 | 0 | 3 | >90% |
| 6 | 0 | 10 | 15 | >95% |
| **Total** | **71** | **27** | **18** | **>92%** |

---

## Quality Gates

### Pre-Commit
- [ ] All unit tests pass
- [ ] Formatting check (rustfmt)
- [ ] Linting check (clippy - zero warnings)

### Pre-Merge
- [ ] All tests pass (unit + integration)
- [ ] Coverage >90% for changed code
- [ ] Benchmarks show <10% regression
- [ ] Documentation updated

### Pre-Release
- [ ] All E2E tests pass
- [ ] Overall coverage >92%
- [ ] Performance targets met
- [ ] Security audit passed
- [ ] User acceptance testing complete

---

## Test Data Strategy

### Test Fixtures

Create reusable test data:

```rust
// tests/fixtures/relationships.rs
pub fn create_test_episode(id: &str, desc: &str) -> Episode {
    Episode::new(
        desc.to_string(),
        TaskContext::default(),
        TaskType::Testing,
    )
}

pub fn create_test_relationship(
    from: Uuid,
    to: Uuid,
    rel_type: RelationshipType,
) -> EpisodeRelationship {
    EpisodeRelationship {
        id: Uuid::new_v4(),
        from_episode_id: from,
        to_episode_id: to,
        relationship_type: rel_type,
        metadata: RelationshipMetadata::default(),
        created_at: Utc::now(),
    }
}

pub fn create_test_graph(nodes: usize, edges: usize) -> HashMap<Uuid, Vec<EpisodeRelationship>> {
    // Generate random DAG for testing
}
```

---

## Continuous Improvement

### Metrics to Track

1. **Test Count**: Target 116+ (currently 11)
2. **Coverage**: Target >92% (currently 100% for Phase 1)
3. **Test Execution Time**: Target <5 minutes for full suite
4. **Flaky Test Rate**: Target <1%
5. **Bug Escape Rate**: Target <5% (bugs found in production)

### Monthly Review

- Analyze test failures
- Identify gaps in coverage
- Update test strategy as needed
- Add tests for production bugs

---

## Conclusion

This testing strategy ensures:
- ✅ High confidence in correctness
- ✅ Early bug detection
- ✅ Performance regression prevention
- ✅ Maintainable test suite
- ✅ Documentation through tests

**Current Status**: Phase 1 complete (11/11 tests passing, 100% coverage)  
**Next Milestone**: Phase 2 - Add 20+ business logic tests
