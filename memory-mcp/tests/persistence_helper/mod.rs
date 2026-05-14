//! Shared helpers for persistent storage tests

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
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig {
            quality_threshold: 0.0, // Zero threshold for test episodes
            ..Default::default()
        },
        redb_storage.clone(),
        redb_storage,
    );

    Ok((Arc::new(memory), temp_dir))
}
