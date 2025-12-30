# Multi-Dimension Schema Design (Option B)

**Date**: 2025-12-29
**Author**: rust-specialist
**Status**: Design Complete, Ready for Implementation
**Phase**: Phase 0 Preparation

## 1. Overview

Current implementation (v0.1.9) uses a single `embeddings` table with `F32_BLOB(384)` column, limiting native vector search to 384-dimensional embeddings only. OpenAI embeddings (1536-dim) fall back to brute-force O(n) scanning.

This design implements **Option B** from the Turso AI recommendations: separate tables per dimension for native vector support.

## 2. Schema Changes

### New Tables Added

Five new tables have been added to `memory-storage-turso/src/schema.rs`:

#### 1. `embeddings_384` - 384-dimensional embeddings
```sql
CREATE TABLE IF NOT EXISTS embeddings_384 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(384),
    dimension INTEGER NOT NULL DEFAULT 384,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
```

#### 2. `embeddings_1024` - 1024-dimensional embeddings (Mistral AI)
```sql
CREATE TABLE IF NOT EXISTS embeddings_1024 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(1024),
    dimension INTEGER NOT NULL DEFAULT 1024,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
```

#### 3. `embeddings_1536` - 1536-dimensional embeddings (OpenAI small/ada)
```sql
CREATE TABLE IF NOT EXISTS embeddings_1536 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(1536),
    dimension INTEGER NOT NULL DEFAULT 1536,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
```

#### 4. `embeddings_3072` - 3072-dimensional embeddings (OpenAI large)
```sql
CREATE TABLE IF NOT EXISTS embeddings_3072 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(3072),
    dimension INTEGER NOT NULL DEFAULT 3072,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
```

#### 5. `embeddings_other` - Arbitrary dimension embeddings (no native vector support)
```sql
CREATE TABLE IF NOT EXISTS embeddings_other (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector BLOB,
    dimension INTEGER NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
```

### New Indexes

For each dimension-specific table (excluding `embeddings_other`):
- **Vector index**: `CREATE INDEX IF NOT EXISTS idx_embeddings_<dim>_vector ON embeddings_<dim>(libsql_vector_idx(embedding_vector))`
- **Item index**: `CREATE INDEX IF NOT EXISTS idx_embeddings_<dim>_item ON embeddings_<dim>(item_id, item_type)`

For `embeddings_other` table:
- **Item index only**: `CREATE INDEX IF NOT EXISTS idx_embeddings_other_item ON embeddings_other(item_id, item_type)`

### Schema Constants

The following new constants have been added to `schema.rs`:

1. `CREATE_EMBEDDINGS_384_TABLE`
2. `CREATE_EMBEDDINGS_1024_TABLE`
3. `CREATE_EMBEDDINGS_1536_TABLE`
4. `CREATE_EMBEDDINGS_3072_TABLE`
5. `CREATE_EMBEDDINGS_OTHER_TABLE`
6. Vector indexes for each dimension (384, 1024, 1536, 3072)
7. Item indexes for all tables (including "other")

## 3. Routing Logic Design

### Core Routing Function

```rust
/// Get table name and native vector support flag for a given dimension
fn get_table_for_dimension(dimension: usize) -> (&'static str, bool) {
    match dimension {
        384 => ("embeddings_384", true),
        1024 => ("embeddings_1024", true),
        1536 => ("embeddings_1536", true),
        3072 => ("embeddings_3072", true),
        _ => ("embeddings_other", false),
    }
}
```

### Storage Methods to Update

The following methods in `memory-storage-turso/src/storage.rs` need to be updated to use dimension-specific tables:

#### 1. `store_embedding` (line ~1080)
- Current: Uses hardcoded `embeddings` table
- Update: Call `get_table_for_dimension(embedding.len())` to get target table
- Update SQL generation to use appropriate table name
- For native dimensions: Use `vector32(?)` with appropriate column
- For other dimensions: Store as BLOB (no vector32)

