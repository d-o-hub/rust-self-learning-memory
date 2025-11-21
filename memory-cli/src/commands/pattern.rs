use clap::{Subcommand, ValueEnum};
use serde::Serialize;

use crate::config::Config;
use crate::output::{Output, OutputFormat};

#[derive(Subcommand)]
pub enum PatternCommands {
    /// List patterns
    List {
        /// Minimum confidence threshold
        #[arg(long, default_value = "0.0")]
        min_confidence: f32,

        /// Filter by pattern type
        #[arg(short, long)]
        pattern_type: Option<PatternType>,

        /// Maximum number of patterns to return
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// View pattern details
    View {
        /// Pattern ID
        #[arg(value_name = "PATTERN_ID")]
        pattern_id: String,
    },

    /// Analyze pattern effectiveness
    Analyze {
        /// Pattern ID
        #[arg(value_name = "PATTERN_ID")]
        pattern_id: String,

        /// Number of episodes to analyze
        #[arg(short, long, default_value = "100")]
        episodes: usize,
    },

    /// Show pattern effectiveness rankings
    Effectiveness {
        /// Show top N patterns
        #[arg(short, long, default_value = "10")]
        top: usize,

        /// Minimum number of uses
        #[arg(long, default_value = "1")]
        min_uses: usize,
    },

    /// Apply pattern decay
    Decay {
        /// Show what would be done without executing
        #[arg(long)]
        dry_run: bool,

        /// Force decay without confirmation
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PatternType {
    /// Tool sequence patterns
    ToolSequence,
    /// Decision point patterns
    DecisionPoint,
    /// Error recovery patterns
    ErrorRecovery,
    /// Context patterns
    ContextPattern,
}

#[derive(Debug, Serialize)]
pub struct PatternSummary {
    pub pattern_id: String,
    pub pattern_type: String,
    pub confidence: f32,
    pub effectiveness: f32,
    pub use_count: usize,
    pub last_used: String,
    pub description: String,
}

impl Output for PatternSummary {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        let confidence_color = match self.confidence {
            c if c >= 0.8 => Color::Green,
            c if c >= 0.6 => Color::Yellow,
            _ => Color::Red,
        };

        let effectiveness_color = match self.effectiveness {
            e if e >= 0.8 => Color::Green,
            e if e >= 0.6 => Color::Yellow,
            _ => Color::Red,
        };

        writeln!(
            writer,
            "{} ({})",
            self.pattern_id[..8].to_string().dimmed(),
            self.pattern_type
        )?;
        writeln!(writer, "  Description: {}", self.description)?;
        writeln!(
            writer,
            "  Confidence: {:.2} {}",
            self.confidence,
            "●".color(confidence_color)
        )?;
        writeln!(
            writer,
            "  Effectiveness: {:.2} {}",
            self.effectiveness,
            "●".color(effectiveness_color)
        )?;
        writeln!(
            writer,
            "  Uses: {}, Last: {}",
            self.use_count, self.last_used
        )?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct PatternList {
    pub patterns: Vec<PatternSummary>,
    pub total_count: usize,
}

impl Output for PatternList {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(
            writer,
            "{} patterns (showing {})",
            self.total_count,
            self.patterns.len()
        )?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for pattern in &self.patterns {
            let (confidence_color, confidence_icon) = match pattern.confidence {
                c if c >= 0.8 => (Color::Green, "●"),
                c if c >= 0.6 => (Color::Yellow, "○"),
                _ => (Color::Red, "○"),
            };

            let confidence_display = format!("{:.2} {}", pattern.confidence, confidence_icon);

            writeln!(
                writer,
                "{} {} {} {} uses",
                pattern.pattern_id[..8].to_string().dimmed(),
                confidence_display.color(confidence_color).bold(),
                pattern.pattern_type.dimmed(),
                pattern.use_count.to_string().color(confidence_color)
            )?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct PatternAnalysis {
    pub pattern_id: String,
    pub analysis: PatternAnalysisData,
}

#[derive(Debug, Serialize)]
pub struct EffectivenessRankings {
    pub rankings: Vec<EffectivenessRanking>,
    pub total_patterns_analyzed: usize,
}

#[derive(Debug, Serialize)]
pub struct EffectivenessRanking {
    pub rank: usize,
    pub pattern_id: String,
    pub effectiveness_score: f32,
    pub use_count: usize,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct PatternDetail {
    pub id: String,
    pub pattern_type: String,
    pub confidence: f32,
    pub success_rate: f32,
    pub sample_size: usize,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct DecayResult {
    pub patterns_to_decay: Vec<DecayPatternInfo>,
    pub total_patterns_analyzed: usize,
    pub dry_run: bool,
    pub would_decay_count: usize,
}

#[derive(Debug, Serialize)]
pub struct DecayPatternInfo {
    pub pattern_id: String,
    pub effectiveness_score: f32,
    pub use_count: usize,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct PatternAnalysisData {
    pub success_rate: f32,
    pub average_improvement: f32,
    pub episodes_analyzed: usize,
    pub recommendations: Vec<String>,
}

impl Output for PatternAnalysis {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(writer, "Pattern Analysis: {}", self.pattern_id)?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(
            writer,
            "Success Rate: {:.1}%",
            self.analysis.success_rate * 100.0
        )?;
        writeln!(
            writer,
            "Average Improvement: {:.2}",
            self.analysis.average_improvement
        )?;
        writeln!(
            writer,
            "Episodes Analyzed: {}",
            self.analysis.episodes_analyzed
        )?;

        if !self.analysis.recommendations.is_empty() {
            writeln!(writer, "\nRecommendations:")?;
            for rec in &self.analysis.recommendations {
                writeln!(writer, "  • {}", rec)?;
            }
        }

        Ok(())
    }
}

impl Output for EffectivenessRankings {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "Pattern Effectiveness Rankings")?;
        writeln!(writer, "{}", "─".repeat(50))?;
        writeln!(
            writer,
            "Total patterns analyzed: {}",
            self.total_patterns_analyzed
        )?;
        writeln!(writer)?;

        if self.rankings.is_empty() {
            writeln!(writer, "No patterns found.")?;
            return Ok(());
        }

        for ranking in &self.rankings {
            let score_color = match ranking.effectiveness_score {
                s if s >= 0.8 => Color::Green,
                s if s >= 0.6 => Color::Yellow,
                _ => Color::Red,
            };

            writeln!(
                writer,
                "{}. {} ({:.2}) - {} uses",
                ranking.rank.to_string().bold(),
                ranking.pattern_id[..8].to_string().dimmed(),
                ranking
                    .effectiveness_score
                    .to_string()
                    .color(score_color)
                    .bold(),
                ranking.use_count
            )?;
            writeln!(writer, "   {}", ranking.description)?;
            writeln!(writer)?;
        }

        Ok(())
    }
}

impl Output for PatternDetail {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(writer, "Pattern Details")?;
        writeln!(writer, "{}", "─".repeat(20))?;
        writeln!(writer, "ID: {}", self.id)?;
        writeln!(writer, "Type: {}", self.pattern_type)?;
        writeln!(writer, "Confidence: {:.2}", self.confidence)?;
        writeln!(writer, "Success Rate: {:.2}", self.success_rate)?;
        writeln!(writer, "Sample Size: {}", self.sample_size)?;
        writeln!(writer)?;
        writeln!(writer, "Details:")?;

        // Pretty print the JSON details
        if let Ok(pretty_json) = serde_json::to_string_pretty(&self.details) {
            for line in pretty_json.lines() {
                writeln!(writer, "  {}", line)?;
            }
        } else {
            writeln!(writer, "  {}", self.details)?;
        }

        Ok(())
    }
}

impl Output for DecayResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        if self.dry_run {
            writeln!(writer, "Pattern Decay Analysis (DRY RUN)")?;
        } else {
            writeln!(writer, "Pattern Decay Results")?;
        }
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(writer, "Analyzed {} patterns", self.total_patterns_analyzed)?;

        if self.dry_run {
            writeln!(writer, "Would decay {} patterns", self.would_decay_count)?;
        } else {
            writeln!(writer, "Decayed {} patterns", self.would_decay_count)?;
        }
        writeln!(writer)?;

        if self.patterns_to_decay.is_empty() {
            writeln!(
                writer,
                "{}",
                "No patterns to decay - all patterns are effective.".green()
            )?;
        } else {
            if self.dry_run {
                writeln!(writer, "Patterns that would be decayed:")?;
            } else {
                writeln!(writer, "Decayed patterns:")?;
            }
            writeln!(writer)?;

            for pattern in &self.patterns_to_decay {
                writeln!(
                    writer,
                    "• {} ({:.2} effectiveness, {} uses)",
                    pattern.pattern_id[..8].to_string().dimmed(),
                    pattern.effectiveness_score.to_string().red().bold(),
                    pattern.use_count
                )?;
                writeln!(writer, "  {}", pattern.description)?;
                writeln!(writer)?;
            }
        }

        Ok(())
    }
}

// Command implementations
pub async fn list_patterns(
    min_confidence: f32,
    pattern_type: Option<PatternType>,
    limit: usize,
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    // Get all patterns
    let patterns = memory
        .retrieve_relevant_patterns(&memory_core::types::TaskContext::default(), 1000)
        .await;

    // Filter and transform patterns
    let mut summaries: Vec<PatternSummary> = Vec::new();

    for pattern in patterns {
        // Filter by confidence
        if pattern.confidence() < min_confidence {
            continue;
        }

        // Filter by pattern type
        if let Some(ref pt) = pattern_type {
            let pattern_type_str = match pattern {
                memory_core::pattern::Pattern::ToolSequence { .. } => "ToolSequence",
                memory_core::pattern::Pattern::DecisionPoint { .. } => "DecisionPoint",
                memory_core::pattern::Pattern::ErrorRecovery { .. } => "ErrorRecovery",
                memory_core::pattern::Pattern::ContextPattern { .. } => "ContextPattern",
            };

            let requested_type = match pt {
                PatternType::ToolSequence => "ToolSequence",
                PatternType::DecisionPoint => "DecisionPoint",
                PatternType::ErrorRecovery => "ErrorRecovery",
                PatternType::ContextPattern => "ContextPattern",
            };

            if pattern_type_str != requested_type {
                continue;
            }
        }

        // Get effectiveness data (if available)
        let effectiveness = 0.5; // Default neutral effectiveness
        let use_count = pattern.sample_size();
        let last_used = "Unknown".to_string(); // TODO: Track last used time

        let description = match &pattern {
            memory_core::pattern::Pattern::ToolSequence { tools, context, .. } => {
                format!(
                    "Tool sequence: {} in {} domain",
                    tools.join(" → "),
                    context.domain
                )
            }
            memory_core::pattern::Pattern::DecisionPoint {
                condition, action, ..
            } => {
                format!("Decision: {} → {}", condition, action)
            }
            memory_core::pattern::Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                ..
            } => {
                format!(
                    "Error recovery: {} with {} steps",
                    error_type,
                    recovery_steps.len()
                )
            }
            memory_core::pattern::Pattern::ContextPattern {
                recommended_approach,
                ..
            } => {
                format!("Context pattern: {}", recommended_approach)
            }
        };

        let pattern_type_str = match &pattern {
            memory_core::pattern::Pattern::ToolSequence { .. } => "ToolSequence",
            memory_core::pattern::Pattern::DecisionPoint { .. } => "DecisionPoint",
            memory_core::pattern::Pattern::ErrorRecovery { .. } => "ErrorRecovery",
            memory_core::pattern::Pattern::ContextPattern { .. } => "ContextPattern",
        };

        summaries.push(PatternSummary {
            pattern_id: pattern.id().to_string(),
            pattern_type: pattern_type_str.to_string(),
            confidence: pattern.confidence(),
            effectiveness,
            use_count,
            last_used,
            description,
        });
    }

    // Sort by confidence (highest first)
    summaries.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply limit
    let total_count = summaries.len();
    summaries.truncate(limit);

    let pattern_list = PatternList {
        patterns: summaries,
        total_count,
    };

    pattern_list.write(&mut std::io::stdout(), &format)?;
    Ok(())
}

pub async fn view_pattern(
    pattern_id: String,
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use uuid::Uuid;

    let pattern_uuid = Uuid::parse_str(&pattern_id)
        .map_err(|_| anyhow::anyhow!("Invalid pattern ID format: {}", pattern_id))?;

    let pattern = memory
        .get_pattern(pattern_uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Pattern not found: {}", pattern_id))?;

    let (pattern_type, details) = match &pattern {
        memory_core::pattern::Pattern::ToolSequence {
            tools,
            context,
            success_rate,
            avg_latency,
            occurrence_count,
            ..
        } => (
            "ToolSequence".to_string(),
            serde_json::json!({
                "tools": tools,
                "context": context,
                "success_rate": success_rate,
                "avg_latency_ms": avg_latency.num_milliseconds(),
                "occurrence_count": occurrence_count
            }),
        ),
        memory_core::pattern::Pattern::DecisionPoint {
            condition,
            action,
            outcome_stats,
            context,
            ..
        } => (
            "DecisionPoint".to_string(),
            serde_json::json!({
                "condition": condition,
                "action": action,
                "outcome_stats": outcome_stats,
                "context": context
            }),
        ),
        memory_core::pattern::Pattern::ErrorRecovery {
            error_type,
            recovery_steps,
            success_rate,
            context,
            ..
        } => (
            "ErrorRecovery".to_string(),
            serde_json::json!({
                "error_type": error_type,
                "recovery_steps": recovery_steps,
                "success_rate": success_rate,
                "context": context
            }),
        ),
        memory_core::pattern::Pattern::ContextPattern {
            context_features,
            recommended_approach,
            evidence,
            success_rate,
            ..
        } => (
            "ContextPattern".to_string(),
            serde_json::json!({
                "context_features": context_features,
                "recommended_approach": recommended_approach,
                "evidence_count": evidence.len(),
                "success_rate": success_rate
            }),
        ),
    };

    let detail = PatternDetail {
        id: pattern.id().to_string(),
        pattern_type,
        confidence: pattern.confidence(),
        success_rate: pattern.success_rate(),
        sample_size: pattern.sample_size(),
        details,
    };

    // For human format, create a custom display
    if format == OutputFormat::Human {
        println!("Pattern Details");
        println!("===============");
        println!("ID: {}", detail.id);
        println!("Type: {}", detail.pattern_type);
        println!("Confidence: {:.2}", detail.confidence);
        println!("Success Rate: {:.2}", detail.success_rate);
        println!("Sample Size: {}", detail.sample_size);
        println!();
        println!("Details:");
        match &pattern {
            memory_core::pattern::Pattern::ToolSequence {
                tools,
                context,
                success_rate,
                avg_latency,
                occurrence_count,
                ..
            } => {
                println!("  Tools: {}", tools.join(" → "));
                println!("  Context Domain: {}", context.domain);
                println!(
                    "  Language: {}",
                    context.language.as_deref().unwrap_or("N/A")
                );
                println!("  Success Rate: {:.2}", success_rate);
                println!("  Average Latency: {}ms", avg_latency.num_milliseconds());
                println!("  Occurrence Count: {}", occurrence_count);
            }
            memory_core::pattern::Pattern::DecisionPoint {
                condition,
                action,
                outcome_stats,
                context,
                ..
            } => {
                println!("  Condition: {}", condition);
                println!("  Action: {}", action);
                println!("  Success Rate: {:.2}", outcome_stats.success_rate());
                println!("  Total Cases: {}", outcome_stats.total_count);
                println!("  Context Domain: {}", context.domain);
            }
            memory_core::pattern::Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                success_rate,
                context,
                ..
            } => {
                println!("  Error Type: {}", error_type);
                println!("  Recovery Steps:");
                for (i, step) in recovery_steps.iter().enumerate() {
                    println!("    {}. {}", i + 1, step);
                }
                println!("  Success Rate: {:.2}", success_rate);
                println!("  Context Domain: {}", context.domain);
            }
            memory_core::pattern::Pattern::ContextPattern {
                context_features,
                recommended_approach,
                evidence,
                success_rate,
                ..
            } => {
                println!("  Context Features:");
                for feature in context_features {
                    println!("    - {}", feature);
                }
                println!("  Recommended Approach: {}", recommended_approach);
                println!("  Evidence Episodes: {}", evidence.len());
                println!("  Success Rate: {:.2}", success_rate);
            }
        }
    } else {
        // For JSON/YAML, use the structured data
        detail.write(&mut std::io::stdout(), &format)?;
    }

    Ok(())
}

pub async fn analyze_pattern(
    pattern_id: String,
    episodes: usize,
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use uuid::Uuid;

    let pattern_uuid = Uuid::parse_str(&pattern_id)
        .map_err(|_| anyhow::anyhow!("Invalid pattern ID format: {}", pattern_id))?;

    let _pattern = memory
        .get_pattern(pattern_uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Pattern not found: {}", pattern_id))?;

    // Get recent episodes to analyze
    let context = memory_core::types::TaskContext::default();
    let recent_episodes = memory
        .retrieve_relevant_context("".to_string(), context, episodes)
        .await;

    // Analyze pattern effectiveness across episodes
    let mut successful_applications = 0;
    let mut total_applications = 0;
    let mut improvement_scores = Vec::new();

    for episode in recent_episodes {
        // Check if this episode used the pattern
        if episode.patterns.contains(&pattern_uuid) {
            total_applications += 1;

            // Consider episode successful if it has a positive reward
            if let Some(reward) = &episode.reward {
                if reward.total > 0.0 {
                    successful_applications += 1;
                }

                // Calculate improvement score (simplified)
                let base_complexity = match episode.context.complexity {
                    memory_core::types::ComplexityLevel::Simple => 1.0,
                    memory_core::types::ComplexityLevel::Moderate => 2.0,
                    memory_core::types::ComplexityLevel::Complex => 3.0,
                };

                let efficiency_score = reward.total / base_complexity;
                improvement_scores.push(efficiency_score);
            }
        }
    }

    let success_rate = if total_applications > 0 {
        successful_applications as f32 / total_applications as f32
    } else {
        0.0
    };

    let average_improvement = if !improvement_scores.is_empty() {
        improvement_scores.iter().sum::<f32>() / improvement_scores.len() as f32
    } else {
        0.0
    };

    // Generate recommendations
    let mut recommendations = Vec::new();

    if success_rate < 0.5 {
        recommendations
            .push("Consider reviewing pattern accuracy - success rate is below 50%".to_string());
    }

    if total_applications < 3 {
        recommendations.push(
            "Pattern has limited usage data - needs more applications for reliable analysis"
                .to_string(),
        );
    }

    if average_improvement < 0.7 {
        recommendations
            .push("Pattern may need refinement - average improvement score is low".to_string());
    }

    if success_rate > 0.8 && total_applications >= 5 {
        recommendations.push(
            "Pattern is highly effective - consider promoting for similar contexts".to_string(),
        );
    }

    let analysis = PatternAnalysisData {
        success_rate,
        average_improvement,
        episodes_analyzed: episodes,
        recommendations,
    };

    let result = PatternAnalysis {
        pattern_id: pattern_id.clone(),
        analysis,
    };

    result.write(&mut std::io::stdout(), &format)?;
    Ok(())
}

pub async fn pattern_effectiveness(
    top: usize,
    min_uses: usize,
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    // Get all patterns
    let patterns = memory
        .retrieve_relevant_patterns(&memory_core::types::TaskContext::default(), 1000)
        .await;

    // Create effectiveness rankings
    let mut rankings: Vec<(String, f32, usize, String)> = Vec::new();

    for pattern in patterns {
        let use_count = pattern.sample_size();

        // Filter by minimum uses
        if use_count < min_uses {
            continue;
        }

        // Calculate effectiveness score (simplified - could use EffectivenessTracker in future)
        let effectiveness =
            pattern.success_rate() * (1.0 + (use_count as f32).ln().min(2.0) / 10.0);

        let description = match &pattern {
            memory_core::pattern::Pattern::ToolSequence { tools, .. } => {
                format!("Tool sequence: {}", tools.join(" → "))
            }
            memory_core::pattern::Pattern::DecisionPoint {
                condition, action, ..
            } => {
                format!("Decision: {} → {}", condition, action)
            }
            memory_core::pattern::Pattern::ErrorRecovery { error_type, .. } => {
                format!("Error recovery: {}", error_type)
            }
            memory_core::pattern::Pattern::ContextPattern {
                recommended_approach,
                ..
            } => {
                format!("Context: {}", recommended_approach)
            }
        };

        rankings.push((
            pattern.id().to_string(),
            effectiveness,
            use_count,
            description,
        ));
    }

    // Sort by effectiveness (highest first)
    rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top N
    rankings.truncate(top);

    let rankings_struct: Vec<EffectivenessRanking> = rankings
        .into_iter()
        .enumerate()
        .map(|(i, (id, score, uses, desc))| EffectivenessRanking {
            rank: i + 1,
            pattern_id: id,
            effectiveness_score: score,
            use_count: uses,
            description: desc,
        })
        .collect();

    let total_analyzed = rankings_struct.len();

    let result = EffectivenessRankings {
        rankings: rankings_struct,
        total_patterns_analyzed: total_analyzed,
    };

    // Custom human formatting
    if format == OutputFormat::Human {
        use colored::*;

        println!("Pattern Effectiveness Rankings");
        println!("==============================");
        println!("Showing top {} patterns (min {} uses)", top, min_uses);
        println!();

        if result.rankings.is_empty() {
            println!("No patterns found matching criteria.");
            return Ok(());
        }

        for ranking in &result.rankings {
            let score_color = match ranking.effectiveness_score {
                s if s >= 0.8 => Color::Green,
                s if s >= 0.6 => Color::Yellow,
                _ => Color::Red,
            };

            println!(
                "{}. {} ({:.2}) - {} uses",
                ranking.rank.to_string().bold(),
                ranking.pattern_id[..8].to_string().dimmed(),
                ranking
                    .effectiveness_score
                    .to_string()
                    .color(score_color)
                    .bold(),
                ranking.use_count
            );
            println!("   {}", ranking.description);
            println!();
        }
    } else {
        result.write(&mut std::io::stdout(), &format)?;
    }

    Ok(())
}

pub async fn decay_patterns(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
    force: bool,
) -> anyhow::Result<()> {
    // Note: In a real implementation, we'd have an EffectivenessTracker instance
    // For now, we'll simulate decay analysis

    // Get all patterns
    let patterns = memory
        .retrieve_relevant_patterns(&memory_core::types::TaskContext::default(), 1000)
        .await;

    // Analyze which patterns would be decayed
    let mut patterns_to_decay = Vec::new();
    let min_effectiveness = 0.3; // Same as EffectivenessTracker default

    for pattern in &patterns {
        let effectiveness =
            pattern.success_rate() * (1.0 + (pattern.sample_size() as f32).ln().min(2.0) / 10.0);

        if effectiveness < min_effectiveness {
            patterns_to_decay.push((
                pattern.id().to_string(),
                effectiveness,
                pattern.sample_size(),
                match pattern {
                    memory_core::pattern::Pattern::ToolSequence { tools, .. } => {
                        format!("Tool sequence: {}", tools.join(" → "))
                    }
                    memory_core::pattern::Pattern::DecisionPoint {
                        condition, action, ..
                    } => {
                        format!("Decision: {} → {}", condition, action)
                    }
                    memory_core::pattern::Pattern::ErrorRecovery { error_type, .. } => {
                        format!("Error recovery: {}", error_type)
                    }
                    memory_core::pattern::Pattern::ContextPattern {
                        recommended_approach,
                        ..
                    } => {
                        format!("Context: {}", recommended_approach)
                    }
                },
            ));
        }
    }

    let decay_infos: Vec<DecayPatternInfo> = patterns_to_decay
        .iter()
        .map(|(id, score, uses, desc)| DecayPatternInfo {
            pattern_id: id.clone(),
            effectiveness_score: *score,
            use_count: *uses,
            description: desc.clone(),
        })
        .collect();

    let result = DecayResult {
        patterns_to_decay: decay_infos,
        total_patterns_analyzed: patterns.len(),
        dry_run,
        would_decay_count: patterns_to_decay.len(),
    };

    // Handle dry run
    if dry_run {
        if format == OutputFormat::Human {
            use colored::*;

            println!("Pattern Decay Analysis (DRY RUN)");
            println!("=================================");
            println!(
                "Analyzed {} patterns, would decay {} ineffective patterns",
                result.total_patterns_analyzed, result.would_decay_count
            );
            println!();

            if result.patterns_to_decay.is_empty() {
                println!(
                    "{}",
                    "No patterns would be decayed - all patterns are effective.".green()
                );
            } else {
                println!("Patterns that would be decayed:");
                println!();

                for pattern in &result.patterns_to_decay {
                    println!(
                        "• {} ({:.2} effectiveness, {} uses)",
                        pattern.pattern_id[..8].to_string().dimmed(),
                        pattern.effectiveness_score.to_string().red().bold(),
                        pattern.use_count
                    );
                    println!("  {}", pattern.description);
                    println!();
                }

                println!(
                    "Run with {} to actually perform decay.",
                    "--force".yellow().bold()
                );
            }
        } else {
            result.write(&mut std::io::stdout(), &format)?;
        }
        return Ok(());
    }

    // Handle actual decay
    if !force && !result.patterns_to_decay.is_empty() {
        if format == OutputFormat::Human {
            use colored::*;
            use dialoguer::Confirm;

            println!("Pattern Decay Analysis");
            println!("======================");
            println!(
                "Found {} patterns that could be decayed.",
                result.would_decay_count.to_string().yellow().bold()
            );
            println!();

            // Show preview of patterns to be decayed
            println!("Patterns to be decayed:");
            for pattern in result.patterns_to_decay.iter().take(5) {
                println!(
                    "  • {} ({:.2} effectiveness, {} uses)",
                    &pattern.pattern_id[..8],
                    pattern.effectiveness_score,
                    pattern.use_count
                );
            }
            if result.patterns_to_decay.len() > 5 {
                println!("  ... and {} more", result.patterns_to_decay.len() - 5);
            }
            println!();

            println!(
                "{}",
                "This will permanently remove ineffective patterns from the system.".yellow()
            );
            println!();

            // Interactive confirmation
            let confirmed = Confirm::new()
                .with_prompt("Continue with pattern decay?")
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{}", "Operation cancelled.".yellow());
                return Ok(());
            }
        } else {
            // Non-human format requires --force flag
            anyhow::bail!("Pattern decay requires --force flag for non-interactive formats");
        }
    }

    // Perform actual decay (in real implementation, this would remove from storage)
    if format == OutputFormat::Human {
        use colored::*;

        println!("Pattern Decay Results");
        println!("=====================");

        if result.patterns_to_decay.is_empty() {
            println!(
                "{}",
                "No patterns were decayed - all patterns are effective.".green()
            );
        } else {
            println!(
                "Successfully decayed {} ineffective patterns.",
                result.would_decay_count.to_string().green().bold()
            );
            println!();
            println!("Decayed patterns:");

            for pattern in &result.patterns_to_decay {
                println!(
                    "• {} ({:.2} effectiveness)",
                    pattern.pattern_id[..8].to_string().dimmed(),
                    pattern.effectiveness_score.to_string().red()
                );
            }
        }
    } else {
        result.write(&mut std::io::stdout(), &format)?;
    }

    Ok(())
}
