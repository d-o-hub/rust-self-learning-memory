//! Evaluation and calibration commands

use clap::Subcommand;
use serde::Serialize;
use std::path::PathBuf;

use do_memory_core::retrieval::{
    BenchmarkReport, FixtureCorpus, RegressionChecker, RegressionThresholds, RetrievalEvaluator,
    RetrievalStrategy, format_markdown_report,
};

use crate::config::Config;
use crate::output::{Output, OutputFormat};

#[derive(Subcommand)]
pub enum EvalCommands {
    /// View domain calibration statistics
    Calibration {
        /// Filter by specific domain
        #[arg(short, long)]
        domain: Option<String>,

        /// Show all domains (including those with few episodes)
        #[arg(short, long)]
        all: bool,

        /// Minimum episodes required to show domain
        #[arg(long, default_value = "5")]
        min_episodes: usize,
    },

    /// View detailed domain statistics
    Stats {
        /// Domain to view
        #[arg(value_name = "DOMAIN")]
        domain: String,
    },

    /// Run reproducible retrieval quality and cost benchmark
    Benchmark {
        /// Path to JSON or JSONL fixture corpus file
        #[arg(long, value_name = "FILE")]
        fixture: Option<PathBuf>,

        /// Strategy to evaluate (adaptive, always_embed, local_only, or all)
        #[arg(long, default_value = "all")]
        strategy: String,

        /// Path to baseline JSON artifact for regression check
        #[arg(long, value_name = "FILE")]
        baseline: Option<PathBuf>,

        /// Fail process exit code if regression thresholds are exceeded
        #[arg(long)]
        fail_on_regression: bool,

        /// Maximum allowed drop in Recall@5 (e.g. 0.05 for 5%)
        #[arg(long, default_value_t = 0.05)]
        max_recall_drop: f64,

        /// Maximum allowed drop in MRR
        #[arg(long, default_value_t = 0.05)]
        max_mrr_drop: f64,

        /// Maximum allowed drop in NDCG@5
        #[arg(long, default_value_t = 0.05)]
        max_ndcg_drop: f64,

        /// Maximum allowed latency increase ratio (e.g. 0.50 for 50%)
        #[arg(long, default_value_t = 0.50)]
        max_latency_increase: f64,

        /// Maximum allowed cost increase ratio (e.g. 0.20 for 20%)
        #[arg(long, default_value_t = 0.20)]
        max_cost_increase: f64,

        /// Path to save machine-readable JSON output
        #[arg(long, value_name = "FILE")]
        output_json: Option<PathBuf>,

        /// Path to save concise Markdown report
        #[arg(long, value_name = "FILE")]
        output_markdown: Option<PathBuf>,

        /// Permit remote embedding provider calls (requires API credentials)
        #[arg(long)]
        remote: bool,
    },
}

#[derive(Debug, Serialize)]
pub struct CalibrationSummary {
    pub domains: Vec<DomainCalibration>,
    pub total_domains: usize,
    pub reliable_domains: usize,
}

#[derive(Debug, Serialize)]
pub struct DomainCalibration {
    pub domain: String,
    pub episode_count: usize,
    pub efficient_duration_secs: f32,
    pub efficient_step_count: usize,
    pub avg_reward: f32,
    pub success_rate: f32,
    pub is_reliable: bool,
}

#[derive(Debug, Serialize)]
pub struct DomainStatsDetail {
    pub domain: String,
    pub episode_count: usize,
    pub duration: DurationStats,
    pub steps: StepStats,
    pub rewards: RewardStats,
    pub success_rate: f32,
    pub last_updated: String,
    pub is_reliable: bool,
    pub is_stale: bool,
}

#[derive(Debug, Serialize)]
pub struct DurationStats {
    pub avg_secs: f32,
    pub median_secs: f32,
    pub p90_secs: f32,
}

#[derive(Debug, Serialize)]
pub struct StepStats {
    pub avg: f32,
    pub median: usize,
    pub p90: usize,
}

#[derive(Debug, Serialize)]
pub struct RewardStats {
    pub avg: f32,
    pub median: f32,
    pub std_dev: f32,
}

