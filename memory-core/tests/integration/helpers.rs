//! Test helpers for integration tests
//!
//! Provides utilities for setting up temporary databases and test environments.

use anyhow::Result;
use memory_core::{MemoryConfig, SelfLearningMemory};
use memory_storage_redb::RedbStorage;
use memory_storage_turso::TursoStorage;
use std::sync::Arc;
use tempfile::TempDir;

/// Test environment with temporary Turso and redb databases
///
/// Provides a complete test environment with:
/// - Temporary directory (auto-cleaned on drop)
/// - Local file-based Turso database
/// - redb cache database
/// - Configured SelfLearningMemory instance
///
/// # Example
///
/// ```no_run
/// use integration::helpers::TestEnvironment;
///
/// #[tokio::test]
/// async fn test_something() {
///     let env = TestEnvironment::new().await.unwrap();
///     // Use env.memory for testing
/// }
/// ```
pub struct TestEnvironment {
    /// Temporary directory (cleaned up on drop)
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    /// Turso database URL
    pub turso_url: String,
    /// redb database path
    pub redb_path: String,
    /// Configured memory instance
    pub memory: SelfLearningMemory,
    /// Reference to Turso storage (for direct access)
    pub turso_storage: Arc<TursoStorage>,
    /// Reference to redb storage (for direct access)
    pub redb_storage: Arc<RedbStorage>,
}

impl TestEnvironment {
    /// Create a new test environment with temporary databases
    ///
    /// Sets up:
    /// 1. Temporary directory
    /// 2. File-based Turso database
    /// 3. redb cache database
    /// 4. Initializes database schemas
    /// 5. Creates SelfLearningMemory with both backends
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Temporary directory creation fails
    /// - Database initialization fails
    /// - Schema creation fails
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let turso_url = format!("file:{}", db_path.display());
        let redb_path = temp_dir.path().join("cache.redb");

        // Create Turso storage
        let turso = TursoStorage::new(&turso_url, "").await?;
        turso.initialize_schema().await?;
        let turso_arc = Arc::new(turso);

        // Create redb storage
        let redb = RedbStorage::new(&redb_path)?;
        let redb_arc = Arc::new(redb);

        // Create memory with both backends
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            turso_arc.clone(),
            redb_arc.clone(),
        );

        Ok(Self {
            temp_dir,
            turso_url,
            redb_path: redb_path.display().to_string(),
            memory,
            turso_storage: turso_arc,
            redb_storage: redb_arc,
        })
    }

    /// Create a new test environment with custom configuration
    ///
    /// Like `new()` but accepts a custom MemoryConfig.
    ///
    /// # Errors
    ///
    /// Returns error if database setup fails
    pub async fn with_config(config: MemoryConfig) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let turso_url = format!("file:{}", db_path.display());
        let redb_path = temp_dir.path().join("cache.redb");

        // Create Turso storage
        let turso = TursoStorage::new(&turso_url, "").await?;
        turso.initialize_schema().await?;
        let turso_arc = Arc::new(turso);

        // Create redb storage
        let redb = RedbStorage::new(&redb_path)?;
        let redb_arc = Arc::new(redb);

        // Create memory with custom config
        let memory = SelfLearningMemory::with_storage(config, turso_arc.clone(), redb_arc.clone());

        Ok(Self {
            temp_dir,
            turso_url,
            redb_path: redb_path.display().to_string(),
            memory,
            turso_storage: turso_arc,
            redb_storage: redb_arc,
        })
    }

    /// Get a reference to the Turso storage backend
    ///
    /// Useful for direct storage operations in tests
    pub fn turso(&self) -> &TursoStorage {
        &self.turso_storage
    }

    /// Get a reference to the redb storage backend
    ///
    /// Useful for direct storage operations in tests
    pub fn redb(&self) -> &RedbStorage {
        &self.redb_storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_creation() {
        let env = TestEnvironment::new().await.unwrap();
        assert!(env.turso_url.starts_with("file:"));
        assert!(env.redb_path.ends_with("cache.redb"));
    }

    #[tokio::test]
    async fn test_environment_with_custom_config() {
        let mut config = MemoryConfig::default();
        config.pattern_extraction_threshold = 0.5;

        let env = TestEnvironment::with_config(config).await.unwrap();
        assert!(env.turso_url.starts_with("file:"));
    }
}
