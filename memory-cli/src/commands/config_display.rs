//! Config command display types and human-readable Output impls.

use colored::Colorize;
use serde::Serialize;

use crate::output::Output;

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
    /// Canonical storage mode: "remote" | "local" | "memory"
    pub storage_mode: Option<String>,
    /// Local Turso SQLite path (when storage_mode = "local")
    pub db_path: Option<String>,
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
        if let Some(mode) = &self.database.storage_mode {
            writeln!(writer, "  Storage Mode: {}", mode)?;
        } else {
            writeln!(
                writer,
                "  Storage Mode: {} (default remote; set [database].storage_mode or --storage-mode)",
                "Not set".yellow()
            )?;
        }
        if let Some(path) = &self.database.db_path {
            writeln!(writer, "  DB Path: {}", path)?;
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