impl Output for CalibrationSummary {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "Domain Calibration Summary")?;
        writeln!(writer, "==========================")?;
        writeln!(
            writer,
            "Total domains: {} (reliable: {})",
            self.total_domains, self.reliable_domains
        )?;
        writeln!(writer)?;

        if self.domains.is_empty() {
            writeln!(writer, "No domains found.")?;
            return Ok(());
        }

        writeln!(
            writer,
            "{:<20} {:>8} {:>12} {:>10} {:>8} {:>8}",
            "Domain", "Episodes", "Duration(s)", "Steps", "Reward", "Success"
        )?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for cal in &self.domains {
            let reliable_marker = if cal.is_reliable { "✓" } else { " " };
            let color = if cal.is_reliable {
                Color::Green
            } else {
                Color::Yellow
            };

            writeln!(
                writer,
                "{} {:<18} {:>8} {:>12.1} {:>10} {:>8.2} {:>7.1}%",
                reliable_marker.color(color),
                cal.domain,
                cal.episode_count,
                cal.efficient_duration_secs,
                cal.efficient_step_count,
                cal.avg_reward,
                cal.success_rate * 100.0
            )?;
        }

        writeln!(writer)?;
        writeln!(writer, "{}", "✓ = Reliable (5+ episodes)".dimmed())?;
        writeln!(
            writer,
            "{}",
            "Duration/Steps show median (p50) values used as 'efficient' baseline".dimmed()
        )?;

        Ok(())
    }
}

impl Output for DomainStatsDetail {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "Domain Statistics: {}", self.domain.bold())?;
        writeln!(writer, "{}", "=".repeat(50))?;
        writeln!(writer)?;

        // Overview
        writeln!(writer, "{}", "Overview:".bold())?;
        writeln!(writer, "  Episodes: {}", self.episode_count)?;
        writeln!(writer, "  Success Rate: {:.1}%", self.success_rate * 100.0)?;
        writeln!(writer, "  Last Updated: {}", self.last_updated)?;

        let status = if !self.is_reliable {
            "Unreliable (needs 5+ episodes)".yellow()
        } else if self.is_stale {
            "Stale (>7 days old)".yellow()
        } else {
            "Reliable".green()
        };
        writeln!(writer, "  Status: {}", status)?;
        writeln!(writer)?;

        // Duration stats
        writeln!(writer, "{}", "Duration Statistics:".bold())?;
        writeln!(writer, "  Average: {:.1}s", self.duration.avg_secs)?;
        writeln!(
            writer,
            "  Median (p50): {:.1}s {}",
            self.duration.median_secs,
            "(baseline)".dimmed()
        )?;
        writeln!(writer, "  90th percentile: {:.1}s", self.duration.p90_secs)?;
        writeln!(writer)?;

        // Step stats
        writeln!(writer, "{}", "Step Count Statistics:".bold())?;
        writeln!(writer, "  Average: {:.1}", self.steps.avg)?;
        writeln!(
            writer,
            "  Median (p50): {} {}",
            self.steps.median,
            "(baseline)".dimmed()
        )?;
        writeln!(writer, "  90th percentile: {}", self.steps.p90)?;
        writeln!(writer)?;

        // Reward stats
        writeln!(writer, "{}", "Reward Statistics:".bold())?;
        writeln!(writer, "  Average: {:.2}", self.rewards.avg)?;
        writeln!(writer, "  Median: {:.2}", self.rewards.median)?;
        writeln!(writer, "  Std Dev: {:.2}", self.rewards.std_dev)?;
        writeln!(writer)?;

        if self.is_reliable {
            writeln!(
                writer,
                "{}",
                "This domain has sufficient data for adaptive reward calibration.".green()
            )?;
            writeln!(
                writer,
                "Episodes faster than {:.1}s or fewer than {} steps will get efficiency bonuses.",
                self.duration.median_secs, self.steps.median
            )?;
        } else {
            writeln!(
                writer,
                "{}",
                "This domain needs more episodes (5+) for reliable calibration.".yellow()
            )?;
            writeln!(
                writer,
                "{}",
                "Fixed thresholds (60s, 10 steps) will be used until then.".yellow()
            )?;
        }

        Ok(())
    }
}

