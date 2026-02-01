//! Evaluation and calibration commands

use clap::Subcommand;
use serde::Serialize;

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

    /// Set custom threshold for a domain (manual override)
    SetThreshold {
        /// Domain to configure
        #[arg(long)]
        domain: String,

        /// Duration threshold in seconds
        #[arg(long)]
        duration: Option<f32>,

        /// Step count threshold
        #[arg(long)]
        steps: Option<usize>,
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
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use memory_core::DomainStatisticsCache;

    // Get all episodes to calculate statistics (returns Vec<Arc<Episode>>)
    let context = memory_core::types::TaskContext::default();
    let arc_episodes = memory
        .retrieve_relevant_context("".to_string(), context, 10000)
        .await;

    // Convert Vec<Arc<Episode>> to Vec<Episode> for grouping
    let all_episodes: Vec<memory_core::Episode> = arc_episodes
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
    calibrations.sort_by(|a, b| b.episode_count.cmp(&a.episode_count));

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
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use memory_core::DomainStatisticsCache;

    // Get all episodes for this domain (returns Vec<Arc<Episode>>)
    let context = memory_core::types::TaskContext {
        domain: domain.clone(),
        ..Default::default()
    };
    let arc_episodes = memory
        .retrieve_relevant_context("".to_string(), context, 10000)
        .await;

    // Convert Vec<Arc<Episode>> to Vec<Episode> for filtering
    let episodes: Vec<memory_core::Episode> = arc_episodes
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

pub async fn set_threshold(
    domain: String,
    duration: Option<f32>,
    steps: Option<usize>,
    _memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    use colored::*;

    println!("Setting custom thresholds for domain: {}", domain.bold());
    println!();

    if let Some(dur) = duration {
        println!("  Duration threshold: {}s", dur.to_string().green());
    }

    if let Some(stp) = steps {
        println!("  Step count threshold: {}", stp.to_string().green());
    }

    println!();
    println!(
        "{}",
        "Note: Custom thresholds are not yet implemented in Phase 2.".yellow()
    );
    println!(
        "{}",
        "This command will allow manual overrides in a future update.".yellow()
    );
    println!();
    println!("Current behavior:");
    println!("  - Domains with 5+ episodes: Use adaptive calibration (median as baseline)");
    println!("  - Domains with <5 episodes: Use fixed thresholds (60s, 10 steps)");

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
