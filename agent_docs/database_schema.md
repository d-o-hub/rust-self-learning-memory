# Database Schema

## Overview

The system uses two primary storage mechanisms:
1. **Turso (libSQL)**: Primary persistent storage
2. **Redb**: High-performance cache layer

## Turso Database Schema

### Episodes Table
```sql
CREATE TABLE episodes (
    id TEXT PRIMARY KEY,
    task_type TEXT NOT NULL,
    domain TEXT NOT NULL,
    context TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    success_rate REAL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    metadata TEXT, -- JSON blob
    INDEX idx_episodes_domain (domain),
    INDEX idx_episodes_status (status),
    INDEX idx_episodes_created (created_at)
);
```

### Steps Table
```sql
CREATE TABLE steps (
    id TEXT PRIMARY KEY,
    episode_id TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    action TEXT NOT NULL,
    result TEXT,
    duration_ms INTEGER,
    success BOOLEAN,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE,
    INDEX idx_steps_episode (episode_id),
    INDEX idx_steps_timestamp (timestamp)
);
```

### Patterns Table
```sql
CREATE TABLE patterns (
    id TEXT PRIMARY KEY,
    pattern_type TEXT NOT NULL,
    pattern_data TEXT NOT NULL, -- JSON
    success_rate REAL NOT NULL,
    usage_count INTEGER DEFAULT 0,
    last_used DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT, -- JSON blob
    INDEX idx_patterns_type (pattern_type),
    INDEX idx_patterns_success (success_rate),
    INDEX idx_patterns_usage (usage_count)
);
```

### Embeddings Table
```sql
CREATE TABLE embeddings (
    id TEXT PRIMARY KEY,
    episode_id TEXT NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB NOT NULL, -- Vector data
    model_name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE,
    INDEX idx_embeddings_episode (episode_id)
);
```

## Redb Cache Schema

### Episode Cache
```
Key: "episode:{id}"
Value: Serialized Episode struct
TTL: Configurable (default: 1 hour)
```

### Pattern Cache
```
Key: "pattern:{type}:{hash}"
Value: Serialized Pattern struct
TTL: Configurable (default: 30 minutes)
```

### Query Cache
```
Key: "query:{hash}"
Value: Search results
TTL: Short (default: 5 minutes)
```

## Data Relationships

```
Episodes (1) ──── (N) Steps
    │                   │
    │                   │
    │                   │
    └── (1) ──── (N) Embeddings
         │
         │
         └── (N) ──── (N) Patterns (many-to-many)
```

## Indexes and Performance

### Primary Indexes
- **Episodes**: `domain`, `status`, `created_at`
- **Steps**: `episode_id`, `timestamp`
- **Patterns**: `pattern_type`, `success_rate`
- **Embeddings**: `episode_id`

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
```

### Full-Text Search
```sql
-- Enable FTS for content search
CREATE VIRTUAL TABLE episodes_fts USING fts5(
    task_type, domain, context, content='episodes', content_rowid='rowid'
);
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

## Migration Scripts

### Version 1.0.0
```sql
-- Initial schema
CREATE TABLE episodes (...);
CREATE TABLE steps (...);
-- ... other tables
```

### Version 1.1.0
```sql
-- Add embeddings support
ALTER TABLE episodes ADD COLUMN embedding_version TEXT;
CREATE TABLE embeddings (...);
```

## Backup and Recovery

### Turso Backup
```bash
# Export database
sqlite3 memory.db ".backup backup.db"

# Import database
sqlite3 memory.db ".restore backup.db"
```

### Redb Backup
```bash
# Copy redb files
cp cache.redb cache.redb.backup
```

### Recovery Procedures
1. **Database Corruption**: Restore from backup
2. **Cache Loss**: Rebuild from primary database
3. **Partial Data Loss**: Use episode IDs to rebuild relationships

## Performance Tuning

### Turso Optimization
```sql
-- Analyze query performance
EXPLAIN QUERY PLAN SELECT * FROM episodes WHERE domain = ?;

-- Update table statistics
ANALYZE episodes;
ANALYZE steps;
```

### Redb Optimization
- Monitor cache hit rates
- Adjust TTL values based on access patterns
- Use compression for large payloads

## Security Considerations

### Data Encryption
- Sensitive episode data encrypted at rest
- API keys and tokens stored securely
- Communication encrypted (TLS)

### Access Control
- Database access restricted to application
- No external systems direct database access from
- Audit logging for all data operations

### Privacy
- Episode data isolated by domain
- Configurable data retention
- GDPR compliance through deletion