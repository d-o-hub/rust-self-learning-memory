//! Storage combination determination and fallback logic.

use super::StorageType;
#[cfg(feature = "turso")]
use super::{ensure_directory_exists, extract_db_path};
use crate::config::types::Config;
use anyhow::Result;
use do_memory_core::{MemoryConfig, SelfLearningMemory, StorageBackend};
use std::sync::Arc;

pub(super) async fn determine_storage_combination(
    config: &Config,
    turso_storage: Option<Arc<dyn StorageBackend>>,
    redb_storage: Option<Arc<dyn StorageBackend>>,
    _turso_messages: Vec<String>,
    _redb_messages: Vec<String>,
) -> Result<(StorageType, StorageType, SelfLearningMemory)> {
    let memory_config = super::create_memory_config(config);
    let memory_config_clone = memory_config.clone();

    let storage_combination = match (turso_storage, redb_storage) {
        (Some(turso), Some(redb)) => {
            let memory = SelfLearningMemory::with_storage(memory_config, turso, redb);
            (StorageType::Turso, StorageType::Redb, memory)
        }
        (Some(turso), None) => {
            #[cfg(feature = "redb")]
            {
                // Use a proper temporary file for the ephemeral redb cache.
                // Note: `:memory:` is NOT a special path for redb (unlike SQLite);
                // it would create a literal file named `:memory:` in the current
                // working directory. Using tempfile ensures proper cleanup.
                let temp_dir = tempfile::tempdir().map_err(|e| {
                    anyhow::anyhow!("Failed to create temp dir for redb cache: {}", e)
                })?;
                let temp_path = temp_dir.path().join("cache.redb");
                let temp_redb = do_memory_storage_redb::RedbStorage::new(&temp_path).await?;
                // Leak the temp dir so the file isn't deleted while in use.
                // The process will clean up on exit.
                std::mem::forget(temp_dir);
                let memory =
                    SelfLearningMemory::with_storage(memory_config, turso, Arc::new(temp_redb));
                (StorageType::Turso, StorageType::Memory, memory)
            }
            #[cfg(not(feature = "redb"))]
            {
                let memory = SelfLearningMemory::with_config(memory_config);
                (StorageType::Turso, StorageType::Memory, memory)
            }
        }
        (None, Some(redb)) => {
            #[cfg(feature = "turso")]
            {
                match try_setup_local_sqlite_for_redis(redb.clone(), memory_config_clone.clone())
                    .await
                {
                    Ok((storage_type, memory)) => (storage_type, StorageType::Redb, memory),
                    Err(_) => create_redb_only_memory(memory_config_clone, redb),
                }
            }
            #[cfg(not(feature = "turso"))]
            {
                create_redb_only_memory(memory_config_clone, redb)
            }
        }
        (None, None) => {
            #[cfg(feature = "turso")]
            {
                match try_setup_fallback_storage(memory_config.clone()).await {
                    Ok((primary, cache, memory)) => (primary, cache, memory),
                    Err(_) => (
                        StorageType::Memory,
                        StorageType::Memory,
                        SelfLearningMemory::with_config(memory_config_clone),
                    ),
                }
            }
            #[cfg(not(feature = "turso"))]
            {
                (
                    StorageType::Memory,
                    StorageType::Memory,
                    SelfLearningMemory::with_config(memory_config_clone),
                )
            }
        }
    };

    Ok(storage_combination)
}

/// Try to set up local SQLite for redb-only configuration
#[cfg(feature = "turso")]
async fn try_setup_local_sqlite_for_redis(
    redb_storage: Arc<dyn StorageBackend>,
    memory_config: MemoryConfig,
) -> Result<(StorageType, SelfLearningMemory)> {
    if let Ok(local_db_url) = std::env::var("LOCAL_DATABASE_URL") {
        if local_db_url.starts_with("sqlite:") || local_db_url.starts_with("file:") {
            let db_path = extract_db_path(&local_db_url);
            ensure_directory_exists(db_path).await?;

            match do_memory_storage_turso::TursoStorage::new(&format!("file:{}", db_path), "").await
            {
                Ok(turso_storage) => {
                    if let Err(e) = turso_storage.initialize_schema().await {
                        eprintln!("Warning: Failed to initialize local SQLite schema: {}", e);
                        Ok(create_redb_only_memory_system(memory_config, redb_storage))
                    } else {
                        eprintln!("Using local SQLite database: {}", db_path);
                        let memory = SelfLearningMemory::with_storage(
                            memory_config,
                            Arc::new(turso_storage),
                            redb_storage,
                        );
                        Ok((StorageType::LocalSqlite, memory))
                    }
                }
                Err(_) => Ok(create_redb_only_memory_system(memory_config, redb_storage)),
            }
        } else {
            Ok(create_redb_only_memory_system(memory_config, redb_storage))
        }
    } else {
        Ok(create_redb_only_memory_system(memory_config, redb_storage))
    }
}

