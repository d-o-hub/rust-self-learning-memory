//! Storage initialization functions for MCP server
//!
//! This module provides various storage backend initialization strategies:
//! - Turso local (default)
//! - Turso cloud + redb cache (dual storage)
//! - redb-only (fallback)
//! - In-memory (last resort)

use anyhow::Context;
use do_memory_core::{Error, MemoryConfig, SelfLearningMemory};
use do_memory_storage_redb::{CacheConfig, RedbStorage};
use do_memory_storage_turso::{TursoConfig, TursoStorage};
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

/// Initialize the memory system with appropriate storage backends
///
/// This function tries storage backends in order of preference:
/// 1. Named storage mode (if MEMORY_STORAGE_MODE is set)
/// 2. Turso local (default, no configuration needed)
/// 3. Turso cloud + redb (if TURSO_DATABASE_URL and TURSO_AUTH_TOKEN are set)
/// 4. redb-only (fallback when Turso is unavailable)
/// 5. In-memory (last resort)
pub async fn initialize_memory_system() -> anyhow::Result<Arc<SelfLearningMemory>> {
    // Check for explicit storage mode override
    if let Ok(mode) = std::env::var("MEMORY_STORAGE_MODE") {
        match mode.to_lowercase().as_str() {
            "local" => {
                if let Ok(memory) = initialize_turso_local().await {
                    info!("Memory system initialized with Turso local database (explicit)");
                    return Ok(memory);
                }
            }
            "memory" => {
                info!("Memory system initialized with in-memory storage (explicit)");

                // Use the new named constructor from do_memory_storage_turso
                let storage = do_memory_storage_turso::TursoStorage::new_in_memory().await?;
                storage.initialize_schema().await?;

                // Also initialize an in-memory redb for caching
                let cache = Arc::new(
                    do_memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:"))
                        .await?,
                );

                let memory = SelfLearningMemory::with_storage(
                    MemoryConfig::default(),
                    Arc::new(storage),
                    cache,
                );

                return Ok(Arc::new(memory));
            }
            "remote" => {
                if let Ok(memory) = initialize_dual_storage().await {
                    info!("Memory system initialized with remote Turso (explicit)");
                    return Ok(memory);
                }
            }
            _ => warn!(
                "Unknown MEMORY_STORAGE_MODE: {}, following default order",
                mode
            ),
        }
    }

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
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| Error::Storage(format!("Failed to create data directory: {e}")))?;
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
    let redb_arc: Arc<dyn do_memory_core::StorageBackend> = Arc::new(redb_storage);
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
    #[allow(clippy::field_reassign_with_default)]
    let turso_config = TursoConfig {
        max_retries: 3,
        retry_base_delay_ms: 100,
        retry_max_delay_ms: 5000,
        enable_pooling: true,
        compression_threshold: 1024,
        compress_episodes: true,
        compress_patterns: true,
        compress_embeddings: true,
        ..Default::default()
    };

    let turso_storage = TursoStorage::with_config(&turso_url, &turso_token, turso_config).await?;
    turso_storage.initialize_schema().await?;

    // Initialize redb cache storage
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    // Create data directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| Error::Storage(format!("Failed to create cache directory: {e}")))?;
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

    // Resolve the local database path. Precedence:
    // 1. `MEMORY_DB_PATH` env var (the explicit local-mode knob)
    // 2. `TURSO_DATABASE_URL` env var if it points at a local file
    //    (strip the `file:` prefix used by the libsql URL syntax)
    // 3. `do_memory_core::memory::default_db_path()` (workspace default)
    //
    // We always end up with a real on-disk path so we can call
    // `TursoStorage::new_local` directly without going through the
    // remote-style URL parsing path.
    let db_path: std::path::PathBuf = if let Ok(p) = std::env::var("MEMORY_DB_PATH") {
        std::path::PathBuf::from(strip_file_prefix(&p))
    } else if let Ok(url) = std::env::var("TURSO_DATABASE_URL") {
        if url.starts_with("file:") || url.starts_with("libsql://") {
            // For backwards compatibility: only treat `file:` URLs as local
            // paths; ignore `libsql://` (that should use the remote path).
            if url.starts_with("file:") {
                std::path::PathBuf::from(strip_file_prefix(&url))
            } else {
                do_memory_core::memory::default_db_path()
            }
        } else {
            // No protocol prefix: treat as a plain file path.
            std::path::PathBuf::from(url)
        }
    } else {
        do_memory_core::memory::default_db_path()
    };

    info!(
        "Connecting to local Turso database at {}",
        db_path.display()
    );

    // Local file connections use a single direct connection (no pool / no
    // keep-alive). This matches the in-storage `TursoConfig::default()`
    // used by `TursoStorage::new_local`/`new_in_memory`.
    let turso_storage = TursoStorage::new_local(&db_path).await?;
    turso_storage.initialize_schema().await?;

    // Initialize redb cache storage for performance
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    // Create data directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| Error::Storage(format!("Failed to create cache directory: {e}")))?;
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

/// Strip an optional `file:` URL prefix from a path or URL string.
fn strip_file_prefix(s: &str) -> &str {
    s.strip_prefix("file:").unwrap_or(s)
}

#[cfg(test)]
#[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_turso_local_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        // SAFETY: single-threaded test environment
        unsafe {
            std::env::set_var("MEMORY_DB_PATH", db_path.to_str().unwrap());
            std::env::set_var(
                "REDB_CACHE_PATH",
                dir.path().join("cache.redb").to_str().unwrap(),
            );
        }
        let result = initialize_turso_local().await;
        // SAFETY: single-threaded test environment
        unsafe {
            std::env::remove_var("MEMORY_DB_PATH");
            std::env::remove_var("REDB_CACHE_PATH");
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_initialize_redb_only_storage_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded test environment
        unsafe {
            std::env::set_var(
                "REDB_CACHE_PATH",
                dir.path().join("cache.redb").to_str().unwrap(),
            );
        }
        let result = initialize_redb_only_storage().await;
        // SAFETY: single-threaded test environment
        unsafe {
            std::env::remove_var("REDB_CACHE_PATH");
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_initialize_memory_system_in_memory_mode() {
        let dir = tempfile::tempdir().unwrap();
        // SAFETY: single-threaded test environment
        unsafe {
            std::env::set_var("MEMORY_STORAGE_MODE", "memory");
            std::env::set_var(
                "REDB_CACHE_PATH",
                dir.path().join("cache.redb").to_str().unwrap(),
            );
        }
        let result = initialize_memory_system().await;
        // SAFETY: single-threaded test environment
        unsafe {
            std::env::remove_var("MEMORY_STORAGE_MODE");
            std::env::remove_var("REDB_CACHE_PATH");
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_initialize_memory_system_unknown_mode_falls_back() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        // SAFETY: single-threaded test environment
        unsafe {
            std::env::set_var("MEMORY_STORAGE_MODE", "unknown_xyz");
            std::env::set_var("MEMORY_DB_PATH", db_path.to_str().unwrap());
            std::env::set_var(
                "REDB_CACHE_PATH",
                dir.path().join("cache.redb").to_str().unwrap(),
            );
        }
        let result = initialize_memory_system().await;
        // SAFETY: single-threaded test environment
        unsafe {
            std::env::remove_var("MEMORY_STORAGE_MODE");
            std::env::remove_var("MEMORY_DB_PATH");
            std::env::remove_var("REDB_CACHE_PATH");
        }
        assert!(result.is_ok());
    }
}
