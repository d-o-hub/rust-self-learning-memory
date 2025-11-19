use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// CLI-specific settings
    pub cli: CliConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Turso database URL
    pub turso_url: Option<String>,
    /// Turso authentication token
    pub turso_token: Option<String>,
    /// redb database path
    pub redb_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum episodes to cache
    pub max_episodes_cache: usize,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Connection pool size
    pub pool_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default output format
    pub default_format: String,
    /// Enable progress bars
    pub progress_bars: bool,
    /// Batch size for bulk operations
    pub batch_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some("memory.redb".to_string()),
            },
            storage: StorageConfig {
                max_episodes_cache: 1000,
                cache_ttl_seconds: 3600, // 1 hour
                pool_size: 10,
            },
            cli: CliConfig {
                default_format: "human".to_string(),
                progress_bars: true,
                batch_size: 100,
            },
        }
    }
}

impl Config {
    /// Load configuration from file or use defaults
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        match path {
            Some(path) => {
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read config file: {}", path.display()))?;

                if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                    || path.extension().and_then(|s| s.to_str()) == Some("yml")
                {
                    serde_yaml::from_str(&content)
                        .with_context(|| format!("Failed to parse YAML config: {}", path.display()))
                } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    toml::from_str(&content)
                        .with_context(|| format!("Failed to parse TOML config: {}", path.display()))
                } else {
                    serde_json::from_str(&content)
                        .with_context(|| format!("Failed to parse JSON config: {}", path.display()))
                }
            }
            None => {
                // Try to load from default locations
                let default_paths = [
                    "memory-cli.toml",
                    "memory-cli.json",
                    "memory-cli.yaml",
                    ".memory-cli.toml",
                    ".memory-cli.json",
                    ".memory-cli.yaml",
                ];

                for path_str in &default_paths {
                    let path = Path::new(path_str);
                    if path.exists() {
                        return Self::load(Some(path));
                    }
                }

                // Fall back to defaults
                Ok(Self::default())
            }
        }
    }

    /// Validate configuration
    #[allow(dead_code)]
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate database configuration
        if self.database.turso_url.is_none() && self.database.redb_path.is_none() {
            anyhow::bail!("Either Turso URL or redb path must be configured");
        }

        // Validate storage configuration
        if self.storage.max_episodes_cache == 0 {
            anyhow::bail!("max_episodes_cache must be greater than 0");
        }

        if self.storage.pool_size == 0 {
            anyhow::bail!("pool_size must be greater than 0");
        }

        // Validate CLI configuration
        match self.cli.default_format.as_str() {
            "human" | "json" | "yaml" => {}
            _ => anyhow::bail!("default_format must be 'human', 'json', or 'yaml'"),
        }

        if self.cli.batch_size == 0 {
            anyhow::bail!("batch_size must be greater than 0");
        }

        Ok(())
    }

    /// Create a SelfLearningMemory instance with configured storage backends
    pub async fn create_memory(&self) -> anyhow::Result<memory_core::SelfLearningMemory> {
        use memory_core::{MemoryConfig, SelfLearningMemory};
        #[allow(unused_imports)]
        use std::sync::Arc;

        let memory_config = MemoryConfig {
            storage: memory_core::StorageConfig {
                max_episodes_cache: self.storage.max_episodes_cache,
                sync_interval_secs: 300, // 5 minutes default
                enable_compression: false,
            },
            enable_embeddings: false,
            pattern_extraction_threshold: 0.1, // Default threshold
            batch_config: Some(memory_core::BatchConfig::default()),
            concurrency: memory_core::ConcurrencyConfig::default(),
        };

        // Create storage backends based on configuration
        #[allow(unused_mut)]
        let mut turso_storage = None;
        #[allow(unused_mut)]
        let mut redb_storage = None;

        // Initialize Turso storage if configured
        #[cfg(feature = "turso")]
        if let Some(turso_url) = &self.database.turso_url {
            let token = self.database.turso_token.as_deref().unwrap_or("");
            let storage = memory_storage_turso::TursoStorage::new(turso_url, token).await?;
            storage.initialize_schema().await?;
            turso_storage = Some(Arc::new(storage) as Arc<dyn memory_core::StorageBackend>);
        }

        // Initialize redb storage if configured
        #[cfg(feature = "redb")]
        if let Some(redb_path) = &self.database.redb_path {
            let path = std::path::Path::new(redb_path);
            let storage = memory_storage_redb::RedbStorage::new(path).await?;
            redb_storage = Some(Arc::new(storage) as Arc<dyn memory_core::StorageBackend>);
        }

        // Create memory system with storage backends
        #[allow(unused_variables)]
        match (turso_storage, redb_storage) {
            (Some(turso), Some(redb)) => {
                Ok(SelfLearningMemory::with_storage(memory_config, turso, redb))
            }
            (Some(turso), None) => {
                // Create a fallback redb for cache if only turso is configured
                #[cfg(feature = "redb")]
                {
                    let temp_redb =
                        memory_storage_redb::RedbStorage::new(std::path::Path::new(":memory:"))
                            .await?;
                    Ok(SelfLearningMemory::with_storage(
                        memory_config,
                        turso,
                        Arc::new(temp_redb),
                    ))
                }
                #[cfg(not(feature = "redb"))]
                {
                    // No redb available, use turso-only (in-memory cache fallback)
                    Ok(SelfLearningMemory::with_config(memory_config))
                }
            }
            (None, Some(_redb)) => {
                // Only redb configured - use in-memory fallback for durable storage
                Ok(SelfLearningMemory::with_config(memory_config))
            }
            (None, None) => {
                // No storage configured - use in-memory only
                Ok(SelfLearningMemory::with_config(memory_config))
            }
        }
    }
}
