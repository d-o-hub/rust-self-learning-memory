use clap::Subcommand;
use colored::Colorize;
use serde::Serialize;

use crate::config::{Config, ConfigWizard};
use crate::output::{Output, OutputFormat};

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
}

#[derive(Debug, Serialize)]
pub struct ConfigValidation {
    pub is_valid: bool,
    pub issues: Vec<ConfigIssue>,
    pub connectivity: ConnectivityStatus,
}

#[derive(Debug, Serialize)]
pub struct ConfigIssue {
    pub level: IssueLevel,
    pub category: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum IssueLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Serialize)]
pub struct ConnectivityStatus {
    pub turso_connected: bool,
    pub redb_accessible: bool,
    pub latency_ms: Option<u64>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigCheck {
    pub validation: ConfigValidation,
    pub recommendations: Vec<String>,
    pub security_issues: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigDisplay {
    pub database: DatabaseConfigDisplay,
    pub storage: StorageConfigDisplay,
    pub cli: CliConfigDisplay,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DatabaseConfigDisplay {
    pub turso_url: Option<String>,
    pub turso_token_configured: bool,
    pub redb_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StorageConfigDisplay {
    pub max_episodes_cache: usize,
    pub cache_ttl_seconds: u64,
    pub pool_size: usize,
}

#[derive(Debug, Serialize)]
pub struct CliConfigDisplay {
    pub default_format: String,
    pub progress_bars: bool,
    pub batch_size: usize,
}

impl Output for ConfigValidation {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        if self.is_valid {
            writeln!(writer, "{}", "✅ Configuration is valid".green().bold())?;
        } else {
            writeln!(writer, "{}", "❌ Configuration has issues".red().bold())?;
        }

        if !self.issues.is_empty() {
            writeln!(writer, "\nIssues found:")?;
            for issue in &self.issues {
                let level_color = match issue.level {
                    IssueLevel::Error => Color::Red,
                    IssueLevel::Warning => Color::Yellow,
                    IssueLevel::Info => Color::Blue,
                };
                let level_str = match issue.level {
                    IssueLevel::Error => "ERROR",
                    IssueLevel::Warning => "WARN",
                    IssueLevel::Info => "INFO",
                };
                writeln!(
                    writer,
                    "  {} [{}] {}",
                    level_str.color(level_color).bold(),
                    issue.category,
                    issue.message
                )?;
                if let Some(suggestion) = &issue.suggestion {
                    writeln!(writer, "    💡 {}", suggestion.italic())?;
                }
            }
        }

        writeln!(writer, "\nConnectivity Status:")?;
        if self.connectivity.turso_connected {
            writeln!(writer, "  {} Turso: Connected", "✅".green())?;
        } else {
            writeln!(writer, "  {} Turso: Not connected", "❌".red())?;
        }

        if self.connectivity.redb_accessible {
            writeln!(writer, "  {} redb: Accessible", "✅".green())?;
        } else {
            writeln!(writer, "  {} redb: Not accessible", "❌".red())?;
        }

        if let Some(latency) = self.connectivity.latency_ms {
            writeln!(writer, "  Latency: {}ms", latency)?;
        }

        if !self.connectivity.errors.is_empty() {
            writeln!(writer, "\nConnection errors:")?;
            for error in &self.connectivity.errors {
                writeln!(writer, "  {}", error.red())?;
            }
        }

        Ok(())
    }
}

impl Output for ConfigCheck {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        // First show validation results
        self.validation.write_human(&mut writer)?;

        if !self.recommendations.is_empty() {
            writeln!(writer, "\nRecommendations:")?;
            for rec in &self.recommendations {
                writeln!(writer, "  💡 {}", rec)?;
            }
        }

        if !self.security_issues.is_empty() {
            writeln!(writer, "\n{} Security Issues:", "⚠️".yellow())?;
            for issue in &self.security_issues {
                writeln!(writer, "  {}", issue.red())?;
            }
        }

        Ok(())
    }
}

impl Output for ConfigDisplay {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Configuration Overview".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;

        writeln!(writer, "\nDatabase:")?;
        if let Some(url) = &self.database.turso_url {
            writeln!(writer, "  Turso URL: {}", url)?;
        } else {
            writeln!(writer, "  Turso URL: {}", "Not configured".yellow())?;
        }
        writeln!(
            writer,
            "  Turso Token: {}",
            if self.database.turso_token_configured {
                "Configured"
            } else {
                "Not configured"
            }
        )?;
        if let Some(path) = &self.database.redb_path {
            writeln!(writer, "  redb Path: {}", path)?;
        } else {
            writeln!(writer, "  redb Path: {}", "Not configured".yellow())?;
        }

        writeln!(writer, "\nStorage:")?;
        writeln!(
            writer,
            "  Max Episodes Cache: {}",
            self.storage.max_episodes_cache
        )?;
        writeln!(writer, "  Cache TTL: {}s", self.storage.cache_ttl_seconds)?;
        writeln!(writer, "  Pool Size: {}", self.storage.pool_size)?;

        writeln!(writer, "\nCLI:")?;
        writeln!(writer, "  Default Format: {}", self.cli.default_format)?;
        writeln!(writer, "  Progress Bars: {}", self.cli.progress_bars)?;
        writeln!(writer, "  Batch Size: {}", self.cli.batch_size)?;

        if !self.features.is_empty() {
            writeln!(writer, "\nEnabled Features:")?;
            for feature in &self.features {
                writeln!(writer, "  • {}", feature)?;
            }
        }

        Ok(())
    }
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
                #[allow(clippy::cast_possible_truncation)]
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
                #[allow(clippy::cast_possible_truncation)]
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

#[cfg(test)]
mod config_command_tests {
    use super::*;
    use crate::output::Output;

    #[test]
    fn test_config_validation_write_human_invalid() {
        let validation = ConfigValidation {
            is_valid: false,
            issues: vec![ConfigIssue {
                level: IssueLevel::Error,
                category: "database".to_string(),
                message: "Test error".to_string(),
                suggestion: Some("Fix it".to_string()),
            }],
            connectivity: ConnectivityStatus {
                turso_connected: false,
                redb_accessible: false,
                latency_ms: None,
                errors: vec!["Connection failed".to_string()],
            },
        };

        let mut buffer = Vec::new();
        validation.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Configuration has issues"));
        assert!(output.contains("ERROR"));
        assert!(output.contains("Test error"));
        assert!(output.contains("Fix it"));
        assert!(output.contains("Turso: Not connected"));
        assert!(output.contains("redb: Not accessible"));
        assert!(output.contains("Connection failed"));
    }

    #[test]
    fn test_config_validation_write_human_with_warnings() {
        let validation = ConfigValidation {
            is_valid: true,
            issues: vec![ConfigIssue {
                level: IssueLevel::Warning,
                category: "performance".to_string(),
                message: "Cache size is low".to_string(),
                suggestion: Some("Increase cache".to_string()),
            }],
            connectivity: ConnectivityStatus {
                turso_connected: true,
                redb_accessible: true,
                latency_ms: Some(50),
                errors: vec![],
            },
        };

        let mut buffer = Vec::new();
        validation.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Configuration is valid"));
        assert!(output.contains("WARN"));
        assert!(output.contains("Cache size is low"));
        assert!(output.contains("Turso: Connected"));
        assert!(output.contains("redb: Accessible"));
        assert!(output.contains("Latency: 50ms"));
    }

    #[test]
    fn test_config_check_write_human_with_security_issues() {
        let check = ConfigCheck {
            validation: ConfigValidation {
                is_valid: true,
                issues: vec![],
                connectivity: ConnectivityStatus {
                    turso_connected: true,
                    redb_accessible: true,
                    latency_ms: None,
                    errors: vec![],
                },
            },
            recommendations: vec!["Enable progress bars".to_string()],
            security_issues: vec!["Token missing".to_string()],
        };

        let mut buffer = Vec::new();
        check.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Recommendations"));
        assert!(output.contains("Enable progress bars"));
        assert!(output.contains("Security Issues"));
        assert!(output.contains("Token missing"));
    }

    #[test]
    fn test_config_display_write_human_with_none_urls() {
        let display = ConfigDisplay {
            database: DatabaseConfigDisplay {
                turso_url: None,
                turso_token_configured: false,
                redb_path: None,
            },
            storage: StorageConfigDisplay {
                max_episodes_cache: 1000,
                cache_ttl_seconds: 3600,
                pool_size: 10,
            },
            cli: CliConfigDisplay {
                default_format: "human".to_string(),
                progress_bars: true,
                batch_size: 100,
            },
            features: vec!["turso".to_string(), "redb".to_string()],
        };

        let mut buffer = Vec::new();
        display.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Turso URL: Not configured"));
        assert!(output.contains("Turso Token: Not configured"));
        assert!(output.contains("redb Path: Not configured"));
        assert!(output.contains("Enabled Features"));
        assert!(output.contains("turso"));
        assert!(output.contains("redb"));
    }

    #[test]
    fn test_config_display_write_human_with_empty_features() {
        let display = ConfigDisplay {
            database: DatabaseConfigDisplay {
                turso_url: Some("file:test.db".to_string()),
                turso_token_configured: true,
                redb_path: Some("memory.redb".to_string()),
            },
            storage: StorageConfigDisplay {
                max_episodes_cache: 500,
                cache_ttl_seconds: 1800,
                pool_size: 5,
            },
            cli: CliConfigDisplay {
                default_format: "json".to_string(),
                progress_bars: false,
                batch_size: 50,
            },
            features: vec![],
        };

        let mut buffer = Vec::new();
        display.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Turso URL: file:test.db"));
        assert!(output.contains("Turso Token: Configured"));
        assert!(output.contains("redb Path: memory.redb"));
        assert!(!output.contains("Enabled Features"));
    }

    #[test]
    fn test_config_issue_level_info() {
        let validation = ConfigValidation {
            is_valid: true,
            issues: vec![ConfigIssue {
                level: IssueLevel::Info,
                category: "tips".to_string(),
                message: "Consider enabling feature X".to_string(),
                suggestion: None,
            }],
            connectivity: ConnectivityStatus {
                turso_connected: true,
                redb_accessible: true,
                latency_ms: None,
                errors: vec![],
            },
        };

        let mut buffer = Vec::new();
        validation.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("INFO"));
        assert!(output.contains("Consider enabling feature X"));
    }

    #[test]
    fn test_config_validation_json_serialization() {
        let validation = ConfigValidation {
            is_valid: true,
            issues: vec![],
            connectivity: ConnectivityStatus {
                turso_connected: true,
                redb_accessible: true,
                latency_ms: Some(100),
                errors: vec![],
            },
        };

        // Test that we can serialize to JSON
        let json = serde_json::to_string(&validation).unwrap();
        assert!(json.contains("is_valid"));
        assert!(json.contains("turso_connected"));
        assert!(json.contains("latency_ms"));
    }

    #[test]
    fn test_connectivity_status_with_errors() {
        let connectivity = ConnectivityStatus {
            turso_connected: false,
            redb_accessible: false,
            latency_ms: None,
            errors: vec![
                "Turso connection timeout".to_string(),
                "redb permission denied".to_string(),
            ],
        };

        let validation = ConfigValidation {
            is_valid: true,
            issues: vec![],
            connectivity,
        };

        let mut buffer = Vec::new();
        validation.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Connection errors"));
        assert!(output.contains("Turso connection timeout"));
        assert!(output.contains("redb permission denied"));
    }
}
