# Database Schema

## Overview

The system uses two primary storage mechanisms:
1. **Turso (libSQL)**: Primary persistent storage (SQLite-based)
2. **Redb**: High-performance cache layer with Postcard serialization

## v0.1.7 Breaking Change: Postcard Serialization

**IMPORTANT**: Since v0.1.7, the cache layer uses **Postcard** for serialization instead of Bincode:

```rust
// ✅ Use Postcard (v0.1.7+)
use postcard::{from_bytes, to_allocvec};
let serialized = postcard::to_allocvec(&episode)?;
let deserialized: Episode = postcard::from_bytes(&serialized)?;

// ❌ Do NOT use Bincode (deprecated)
// let serialized = bincode::serialize(&episode)?;
```

**Why Postcard?**
- Safer: No unsafe code operations
- Smaller: More compact binary representation
- `#[no_std]` compatible
- Better performance for embedded use cases

## Turso Database Schema

### Episodes Table
```sql
CREATE TABLE IF NOT EXISTS episodes (
    id TEXT PRIMARY KEY,
    task_type TEXT NOT NULL,
    domain TEXT NOT NULL,
    context TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    success_rate REAL,
    reward_score REAL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    reflection TEXT,
    metadata TEXT,
    INDEX idx_episodes_domain (domain),
    INDEX idx_episodes_status (status),
    INDEX idx_episodes_created (created_at),
    INDEX idx_episodes_task_type (task_type)
);
```

### Steps Table
```sql
CREATE TABLE IF NOT EXISTS steps (
    id TEXT PRIMARY KEY,
    episode_id TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    action TEXT NOT NULL,
    result TEXT,
    duration_ms INTEGER,
    success BOOLEAN,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    tool_name TEXT,
    FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE,
    INDEX idx_steps_episode (episode_id),
    INDEX idx_steps_timestamp (timestamp),
    INDEX idx_steps_tool (tool_name)
);
```

### Patterns Table
```sql
CREATE TABLE IF NOT EXISTS patterns (
    id TEXT PRIMARY KEY,
    pattern_type TEXT NOT NULL,
    pattern_data TEXT NOT NULL,
    success_rate REAL NOT NULL,
    usage_count INTEGER DEFAULT 0,
    last_used DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT,
    INDEX idx_patterns_type (pattern_type),
    INDEX idx_patterns_success (success_rate),
    INDEX idx_patterns_usage (usage_count),
    INDEX idx_patterns_domain (domain)
);
```

### Embeddings Table
```sql
CREATE TABLE IF NOT EXISTS embeddings (
    id TEXT PRIMARY KEY,
    episode_id TEXT NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB NOT NULL,
    model_name TEXT NOT NULL,
    vector_dim INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE,
    INDEX idx_embeddings_episode (episode_id),
    INDEX idx_embeddings_model (model_name)
);
```

### Spatiotemporal Index Table
```sql
CREATE TABLE IF NOT EXISTS spatiotemporal_index (
    id TEXT PRIMARY KEY,
    episode_id TEXT NOT NULL,
    spatial_key TEXT NOT NULL,
    temporal_key TEXT NOT NULL,
    diversity_score REAL,
    FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE,
    INDEX idx_spatial (spatial_key),
    INDEX idx_temporal (temporal_key),
    INDEX idx_diversity (diversity_score)
);
```

## Redb Cache Schema

### Postcard Serialization Format
```rust
// All cache values use Postcard serialization
use postcard::to_allocvec;

// Episode Cache
table: "episodes"
key: episode_id (String)
value: Vec<u8> (Postcard-serialized Episode)

// Pattern Cache
table: "patterns"
key: pattern_type:hash (String)
value: Vec<u8> (Postcard-serialized Pattern)

// Embedding Cache
table: "embeddings"
key: episode_id (String)
value: Vec<u8> (Postcard-serialized Embedding)

// Query Cache
table: "queries"
key: query_hash (String)
value: Vec<u8> (Postcard-serialized QueryResult)
```

