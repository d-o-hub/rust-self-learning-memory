//! Pattern effectiveness command implementation

use crate::config::Config;
use crate::output::{Output, OutputFormat};
use memory_core::SelfLearningMemory;
use super::types::{EffectivenessRankings, EffectivenessRanking};

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
        let effectiveness_data = pattern.effectiveness();
        let use_count = effectiveness_data.times_applied;

        // Filter by minimum uses
        if use_count < min_uses {
            continue;
        }

        // Use the actual effectiveness score from tracking
        let effectiveness = effectiveness_data.effectiveness_score();

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