#### 2. `get_embedding` (line ~1130)
- Current: Queries `embeddings` table by `embedding_id`
- Challenge: Need to know dimension to query correct table
- Options:
  a) Store dimension mapping in separate metadata table
  b) Search across all dimension tables (fallback)
  c) Include dimension in `embedding_id` format
- Recommended: Add `dimension` parameter to method signature (breaking change) or infer from existing data by searching all tables

#### 3. `store_embedding_backend` (line ~1162)
- Similar to `store_embedding` but uses `StorageBackend` trait
- Update to route based on dimension

#### 4. `get_embedding_backend` (line ~1221)
- Similar challenges as `get_embedding`

#### 5. Batch operations (`store_embeddings_batch_backend`, `get_embeddings_batch_backend`)
- Need to handle mixed dimensions within a single batch
- Option: Group by dimension and execute per table

#### 6. `find_similar_episodes_native` (line ~1495)
- Current: Uses `embeddings` table and `idx_embeddings_vector` index
- Update: Use dimension-specific table based on `query_embedding.len()`
- Update SQL to use appropriate table and index names
- For non-native dimensions (other), fall back to brute-force search

#### 7. `find_similar_patterns_native` (line ~1685)
- Same changes as episodes

#### 8. `delete_embedding_backend`
- Need to know which table contains the embedding
- Could search across all tables or maintain mapping

### Backward Compatibility Considerations

1. **Existing Data**: The original `embeddings` table remains for backward compatibility during migration
2. **Dual-Write Phase**: Optionally write to both old and new tables during transition
3. **Read Fallback**: If embedding not found in new tables, fall back to old table
4. **Migration Flag**: Configuration flag to control migration behavior

## 4. Migration Script Design

### Purpose
Backfill existing embeddings from the original `embeddings` table to dimension-specific tables.

### Script Location
`scripts/migrate_embeddings_to_multi_dim.rs`

### Algorithm
```rust
// 1. Connect to database
// 2. Read all rows from embeddings table
// 3. For each row:
//    a. Parse dimension from dimension column
//    b. Determine target table using get_table_for_dimension()
//    c. Insert into target table
//       - For native dimensions: Convert embedding_data to vector32() format
//       - For other dimensions: Store embedding_vector as BLOB (or NULL)
// 4. Verify counts match: SUM(all dimension tables) == COUNT(embeddings)
// 5. Optional: Create triggers to keep tables in sync during transition
// 6. Optional: Drop old table after verification (configurable)
```

### Safety Features
- Transaction-based migration (rollback on error)
- Progress logging with checkpoint restart capability
- Dry-run mode to preview changes
- Validation step to ensure data integrity
- Backup of original table before migration

### Integration Points
- Called from `initialize_schema()` with feature flag
- Optional CLI command in `memory-cli`
- Integration test suite

## 5. Embedding Dimension Review

### Current Implementation Status

Reviewed `embedding_dimension()` implementations in `memory-core/src/embeddings/provider.rs`:

#### 1. `OpenAIEmbeddingProvider` (openai.rs)
- Returns `self.config.embedding_dimension`
- Configurations:
  - `openai_ada_002()`: 1536
  - `openai_3_small()`: 1536  
  - `openai_3_large()`: 3072
- ✅ Correctly reports dimensions

#### 2. `LocalEmbeddingProvider` (local.rs)
- Returns `self.config.embedding_dimension`
- Default configuration: 384 (sentence-transformers/all-MiniLM-L6-v2)
- ✅ Correctly reports dimensions

#### 3. Other Providers (Cohere, Mistral via OpenAIEmbeddingProvider)
- Mistral configuration: 1024 dimensions
- ✅ Correctly reports dimensions

### Issues Identified

1. **Feature Flag Mismatch**: Warnings about `openai-embeddings` and `cohere-embeddings` feature names not matching Cargo.toml expectations
   - Impact: Compilation warnings only, no functional issues
   - Recommendation: Update feature names in Cargo.toml or constants.rs