### Cache Tables Structure
```rust
// Episodes
episodes: {
    key: String,      // episode ID
    value: Vec<u8>,   // Postcard-serialized Episode
}

// Patterns
patterns: {
    key: String,      // pattern_type:hash
    value: Vec<u8>,   // Postcard-serialized Pattern
}

// Embeddings
embeddings: {
    key: String,      // episode ID
    value: Vec<u8>,   // Postcard-serialized Embedding
}

// Queries
queries: {
    key: String,      // query hash
    value: Vec<u8>,   // Postcard-serialized QueryResult
}

// Metadata
metadata: {
    key: String,      // metadata key
    value: String,   // metadata value
}
```

### TTL Configuration
- **Episodes**: 1 hour default
- **Patterns**: 30 minutes default
- **Queries**: 5 minutes default
- **Embeddings**: 2 hours default

## Data Relationships

```
Episodes (1) ──── (N) Steps
    │                   │
    │                   │
    │                   │
    └── (1) ──── (N) Embeddings
         │
         │
         └── (1) ──── (N) Patterns
         │
         └── (1) ──── (N) Spatiotemporal Index
```

## Indexes and Performance

### Primary Indexes
- **Episodes**: `domain`, `status`, `created_at`, `task_type`
- **Steps**: `episode_id`, `timestamp`, `tool_name`
- **Patterns**: `pattern_type`, `success_rate`, `usage_count`, `domain`
- **Embeddings**: `episode_id`, `model_name`
- **Spatiotemporal**: `spatial_key`, `temporal_key`, `diversity_score`

### Query Optimization
```sql
-- Fast episode retrieval by domain and time
SELECT * FROM episodes
WHERE domain = ? AND created_at > ?
ORDER BY created_at DESC
LIMIT ?;

-- Pattern search by type and success rate
SELECT * FROM patterns
WHERE pattern_type = ? AND success_rate > ?
ORDER BY success_rate DESC;

-- Spatiotemporal query with diversity
SELECT e.*, st.diversity_score
FROM episodes e
JOIN spatiotemporal_index st ON e.id = st.episode_id
WHERE st.spatial_key = ? AND st.temporal_key = ?
ORDER BY st.diversity_score DESC
LIMIT ?;
```

### Full-Text Search
```sql
-- Enable FTS for content search
CREATE VIRTUAL TABLE IF NOT EXISTS episodes_fts USING fts5(
    task_type, domain, context, content='episodes', content_rowid='rowid'
);
```

## Connection Pooling

### Turso Connection Pool
```rust
// Semaphore-based connection limiting
use tokio::sync::Semaphore;
use libsql::Connection;

pub struct TursoPool {
    database_url: String,
    semaphore: Arc<Semaphore>,  // Default: 10 permits
}

impl TursoPool {
    pub async fn get_connection(&self) -> Result<Connection> {
        let _permit = self.semaphore.acquire().await?;
        let conn = libsql::connect(self.database_url.clone()).await?;
        Ok(conn)
    }
}
```

### Configuration
```bash
# Connection pool size (default: 10)
TURSO_POOL_SIZE=10

# Connection timeout (default: 30s)
TURSO_CONNECTION_TIMEOUT=30

# Max connection lifetime (default: 1h)
TURSO_MAX_CONNECTION_LIFETIME=3600
```

## Data Types and Validation

### Episode Status Values
- `'active'` - Episode in progress
- `'completed'` - Successfully completed
- `'failed'` - Failed to complete
- `'archived'` - Moved to long-term storage

### Task Types
- `'code_generation'`
- `'debugging'`
- `'refactoring'`
- `'testing'`
- `'analysis'`
- `'documentation'`

### Pattern Types
- `'tool_sequence'` - Sequence of tool usage
- `'error_resolution'` - Error handling patterns
- `'optimization'` - Performance improvements
- `'testing_strategy'` - Testing approaches
- `'decision_point'` - Decision-making patterns
- `'context_pattern'` - Contextual patterns

### Embedding Models
- `'text-embedding-3-small'` (OpenAI)
- `'text-embedding-3-large'` (OpenAI)
- `'text-embedding-ada-002'` (OpenAI)
- `'embed-english-v3.0'` (Cohere)
- `'nomic-embed-text-v1'` (Ollama)
- Local models via ONNX Runtime

## Synchronization Strategy