pub async fn calibration(
    domain_filter: Option<String>,
    show_all: bool,
    min_episodes: usize,
    memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use do_memory_core::DomainStatisticsCache;

    // Get all episodes to calculate statistics (returns Vec<Arc<Episode>>)
    let context = do_memory_core::types::TaskContext::default();
    let arc_episodes = memory
        .retrieve_relevant_context("".to_string(), context, 10000)
        .await;

    // Convert Vec<Arc<Episode>> to Vec<Episode> for grouping
    let all_episodes: Vec<do_memory_core::Episode> = arc_episodes
        .iter()
        .map(|arc_ep| arc_ep.as_ref().clone())
        .collect();

    // Group episodes by domain
    let mut domain_episodes: std::collections::HashMap<String, Vec<_>> =
        std::collections::HashMap::new();
    for episode in &all_episodes {
        domain_episodes
            .entry(episode.context.domain.clone())
            .or_default()
            .push(episode.clone());
    }

    // Calculate statistics for each domain
    let mut stats_cache = DomainStatisticsCache::new();
    for (domain, episodes) in domain_episodes {
        stats_cache.calculate_from_episodes(domain, &episodes);
    }

    // Filter and format results
    let mut calibrations: Vec<DomainCalibration> = Vec::new();

    for (domain, stats) in &stats_cache.stats {
        // Apply filters
        if let Some(ref filter) = domain_filter {
            if domain != filter {
                continue;
            }
        }

        if !show_all && stats.episode_count < min_episodes {
            continue;
        }

        calibrations.push(DomainCalibration {
            domain: domain.clone(),
            episode_count: stats.episode_count,
            efficient_duration_secs: stats.p50_duration_secs,
            efficient_step_count: stats.p50_step_count,
            avg_reward: stats.avg_reward,
            success_rate: stats.success_rate(),
            is_reliable: stats.is_reliable(),
        });
    }

    // Sort by episode count (most data first)
    calibrations.sort_by_key(|b| std::cmp::Reverse(b.episode_count));

    let reliable_count = calibrations.iter().filter(|c| c.is_reliable).count();

    let summary = CalibrationSummary {
        total_domains: calibrations.len(),
        reliable_domains: reliable_count,
        domains: calibrations,
    };

    summary.write(&mut std::io::stdout(), &format)?;
    Ok(())
}

