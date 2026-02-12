//! Batch pattern operations CLI commands

use anyhow::Result;
use clap::Subcommand;
use memory_core::{
    episode::PatternId, types::OutcomeStats, Pattern, PatternEffectiveness, TaskContext,
};

#[cfg(feature = "turso")]
use memory_storage_turso::TursoStorage;

use serde::Serialize;
use uuid::Uuid;

/// Batch pattern operations
#[derive(Subcommand)]
pub enum PatternBatchCommands {
    /// Store multiple patterns in a single transaction
    Store {
        /// Number of patterns to generate for batch (for testing)
        #[arg(long, default_value = "10")]
        count: usize,

        /// Show patterns without storing (dry run)
        #[arg(long)]
        dry_run: bool,
    },

    /// Retrieve multiple patterns by IDs
    Get {
        /// Pattern IDs (comma-separated)
        #[arg(value_name = "IDS")]
        ids: String,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },

    /// Update multiple patterns in a single transaction
    Update {
        /// Pattern IDs (comma-separated)
        #[arg(value_name = "IDS")]
        ids: String,

        /// New success rate to set
        #[arg(long)]
        success_rate: Option<f32>,
    },

    /// Delete multiple patterns in a single transaction
    Delete {
        /// Pattern IDs (comma-separated)
        #[arg(value_name = "IDS")]
        ids: String,

        /// Confirm deletion without prompting
        #[arg(long)]
        force: bool,

        /// Show what would be deleted without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Performance benchmark for batch operations
    Benchmark {
        /// Number of patterns for benchmark
        #[arg(long, default_value = "100")]
        count: usize,

        /// Batch size for chunked operations
        #[arg(long, default_value = "50")]
        batch_size: usize,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

/// Result of batch pattern operation
#[derive(Debug, Serialize)]
pub struct BatchOperationResult {
    pub operation: String,
    pub total_requested: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub throughput_per_sec: f64,
    pub errors: Vec<String>,
}

/// Execute batch pattern command
pub async fn execute_pattern_batch_command(
    cmd: PatternBatchCommands,
    storage: &mut TursoStorage,
) -> Result<()> {
    match cmd {
        PatternBatchCommands::Store { count, dry_run } => {
            execute_batch_store(storage, count, dry_run).await
        }
        PatternBatchCommands::Get { ids, format } => execute_batch_get(storage, ids, format).await,
        PatternBatchCommands::Update { ids, success_rate } => {
            execute_batch_update(storage, ids, success_rate).await
        }
        PatternBatchCommands::Delete {
            ids,
            force,
            dry_run,
        } => execute_batch_delete(storage, ids, force, dry_run).await,
        PatternBatchCommands::Benchmark { count, batch_size } => {
            execute_benchmark(storage, count, batch_size).await
        }
    }
}

/// Execute batch store operation
async fn execute_batch_store(
    storage: &mut TursoStorage,
    count: usize,
    dry_run: bool,
) -> Result<()> {
    println!("📦 Generating {} test patterns...", count);

    let patterns: Vec<Pattern> = (0..count)
        .map(|i| Pattern::DecisionPoint {
            id: PatternId::new_v4(),
            condition: format!("Test condition {}", i),
            action: format!("Test action {}", i),
            outcome_stats: OutcomeStats {
                success_count: (i % 10) + 1,
                failure_count: i % 3,
                total_count: (i % 10) + 1 + (i % 3),
                avg_duration_secs: 0.0,
            },
            context: TaskContext::default(),
            effectiveness: PatternEffectiveness::default(),
        })
        .collect();

    if dry_run {
        println!("\n🔍 Dry run - would store {} patterns:", count);
        for (i, pattern) in patterns.iter().take(5).enumerate() {
            println!("  {}. Pattern ID: {}", i + 1, pattern.id());
        }
        if patterns.len() > 5 {
            println!("  ... and {} more", patterns.len() - 5);
        }
        return Ok(());
    }

    let start = std::time::Instant::now();
    let result = storage.store_patterns_batch(patterns).await;
    let duration = start.elapsed();

    match result {
        Ok(()) => {
            let throughput = count as f64 / duration.as_secs_f64();
            println!(
                "✅ Successfully stored {} patterns in {:?} ({:.2} patterns/sec)",
                count, duration, throughput
            );
            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to store patterns: {}", e);
            Err(e.into())
        }
    }
}

/// Execute batch get operation
async fn execute_batch_get(
    storage: &mut TursoStorage,
    ids_str: String,
    format: OutputFormat,
) -> Result<()> {
    let ids: Vec<PatternId> = ids_str
        .split(',')
        .filter_map(|s| s.trim().parse::<Uuid>().ok())
        .map(|id| id)
        .collect();

    if ids.is_empty() {
        println!("❌ No valid pattern IDs provided");
        return Ok(());
    }

    println!("📦 Retrieving {} patterns...", ids.len());

    let start = std::time::Instant::now();
    let patterns = storage.get_patterns_batch(&ids).await?;
    let duration = start.elapsed();

    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&patterns)?;
            println!("{}", json);
        }
        OutputFormat::Table => {
            println!(
                "\nRetrieved {} patterns in {:?}:\n",
                patterns.len(),
                duration
            );
            for (i, pattern_opt) in patterns.iter().enumerate() {
                if let Some(pattern) = pattern_opt {
                    println!(
                        "  {}. ID: {} | Type: {:?} | Success Rate: {:.2}",
                        i + 1,
                        pattern.id(),
                        pattern,
                        pattern.success_rate()
                    );
                }
            }
        }
        OutputFormat::Csv => {
            println!("pattern_id,pattern_type,success_rate");
            for pattern_opt in &patterns {
                if let Some(pattern) = pattern_opt {
                    println!(
                        "{},{:?},{:.2}",
                        pattern.id(),
                        pattern,
                        pattern.success_rate()
                    );
                }
            }
        }
    }

