//! Backup command types and output implementations.

use clap::Subcommand;
use serde::Serialize;
use std::path::PathBuf;

use crate::output::Output;

#[derive(Subcommand)]
pub enum BackupCommands {
    /// Create a new backup
    Create {
        /// Backup destination path
        #[arg(short, long)]
        path: PathBuf,
        /// Backup format (json, jsonl, sql)
        #[arg(short, long, default_value = "json")]
        format: BackupFormat,
        /// Compress the backup
        #[arg(short, long)]
        compress: bool,
    },
    /// List available backups
    List {
        /// Backup directory path
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Restore from backup
    Restore {
        /// Backup directory path
        #[arg(short, long)]
        path: PathBuf,
        /// Backup ID to restore
        #[arg(short, long)]
        backup_id: String,
        /// Force restore (overwrite existing data)
        #[arg(short, long)]
        force: bool,
    },
    /// Verify backup integrity
    Verify {
        /// Backup directory path
        #[arg(short, long)]
        path: PathBuf,
        /// Backup ID to verify
        #[arg(short, long)]
        backup_id: String,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum BackupFormat {
    Json,
    Jsonl,
    Sql,
}

#[derive(Debug, Serialize)]
pub struct BackupResult {
    pub backup_id: String,
    pub path: String,
    pub format: String,
    pub compressed: bool,
    pub episodes_count: usize,
    pub patterns_count: usize,
    pub size_bytes: u64,
    pub duration_ms: u64,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct BackupList {
    pub backups: Vec<BackupInfo>,
    pub total_size_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct BackupInfo {
    pub id: String,
    pub timestamp: String,
    pub format: String,
    pub compressed: bool,
    pub episodes_count: usize,
    pub patterns_count: usize,
    pub size_bytes: u64,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct RestoreResult {
    pub backup_id: String,
    pub episodes_restored: usize,
    pub patterns_restored: usize,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct VerifyResult {
    pub backup_id: String,
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub episodes_count: usize,
    pub patterns_count: usize,
}

// Output implementations
impl Output for BackupResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Backup Created Successfully".bold().green())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(writer, "Backup ID: {}", self.backup_id.cyan())?;
        writeln!(writer, "Path: {}", self.path)?;
        writeln!(writer, "Format: {}", self.format)?;
        writeln!(
            writer,
            "Compressed: {}",
            if self.compressed { "Yes" } else { "No" }
        )?;
        writeln!(writer, "Episodes: {}", self.episodes_count)?;
        writeln!(writer, "Patterns: {}", self.patterns_count)?;
        writeln!(
            writer,
            "Size: {:.2} MB",
            self.size_bytes as f64 / 1_000_000.0
        )?;
        writeln!(writer, "Duration: {}ms", self.duration_ms)?;
        writeln!(writer, "Timestamp: {}", self.timestamp)?;

        Ok(())
    }
}

impl Output for BackupList {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Available Backups".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;

        if self.backups.is_empty() {
            writeln!(writer, "No backups found")?;
            return Ok(());
        }

        writeln!(
            writer,
            "{:<36} {:<12} {:<8} {:<8} {:<10} {:<12}",
            "ID", "Timestamp", "Format", "Episodes", "Patterns", "Size (MB)"
        )?;
        writeln!(writer, "{}", "─".repeat(90))?;

        for backup in &self.backups {
            writeln!(
                writer,
                "{:<36} {:<12} {:<8} {:<8} {:<10} {:.2}",
                backup.id,
                backup.timestamp,
                backup.format,
                backup.episodes_count,
                backup.patterns_count,
                backup.size_bytes as f64 / 1_000_000.0
            )?;
        }

        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(
            writer,
            "Total Size: {:.2} MB",
            self.total_size_bytes as f64 / 1_000_000.0
        )?;

        Ok(())
    }
}

impl Output for RestoreResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Restore Complete".bold().green())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(writer, "Backup ID: {}", self.backup_id.cyan())?;
        writeln!(writer, "Episodes Restored: {}", self.episodes_restored)?;
        writeln!(writer, "Patterns Restored: {}", self.patterns_restored)?;
        writeln!(writer, "Duration: {}ms", self.duration_ms)?;

        if !self.errors.is_empty() {
            writeln!(writer, "\nErrors ({}):", self.errors.len())?;
            for (i, error) in self.errors.iter().enumerate() {
                writeln!(writer, "{}. {}", i + 1, error.red())?;
            }
        }

        Ok(())
    }
}

impl Output for VerifyResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Backup Verification".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(writer, "Backup ID: {}", self.backup_id.cyan())?;

        if self.is_valid {
            writeln!(writer, "Status: {}", "VALID".green())?;
        } else {
            writeln!(writer, "Status: {}", "INVALID".red())?;
        }

        writeln!(writer, "Episodes: {}", self.episodes_count)?;
        writeln!(writer, "Patterns: {}", self.patterns_count)?;

        if !self.issues.is_empty() {
            writeln!(writer, "\nIssues Found ({}):", self.issues.len())?;
            for (i, issue) in self.issues.iter().enumerate() {
                writeln!(writer, "{}. {}", i + 1, issue.red())?;
            }
        }

        Ok(())
    }
}
