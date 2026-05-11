use do_memory_core::{MemoryConfig, SelfLearningMemory};
use do_memory_storage_redb::RedbStorage;
use std::sync::Arc;
use tempfile::TempDir;

/// Setup a memory system with redb storage backend for persistence testing
pub async fn setup_persistent_memory() -> anyhow::Result<(Arc<SelfLearningMemory>, TempDir)> {
    // Create temporary directory for redb file
    let temp_dir = TempDir::new()?;
    let redb_path = temp_dir.path().join("test_memory.redb");

    // Create redb storage (cache layer)
    let redb_storage: Arc<dyn do_memory_core::StorageBackend> =
        Arc::new(RedbStorage::new(&redb_path).await?);

    // Create memory system with redb storage
    // Note: For this test, we only use redb since Turso requires external setup
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig {
            quality_threshold: 0.0, // Zero threshold for test episodes
            ..Default::default()
        },
        redb_storage.clone(), // Use redb as both turso and cache for testing
        redb_storage,
    );

    Ok((Arc::new(memory), temp_dir))
}

/// Setup a memory system with DuckDB storage backend for persistence testing
#[cfg(feature = "duckdb")]
pub async fn setup_duckdb_persistent_memory() -> anyhow::Result<(Arc<SelfLearningMemory>, TempDir)>
{
    use do_memory_storage_duckdb::DuckDbStorage;

    // Create temporary directory for DuckDB file
    let temp_dir = TempDir::new()?;
    let duckdb_path = temp_dir.path().join("test_memory.duckdb");

    // Create DuckDB storage
    let duckdb_storage: Arc<dyn do_memory_core::StorageBackend> =
        Arc::new(DuckDbStorage::new(&duckdb_path).await?);

    // Create redb storage (cache layer)
    let redb_path = temp_dir.path().join("test_cache.redb");
    let redb_storage: Arc<dyn do_memory_core::StorageBackend> =
        Arc::new(do_memory_storage_redb::RedbStorage::new(&redb_path).await?);

    // Create memory system with DuckDB and redb cache
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig {
            quality_threshold: 0.0, // Zero threshold for test episodes
            ..Default::default()
        },
        duckdb_storage,
        redb_storage,
    );

    Ok((Arc::new(memory), temp_dir))
}
