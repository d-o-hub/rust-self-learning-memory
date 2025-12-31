# Migration Script Design: Embeddings to Multi-Dimension Tables

## Overview

Migration script to backfill existing embeddings from the original `embeddings` table to dimension-specific tables.

## Script Location

`scripts/migrate_embeddings_to_multi_dim.rs`

## Dependencies

- `memory-storage-turso` crate for database connection
- `clap` for CLI argument parsing (optional)
- `tracing` for logging

## Command-Line Interface

```bash
# Dry run (preview changes)
cargo run --bin migrate_embeddings -- --dry-run

# Execute migration
cargo run --bin migrate_embeddings -- --execute

# Execute with backup
cargo run --bin migrate_embeddings -- --execute --backup

# Execute and drop old table (dangerous)
cargo run --bin migrate_embeddings -- --execute --drop-old-table
```

## Algorithm

### Phase 1: Preparation
1. Connect to database
2. Verify source table (`embeddings`) exists
3. Verify target tables exist (create if missing)
4. Record initial counts for verification
5. Create backup table if requested

### Phase 2: Data Migration
```rust
// Pseudocode
let mut migrated_count = 0;
let mut error_count = 0;

let rows = conn.query("SELECT * FROM embeddings", ()).await?;

while let Some(row) = rows.next().await? {
    let embedding_id: String = row.get(0)?;
    let item_id: String = row.get(1)?;
    let item_type: String = row.get(2)?;
    let embedding_data: String = row.get(3)?;
    let embedding_vector: Option<Vec<u8>> = row.get(4)?;
    let dimension: i64 = row.get(5)?;
    let model: String = row.get(6)?;
    let created_at: i64 = row.get(7)?;
    
    // Parse embedding from JSON
    let embedding: Vec<f32> = serde_json::from_str(&embedding_data)?;
    
    // Determine target table
    let (target_table, native_support) = get_table_for_dimension(dimension as usize);
    
    // Prepare insert statement
    let sql = if native_support {
        // Convert embedding to vector string
        let vector_str = format!("[{}]", embedding.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", "));
        
        format!(
            "INSERT OR REPLACE INTO {} (embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model, created_at) VALUES (?, ?, ?, ?, vector32(?), ?, ?, ?)",
            target_table
        )
    } else {
        // Store as BLOB
        let blob_data = embedding.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>();
        
        format!(
            "INSERT OR REPLACE INTO {} (embedding_id, item_id, item_type, embedding_data, embedding_vector, dimension, model, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            target_table
        )
    };
    
    // Execute insert
    match conn.execute(&sql, params![...]).await {
        Ok(_) => migrated_count += 1,
        Err(e) => {
            error_count += 1;
            tracing::error!("Failed to migrate embedding {}: {}", embedding_id, e);
        }
    }
    
    // Progress reporting
    if migrated_count % 1000 == 0 {
        tracing::info!("Migrated {} embeddings...", migrated_count);
    }
}
```

### Phase 3: Verification
1. Count rows in each target table
2. Compare sum with original count
3. Validate sample embeddings match
4. Report discrepancies

### Phase 4: Cleanup (Optional)
1. Create backup of old table (if not already)
2. Drop old table (if `--drop-old-table` flag)
3. Update metadata table with migration timestamp

## Error Handling

### Retry Logic
- Transient errors: exponential backoff with max retries
- Permanent errors: log and continue, record in error log

### Data Integrity
- Use transactions for each batch (e.g., 1000 rows)
- Rollback on batch failure
- Resume from checkpoint using last successful embedding_id

### Validation
- Compare embedding vectors before/after migration
- Verify dimension column matches table
- Check for duplicate embedding_id across tables

## Performance Considerations

### Batch Size
- Default: 1000 rows per transaction
- Configurable via CLI argument
- Memory usage: ~4MB per 1000 embeddings (1536-dim)

### Parallel Processing
- Process different dimension tables in parallel
- Use connection pooling for concurrent inserts

### Progress Monitoring
- Real-time progress bar
- Estimated time remaining
- Resource usage reporting

## Rollback Strategy

### Automatic Rollback
- Transaction-based: uncommitted changes automatically rolled back
- Manual intervention required for committed changes

### Manual Rollback
```sql
-- If migration fails, restore from backup
DROP TABLE IF EXISTS embeddings;
ALTER TABLE embeddings_backup RENAME TO embeddings;
```

### Partial Rollback
- Migration script creates checkpoint file
- Can resume from last checkpoint
- Can rollback specific batches

## Integration with System

### As Part of Schema Initialization
```rust
// In initialize_schema()
if should_migrate_embeddings() {
    migrate_embeddings_to_multi_dim().await?;
}
```

### Standalone Binary
```toml
# Cargo.toml
[[bin]]
name = "migrate_embeddings"
path = "scripts/migrate_embeddings_to_multi_dim.rs"
```

### CLI Integration
```bash
memory-cli migrate-embeddings --dry-run
```

## Testing Strategy

### Unit Tests
- Mock database connection
- Test dimension routing logic
- Test error handling

### Integration Tests
- Real database (in-memory SQLite)
- Test full migration workflow
- Verify data integrity

### Performance Tests
- Benchmark migration speed
- Measure memory usage
- Scalability with large datasets

## Success Criteria

1. **Completeness**: All embeddings migrated
2. **Accuracy**: Data matches exactly (bit-for-bit)
3. **Performance**: Migration completes within reasonable time (e.g., 1M embeddings/hour)
4. **Safety**: No data loss, rollback possible
5. **Observability**: Comprehensive logging and progress reporting

## Risk Mitigation

### Data Loss Risk
- Backup created before migration
- Transaction isolation
- Verification step before commit

### Performance Risk
- Batch size tuning
- Index management (disable during migration, rebuild after)
- Connection pooling

### Compatibility Risk
- Support for both old and new schema during transition
- Feature flag control
- Version checking

## Example Usage

```rust
// Programmatic usage
use memory_storage_turso::migration::migrate_embeddings;

let result = migrate_embeddings(
    &db_url,
    &auth_token,
    MigrationOptions {
        dry_run: false,
        backup: true,
        batch_size: 1000,
        drop_old_table: false,
    },
).await?;

println!("Migration completed: {:?}", result);
```

## Next Steps

1. Implement migration script prototype
2. Add unit tests
3. Integrate with CI/CD pipeline
4. Create documentation for users
5. Performance optimization