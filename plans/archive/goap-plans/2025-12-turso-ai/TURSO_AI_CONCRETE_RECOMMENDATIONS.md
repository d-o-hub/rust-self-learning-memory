# Turso AI & Embeddings: Concrete Recommendations with File References

**Date**: 2025-12-29
**Status**: Actionable Implementation Items
**Priority**: High (Performance & Capability Enhancement)

## Executive Summary

Based on analysis of Turso's AI/embeddings features and SQLite extensions, here are concrete recommendations with specific file references. The current implementation (v0.1.9) uses basic Turso vector capabilities but misses significant optimization opportunities.

## 1. Multi-Dimension Vector Support (HIGH PRIORITY)

### Current Limitation
**File**: `memory-storage-turso/src/schema.rs:55-65`
```sql
embedding_vector F32_BLOB(384)  -- ❌ Fixed to 384 dimensions
```

**File**: `memory-storage-turso/src/storage.rs:1171-1215`
```rust
// Only use vector32() for 384-dim embeddings
let _sql = if dimension == 384 {
    // ... use vector32()
} else {
    // ... set embedding_vector = NULL ❌
};
```

**Impact**: OpenAI embeddings (1536-dim) cannot use native vector search, fall back to brute-force O(n) scan.

### Recommended Implementation

**Option B (Recommended)**: Separate tables per dimension

**File Modifications**:

1. **`memory-storage-turso/src/schema.rs`**:
```sql
-- Add after CREATE_EMBEDDINGS_TABLE
pub const CREATE_EMBEDDINGS_384_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS embeddings_384 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(384),  -- ✅ Native 384-dim
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

pub const CREATE_EMBEDDINGS_1536_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS embeddings_1536 (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    embedding_vector F32_BLOB(1536),  -- ✅ Native 1536-dim
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

-- Corresponding indexes
pub const CREATE_EMBEDDINGS_384_VECTOR_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_384_vector
ON embeddings_384(libsql_vector_idx(embedding_vector))
"#;

pub const CREATE_EMBEDDINGS_1536_VECTOR_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_1536_vector
ON embeddings_1536(libsql_vector_idx(embedding_vector))
"#;
```

2. **`memory-storage-turso/src/storage.rs`**:
- Update `store_embedding_backend()` to route to correct table based on dimension
- Update `find_similar_episodes_native()` to query appropriate table
- Add helper method `get_table_for_dimension(dim: usize) -> &str`

3. **`memory-core/src/embeddings/provider.rs`**:
- Add `embedding_dimension()` method to EmbeddingProvider trait (already exists)
- Ensure providers report correct dimensions

### Migration Script
Create `scripts/migrate_embeddings_to_multi_dim.rs`:
```rust
// Backfill existing embeddings to dimension-specific tables
// 1. Read from old embeddings table
// 2. Determine dimension from embedding_data JSON
// 3. Insert into embeddings_384 or embeddings_1536
// 4. Verify count matches
// 5. Optional: Drop old table after verification
```

## 2. Vector Index Optimization (MEDIUM PRIORITY)

### Current Implementation
**File**: `memory-storage-turso/src/schema.rs:71-78`
```sql
CREATE INDEX IF NOT EXISTS idx_embeddings_vector
ON embeddings(libsql_vector_idx(embedding_vector))  -- ❌ Default settings
```

### Recommended Optimization

**File**: `memory-storage-turso/src/schema.rs` (add new constants):
```sql
-- Optimized for episode search (high recall, frequent domain filtering)
pub const CREATE_EPISODE_VECTOR_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_episodes_vector
ON embeddings_384(libsql_vector_idx(embedding_vector,
    'metric=cosine',
    'compress_neighbors=float16',
    'max_neighbors=15',
    'alpha=1.1',
    'search_l=100'
)) WHERE item_type = 'episode'
"#;

-- Optimized for pattern search (high precision, lower volume)
pub const CREATE_PATTERN_VECTOR_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_patterns_vector
ON embeddings_384(libsql_vector_idx(embedding_vector,
    'metric=cosine',
    'compress_neighbors=float32',  -- No compression for max precision
    'max_neighbors=20',
    'alpha=1.3',
    'search_l=150'
)) WHERE item_type = 'pattern'
"#;
```

