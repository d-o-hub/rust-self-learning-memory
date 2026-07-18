use clap::Subcommand;
use std::path::PathBuf;

use crate::config::{Config, ConfigWizard};
use crate::output::OutputFormat;

// Re-export template helpers so `config::show_config_template` / `init_config`
// keep working from the command dispatcher.
pub use super::config_template::{init_config, show_config_template};
// Display/validation result types used by handlers and Output impls.
pub use super::config_display::*;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Validate configuration and connectivity
    Validate,
    /// Check configuration for issues and recommendations
    Check,
    /// Show current configuration (with sensitive data masked)
    Show,
    /// Run interactive configuration wizard
    Wizard,
    /// Print a starter configuration template (TOML) to stdout
    ShowTemplate,
    /// Write a starter configuration to a file (default: do-memory-cli.toml)
    Init {
        /// Path to write the starter config
        #[arg(default_value = "do-memory-cli.toml")]
        path: PathBuf,
    },
}

// Command implementations
pub async fn validate_config(
    memory: &do_memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let mut issues = Vec::new();
    let mut connectivity_errors = Vec::new();
    let mut turso_connected = false;
    let mut redb_accessible = false;
    let mut latency_ms = None;

    // Basic configuration validation
    if config.database.turso_url.is_none() && config.database.redb_path.is_none() {
        issues.push(ConfigIssue {
            level: IssueLevel::Error,
            category: "database".to_string(),
            message: "Neither Turso URL nor redb path is configured".to_string(),
            suggestion: Some("Configure at least one storage backend".to_string()),
        });
    }

    if config.storage.max_episodes_cache == 0 {
        issues.push(ConfigIssue {
            level: IssueLevel::Error,
            category: "storage".to_string(),
            message: "max_episodes_cache cannot be zero".to_string(),
            suggestion: Some("Set max_episodes_cache to a positive value".to_string()),
        });
    }

    if config.storage.pool_size == 0 {
        issues.push(ConfigIssue {
            level: IssueLevel::Error,
            category: "storage".to_string(),
            message: "pool_size cannot be zero".to_string(),
            suggestion: Some("Set pool_size to a positive value".to_string()),
        });
    }

    if config.cli.batch_size == 0 {
        issues.push(ConfigIssue {
            level: IssueLevel::Error,
            category: "cli".to_string(),
            message: "batch_size cannot be zero".to_string(),
            suggestion: Some("Set batch_size to a positive value".to_string()),
        });
    }

    // Test connectivity
    if let Some(turso) = memory.turso_storage() {
        let start = std::time::Instant::now();
        match turso.get_episode(uuid::Uuid::new_v4()).await {
            Ok(_) => {
                turso_connected = true;
                #[expect(clippy::cast_possible_truncation)]
                let latency = start.elapsed().as_millis() as u64;
                latency_ms = Some(latency);
            }
            Err(e) => {
                connectivity_errors.push(format!("Turso connection failed: {}", e));
            }
        }
    }

    if let Some(cache) = memory.cache_storage() {
        match cache.get_episode(uuid::Uuid::new_v4()).await {
            Ok(_) => redb_accessible = true,
            Err(e) => {
                connectivity_errors.push(format!("redb access failed: {}", e));
            }
        }
    }

    let is_valid = issues.iter().all(|i| i.level != IssueLevel::Error);

    let validation = ConfigValidation {
        is_valid,
        issues,
        connectivity: ConnectivityStatus {
            turso_connected,
            redb_accessible,
            latency_ms,
            errors: connectivity_errors,
        },
    };

    format.print_output(&validation)?;
    Ok(())
}

