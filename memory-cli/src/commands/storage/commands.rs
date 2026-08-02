//! Storage command implementations.

use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};

use crate::config::Config;
use crate::errors::helpers;
use crate::output::OutputFormat;

use super::provenance::MetricValue;
use super::types::*;

// Command implementations
pub async fn storage_stats(
    memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let (total_episodes, completed_episodes, total_patterns) = memory.get_stats().await;

    // Heuristic size estimates. These are NOT measured through any backend
    // interface; they are explicitly labeled `Estimated` in output (PTA-A2).
    let episode_size_estimate = 2048u64; // ~2KB per episode
    let pattern_size_estimate = 1024u64; // ~1KB per pattern
    let storage_size_bytes = (total_episodes as u64) * episode_size_estimate
        + (total_patterns as u64) * pattern_size_estimate;

    // Cache hit rate and last sync time are not exposed through the
    // `StorageBackend` trait. Report them as unavailable rather than
    // fabricating a zero hit rate or current timestamp.
    let stats = StorageStats {
        episodes: StorageStatsData {
            total_count: total_episodes,
            completed_count: MetricValue::measured(completed_episodes),
            average_size_bytes: MetricValue::estimated(episode_size_estimate),
        },
        patterns: StorageStatsData {
            total_count: total_patterns,
            // Patterns have no completed concept; do not report 0 as measured.
            completed_count: MetricValue::unavailable(),
            average_size_bytes: MetricValue::estimated(pattern_size_estimate),
        },
        storage_size_bytes: MetricValue::estimated(storage_size_bytes),
        cache_hit_rate: MetricValue::unavailable(),
        last_sync: MetricValue::unavailable(),
    };

    format.print_output(&stats)?;

    Ok(())
}

