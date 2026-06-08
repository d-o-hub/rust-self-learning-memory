//! Storage initialization module – unified backend setup logic.

mod storage_config;

use super::types::{Config, DatabaseConfig};
use anyhow::{Context, Result};
use do_memory_core::{MemoryConfig, SelfLearningMemory, StorageBackend};
use std::sync::Arc;
pub use storage_config::create_memory_config;

/// Storage initialization result with detailed information
pub struct StorageInitResult {
    /// Successfully initialized memory system
    pub memory: SelfLearningMemory,
    /// Information about what storage backends are being used
    pub storage_info: StorageInfo,
}

/// Information about configured storage backends
#[derive(Debug)]
pub struct StorageInfo {
    /// Primary storage backend type
    pub primary_storage: StorageType,
    /// Cache storage backend type
    pub cache_storage: StorageType,
    /// Detailed status messages
    pub status_messages: Vec<String>,
}

/// Storage backend type enumeration
#[derive(Debug, Clone)]
pub enum StorageType {
    /// Turso/libSQL remote storage
    Turso,
    /// Local SQLite via Turso storage
    LocalSqlite,
    /// redb local cache storage
    Redb,
    /// In-memory storage (fallback)
    Memory,
    /// No storage configured
    None,
}

/// Initialize storage backends based on configuration
pub async fn initialize_storage(config: &Config) -> Result<StorageInitResult> {
    tracing::info!("Initializing storage backends based on configuration");

    let _memory_config = create_memory_config(config);
    let mut storage_info = StorageInfo {
        primary_storage: StorageType::None,
        cache_storage: StorageType::None,
        status_messages: Vec::new(),
    };

    // Initialize storage backends based on configuration
    #[cfg(feature = "turso")]
    let (turso_storage, turso_messages) = initialize_turso_storage(&config.database).await?;
    #[cfg(not(feature = "turso"))]
    let (turso_storage, turso_messages) = (None, Vec::new());

    #[cfg(feature = "redb")]
    let (redb_storage, redb_messages) = initialize_redb_storage(&config.database).await?;
    #[cfg(not(feature = "redb"))]
    let (redb_storage, redb_messages) = (None, Vec::new());

    // Combine status messages
    storage_info.status_messages.extend(turso_messages.clone());
    storage_info.status_messages.extend(redb_messages.clone());

    // Determine primary and cache storage types
    let (primary_storage, cache_storage, memory) = determine_storage_combination(
        &config,
        turso_storage,
        redb_storage,
        turso_messages,
        redb_messages,
    )
    .await?;

    storage_info.primary_storage = primary_storage;
    storage_info.cache_storage = cache_storage;

    if let Err(e) = memory.get_all_episodes().await {
        storage_info.status_messages.push(format!(
            "Warning: failed to warm-start episodes from storage: {}",
            e
        ));
    }

    Ok(StorageInitResult {
        memory,
        storage_info,
    })
}

/// Initialize Turso storage if configured
#[cfg(feature = "turso")]
async fn initialize_turso_storage(
    db_config: &DatabaseConfig,
) -> Result<(Option<Arc<dyn StorageBackend>>, Vec<String>)> {
    let mut storage = None;
    let mut status_messages = Vec::new();

    let storage_mode = db_config.storage_mode.as_deref().unwrap_or("remote");

    match storage_mode {
        "memory" => match do_memory_storage_turso::TursoStorage::new_in_memory().await {
            Ok(turso) => {
                if let Err(e) = turso.initialize_schema().await {
                    status_messages.push(format!(
                        "Warning: Failed to initialize in-memory Turso schema: {}",
                        e
                    ));
                } else {
                    storage = Some(Arc::new(turso) as Arc<dyn StorageBackend>);
                    status_messages.push("In-memory Turso storage initialized".to_string());
                }
            }
            Err(e) => {
                status_messages.push(format!(
                    "Warning: Failed to create in-memory Turso storage: {}",
                    e
                ));
            }
        },
        "local" => {
            let path = db_config
                .db_path
                .as_ref()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(do_memory_core::memory::default_db_path);

            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await.context(format!(
                    "Failed to create directory for local database: {}",
                    parent.display()
                ))?;
            }

            match do_memory_storage_turso::TursoStorage::new_local(&path).await {
                Ok(turso) => {
                    if let Err(e) = turso.initialize_schema().await {
                        status_messages.push(format!(
                            "Warning: Failed to initialize local Turso schema: {}",
                            e
                        ));
                    } else {
                        storage = Some(Arc::new(turso) as Arc<dyn StorageBackend>);
                        status_messages.push(format!(
                            "Local Turso storage initialized: {}",
                            path.display()
                        ));
                    }
                }
                Err(e) => {
                    status_messages.push(format!(
                        "Warning: Failed to create local Turso storage at {}: {}",
                        path.display(),
                        e
                    ));
                }
            }
        }
        _ => {
            // Default to remote
            if let Some(turso_url) = &db_config.turso_url {
                let token = db_config.turso_token.as_deref().unwrap_or("");

                match do_memory_storage_turso::TursoStorage::new(turso_url, token).await {
                    Ok(turso) => {
                        if let Err(e) = turso.initialize_schema().await {
                            status_messages
                                .push(format!("Warning: Failed to initialize Turso schema: {}", e));
                        } else {
                            storage = Some(Arc::new(turso) as Arc<dyn StorageBackend>);
                            status_messages
                                .push(format!("Turso storage initialized: {}", turso_url));
                        }
                    }
                    Err(e) => {
                        status_messages
                            .push(format!("Warning: Failed to create Turso storage: {}", e));
                    }
                }
            }

            // If no explicit Turso config, try local SQLite fallback
            if storage.is_none() {
                let (local_storage, local_messages) = try_local_sqlite_fallback(db_config).await?;
                storage = local_storage;
                status_messages.extend(local_messages);
            }
        }
    }

    Ok((storage, status_messages))
}