fn create_redb_only_memory(
    memory_config: MemoryConfig,
    redb_storage: Arc<dyn StorageBackend>,
) -> (StorageType, StorageType, SelfLearningMemory) {
    let memory =
        SelfLearningMemory::with_storage(memory_config, Arc::clone(&redb_storage), redb_storage);
    (StorageType::Redb, StorageType::Redb, memory)
}

#[cfg(feature = "turso")]
fn create_redb_only_memory_system(
    memory_config: MemoryConfig,
    redb_storage: Arc<dyn StorageBackend>,
) -> (StorageType, SelfLearningMemory) {
    let (_, _, memory) = create_redb_only_memory(memory_config, redb_storage);
    (StorageType::Redb, memory)
}

/// Try to set up fallback storage when no explicit configuration
#[cfg(feature = "turso")]
#[cfg(feature = "redb")]
async fn create_fallback_with_redb(
    turso_storage: do_memory_storage_turso::TursoStorage,
    memory_config: MemoryConfig,
) -> Result<(StorageType, StorageType, SelfLearningMemory)> {
    let temp_dir = tempfile::tempdir()
        .map_err(|e| anyhow::anyhow!("Failed to create temp dir for redb cache: {}", e))?;
    let temp_path = temp_dir.path().join("cache.redb");
    let temp_redb = do_memory_storage_redb::RedbStorage::new(&temp_path).await?;
    std::mem::forget(temp_dir);
    let memory = SelfLearningMemory::with_storage(
        memory_config,
        Arc::new(turso_storage),
        Arc::new(temp_redb),
    );
    Ok((StorageType::LocalSqlite, StorageType::Memory, memory))
}

#[cfg(feature = "turso")]
async fn try_setup_fallback_storage(
    memory_config: MemoryConfig,
) -> Result<(StorageType, StorageType, SelfLearningMemory)> {
    if let Ok(local_db_url) = std::env::var("LOCAL_DATABASE_URL") {
        if local_db_url.starts_with("sqlite:") || local_db_url.starts_with("file:") {
            let db_path = extract_db_path(&local_db_url);
            ensure_directory_exists(db_path).await?;

            match do_memory_storage_turso::TursoStorage::new(&format!("file:{}", db_path), "").await
            {
                Ok(turso_storage) => {
                    if let Err(e) = turso_storage.initialize_schema().await {
                        eprintln!("Warning: Failed to initialize local SQLite schema: {}", e);
                        Ok((
                            StorageType::Memory,
                            StorageType::Memory,
                            SelfLearningMemory::with_config(memory_config),
                        ))
                    } else {
                        eprintln!("Using local SQLite database: {}", db_path);

                        #[cfg(feature = "redb")]
                        return create_fallback_with_redb(turso_storage, memory_config).await;
                        #[cfg(not(feature = "redb"))]
                        {
                            let turso_arc = Arc::new(turso_storage);
                            let temp_dir = tempfile::tempdir().map_err(|e| {
                                anyhow::anyhow!("Failed to create temp dir for redb cache: {}", e)
                            })?;
                            let temp_path = temp_dir.path().join("cache.redb");
                            let memory = SelfLearningMemory::with_storage(
                                memory_config,
                                turso_arc,
                                Arc::new(
                                    do_memory_storage_redb::RedbStorage::new(&temp_path).await?,
                                ),
                            );
                            std::mem::forget(temp_dir);
                            Ok((StorageType::LocalSqlite, StorageType::Memory, memory))
                        }
                    }
                }
                Err(_) => Ok((
                    StorageType::Memory,
                    StorageType::Memory,
                    SelfLearningMemory::with_config(memory_config),
                )),
            }
        } else {
            Ok((
                StorageType::Memory,
                StorageType::Memory,
                SelfLearningMemory::with_config(memory_config),
            ))
        }
    } else {
        Ok((
            StorageType::Memory,
            StorageType::Memory,
            SelfLearningMemory::with_config(memory_config),
        ))
    }
}