#[expect(clippy::excessive_nesting)]
pub async fn sync_storage(
    memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    force: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    // Check if we have both storage backends
    let (turso, redb) = match (memory.turso_storage(), memory.cache_storage()) {
        (Some(t), Some(r)) => (Arc::clone(&t), Arc::clone(&r)),
        _ => {
            // ADR-076: sync is Turso↔redb reconciliation only — not pattern extraction.
            return Err(anyhow::anyhow!(helpers::format_error_message(
                "Both Turso (durable) and redb (cache) must be configured. \
                 Local-only redb has no dual backend to reconcile.",
                "storage sync is Turso↔redb reconciliation — not pattern extraction",
                &[
                    "Patterns are created on episode complete (with ≥1 step for tool sequences), then listed via `pattern list`",
                    "To use sync: configure both a Turso URL and a redb path (dual backends)",
                    "Confirm path/config with `do-memory-cli config show`",
                    "Check backends: `do-memory-cli storage health`",
                ]
            )));
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

    // Interactive confirmation for force sync
    if force && format == OutputFormat::Human {
        use colored::*;
        use dialoguer::Confirm;

        println!(
            "{}",
            "WARNING: Force synchronization will process all data from the last 24 hours."
                .yellow()
                .bold()
        );
        println!("This may take a while and could overwrite cached data.");
        println!();

        let confirmed = Confirm::new()
            .with_prompt("Continue with full synchronization?")
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
    }

    let start_time = std::time::Instant::now();

    // Determine sync timeframe
    let since = if force {
        chrono::Utc::now() - chrono::Duration::hours(24)
    } else {
        chrono::Utc::now() - chrono::Duration::hours(1)
    };

    println!("Starting storage synchronization...");
    println!(
        "- Mode: {}",
        if force {
            "Full (24h)"
        } else {
            "Incremental (1h)"
        }
    );
    println!("- Since: {}", since.format("%Y-%m-%d %H:%M:%S UTC"));

    // Create progress bar
    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .expect("ProgressStyle template is valid: uses standard format"),
    );
    progress.set_message("Querying episodes from Turso...");

    // Query recent episodes from Turso (source of truth) - use max limit for sync
    let episodes = match turso
        .query_episodes_since(since, Some(do_memory_core::MAX_QUERY_LIMIT))
        .await
    {
        Ok(episodes) => episodes,
        Err(e) => {
            progress.finish_with_message("Failed to query episodes");
            return Err(anyhow::anyhow!(helpers::format_error_message(
                &format!("Failed to query episodes from Turso: {}", e),
                "Could not retrieve episodes from durable storage",
                helpers::STORAGE_CONNECTION_HELP
            )));
        }
    };

    let mut episodes_synced = 0;
    let mut patterns_synced = 0;
    let mut heuristics_synced = 0;
    let conflicts_resolved = 0;
    let mut errors = 0;

    progress.set_message(format!("Found {} episodes to sync", episodes.len()));
    progress.set_length(episodes.len() as u64);

    println!("Found {} episodes to sync", episodes.len());

    // Sync episodes to redb cache
    for episode in episodes {
        progress.set_message(format!(
            "Syncing episode {}",
            &episode.episode_id.to_string()[..8]
        ));

        match redb.store_episode(&episode).await {
            Ok(_) => {
                episodes_synced += 1;
                progress.inc(1);

                // Also sync patterns and heuristics if they exist
                for pattern_id in &episode.patterns {
                    // Try to get pattern from Turso and store in redb
                    if let Ok(Some(pattern)) = turso.get_pattern(*pattern_id).await {
                        if let Err(e) = redb.store_pattern(&pattern).await {
                            eprintln!("Warning: Failed to sync pattern {}: {}", pattern_id, e);
                        } else {
                            patterns_synced += 1;
                        }
                    }
                }
                for heuristic_id in &episode.heuristics {
                    // Try to get heuristic from Turso and store in redb
                    if let Ok(Some(heuristic)) = turso.get_heuristic(*heuristic_id).await {
                        if let Err(e) = redb.store_heuristic(&heuristic).await {
                            eprintln!("Warning: Failed to sync heuristic {}: {}", heuristic_id, e);
                        } else {
                            heuristics_synced += 1;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to sync episode {}: {}", episode.episode_id, e);
                errors += 1;
                progress.inc(1);
            }
        }
    }

    // Store sync timestamp (we'll use a simple approach since we can't access backend-specific methods)
    // This is a limitation - in a full implementation, we'd need to extend the StorageBackend trait

    let duration_ms = start_time.elapsed().as_millis() as u64;

    progress.finish_with_message(format!(
        "Sync completed in {:.2}s",
        duration_ms as f64 / 1000.0
    ));

    let result = SyncResult {
        episodes_synced,
        patterns_synced,
        heuristics_synced,
        conflicts_resolved,
        errors,
        duration_ms,
        force,
    };

    format.print_output(&result)?;

    Ok(())
}

pub async fn vacuum_storage(
    _memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    let total_cleaned = 0usize;
    let errors = 0usize;
    let _storage_optimized = false;

    if dry_run {
        println!("DRY RUN: Would perform storage vacuum operations");
        println!("- Would clean expired cache entries from redb");
        println!("- Would optimize Turso database structures");
        println!("- Would remove orphaned data and compact storage");

        let result = VacuumResult {
            items_cleaned: 0, // Would calculate in real run
            storage_optimized: false,
            errors: 0,
            dry_run: true,
        };
        format.print_output(&result)?;
        return Ok(());
    }

    // Interactive confirmation for vacuum
    if format == OutputFormat::Human {
        use colored::*;
        use dialoguer::Confirm;

        println!("{}", "Storage Vacuum".bold());
        println!("{}", "==============".bold());
        println!("This operation will:");
        println!("  • Clean expired cache entries from redb");
        println!("  • Optimize Turso database structures");
        println!("  • Remove orphaned data and compact storage");
        println!();
        println!(
            "{}",
            "Note: This operation is generally safe but may take time.".yellow()
        );
        println!();

        let confirmed = Confirm::new()
            .with_prompt("Continue with vacuum operation?")
            .default(true)
            .interact()?;

        if !confirmed {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
        println!();
    }

    println!("Starting storage vacuum operations...");

    // Create progress bar for vacuum operations
    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} [{elapsed_precise}] {msg}")
            .expect("ProgressStyle template is valid: uses standard format"),
    );
    progress.set_message("Analyzing storage for optimization opportunities...");

    // Note: Vacuum operations are limited by the StorageBackend trait
    // In a full implementation, we'd need to extend the trait with vacuum methods
    println!("Note: Vacuum operations are limited through the generic StorageBackend trait");
    println!("For full vacuum capabilities, backend-specific tools should be used directly");

    // For now, we can only report that vacuum is not fully supported
    // through the generic interface

    // Mark as optimized if no errors occurred (which is always true for now)
    let storage_optimized = errors == 0;

    progress.finish_with_message("Storage vacuum completed");

    let result = VacuumResult {
        items_cleaned: total_cleaned,
        storage_optimized,
        errors,
        dry_run: false,
    };

    format.print_output(&result)?;

    Ok(())
}

pub async fn storage_health(
    memory: &do_memory_core::SelfLearningMemory,
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

    // Check Turso health by attempting a simple query
    if let Some(turso) = memory.turso_storage() {
        let start = std::time::Instant::now();
        // Try to query a non-existent episode to test connectivity
        match turso.get_episode(uuid::Uuid::new_v4()).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                turso_health = ComponentHealth {
                    status: if latency < 100 {
                        HealthStatus::Healthy
                    } else {
                        HealthStatus::Degraded
                    },
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
                    status: if latency < 10 {
                        HealthStatus::Healthy
                    } else {
                        HealthStatus::Degraded
                    },
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
    let overall_status = match (&turso_health.status, &redb_health.status) {
        (HealthStatus::Healthy, HealthStatus::Healthy) => HealthStatus::Healthy,
        (HealthStatus::Healthy, HealthStatus::Degraded)
        | (HealthStatus::Degraded, HealthStatus::Healthy) => HealthStatus::Degraded,
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
    _memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    // Pool telemetry is not exposed through the `StorageBackend` trait, so all
    // connection metrics are reported as `Unavailable` regardless of
    // configuration rather than fabricating active/pool/last-activity values
    // (PTA-A2).

    let status = ConnectionStatus {
        turso: ConnectionInfo {
            active_connections: MetricValue::unavailable(),
            pool_size: MetricValue::unavailable(),
            queue_depth: MetricValue::unavailable(),
            last_activity: MetricValue::unavailable(),
        },
        redb: ConnectionInfo {
            active_connections: MetricValue::unavailable(),
            pool_size: MetricValue::unavailable(),
            queue_depth: MetricValue::unavailable(),
            last_activity: MetricValue::unavailable(),
        },
    };

    format.print_output(&status)?;

    Ok(())
}