pub async fn check_config(
    memory: &do_memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    // First get validation results
    let mut issues = Vec::new();
    let mut connectivity_errors = Vec::new();
    let mut turso_connected = false;
    let mut redb_accessible = false;
    let mut latency_ms = None;

    // Basic configuration validation
    if config.database.turso_url.is_none() && config.database.redb_path.is_none() {
        issues.push(ConfigIssue {
            level: IssueLevel::Error,
            category: "database".to_string(),
            message: "Neither Turso URL nor redb path is configured".to_string(),
            suggestion: Some("Configure at least one storage backend".to_string()),
        });
    }

    if config.storage.max_episodes_cache == 0 {
        issues.push(ConfigIssue {
            level: IssueLevel::Error,
            category: "storage".to_string(),
            message: "max_episodes_cache cannot be zero".to_string(),
            suggestion: Some("Set max_episodes_cache to a positive value".to_string()),
        });
    }

    // Performance recommendations
    if config.storage.max_episodes_cache < 100 {
        issues.push(ConfigIssue {
            level: IssueLevel::Warning,
            category: "performance".to_string(),
            message: "max_episodes_cache is quite low".to_string(),
            suggestion: Some(
                "Consider increasing to at least 1000 for better performance".to_string(),
            ),
        });
    }

    if config.storage.pool_size < 5 {
        issues.push(ConfigIssue {
            level: IssueLevel::Warning,
            category: "performance".to_string(),
            message: "pool_size is quite low".to_string(),
            suggestion: Some("Consider increasing pool_size for better concurrency".to_string()),
        });
    }

    // Test connectivity
    if let Some(turso) = memory.turso_storage() {
        let start = std::time::Instant::now();
        match turso.get_episode(uuid::Uuid::new_v4()).await {
            Ok(_) => {
                turso_connected = true;
                #[expect(clippy::cast_possible_truncation)]
                let latency = start.elapsed().as_millis() as u64;
                latency_ms = Some(latency);
            }
            Err(e) => {
                connectivity_errors.push(format!("Turso connection failed: {}", e));
            }
        }
    }

    if let Some(cache) = memory.cache_storage() {
        match cache.get_episode(uuid::Uuid::new_v4()).await {
            Ok(_) => redb_accessible = true,
            Err(e) => {
                connectivity_errors.push(format!("redb access failed: {}", e));
            }
        }
    }

    let is_valid = issues.iter().all(|i| i.level != IssueLevel::Error);

    let validation = ConfigValidation {
        is_valid,
        issues,
        connectivity: ConnectivityStatus {
            turso_connected,
            redb_accessible,
            latency_ms,
            errors: connectivity_errors,
        },
    };

    // Generate recommendations
    let mut recommendations = Vec::new();
    let mut security_issues = Vec::new();

    if config.database.turso_token.is_none() && config.database.turso_url.is_some() {
        security_issues.push("Turso URL configured but no token provided".to_string());
    }

    if config
        .database
        .redb_path
        .as_ref()
        .is_some_and(|p| p.starts_with("/tmp"))
    {
        recommendations
            .push("Consider using a persistent path for redb instead of /tmp".to_string());
    }

    if !config.cli.progress_bars {
        recommendations
            .push("Consider enabling progress bars for better user experience".to_string());
    }

    let check = ConfigCheck {
        validation,
        recommendations,
        security_issues,
    };

    format.print_output(&check)?;
    Ok(())
}

pub async fn show_config(
    _memory: &do_memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let display = ConfigDisplay {
        database: DatabaseConfigDisplay {
            turso_url: config.database.turso_url.clone(),
            turso_token_configured: config.database.turso_token.is_some(),
            redb_path: config.database.redb_path.clone(),
            storage_mode: config.database.storage_mode.clone(),
            db_path: config.database.db_path.clone(),
        },
        storage: StorageConfigDisplay {
            max_episodes_cache: config.storage.max_episodes_cache,
            cache_ttl_seconds: config.storage.cache_ttl_seconds,
            pool_size: config.storage.pool_size,
        },
        cli: CliConfigDisplay {
            default_format: config.cli.default_format.clone(),
            progress_bars: config.cli.progress_bars,
            batch_size: config.cli.batch_size,
        },
        features: vec![
            #[cfg(feature = "turso")]
            "turso".to_string(),
            #[cfg(feature = "redb")]
            "redb".to_string(),
        ]
        .into_iter()
        .filter(|_| true)
        .collect(),
    };

    format.print_output(&display)?;
    Ok(())
}

/// Run the interactive configuration wizard
///
/// This launches the step-by-step configuration wizard that helps users
/// set up their memory-cli configuration with optimal defaults.
pub async fn run_wizard() -> anyhow::Result<()> {
    let wizard = ConfigWizard::new();
    let _config = wizard.run().await?;
    Ok(())
}
