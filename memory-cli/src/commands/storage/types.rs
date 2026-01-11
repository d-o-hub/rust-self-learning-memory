//! Storage command types and data structures.

use clap::Subcommand;
use serde::Serialize;

use crate::output::Output;

#[derive(Subcommand)]
pub enum StorageCommands {
    /// Show storage statistics
    Stats,
    /// Synchronize storage backends
    Sync {
        /// Force full synchronization
        #[arg(long)]
        force: bool,

        /// Show what would be done without executing
        #[arg(long)]
        dry_run: bool,
    },
    /// Vacuum and optimize storage
    Vacuum {
        /// Show what would be done without executing
        #[arg(long)]
        dry_run: bool,
    },
    /// Check storage health
    Health,
    /// Show connection status
    Connections,
}

#[derive(Debug, Serialize)]
pub struct StorageStats {
    pub episodes: StorageStatsData,
    pub patterns: StorageStatsData,
    pub storage_size_bytes: u64,
    pub cache_hit_rate: f32,
    pub last_sync: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StorageStatsData {
    pub total_count: usize,
    pub recent_count: usize, // Last 24 hours
    pub average_size_bytes: u64,
}

impl Output for StorageStats {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Storage Statistics".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;

        writeln!(writer, "Episodes:")?;
        writeln!(writer, "  Total: {}", self.episodes.total_count)?;
        writeln!(writer, "  Recent (24h): {}", self.episodes.recent_count)?;
        writeln!(
            writer,
            "  Avg Size: {} bytes",
            self.episodes.average_size_bytes
        )?;

        writeln!(writer, "\nPatterns:")?;
        writeln!(writer, "  Total: {}", self.patterns.total_count)?;
        writeln!(writer, "  Recent (24h): {}", self.patterns.recent_count)?;
        writeln!(
            writer,
            "  Avg Size: {} bytes",
            self.patterns.average_size_bytes
        )?;

        writeln!(writer, "\nStorage:")?;
        writeln!(
            writer,
            "  Total Size: {:.2} MB",
            self.storage_size_bytes as f32 / 1_000_000.0
        )?;
        writeln!(
            writer,
            "  Cache Hit Rate: {:.1}%",
            self.cache_hit_rate * 100.0
        )?;

        if let Some(last_sync) = &self.last_sync {
            writeln!(writer, "  Last Sync: {}", last_sync)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct StorageHealth {
    pub turso: ComponentHealth,
    pub redb: ComponentHealth,
    pub overall: HealthStatus,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Degraded => write!(f, "Degraded"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
        }
    }
}

impl Output for StorageHealth {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        let overall_color = match self.overall {
            HealthStatus::Healthy => Color::Green,
            HealthStatus::Degraded => Color::Yellow,
            HealthStatus::Unhealthy => Color::Red,
        };

        writeln!(writer, "{}", "Storage Health".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(
            writer,
            "Overall: {}",
            format!("{:?}", self.overall).color(overall_color).bold()
        )?;

        writeln!(writer, "\nTurso:")?;
        let turso_color = match self.turso.status {
            HealthStatus::Healthy => Color::Green,
            HealthStatus::Degraded => Color::Yellow,
            HealthStatus::Unhealthy => Color::Red,
        };
        writeln!(
            writer,
            "  Status: {}",
            format!("{:?}", self.turso.status).color(turso_color)
        )?;
        if let Some(latency) = self.turso.latency_ms {
            writeln!(writer, "  Latency: {}ms", latency)?;
        }
        if let Some(error) = &self.turso.error {
            writeln!(writer, "  Error: {}", error.red())?;
        }

        writeln!(writer, "\nredb:")?;
        let redb_color = match self.redb.status {
            HealthStatus::Healthy => Color::Green,
            HealthStatus::Degraded => Color::Yellow,
            HealthStatus::Unhealthy => Color::Red,
        };
        writeln!(
            writer,
            "  Status: {}",
            format!("{:?}", self.redb.status).color(redb_color)
        )?;
        if let Some(latency) = self.redb.latency_ms {
            writeln!(writer, "  Latency: {}ms", latency)?;
        }
        if let Some(error) = &self.redb.error {
            writeln!(writer, "  Error: {}", error.red())?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionStatus {
    pub turso: ConnectionInfo,
    pub redb: ConnectionInfo,
}

#[derive(Debug, Serialize)]
pub struct ConnectionInfo {
    pub active_connections: usize,
    pub pool_size: usize,
    pub queue_depth: usize,
    pub last_activity: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub episodes_synced: usize,
    pub patterns_synced: usize,
    pub heuristics_synced: usize,
    pub conflicts_resolved: usize,
    pub errors: usize,
    pub duration_ms: u64,
    pub force: bool,
}

impl Output for SyncResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Storage Synchronization Complete".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;

        writeln!(
            writer,
            "Mode: {}",
            if self.force {
                "Full Sync".green()
            } else {
                "Incremental".blue()
            }
        )?;
        writeln!(writer, "Episodes synced: {}", self.episodes_synced)?;
        writeln!(writer, "Patterns synced: {}", self.patterns_synced)?;
        writeln!(writer, "Heuristics synced: {}", self.heuristics_synced)?;
        writeln!(writer, "Conflicts resolved: {}", self.conflicts_resolved)?;

        if self.errors > 0 {
            writeln!(writer, "Errors: {}", self.errors.to_string().red())?;
        }

        if self.duration_ms > 0 {
            writeln!(writer, "Duration: {}ms", self.duration_ms)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct VacuumResult {
    pub items_cleaned: usize,
    pub storage_optimized: bool,
    pub errors: usize,
    pub dry_run: bool,
}

impl Output for VacuumResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Storage Vacuum Complete".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;

        if self.dry_run {
            writeln!(writer, "Mode: {}", "Dry Run".yellow())?;
        } else {
            writeln!(writer, "Mode: {}", "Live Operation".green())?;
        }

        writeln!(writer, "Items cleaned: {}", self.items_cleaned)?;

        if self.storage_optimized {
            writeln!(writer, "Storage: {}", "Optimized".green())?;
        } else {
            writeln!(writer, "Storage: {}", "Not optimized".yellow())?;
        }

        if self.errors > 0 {
            writeln!(writer, "Errors: {}", self.errors.to_string().red())?;
        }

        Ok(())
    }
}

impl Output for ConnectionStatus {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Connection Status".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;

        writeln!(writer, "Turso:")?;
        writeln!(
            writer,
            "  Active: {}/{}",
            self.turso.active_connections, self.turso.pool_size
        )?;
        writeln!(writer, "  Queue: {}", self.turso.queue_depth)?;
        if let Some(activity) = &self.turso.last_activity {
            writeln!(writer, "  Last Activity: {}", activity)?;
        }

        writeln!(writer, "\nredb:")?;
        writeln!(
            writer,
            "  Active: {}/{}",
            self.redb.active_connections, self.redb.pool_size
        )?;
        writeln!(writer, "  Queue: {}", self.redb.queue_depth)?;
        if let Some(activity) = &self.redb.last_activity {
            writeln!(writer, "  Last Activity: {}", activity)?;
        }

        Ok(())
    }
}
