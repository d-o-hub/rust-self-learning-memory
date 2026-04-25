//! Storage initialization functions for MCP server
//!
//! This module provides various storage backend initialization strategies:
//! - Turso local (default)
//! - Turso cloud + redb cache (dual storage)
//! - redb-only (fallback)
//! - In-memory (last resort)

use anyhow::Context;
use do_memory_core::{MemoryConfig, SelfLearningMemory};
use do_memory_storage_redb::{CacheConfig, RedbStorage};
use do_memory_storage_turso::{TursoConfig, TursoStorage};
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
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create storage directory: {}", e))?;
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
    // 1. Prepare REDB cache path and ensure directory exists early
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    if let Some(parent) = cache_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create cache directory: {}", e))?;
    }

    // 2. Read Turso configuration from environment
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

    // 3. Initialize redb cache storage
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

    // 1. Use local Turso database file
    let turso_url =
        std::env::var("TURSO_DATABASE_URL").unwrap_or_else(|_| "file:./data/memory.db".to_string());

    // Create directory for local Turso database if applicable
    if let Some(path_str) = turso_url.strip_prefix("file:") {
        if let Some(parent) = Path::new(path_str).parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create database directory: {}", e))?;
        }
    }

    // For local files, no token is needed
    let turso_token = if turso_url.starts_with("file:") {
        "".to_string()
    } else {
        std::env::var("TURSO_AUTH_TOKEN").unwrap_or_default()
    };

    info!("Connecting to Turso database at {}", turso_url);

    // Initialize Turso storage with basic config for local use
    #[allow(clippy::field_reassign_with_default)]
    let turso_config = TursoConfig {
        max_retries: 1, // Fewer retries for local
        retry_base_delay_ms: 50,
        retry_max_delay_ms: 1000,
        enable_pooling: false, // No pooling needed for local
        compression_threshold: 1024,
        compress_episodes: true,
        compress_patterns: true,
        compress_embeddings: true,
        ..Default::default()
    };

    let turso_storage = TursoStorage::with_config(&turso_url, &turso_token, turso_config).await?;
    turso_storage.initialize_schema().await?;

    // 2. Initialize redb cache storage for performance
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    // Create data directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create storage directory: {}", e))?;
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

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[tokio::test]
    #[serial]
    async fn test_initialize_redb_only_storage() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().join("redb_only_test");
        let cache_path = cache_dir.join("cache.redb");
        let original_cache_path = std::env::var("REDB_CACHE_PATH").ok();
        let cache_path_str = cache_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid cache path"))?;

        unsafe {
            std::env::set_var("REDB_CACHE_PATH", cache_path_str);
        }

        let result = initialize_redb_only_storage().await;

        // Restore environment
        if let Some(val) = original_cache_path {
            unsafe {
                std::env::set_var("REDB_CACHE_PATH", val);
            }
        } else {
            unsafe {
                std::env::remove_var("REDB_CACHE_PATH");
            }
        }

        assert!(result.is_ok());
        assert!(cache_path.exists());
        assert!(cache_dir.exists());
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_initialize_turso_local() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let db_dir = temp_dir.path().join("turso_local_test");
        let db_path = db_dir.join("memory.db");
        let cache_dir = temp_dir.path().join("turso_local_cache");
        let cache_path = cache_dir.join("cache.redb");

        let original_db_url = std::env::var("TURSO_DATABASE_URL").ok();
        let original_cache_path = std::env::var("REDB_CACHE_PATH").ok();

        let db_path_str = db_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid db path"))?;
        let cache_path_str = cache_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid cache path"))?;

        unsafe {
            std::env::set_var("TURSO_DATABASE_URL", format!("file:{}", db_path_str));
            std::env::set_var("REDB_CACHE_PATH", cache_path_str);
        }

        let result = initialize_turso_local().await;

        // Restore environment
        if let Some(val) = original_db_url {
            unsafe {
                std::env::set_var("TURSO_DATABASE_URL", val);
            }
        } else {
            unsafe {
                std::env::remove_var("TURSO_DATABASE_URL");
            }
        }
        if let Some(val) = original_cache_path {
            unsafe {
                std::env::set_var("REDB_CACHE_PATH", val);
            }
        } else {
            unsafe {
                std::env::remove_var("REDB_CACHE_PATH");
            }
        }

        assert!(result.is_ok());
        assert!(db_path.exists());
        assert!(cache_path.exists());
        assert!(db_dir.exists());
        assert!(cache_dir.exists());
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_initialize_dual_storage() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_dir = temp_dir.path().join("dual_storage_test");
        let cache_path = cache_dir.join("cache.redb");

        let original_db_url = std::env::var("TURSO_DATABASE_URL").ok();
        let original_db_token = std::env::var("TURSO_AUTH_TOKEN").ok();
        let original_cache_path = std::env::var("REDB_CACHE_PATH").ok();

        let cache_path_str = cache_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid cache path"))?;

        // We use a mock URL for Turso cloud in this test to trigger the dual storage path
        unsafe {
            std::env::set_var("TURSO_DATABASE_URL", "http://localhost:8080");
            std::env::set_var("TURSO_AUTH_TOKEN", "mock-token");
            std::env::set_var("REDB_CACHE_PATH", cache_path_str);
        }

        // This will fail to connect to the mock URL, but we want to check if it creates the directory early
        let result = initialize_dual_storage().await;

        // Restore environment
        if let Some(val) = original_db_url {
            unsafe {
                std::env::set_var("TURSO_DATABASE_URL", val);
            }
        } else {
            unsafe {
                std::env::remove_var("TURSO_DATABASE_URL");
            }
        }
        if let Some(val) = original_db_token {
            unsafe {
                std::env::set_var("TURSO_AUTH_TOKEN", val);
            }
        } else {
            unsafe {
                std::env::remove_var("TURSO_AUTH_TOKEN");
            }
        }
        if let Some(val) = original_cache_path {
            unsafe {
                std::env::set_var("REDB_CACHE_PATH", val);
            }
        } else {
            unsafe {
                std::env::remove_var("REDB_CACHE_PATH");
            }
        }

        // Directory should exist because it's created before the fallible connect call
        assert!(cache_dir.exists());
        // result should be Err because of connection failure
        assert!(result.is_err());
        Ok(())
    }
}