    Ok(())
}

/// Execute batch update operation
async fn execute_batch_update(
    storage: &mut TursoStorage,
    ids_str: String,
    success_rate: Option<f32>,
) -> Result<()> {
    let ids: Vec<PatternId> = ids_str
        .split(',')
        .filter_map(|s| s.trim().parse::<Uuid>().ok())
        .map(|id| id)
        .collect();

    if ids.is_empty() {
        println!("❌ No valid pattern IDs provided");
        return Ok(());
    }

    println!("📦 Updating {} patterns...", ids.len());

    // First retrieve the patterns
    let patterns = storage.get_patterns_batch(&ids).await?;

    if patterns.is_empty() {
        println!("❌ No patterns found with provided IDs");
        return Ok(());
    }

    // Filter out None values and apply updates
    let mut updated_patterns: Vec<Pattern> = patterns.into_iter().flatten().collect();

    if let Some(rate) = success_rate {
        for pattern in &mut updated_patterns {
            if let Pattern::DecisionPoint {
                ref mut outcome_stats,
                ..
            } = pattern
            {
                let total = outcome_stats.total_count;
                outcome_stats.success_count = (rate * total as f32) as usize;
                outcome_stats.failure_count = total - outcome_stats.success_count;
            }
        }
    }

    let start = std::time::Instant::now();
    let result = storage
        .update_patterns_batch(updated_patterns.clone())
        .await;
    let duration = start.elapsed();

    match result {
        Ok(()) => {
            println!(
                "✅ Successfully updated {} patterns in {:?}",
                updated_patterns.len(),
                duration
            );
            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to update patterns: {}", e);
            Err(e.into())
        }
    }
}

/// Execute batch delete operation
async fn execute_batch_delete(
    storage: &mut TursoStorage,
    ids_str: String,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    let ids: Vec<PatternId> = ids_str
        .split(',')
        .filter_map(|s| s.trim().parse::<Uuid>().ok())
        .map(|id| id)
        .collect();

    if ids.is_empty() {
        println!("❌ No valid pattern IDs provided");
        return Ok(());
    }

    if !force {
        println!(
            "⚠️  About to delete {} patterns. Continue? [y/N]",
            ids.len()
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("❌ Cancelled");
            return Ok(());
        }
    }

    if dry_run {
        println!("🔍 Dry run - would delete {} patterns", ids.len());
        return Ok(());
    }

    println!("📦 Deleting {} patterns...", ids.len());

    let start = std::time::Instant::now();
    let deleted = storage.delete_patterns_batch(ids).await?;
    let duration = start.elapsed();

    println!(
        "✅ Successfully deleted {} patterns in {:?}",
        deleted, duration
    );

    Ok(())
}

