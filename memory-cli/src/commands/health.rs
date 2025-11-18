use clap::Subcommand;
use serde::Serialize;
use std::time::Duration;
use tokio::time;

use crate::config::Config;
use crate::output::{Output, OutputFormat};

#[derive(Subcommand)]
pub enum HealthCommands {
    /// Perform comprehensive health check
    Check,
    /// Show current health status
    Status,
    /// Monitor health continuously
    Monitor {
        /// Monitoring interval in seconds
        #[arg(short, long, default_value = "30")]
        interval: u64,
        /// Monitoring duration in seconds (0 for indefinite)
        #[arg(short, long, default_value = "300")]
        duration: u64,
    },
}

#[derive(Debug, Serialize)]
pub struct HealthCheckResult {
    pub overall_status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub metrics: SystemMetrics,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
    pub uptime_seconds: u64,
    pub active_connections: usize,
}

#[derive(Debug, Serialize)]
pub struct HealthStatusSummary {
    pub status: HealthStatus,
    pub last_check: String,
    pub next_check: Option<String>,
    pub issues: Vec<String>,
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

impl Output for HealthCheckResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        let status_color = match self.overall_status {
            HealthStatus::Healthy => Color::Green,
            HealthStatus::Degraded => Color::Yellow,
            HealthStatus::Unhealthy => Color::Red,
        };

        writeln!(writer, "{}", "Health Check Results".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(
            writer,
            "Overall Status: {}",
            format!("{:?}", self.overall_status)
                .color(status_color)
                .bold()
        )?;
        writeln!(writer, "Timestamp: {}", self.timestamp)?;

        writeln!(writer, "\nComponent Status:")?;
        for component in &self.components {
            let comp_color = match component.status {
                HealthStatus::Healthy => Color::Green,
                HealthStatus::Degraded => Color::Yellow,
                HealthStatus::Unhealthy => Color::Red,
            };
            write!(
                writer,
                "  {}: {}",
                component.name,
                format!("{:?}", component.status).color(comp_color)
            )?;
            if let Some(latency) = component.latency_ms {
                write!(writer, " ({}ms)", latency)?;
            }
            writeln!(writer)?;

            if let Some(error) = &component.error {
                writeln!(writer, "    Error: {}", error.red())?;
            }
        }

        writeln!(writer, "\nSystem Metrics:")?;
        if let Some(mem) = self.metrics.memory_usage_mb {
            writeln!(writer, "  Memory Usage: {:.1} MB", mem)?;
        }
        if let Some(cpu) = self.metrics.cpu_usage_percent {
            writeln!(writer, "  CPU Usage: {:.1}%", cpu)?;
        }
        writeln!(writer, "  Uptime: {}s", self.metrics.uptime_seconds)?;
        writeln!(
            writer,
            "  Active Connections: {}",
            self.metrics.active_connections
        )?;

        Ok(())
    }
}

impl Output for HealthStatusSummary {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        let status_color = match self.status {
            HealthStatus::Healthy => Color::Green,
            HealthStatus::Degraded => Color::Yellow,
            HealthStatus::Unhealthy => Color::Red,
        };

        writeln!(writer, "{}", "Health Status".bold())?;
        writeln!(
            writer,
            "Status: {}",
            format!("{:?}", self.status).color(status_color).bold()
        )?;
        writeln!(writer, "Last Check: {}", self.last_check)?;

        if let Some(next) = &self.next_check {
            writeln!(writer, "Next Check: {}", next)?;
        }

        if !self.issues.is_empty() {
            writeln!(writer, "\nIssues:")?;
            for issue in &self.issues {
                writeln!(writer, "  {}", issue.red())?;
            }
        }

        Ok(())
    }
}

