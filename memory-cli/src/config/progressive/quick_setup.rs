//! Quick setup module.
//!
//! This module provides ultra-simple configuration setup (30-second setup).

use crate::config::types::{Config, ConfigPreset};
use crate::config::validate_config;
use crate::config::EnvironmentCheck;
use anyhow::{Context, Result};
use tracing::{info, warn};

/// Ultra-Simple Mode: 30-second setup for basic redb usage.
///
/// This is the fastest way to get started with memory-cli. It automatically:
/// - Detects the best local storage path
/// - Creates a minimal configuration optimized for quick start
/// - Validates the environment and provides clear feedback
/// - Returns a ready-to-use configuration
///
/// # Success Criteria
/// - ✅ Setup time: < 30 seconds
/// - ✅ Clear success/error feedback
/// - ✅ No user decisions required
/// - ✅ Auto-detect optimal settings
///
/// # Example
///
/// ```no_run
/// use memory_cli::config::progressive::setup_quick_redb;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = setup_quick_redb().await?;
///     println!("🚀 Memory CLI ready for basic usage!");
///     Ok(())
/// }
/// ```
pub async fn setup_quick_redb() -> Result<Config> {
    info!("🚀 Starting 30-second quick setup for redb storage");

    // Step 1: Auto-detect environment and create minimal config
    let config = create_quick_config()?;

    // Step 2: Quick validation with helpful feedback
    let validation_result = validate_config(&config);

    if !validation_result.is_valid {
        let error_summary = validation_result
            .errors
            .iter()
            .map(|e| format!("  ❌ {}", e))
            .collect::<Vec<_>>()
            .join("\n");

        return Err(anyhow::anyhow!(
            "Quick setup failed - environment issues detected:\n{}\n\n💡 Tip: Run `setup_simple()` for guided configuration",
            error_summary
        ));
    }

    // Step 3: Provide clear success feedback
    if !validation_result.warnings.is_empty() {
        let warning_summary = validation_result
            .warnings
            .iter()
            .map(|w| format!("  ⚠️  {}", w))
            .collect::<Vec<_>>()
            .join("\n");

        warn!("Quick setup completed with warnings:\n{}", warning_summary);
    }

    info!("✅ Quick setup completed successfully - redb ready for use!");
    info!("📁 Data will be stored in: {:?}", config.database.redb_path);

    Ok(config)
}

/// Create minimal configuration optimized for quick start.
fn create_quick_config() -> Result<Config> {
    let env_check = EnvironmentCheck::new();

    let preset = if env_check.redb_available {
        ConfigPreset::Local
    } else {
        ConfigPreset::Memory
    };

    let mut config = preset.create_config();

    // Optimize for quick start: smaller cache, shorter TTL, faster setup
    config.storage.max_episodes_cache = std::cmp::min(500, config.storage.max_episodes_cache);
    config.storage.cache_ttl_seconds = std::cmp::min(1800, config.storage.cache_ttl_seconds);
    config.storage.pool_size = std::cmp::min(5, config.storage.pool_size);
    config.cli.batch_size = std::cmp::min(25, config.cli.batch_size);

    if config.database.redb_path.is_none() {
        config.database.redb_path = Some(quick_redb_path_sync()?);
    }

    Ok(config)
}

/// Get a quick redb path for immediate use.
pub fn quick_redb_path_sync() -> Result<String> {
    let temp_dir = std::env::temp_dir().join("memory-cli-quick");

    std::fs::create_dir_all(&temp_dir)
        .context("Failed to create temporary directory for quick setup")?;

    let redb_path = temp_dir.join("quick-setup.redb");
    Ok(redb_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_redb_setup() {
        let config = setup_quick_redb().await.unwrap();

        assert!(config.database.redb_path.is_some());
        assert!(config.storage.max_episodes_cache > 0);
        assert!(config.storage.pool_size > 0);
    }
}
