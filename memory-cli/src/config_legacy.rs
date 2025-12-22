//! Legacy configuration module facade
//!
//! This module provides backward compatibility for the old configuration API
//! while delegating to the new modular configuration system.

pub mod config {
    // Re-export types from the new modular system
    pub use super::config_mod::{
        Config, DatabaseConfig, StorageConfig, CliConfig,
        ConfigPreset, ValidationResult, ValidationError, ValidationWarning
    };

    // Re-export main functions from the new modular system
    pub use super::config_mod::{
        load_config_with_validation, load_and_init, validate_detailed,
        create_writer, save_config, auto_configure, check_readiness,
        get_config_summary
    };

    // Legacy functions for backward compatibility
    use super::config_mod::{load_config as new_load_config, initialize_storage};

    /// Load configuration from file or use defaults
    /// 
    /// This is the legacy API that delegates to the new modular system.
    pub fn load(path: Option<&std::path::Path>) -> Result<Config, anyhow::Error> {
        super::config_mod::load_config_with_validation(path)
    }

    /// Validate configuration
    /// 
    /// This is the legacy validation API that delegates to the new modular system.
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        let validation_result = super::config_mod::validate_detailed(self);
        if validation_result.is_valid {
            Ok(())
        } else {
            let errors = validation_result.errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            Err(anyhow::anyhow!("Configuration validation failed:\n{}", errors))
        }
    }

    /// Create a SelfLearningMemory instance with configured storage backends
    /// 
    /// This is the legacy storage initialization API that delegates to the new modular system.
    pub async fn create_memory(&self) -> Result<memory_core::SelfLearningMemory, anyhow::Error> {
        let storage_result = initialize_storage(self).await?;
        Ok(storage_result.memory)
    }
}

// Re-export everything from the new modular system for direct use
pub use super::config_mod::*;

#[cfg(test)]
mod legacy_tests {
    use super::*;

    #[test]
    fn test_legacy_api_compatibility() {
        // Test that legacy types are re-exported correctly
        let config = Config::default();
        assert_eq!(config.database.redb_path, Some("./data/memory.redb".to_string()));
        
        // Test that validation works through legacy API
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_backward_compatibility() {
        // Ensure that the legacy API still works
        use super::config::Config;
        
        let config = Config::default();
        assert!(config.database.turso_url.is_none());
        assert!(config.database.turso_token.is_none());
        assert_eq!(config.storage.max_episodes_cache, 1000);
        assert_eq!(config.cli.default_format, "human");
    }
}