pub async fn domain_stats(
    domain: String,
    memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use do_memory_core::DomainStatisticsCache;

    // Get all episodes for this domain (returns Vec<Arc<Episode>>)
    let context = do_memory_core::types::TaskContext {
        domain: domain.clone(),
        ..Default::default()
    };
    let arc_episodes = memory
        .retrieve_relevant_context("".to_string(), context, 10000)
        .await;

    // Convert Vec<Arc<Episode>> to Vec<Episode> for filtering
    let episodes: Vec<do_memory_core::Episode> = arc_episodes
        .iter()
        .map(|arc_ep| arc_ep.as_ref().clone())
        .collect();

    let domain_episodes: Vec<_> = episodes
        .iter()
        .filter(|e| e.context.domain == domain)
        .cloned()
        .collect();

    if domain_episodes.is_empty() {
        anyhow::bail!(
            "No episodes found for domain '{}'. Available domains can be seen with: memory-cli eval calibration",
            domain
        );
    }

    // Calculate statistics
    let mut stats_cache = DomainStatisticsCache::new();
    stats_cache.calculate_from_episodes(domain.clone(), &domain_episodes);

    let stats = stats_cache
        .get(&domain)
        .ok_or_else(|| anyhow::anyhow!("Failed to calculate statistics for domain"))?;

    let detail = DomainStatsDetail {
        domain: stats.domain.clone(),
        episode_count: stats.episode_count,
        duration: DurationStats {
            avg_secs: stats.avg_duration_secs,
            median_secs: stats.p50_duration_secs,
            p90_secs: stats.p90_duration_secs,
        },
        steps: StepStats {
            avg: stats.avg_step_count,
            median: stats.p50_step_count,
            p90: stats.p90_step_count,
        },
        rewards: RewardStats {
            avg: stats.avg_reward,
            median: stats.p50_reward,
            std_dev: stats.reward_std_dev,
        },
        success_rate: stats.success_rate(),
        last_updated: format_time(stats.last_updated),
        is_reliable: stats.is_reliable(),
        is_stale: stats.is_stale(),
    };

    detail.write(&mut std::io::stdout(), &format)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn benchmark(
    fixture_path: Option<PathBuf>,
    strategy_str: String,
    baseline_path: Option<PathBuf>,
    fail_on_regression: bool,
    max_recall_drop: f64,
    max_mrr_drop: f64,
    max_ndcg_drop: f64,
    max_latency_increase: f64,
    max_cost_increase: f64,
    output_json: Option<PathBuf>,
    output_markdown: Option<PathBuf>,
    _remote: bool,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    // 1. Resolve fixture path
    let default_fixture = PathBuf::from("benches/fixtures/retrieval_benchmark_corpus.json");
    let target_fixture = fixture_path.unwrap_or(default_fixture);

    if !target_fixture.exists() {
        anyhow::bail!(
            "Fixture corpus file not found at path: {}",
            target_fixture.display()
        );
    }

    let fixture_content = tokio::fs::read_to_string(&target_fixture).await?;
    let corpus = if target_fixture.extension().and_then(|s| s.to_str()) == Some("jsonl") {
        FixtureCorpus::from_jsonl_str(&fixture_content)?
    } else {
        FixtureCorpus::from_json_str(&fixture_content)?
    };

    let evaluator = RetrievalEvaluator::new(corpus);

    // 2. Execute benchmark
    let report: BenchmarkReport = if strategy_str.eq_ignore_ascii_case("all") {
        evaluator.evaluate_all()?
    } else {
        let strategy: RetrievalStrategy = strategy_str.parse().map_err(anyhow::Error::msg)?;
        let metrics = evaluator.evaluate_strategy(strategy)?;
        let mut strategies = std::collections::HashMap::new();
        strategies.insert(strategy.to_string(), metrics);
        BenchmarkReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            corpus_version: evaluator.corpus_version().to_string(),
            corpus_size: evaluator.corpus_size(),
            query_count: evaluator.query_count(),
            strategies,
        }
    };

    // 3. Baseline comparison
    let default_baseline = PathBuf::from("benches/fixtures/retrieval_baseline.json");
    let active_baseline_path = baseline_path.or_else(|| {
        if default_baseline.exists() {
            Some(default_baseline)
        } else {
            None
        }
    });

    let check_result = if let Some(ref base_path) = active_baseline_path {
        if base_path.exists() {
            let base_content = tokio::fs::read_to_string(base_path).await?;
            let baseline_report: BenchmarkReport = serde_json::from_str(&base_content)?;
            let thresholds = RegressionThresholds {
                max_recall_drop,
                max_mrr_drop,
                max_ndcg_drop,
                max_latency_increase_ratio: max_latency_increase,
                max_cost_increase_ratio: max_cost_increase,
            };
            let checker = RegressionChecker::new(thresholds);
            Some(checker.check(&report, &baseline_report))
        } else {
            None
        }
    } else {
        None
    };

    // 4. Output artifacts
    let markdown_str = format_markdown_report(&report, check_result.as_ref());

    if let Some(ref md_out) = output_markdown {
        tokio::fs::write(md_out, &markdown_str).await?;
    }

    if let Some(ref json_out) = output_json {
        let json_str = serde_json::to_string_pretty(&report)?;
        tokio::fs::write(json_out, json_str).await?;
    }

    // Print summary report to stdout
    println!("{markdown_str}");

    // 5. Fail on regression if configured
    if fail_on_regression {
        if let Some(ref res) = check_result {
            if !res.passed {
                anyhow::bail!(
                    "Retrieval benchmark failed regression checks: {}",
                    res.summary
                );
            }
        }
    }

    Ok(())
}

fn format_time(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now - dt;

    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{} minutes ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{} hours ago", diff.num_hours())
    } else if diff.num_days() < 7 {
        format!("{} days ago", diff.num_days())
    } else {
        format!("{} weeks ago", diff.num_weeks())
    }
}