**Configuration File**: `memory-cli/config/storage.toml`
```toml
[turso.vector_index]
# Episode search optimization
episode_metric = "cosine"
episode_compress_neighbors = "float16"
episode_max_neighbors = 15
episode_alpha = 1.1
episode_search_l = 100

# Pattern search optimization  
pattern_metric = "cosine"
pattern_compress_neighbors = "float32"
pattern_max_neighbors = 20
pattern_alpha = 1.3
pattern_search_l = 150
```

## 3. Hybrid Search with FTS5 (MEDIUM PRIORITY)

### Current Limitation
No full-text search capability, only vector semantic search.

### Recommended Implementation

**New File**: `memory-storage-turso/src/fts5_schema.rs`
```rust
//! FTS5 virtual tables for hybrid search

pub const CREATE_EPISODES_FTS_TABLE: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS episodes_fts USING fts5(
    episode_id UNINDEXED,
    task_description,
    context,
    domain,
    tokenize='porter unicode61'
)
"#;

pub const CREATE_PATTERNS_FTS_TABLE: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS patterns_fts USING fts5(
    pattern_id UNINDEXED,
    pattern_data,
    context_domain,
    context_language,
    tokenize='porter unicode61'
)
"#;

// Synchronization triggers
pub const CREATE_EPISODES_FTS_TRIGGERS: &str = r#"
CREATE TRIGGER IF NOT EXISTS episodes_ai AFTER INSERT ON episodes BEGIN
    INSERT INTO episodes_fts(episode_id, task_description, context, domain)
    VALUES (new.episode_id, new.task_description, new.context, new.domain);
END;

CREATE TRIGGER IF NOT EXISTS episodes_au AFTER UPDATE ON episodes BEGIN
    UPDATE episodes_fts 
    SET task_description = new.task_description,
        context = new.context,
        domain = new.domain
    WHERE episode_id = new.episode_id;
END;

CREATE TRIGGER IF NOT EXISTS episodes_ad AFTER DELETE ON episodes BEGIN
    DELETE FROM episodes_fts WHERE episode_id = old.episode_id;
END;
"#;
```

**File Modification**: `memory-storage-turso/src/storage.rs`
- Add `search_hybrid()` method combining vector and FTS5 results
- Implement ranking algorithm: `alpha * vector_similarity + (1 - alpha) * fts_relevance`

**New File**: `memory-core/src/search/hybrid.rs`
```rust
pub struct HybridSearch {
    vector_weight: f32,  // 0.0 = keyword only, 1.0 = vector only
    fts_weight: f32,
}

impl HybridSearch {
    pub async fn search_episodes(&self, query: &str, limit: usize) -> Result<Vec<Episode>> {
        // 1. Get vector similarity results
        // 2. Get FTS5 keyword results  
        // 3. Combine using weights
        // 4. Return ranked results
    }
}
```

## 4. SQLite Extensions Integration (LOW PRIORITY)

### JSON Functions Enhancement

**File**: `memory-storage-turso/src/storage.rs` (query methods)
```rust
// Instead of deserializing JSON in Rust, query directly in SQL
pub async fn get_episodes_by_metadata_key(&self, key: &str, value: &str) -> Result<Vec<Episode>> {
    let sql = r#"
        SELECT * FROM episodes 
        WHERE json_extract(metadata, '$.' || ?) = ?
    "#;
    // ...
}
```

### SQLean Stats for Analytics

**New File**: `scripts/analytics.sql`
```sql
-- Usage statistics using SQLean Stats extension
SELECT 
    domain,
    COUNT(*) as total_episodes,
    AVG(json_extract(outcome, '$.success')) as avg_success_rate,
    median(json_extract(reward, '$.score')) as median_reward,
    stddev(json_extract(reward, '$.score')) as reward_stddev
FROM episodes
GROUP BY domain
ORDER BY avg_success_rate DESC;
```

### SQLean Crypto for Security

**File**: `memory-storage-turso/src/security.rs` (new)
```rust
pub fn hash_identifier(id: &str) -> String {
    // Use SQLean crypto functions via SQL
    // SELECT crypto_hash('sha256', ?) as hashed_id
}
```

### SQLean UUID Enhancement

**File Modification**: `memory-storage-turso/src/schema.rs`
```sql
-- Instead of TEXT for UUIDs, use UUID extension for validation
-- episode_id UUID PRIMARY KEY NOT NULL DEFAULT (uuid())
```

## 5. Performance Benchmark Suite (HIGH PRIORITY)

### New Benchmark Files

