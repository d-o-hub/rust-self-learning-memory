//! Storage initialization module
//!
//! This module provides unified storage backend initialization logic,
//! eliminating code duplication and providing consistent setup across
//! different configuration scenarios.

mod combination;

use super::types::{Config, DatabaseConfig};
use anyhow::Context;
use anyhow::Result;
use do_memory_core::{MemoryConfig, SelfLearningMemory, StorageBackend};
use std::sync::Arc;

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
    let (primary_storage, cache_storage, memory) = combination::determine_storage_combination(
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
pub(super) fn extract_db_path(url: &str) -> &str {
    url.strip_prefix("sqlite:")
        .unwrap_or(url)
        .strip_prefix("file:")
        .unwrap_or(url)
}

/// Ensure parent directory exists
pub(super) async fn ensure_directory_exists(path: &str) -> Result<()> {
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

        // Ensure parent directory exists before opening redb file.
        // On fresh systems the default cache directory may not exist yet,
        // causing redb initialization to silently fail and episodes to be
        // stored only in a volatile in-memory fallback.
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    status_messages.push(format!(
                        "Warning: Failed to create redb directory {}: {}",
                        parent.display(),
                        e
                    ));
                }
            }
        }

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

/// Create memory system configuration
pub(super) fn create_memory_config(config: &Config) -> MemoryConfig {
    MemoryConfig {
        storage: do_memory_core::StorageConfig {
            max_episodes_cache: config.storage.max_episodes_cache,
            sync_interval_secs: 300, // 5 minutes default
            enable_compression: false,
        },
        enable_embeddings: config.embeddings.enabled, // Use config value
        pattern_extraction_threshold: 0.1,
        quality_threshold: 0.0, // Allow CLI workflows to complete minimal episodes
        batch_config: None,     // Disable batching for CLI - each command is a separate process
        concurrency: do_memory_core::ConcurrencyConfig::default(),
        // Phase 2 (GENESIS) - Capacity management
        max_episodes: None, // No capacity limit by default
        eviction_policy: Some(do_memory_core::episode::EvictionPolicy::RelevanceWeighted),
        // Phase 2 (GENESIS) - Semantic summarization
        enable_summarization: true,
        summary_min_length: 100,
        summary_max_length: 200,
        // Phase 3 (Spatiotemporal) - Hierarchical retrieval
        enable_spatiotemporal_indexing: true,
        enable_diversity_maximization: true,
        diversity_lambda: 0.7,
        temporal_bias_weight: 0.3,
        max_clusters_to_search: 5,
        retrieval_mode: do_memory_core::types::RetrievalMode::Keyword,
        semantic_search_mode: "hybrid".to_string(),
        enable_query_embedding_cache: true,
        semantic_similarity_threshold: 0.6,
        semantic_weight: 0.5,
        recency_weight: 0.25,
        reward_weight: 0.15,
        context_overlap_weight: 0.10,
        ann_index_path: None,
        // Audit configuration
        audit_config: do_memory_core::AuditConfig::default(),
        // CloudEvents EventEmitter (WG-149)
        event_emitter_mode: do_memory_core::types::emitter::EventEmitterMode::default(),
    }
}

/// Determine storage combination and create memory system
#[cfg(test)]
mod tests;