### Dual Storage Synchronization
```rust
// Write: Store to both Turso and redb
pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
    // Store in Turso (durable)
    self.turso.store_episode(episode).await?;

    // Store in redb cache (fast)
    let serialized = postcard::to_allocvec(episode)?;
    self.redb.set(&episode.id, &serialized).await?;

    Ok(())
}

// Read: Check cache first, fallback to Turso
pub async fn get_episode(&self, id: &str) -> Result<Option<Episode>> {
    // Check redb cache
    if let Some(cached) = self.redb.get(id).await? {
        let episode = postcard::from_bytes(&cached)?;
        return Ok(Some(episode));
    }

    // Fallback to Turso
    if let Some(episode) = self.turso.get_episode(id).await? {
        // Update cache
        let serialized = postcard::to_allocvec(&episode)?;
        self.redb.set(id, &serialized).await?;
        return Ok(Some(episode));
    }

    Ok(None)
}
```

### Cache Invalidation
- **Write-through**: Cache updated on every write
- **TTL-based**: Automatic expiration
- **Manual**: Explicit invalidation on update
- **Size-based**: LRU eviction when full

## Migration Scripts

### Version 0.1.7 (Postcard Migration, current: v0.1.13)
```sql
-- Add embedding model info
ALTER TABLE embeddings ADD COLUMN model_name TEXT;
ALTER TABLE embeddings ADD COLUMN vector_dim INTEGER;

-- Add spatiotemporal index
CREATE TABLE spatiotemporal_index (...);
```

**Cache Migration**:
```rust
// On startup, check cache format
// If old bincode format, re-serialize with postcard
async fn migrate_cache() -> Result<()> {
    // Check version
    let version = self.redb.get("metadata:version").await?;

    if version != Some("0.1.12".to_string()) {
        // Migrate to postcard (from v0.1.7, current: v0.1.13)
        self.redb.clear().await?;
        self.redb.set("metadata:version", "0.1.12").await?;
    }

    Ok(())
}
```

## Backup and Recovery

### Turso Backup
```bash
# Export database
sqlite3 memory.db ".backup backup.db"

# Import database
sqlite3 memory.db ".restore backup.db"

# Online backup (libSQL)
curl -X POST https://turso.example.com/backups/create
```

### Redb Backup
```bash
# Copy redb files
cp cache.redb cache.redb.backup

# Rebuild from Turso if cache lost
# (automatic on startup)
```

### Recovery Procedures
1. **Database Corruption**: Restore from backup
2. **Cache Loss**: Rebuild from primary database (automatic)
3. **Partial Data Loss**: Use episode IDs to rebuild relationships
4. **Postcard Migration**: Automatic on version mismatch

## Performance Tuning

### Turso Optimization
```sql
-- Analyze query performance
EXPLAIN QUERY PLAN SELECT * FROM episodes WHERE domain = ?;

-- Update table statistics
ANALYZE episodes;
ANALYZE steps;
ANALYZE patterns;

-- Reindex tables
REINDEX episodes;
REINDEX steps;
```

### Redb Optimization
- Monitor cache hit rates (>80% target)
- Adjust TTL values based on access patterns
- Use compression for large payloads (Postcard is already compact)
- Tune cache size (default: 100MB)

### Connection Pool Tuning
```bash
# Increase for high concurrency
TURSO_POOL_SIZE=20

# Decrease for low memory
TURSO_POOL_SIZE=5
```

## Security Considerations

### Data Encryption
- Sensitive episode data encrypted at rest (optional)
- API keys and tokens stored in environment variables
- Communication encrypted (TLS in production)

### Access Control
- Database access restricted to application
- No external direct database access
- Audit logging for all data operations
- Parameterized queries (SQL injection prevention)

### SQL Injection Prevention
```rust
// ✅ Use parameterized queries
db.query(
    "SELECT * FROM episodes WHERE id = ?1",
    &[episode_id]
).await?;

// ❌ Never string interpolation
db.query(&format!(
    "SELECT * FROM episodes WHERE id = '{}'",
    episode_id
), &[]).await?;
```

### Privacy
- Episode data isolated by domain
- Configurable data retention
- GDPR compliance through deletion
- Optional encryption for sensitive data

### Postcard Security
- Postcard is safer than bincode (no unsafe code)
- Still validate input before deserialization
- Use bounded reads to prevent denial of service
- Consider size limits for cached values