/// Try to set up local SQLite as fallback
#[cfg(feature = "turso")]
async fn try_local_sqlite_fallback(
    db_config: &DatabaseConfig,
) -> Result<(Option<Arc<dyn StorageBackend>>, Vec<String>)> {
    let mut storage = None;
    let mut status_messages = Vec::new();

    // Try environment variables first
    if let Ok(local_db_url) = std::env::var("LOCAL_DATABASE_URL") {
        if local_db_url.starts_with("sqlite:") || local_db_url.starts_with("file:") {
            let db_path = extract_db_path(&local_db_url);

            ensure_directory_exists(db_path).await?;

            match do_memory_storage_turso::TursoStorage::new(&format!("file:{}", db_path), "").await
            {
                Ok(turso_storage) => {
                    if let Err(e) = turso_storage.initialize_schema().await {
                        status_messages.push(format!(
                            "Warning: Failed to initialize local SQLite schema: {}",
                            e
                        ));
                    } else {
                        storage = Some(Arc::new(turso_storage) as Arc<dyn StorageBackend>);
                        status_messages.push(format!("Using local SQLite database: {}", db_path));
                    }
                }
                Err(e) => {
                    status_messages.push(format!(
                        "Warning: Failed to create local SQLite storage: {}",
                        e
                    ));
                }
            }
        }
    }

    Ok((storage, status_messages))
}

/// Extract database path from URL
fn extract_db_path(url: &str) -> &str {
    url.strip_prefix("sqlite:")
        .unwrap_or(url)
        .strip_prefix("file:")
        .unwrap_or(url)
}

/// Ensure parent directory exists
async fn ensure_directory_exists(path: &str) -> Result<()> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context(format!("Failed to create directory: {}", parent.display()))?;
    }
    Ok(())
}

/// Initialize redb storage if configured
#[cfg(feature = "redb")]
async fn initialize_redb_storage(
    db_config: &DatabaseConfig,
) -> Result<(Option<Arc<dyn StorageBackend>>, Vec<String>)> {
    let mut storage = None;
    let mut status_messages = Vec::new();

    if let Some(redb_path) = &db_config.redb_path {
        let path = std::path::Path::new(redb_path);

        match do_memory_storage_redb::RedbStorage::new(path).await {
            Ok(redb) => {
                storage = Some(Arc::new(redb) as Arc<dyn StorageBackend>);
                status_messages.push(format!("redb storage initialized: {}", redb_path));
            }
            Err(e) => {
                status_messages.push(format!("Warning: Failed to create redb storage: {}", e));
            }
        }
    }

    Ok((storage, status_messages))
}



/// Determine storage combination and create memory system
async fn determine_storage_combination(
    config: &Config,
    turso_storage: Option<Arc<dyn StorageBackend>>,
    redb_storage: Option<Arc<dyn StorageBackend>>,
    _turso_messages: Vec<String>,
    _redb_messages: Vec<String>,
) -> Result<(StorageType, StorageType, SelfLearningMemory)> {
    let memory_config = create_memory_config(config);
    let memory_config_clone = memory_config.clone(); // Clone for multiple uses

    let storage_combination = match (turso_storage, redb_storage) {
        (Some(turso), Some(redb)) => {
            // Both configured - use both
            let memory = SelfLearningMemory::with_storage(memory_config, turso, redb);
            (StorageType::Turso, StorageType::Redb, memory)
        }
        (Some(turso), None) => {
            // Only Turso configured - create fallback redb for cache
            #[cfg(feature = "redb")]
            {
                let temp_redb =
                    do_memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:"))
                        .await?;
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
            // Only redb configured - try to set up local SQLite as durable storage
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
            // No storage configured - try to set up local SQLite fallback
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
    let temp_redb =
        do_memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:")).await?;
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
                            let memory = SelfLearningMemory::with_storage(
                                memory_config,
                                turso_arc,
                                Arc::new(
                                    do_memory_storage_redb::RedbStorage::new(std::path::Path::new(
                                        ":memory:",
                                    ))
                                    .await?,
                                ),
                            );
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

#[cfg(test)]
mod tests;