1. **`benches/turso_vector_performance.rs`**:
   - Compare native vs brute-force search
   - Measure different dimension performance
   - Test index optimization impact

2. **`benches/hybrid_search_performance.rs`**:
   - Vector-only vs hybrid search
   - Different weight configurations
   - Scaling with dataset size

3. **`benches/sqlite_extensions.rs`**:
   - JSON query performance vs Rust deserialization
   - FTS5 search speed
   - Extension overhead measurement

### Expected Results
| Operation | Current | Target | Improvement |
|-----------|---------|--------|-------------|
| OpenAI embedding search | ~50ms (brute) | ~5ms (native) | 10x |
| 384-dim search latency | ~5ms | ~2ms | 2.5x |
| Hybrid search relevance | N/A | Better than vector-only | Qualitative |
| JSON query performance | ~10ms (Rust) | ~2ms (SQL) | 5x |

## Implementation Order Recommendation

### Phase 1: Foundation (Week 1)
1. **Multi-dimension vector schema** (separate tables)
2. **Migration scripts** for existing data
3. **Basic benchmarks** established

### Phase 2: Optimization (Week 2)
1. **Vector index tuning** (episode vs pattern indexes)
2. **Performance validation** against benchmarks
3. **Configuration system** for index settings

### Phase 3: Enhancement (Week 3)
1. **FTS5 hybrid search** implementation
2. **JSON function usage** in queries
3. **SQLean Stats integration** for analytics

### Phase 4: Polish (Week 4)
1. **Security enhancements** (Crypto extension)
2. **UUID improvements** (validation/generation)
3. **Comprehensive documentation**

## Risk Mitigation Strategies

### Technical Risks
1. **Schema Migration**: Keep old table during transition, verify data integrity
2. **Performance Regression**: Feature flags, A/B testing capability
3. **Extension Compatibility**: Check Turso version requirements, fallback paths

### Integration Risks
1. **API Changes**: Maintain backward compatibility, deprecation warnings
2. **Configuration Complexity**: Sensible defaults, clear documentation
3. **Testing Coverage**: Maintain >90% coverage, add integration tests

## Success Metrics

### Quantitative
- ✅ 100% of embeddings use native vector search (currently ~50%)
- ✅ 2-10x faster search for OpenAI embeddings
- ✅ <100ms hybrid search latency for 10K episodes
- ✅ >95% search accuracy vs brute-force baseline

### Qualitative
- ✅ Hybrid search provides better relevance than vector-only
- ✅ JSON queries simplify application code
- ✅ Analytics provide actionable insights
- ✅ Security enhanced with proper hashing

## File Impact Summary

### Modified Files (14)
1. `memory-storage-turso/src/schema.rs` (+200 lines)
2. `memory-storage-turso/src/storage.rs` (+500 lines)
3. `memory-storage-turso/src/lib.rs` (+20 lines)
4. `memory-core/src/embeddings/mod.rs` (+100 lines)
5. `memory-core/src/memory/retrieval.rs` (+150 lines)
6. `memory-cli/config/storage.toml` (+50 lines)
7. `memory-cli/src/config/mod.rs` (+30 lines)
8. `Cargo.toml` (workspace) (+5 lines, feature flags)
9. `docs/STORAGE_OPTIMIZATION.md` (new)
10. `docs/HYBRID_SEARCH_GUIDE.md` (new)
11. `scripts/migrate_embeddings.rs` (new)
12. `benches/turso_vector_performance.rs` (new)
13. `benches/hybrid_search_performance.rs` (new)
14. `tests/integration/turso_extensions_test.rs` (new)

### New Modules (3)
1. `memory-storage-turso/src/fts5_schema.rs` (FTS5 tables/triggers)
2. `memory-core/src/search/hybrid.rs` (Hybrid ranking algorithm)
3. `memory-core/src/analytics/stats.rs` (Statistical queries)

## Conclusion

These concrete recommendations provide a clear path to significantly enhance the memory system's capabilities using Turso's advanced features. The implementation is structured to deliver measurable performance improvements while maintaining backward compatibility and system stability.

**Immediate Next Steps**:
1. Review and approve multi-dimension schema design (Option B)
2. Create detailed implementation tickets for Phase 1
3. Assign resources to rust-specialist and performance agents
4. Begin baseline performance measurements

---
*Recommendations v1.0 - 2025-12-29*
*Based on Turso documentation and current v0.1.9 implementation*