// Command implementations
pub async fn health_check(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let mut components = Vec::new();

    // Check Turso storage
    if let Some(turso) = memory.turso_storage() {
        let start = std::time::Instant::now();
        match turso.get_episode(uuid::Uuid::new_v4()).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                components.push(ComponentHealth {
                    name: "Turso Storage".to_string(),
                    status: if latency < 100 {
                        HealthStatus::Healthy
                    } else {
                        HealthStatus::Degraded
                    },
                    latency_ms: Some(latency),
                    error: None,
                    details: serde_json::json!({
                        "type": "durable_storage",
                        "latency_threshold_ms": 100
                    }),
                });
            }
            Err(e) => {
                components.push(ComponentHealth {
                    name: "Turso Storage".to_string(),
                    status: HealthStatus::Unhealthy,
                    latency_ms: None,
                    error: Some(format!("Connection failed: {}", e)),
                    details: serde_json::json!({
                        "type": "durable_storage",
                        "error_type": "connection_error"
                    }),
                });
            }
        }
    } else {
        components.push(ComponentHealth {
            name: "Turso Storage".to_string(),
            status: HealthStatus::Unhealthy,
            latency_ms: None,
            error: Some("Not configured".to_string()),
            details: serde_json::json!({
                "type": "durable_storage",
                "configured": false
            }),
        });
    }

    // Check redb storage
    if let Some(cache) = memory.cache_storage() {
        let start = std::time::Instant::now();
        match cache.get_episode(uuid::Uuid::new_v4()).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                components.push(ComponentHealth {
                    name: "redb Cache".to_string(),
                    status: if latency < 10 {
                        HealthStatus::Healthy
                    } else {
                        HealthStatus::Degraded
                    },
                    latency_ms: Some(latency),
                    error: None,
                    details: serde_json::json!({
                        "type": "cache_storage",
                        "latency_threshold_ms": 10
                    }),
                });
            }
            Err(e) => {
                components.push(ComponentHealth {
                    name: "redb Cache".to_string(),
                    status: HealthStatus::Unhealthy,
                    latency_ms: None,
                    error: Some(format!("Access failed: {}", e)),
                    details: serde_json::json!({
                        "type": "cache_storage",
                        "error_type": "access_error"
                    }),
                });
            }
        }
    } else {
        components.push(ComponentHealth {
            name: "redb Cache".to_string(),
            status: HealthStatus::Unhealthy,
            latency_ms: None,
            error: Some("Not configured".to_string()),
            details: serde_json::json!({
                "type": "cache_storage",
                "configured": false
            }),
        });
    }

    // Get basic stats
    let (total_episodes, completed_episodes, total_patterns) = memory.get_stats().await;

    components.push(ComponentHealth {
        name: "Memory System".to_string(),
        status: HealthStatus::Healthy,
        latency_ms: None,
        error: None,
        details: serde_json::json!({
            "total_episodes": total_episodes,
            "completed_episodes": completed_episodes,
            "total_patterns": total_patterns
        }),
    });

    // Determine overall status
    let overall_status = if components
        .iter()
        .any(|c| c.status == HealthStatus::Unhealthy)
    {
        HealthStatus::Unhealthy
    } else if components
        .iter()
        .any(|c| c.status == HealthStatus::Degraded)
    {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };

    // Get system metrics (simplified)
    let metrics = SystemMetrics {
        memory_usage_mb: None,                     // Would need system monitoring
        cpu_usage_percent: None,                   // Would need system monitoring
        uptime_seconds: std::process::id() as u64, // Placeholder
        active_connections: components.len(),
    };

    let result = HealthCheckResult {
        overall_status,
        components,
        metrics,
        timestamp: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    };

    format.print_output(&result)?;
    Ok(())
}

pub async fn health_status(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    // Quick status check
    let mut issues = Vec::new();
    let mut status = HealthStatus::Healthy;

    // Check storage connectivity
    if let Some(turso) = memory.turso_storage() {
        if let Err(e) = turso.get_episode(uuid::Uuid::new_v4()).await {
            issues.push(format!("Turso storage unavailable: {}", e));
            status = HealthStatus::Unhealthy;
        }
    }

    if let Some(cache) = memory.cache_storage() {
        if let Err(e) = cache.get_episode(uuid::Uuid::new_v4()).await {
            issues.push(format!("redb cache unavailable: {}", e));
            if status == HealthStatus::Healthy {
                status = HealthStatus::Degraded;
            }
        }
    }

    let summary = HealthStatusSummary {
        status,
        last_check: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        next_check: None,
        issues,
    };

    format.print_output(&summary)?;
    Ok(())
}

pub async fn health_monitor(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    _format: OutputFormat,
    interval: u64,
    duration: u64,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!(
            "DRY RUN: Would monitor health every {}s for {}s",
            interval, duration
        );
        return Ok(());
    }

    let interval_duration = Duration::from_secs(interval);
    let total_duration = if duration > 0 {
        Some(Duration::from_secs(duration))
    } else {
        None
    };

    let start_time = std::time::Instant::now();

    println!(
        "Starting health monitoring (interval: {}s, duration: {}s)",
        interval, duration
    );
    println!("Press Ctrl+C to stop...");

    loop {
        // Perform health check
        let mut issues = Vec::new();
        let mut status = HealthStatus::Healthy;

        // Check storage connectivity
        if let Some(turso) = memory.turso_storage() {
            if let Err(e) = turso.get_episode(uuid::Uuid::new_v4()).await {
                issues.push(format!("Turso: {}", e));
                status = HealthStatus::Unhealthy;
            }
        }

        if let Some(cache) = memory.cache_storage() {
            if let Err(e) = cache.get_episode(uuid::Uuid::new_v4()).await {
                issues.push(format!("redb: {}", e));
                if status == HealthStatus::Healthy {
                    status = HealthStatus::Degraded;
                }
            }
        }

        let timestamp = chrono::Utc::now().format("%H:%M:%S");

        match status {
            HealthStatus::Healthy => println!("{} ✅ System healthy", timestamp),
            HealthStatus::Degraded => println!("{} ⚠️  System degraded", timestamp),
            HealthStatus::Unhealthy => println!("{} ❌ System unhealthy", timestamp),
        }

        if !issues.is_empty() {
            for issue in &issues {
                println!("  - {}", issue);
            }
        }

        // Check if we should stop
        if let Some(total) = total_duration {
            if start_time.elapsed() >= total {
                println!("Monitoring duration completed");
                break;
            }
        }

        // Wait for next check
        time::sleep(interval_duration).await;
    }

    Ok(())
}