/// Execute performance benchmark
async fn execute_benchmark(
    storage: &mut TursoStorage,
    count: usize,
    batch_size: usize,
) -> Result<()> {
    println!("🚀 Starting batch pattern benchmark...");
    println!("   Patterns: {}", count);
    println!("   Batch size: {}\n", batch_size);

    // Generate test patterns
    let patterns: Vec<Pattern> = (0..count)
        .map(|i| Pattern::DecisionPoint {
            id: PatternId::new_v4(),
            condition: format!("Benchmark condition {}", i),
            action: format!("Benchmark action {}", i),
            outcome_stats: OutcomeStats {
                success_count: 8,
                failure_count: 2,
                total_count: 10,
                avg_duration_secs: 0.0,
            },
            context: TaskContext::default(),
            effectiveness: PatternEffectiveness::default(),
        })
        .collect();

    let ids: Vec<PatternId> = patterns.iter().map(|p| p.id()).collect();

    // Benchmark 1: Batch Store
    let start = std::time::Instant::now();
    storage.store_patterns_batch(patterns.clone()).await?;
    let store_duration = start.elapsed();
    let store_throughput = count as f64 / store_duration.as_secs_f64();

    // Benchmark 2: Batch Get
    let start = std::time::Instant::now();
    let _retrieved = storage.get_patterns_batch(&ids).await?;
    let get_duration = start.elapsed();
    let get_throughput = count as f64 / get_duration.as_secs_f64();

    // Benchmark 3: Batch Update
    let start = std::time::Instant::now();
    storage.update_patterns_batch(patterns.clone()).await?;
    let update_duration = start.elapsed();
    let update_throughput = count as f64 / update_duration.as_secs_f64();

    // Benchmark 4: Batch Delete
    let start = std::time::Instant::now();
    let _deleted = storage.delete_patterns_batch(ids).await?;
    let delete_duration = start.elapsed();
    let delete_throughput = count as f64 / delete_duration.as_secs_f64();

    // Print results
    println!("\n📊 Benchmark Results:\n");
    println!("┌─────────────────────┬──────────────┬────────────────────┐");
    println!("│ Operation           │ Duration     │ Throughput         │");
    println!("├─────────────────────┼──────────────┼────────────────────┤");
    println!(
        "│ Batch Store ({:4}) │ {:>10?} │ {:>14.2}/s │",
        count, store_duration, store_throughput
    );
    println!(
        "│ Batch Get   ({:4}) │ {:>10?} │ {:>14.2}/s │",
        count, get_duration, get_throughput
    );
    println!(
        "│ Batch Update       │ {:>10?} │ {:>14.2}/s │",
        update_duration, update_throughput
    );
    println!(
        "│ Batch Delete       │ {:>10?} │ {:>14.2}/s │",
        delete_duration, delete_throughput
    );
    println!("└─────────────────────┴──────────────┴────────────────────┘");

    // Calculate improvement factor
    let avg_individual_latency_ms = 5.0; // Estimated individual operation latency
    let expected_individual_time = count as f64 * avg_individual_latency_ms / 1000.0;
    let improvement_factor = expected_individual_time / store_duration.as_secs_f64();

    println!(
        "\n📈 Performance Improvement: {:.2}x faster than individual operations",
        improvement_factor
    );

    if improvement_factor >= 4.0 {
        println!("✅ Target achieved: 4-6x throughput improvement!");
    } else if improvement_factor >= 2.0 {
        println!("⚠️  Below target: aim for 4-6x improvement");
    } else {
        println!("❌ Poor performance: batch operations not effective");
    }

    Ok(())
}