2. **Azure OpenAI Custom Dimensions**: Azure configurations allow custom dimensions via `azure_openai()` constructor
   - This could produce dimensions not in our predefined set (384, 1024, 1536, 3072)
   - These will be routed to `embeddings_other` table (no native vector support)

3. **Custom Model Dimensions**: `ModelConfig::custom()` allows any dimension
   - Same handling as Azure custom dimensions

### Recommendations

1. **Update Feature Flags**: Align feature names with Cargo.toml to eliminate warnings
2. **Dimension Validation**: Consider adding validation in provider constructors to warn about non-native dimensions
3. **Provider Metadata**: Enhance `metadata()` method to include native vector support flag

## 6. Implementation Plan

### Phase 1: Schema Creation (Complete)
- ✅ Added new table definitions to schema.rs
- ✅ Added new index definitions

### Phase 2: Routing Logic Implementation
1. Implement `get_table_for_dimension()` helper
2. Update `store_embedding()` and `store_embedding_backend()`
3. Update `get_embedding()` with dimension parameter (breaking change requires careful coordination)
4. Update batch operations
5. Update similarity search methods
6. Add tests for routing logic

### Phase 3: Schema Initialization Updates
1. Update `initialize_schema()` in lib.rs to create new tables and indexes
2. Add migration path (optional)
3. Add feature flag control

### Phase 4: Migration Script Implementation
1. Implement migration script
2. Add CLI integration
3. Create comprehensive tests

### Phase 5: Performance Validation
1. Benchmark native vector search for each dimension
2. Compare with brute-force baseline
3. Verify performance improvements meet targets

## 7. Quality Gates

### Code Quality
- Zero clippy warnings
- All existing tests pass
- >90% test coverage maintained

### Functional Requirements
- All embedding dimensions supported (native where possible)
- No data loss during migration
- Backward compatibility maintained
- Performance improvements documented

### Performance Targets
- 384-dim search: < 5ms (current: ~5ms)
- 1536-dim search: < 10ms (current: ~50ms brute-force)
- 3072-dim search: < 20ms (baseline to be established)

## 8. Risks and Mitigations

### Technical Risks
1. **Schema Migration Complexity**: Multiple tables increase complexity
   - Mitigation: Thorough testing, rollback capability
2. **Mixed Dimension Batches**: Batch operations with mixed dimensions
   - Mitigation: Group by dimension, execute per table
3. **Query Dimension Mismatch**: Searching across dimensions not comparable
   - Mitigation: Validate query dimension matches table dimension

### Integration Risks
1. **Breaking API Changes**: `get_embedding()` may require dimension parameter
   - Mitigation: Overloaded method or new method, deprecation strategy
2. **Performance Regression**: Additional table lookups could slow operations
   - Mitigation: Benchmarking, optimization

## 9. Next Steps

### Immediate (Phase 0 Completion)
1. ✅ Review current schema and embedding dimension implementations
2. ✅ Design multi-dimension schema approach (Option B)
3. ✅ Update schema.rs with new table definitions
4. Provide design to feature-implementer for implementation

### Phase 1 Implementation
1. Implement routing logic in storage.rs
2. Update schema initialization
3. Add comprehensive tests
4. Performance benchmarking

### Handoff to Feature-Implementer
The design is now ready for implementation. Key files to modify:
- `memory-storage-turso/src/schema.rs` (already updated)
- `memory-storage-turso/src/storage.rs` (routing logic)
- `memory-storage-turso/src/lib.rs` (schema initialization)
- `scripts/migrate_embeddings_to_multi_dim.rs` (new migration script)

## 10. Success Metrics

- [ ] 100% of embeddings use native vector search (currently ~50%)
- [ ] 2-10x faster search for OpenAI embeddings
- [ ] Zero data loss during migration
- [ ] All existing functionality preserved
- [ ] Comprehensive test coverage maintained