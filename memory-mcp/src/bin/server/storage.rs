//! Storage initialization functions for MCP server
//!
//! This module provides various storage backend initialization strategies:
//! - Turso local (default)
//! - Turso cloud + redb cache (dual storage)
//! - redb-only (fallback)
//! - In-memory (last resort)

use anyhow::Context;
use memory_core::{Error, MemoryConfig, SelfLearningMemory};
use memory_storage_redb::{CacheConfig, RedbStorage};
use memory_storage_turso::{TursoConfig, TursoStorage};
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

/// Initialize the memory system with appropriate storage backends
///
/// This function tries storage backends in order of preference:
/// 1. Turso local (default, no configuration needed)
/// 2. Turso cloud + redb (if TURSO_DATABASE_URL and TURSO_AUTH_TOKEN are set)
/// 3. redb-only (fallback when Turso is unavailable)
/// 4. In-memory (last resort)
pub async fn initialize_memory_system() -> anyhow::Result<Arc<SelfLearningMemory>> {
    // Try Turso local first (default behavior)
    if let Ok(memory) = initialize_turso_local().await {
        info!("Memory system initialized with Turso local database (default)");
        return Ok(memory);
    }

    // If Turso local fails, try dual storage (Turso cloud + redb)
    if let Ok(memory) = initialize_dual_storage().await {
        info!("Memory system initialized with persistent storage (Turso cloud + redb)");
        return Ok(memory);
    }

    // If dual storage fails, try redb-only storage
    if let Ok(memory) = initialize_redb_only_storage().await {
        info!("Memory system initialized with redb cache storage (Turso unavailable)");
        return Ok(memory);
    }

    // Final fallback to in-memory storage
    warn!("Failed to initialize any persistent storage, falling back to in-memory");
    info!("To enable persistence:");
    info!("  - Default: Turso local database (no configuration needed)");
    info!("  - Cloud: set TURSO_DATABASE_URL and TURSO_AUTH_TOKEN");
    info!("  - Cache-only: ensure REDB_CACHE_PATH is accessible");
    Ok(Arc::new(SelfLearningMemory::new()))
}

/// Initialize memory system with redb cache storage only (fallback when Turso is unavailable)
pub async fn initialize_redb_only_storage() -> anyhow::Result<Arc<SelfLearningMemory>> {
    info!("Attempting to initialize redb-only storage...");

    // Initialize redb cache storage
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    // Create data directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::Storage(format!("Failed to create data directory: {}", e)))?;
    }

    let cache_config = CacheConfig {
        max_size: std::env::var("REDB_MAX_CACHE_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000),
        default_ttl_secs: 1800,     // 30 minutes
        cleanup_interval_secs: 600, // 10 minutes
        enable_background_cleanup: true,
    };

    let redb_storage = RedbStorage::new_with_cache_config(cache_path, cache_config).await?;
    info!(
        "Successfully initialized redb storage at {}",
        cache_path.display()
    );

    // Create memory system with redb cache and in-memory fallbacks for Turso
    // Note: We use the same redb instance for both turso and cache since we only have redb
    let memory_config = MemoryConfig::default();
    let redb_arc: Arc<dyn memory_core::StorageBackend> = Arc::new(redb_storage);
    let memory = SelfLearningMemory::with_storage(memory_config, Arc::clone(&redb_arc), redb_arc);

    Ok(Arc::new(memory))
}

/// Initialize memory system with both Turso (durable) and redb (cache) storage
pub async fn initialize_dual_storage() -> anyhow::Result<Arc<SelfLearningMemory>> {
    // Read Turso configuration from environment
    let turso_url = std::env::var("TURSO_DATABASE_URL")
        .context("TURSO_DATABASE_URL environment variable not set")?;
    let turso_token = std::env::var("TURSO_AUTH_TOKEN")
        .context("TURSO_AUTH_TOKEN environment variable not set")?;

    info!("Connecting to Turso database at {}", turso_url);

    // Initialize Turso storage with connection pooling
    let turso_config = TursoConfig {
        max_retries: 3,
        retry_base_delay_ms: 100,
        retry_max_delay_ms: 5000,
        enable_pooling: true,
        compression_threshold: 1024,
        compress_episodes: true,
        compress_patterns: true,
        compress_embeddings: true,
        cache_config: None,
    };

    let turso_storage = TursoStorage::with_config(&turso_url, &turso_token, turso_config).await?;
    turso_storage.initialize_schema().await?;

    // Initialize redb cache storage
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    let cache_config = CacheConfig {
        max_size: std::env::var("REDB_MAX_CACHE_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000),
        default_ttl_secs: 1800,     // 30 minutes
        cleanup_interval_secs: 600, // 10 minutes
        enable_background_cleanup: true,
    };

    let redb_storage = RedbStorage::new_with_cache_config(cache_path, cache_config).await?;

    // Create memory system with both storage backends
    let memory_config = MemoryConfig::default();
    let memory = SelfLearningMemory::with_storage(
        memory_config,
        Arc::new(turso_storage),
        Arc::new(redb_storage),
    );

    Ok(Arc::new(memory))
}

/// Initialize memory system with Turso local database (default behavior)
pub async fn initialize_turso_local() -> anyhow::Result<Arc<SelfLearningMemory>> {
    info!("Attempting to initialize Turso local database (default)...");

    // Use local Turso database file
    let turso_url =
        std::env::var("TURSO_DATABASE_URL").unwrap_or_else(|_| "file:./data/memory.db".to_string());

    // For local files, no token is needed
    let turso_token = if turso_url.starts_with("file:") {
        "".to_string()
    } else {
        std::env::var("TURSO_AUTH_TOKEN").unwrap_or_default()
    };

    info!("Connecting to Turso database at {}", turso_url);

    // Initialize Turso storage with basic config for local use
    let turso_config = TursoConfig {
        max_retries: 1, // Fewer retries for local
        retry_base_delay_ms: 50,
        retry_max_delay_ms: 1000,
        enable_pooling: false, // No pooling needed for local
        compression_threshold: 1024,
        compress_episodes: true,
        compress_patterns: true,
        compress_embeddings: true,
        cache_config: None,
    };

    let turso_storage = TursoStorage::with_config(&turso_url, &turso_token, turso_config).await?;
    turso_storage.initialize_schema().await?;

    // Initialize redb cache storage for performance
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    let cache_config = CacheConfig {
        max_size: std::env::var("REDB_MAX_CACHE_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000),
        default_ttl_secs: 1800,     // 30 minutes
        cleanup_interval_secs: 600, // 10 minutes
        enable_background_cleanup: true,
    };

    let redb_storage = RedbStorage::new_with_cache_config(cache_path, cache_config).await?;

    // Create memory system with both storage backends
    let memory_config = MemoryConfig::default();
    let memory = SelfLearningMemory::with_storage(
        memory_config,
        Arc::new(turso_storage),
        Arc::new(redb_storage),
    );

    info!("Successfully initialized Turso local + redb cache storage");
    Ok(Arc::new(memory))
}
