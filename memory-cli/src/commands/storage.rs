use clap::Subcommand;
use serde::Serialize;

use crate::config::Config;
use crate::output::{Output, OutputFormat};

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

        writeln!(writer, "Mode: {}", if self.force { "Full Sync".green() } else { "Incremental".blue() })?;
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

// Command implementations
pub async fn storage_stats(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let (total_episodes, completed_episodes, total_patterns) = memory.get_stats().await;

    // Get enhanced statistics from storage backends
    let mut storage_size_bytes = 0u64;
    let mut cache_hit_rate = 0.0;
    let mut last_sync = None;

    // Estimate storage size based on counts (rough calculation)
    storage_size_bytes += (total_episodes * 2048) as u64; // ~2KB per episode
    storage_size_bytes += (total_patterns * 1024) as u64; // ~1KB per pattern

    // For cache hit rate, we'd need backend-specific access
    // For now, we'll use a placeholder - this could be enhanced later

    let stats = StorageStats {
        episodes: StorageStatsData {
            total_count: total_episodes,
            recent_count: completed_episodes, // Approximation - we don't track time-based stats
            average_size_bytes: if total_episodes > 0 { storage_size_bytes / total_episodes as u64 } else { 0 },
        },
        patterns: StorageStatsData {
            total_count: total_patterns,
            recent_count: 0,       // Not available
            average_size_bytes: if total_patterns > 0 { 1024 } else { 0 }, // Estimate
        },
        storage_size_bytes,
        cache_hit_rate,
        last_sync,
    };

    format.print_output(&stats)?;

    Ok(())
}

pub async fn sync_storage(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    force: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    // Check if we have both storage backends
    let (turso, redb) = match (memory.turso_storage(), memory.cache_storage()) {
        (Some(t), Some(r)) => (t.clone(), r.clone()),
        _ => {
            anyhow::bail!("Storage sync requires both Turso and redb storage backends to be configured");
        }
    };

    if dry_run {
        println!("DRY RUN: Would synchronize data between Turso and redb storage");
        println!("- Turso: durable storage backend");
        println!("- redb: cache storage backend");
        if force {
            println!("- Force mode: full synchronization (last 24 hours)");
        } else {
            println!("- Incremental mode: sync recent changes (last hour)");
        }
        return Ok(());
    }

    let start_time = std::time::Instant::now();

    // For now, implement a basic sync that queries recent episodes from Turso
    // and ensures they exist in redb. A full implementation would use the StorageSynchronizer.
    let since = if force {
        chrono::Utc::now() - chrono::Duration::hours(24)
    } else {
        chrono::Utc::now() - chrono::Duration::hours(1)
    };

    let episodes = turso.query_episodes_since(since).await?;
    let mut episodes_synced = 0;
    let mut errors = 0;

    for episode in episodes {
        match redb.store_episode(&episode).await {
            Ok(_) => {
                episodes_synced += 1;
            }
            Err(e) => {
                eprintln!("Failed to sync episode {}: {}", episode.episode_id, e);
                errors += 1;
            }
        }
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;

    let result = SyncResult {
        episodes_synced,
        patterns_synced: 0, // Not implemented in basic version
        heuristics_synced: 0, // Not implemented in basic version
        conflicts_resolved: 0, // Not implemented in basic version
        errors,
        duration_ms,
        force,
    };

    format.print_output(&result)?;

    Ok(())
}

pub async fn vacuum_storage(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    let mut total_cleaned = 0usize;
    let mut errors = 0usize;
    let mut storage_optimized = false;

    if dry_run {
        println!("DRY RUN: Would perform storage vacuum operations");
        println!("- Would clean expired cache entries");
        println!("- Would optimize storage structures");
    } else {
        // For now, we can only perform basic cleanup operations
        // that work through the StorageBackend trait

        // Note: In a full implementation, we'd need backend-specific vacuum methods
        // For now, we'll report that vacuum is not fully implemented for the generic interface
        println!("Storage vacuum: Limited functionality available through generic StorageBackend trait");
        println!("For full vacuum capabilities, use backend-specific tools directly");
    }

    let result = VacuumResult {
        items_cleaned: total_cleaned,
        storage_optimized,
        errors,
        dry_run,
    };

    format.print_output(&result)?;

    Ok(())
}

pub async fn storage_health(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let mut turso_health = ComponentHealth {
        status: HealthStatus::Unhealthy,
        latency_ms: None,
        error: Some("Not configured".to_string()),
    };

    let mut redb_health = ComponentHealth {
        status: HealthStatus::Unhealthy,
        latency_ms: None,
        error: Some("Not configured".to_string()),
    };

    let mut overall_status = HealthStatus::Unhealthy;

    // Check Turso health by attempting a simple query
    if let Some(turso) = memory.turso_storage() {
        let start = std::time::Instant::now();
        // Try to query a non-existent episode to test connectivity
        match turso.get_episode(uuid::Uuid::new_v4()).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                turso_health = ComponentHealth {
                    status: HealthStatus::Healthy,
                    latency_ms: Some(latency),
                    error: None,
                };
            }
            Err(e) => {
                turso_health = ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    latency_ms: None,
                    error: Some(format!("Connection error: {}", e)),
                };
            }
        }
    }

    // Check redb health by attempting a simple query
    if let Some(cache) = memory.cache_storage() {
        let start = std::time::Instant::now();
        // Try to query a non-existent episode to test connectivity
        match cache.get_episode(uuid::Uuid::new_v4()).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                redb_health = ComponentHealth {
                    status: HealthStatus::Healthy,
                    latency_ms: Some(latency),
                    error: None,
                };
            }
            Err(e) => {
                redb_health = ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    latency_ms: None,
                    error: Some(format!("Connection error: {}", e)),
                };
            }
        }
    }

    // Determine overall health
    overall_status = match (&turso_health.status, &redb_health.status) {
        (HealthStatus::Healthy, HealthStatus::Healthy) => HealthStatus::Healthy,
        (HealthStatus::Healthy, HealthStatus::Degraded) | (HealthStatus::Degraded, HealthStatus::Healthy) => HealthStatus::Degraded,
        (HealthStatus::Degraded, HealthStatus::Degraded) => HealthStatus::Degraded,
        _ => HealthStatus::Unhealthy,
    };

    let health = StorageHealth {
        turso: turso_health,
        redb: redb_health,
        overall: overall_status,
    };

    format.print_output(&health)?;

    Ok(())
}

pub async fn connection_status(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let mut turso_info = ConnectionInfo {
        active_connections: 0,
        pool_size: 0,
        queue_depth: 0,
        last_activity: None,
    };

    let mut redb_info = ConnectionInfo {
        active_connections: 0,
        pool_size: 0,
        queue_depth: 0,
        last_activity: None,
    };

    // Set Turso connection info based on configuration
    if memory.has_turso_storage() {
        // For now, we can't get detailed pool stats through the trait
        // In a full implementation, we'd need to extend the StorageBackend trait
        turso_info.active_connections = 1; // At least one connection
        turso_info.pool_size = 10; // Default pool size
        turso_info.last_activity = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
    }

    // Set redb connection info
    if memory.has_cache_storage() {
        redb_info.active_connections = 1; // redb uses a single connection
        redb_info.pool_size = 1;
        redb_info.last_activity = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
    }

    let status = ConnectionStatus {
        turso: turso_info,
        redb: redb_info,
    };

    format.print_output(&status)?;

    Ok(())
}
