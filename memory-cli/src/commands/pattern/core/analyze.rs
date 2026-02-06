//! Analyze pattern command implementation

use super::types::{PatternAnalysisData, PatternAnalysisResult};
use crate::config::Config;
use crate::errors::{helpers, EnhancedError};
use crate::output::{Output, OutputFormat};
use uuid::Uuid;

pub async fn analyze_pattern(
    pattern_id: String,
    episodes: usize,
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let pattern_uuid = Uuid::parse_str(&pattern_id).context_with_help(
        &format!("Invalid pattern ID format: {}", pattern_id),
        helpers::INVALID_INPUT_HELP,
    )?;

    let _pattern = memory
        .get_pattern(pattern_uuid)
        .await
        .context_with_help(
            "Failed to retrieve pattern from storage",
            helpers::DATABASE_OPERATION_HELP,
        )?
        .ok_or_else(|| {
            anyhow::anyhow!(helpers::format_error_message(
                &format!("Pattern not found: {}", pattern_id),
                "Pattern does not exist in storage",
                helpers::PATTERN_NOT_FOUND_HELP
            ))
        })?;

    // Get recent episodes to analyze (returns Vec<Arc<Episode>>)
    let context = memory_core::types::TaskContext::default();
    let arc_episodes = memory
        .retrieve_relevant_context("".to_string(), context, episodes)
        .await;

    // Analyze pattern effectiveness across episodes
    let mut successful_applications = 0;
    let mut total_applications = 0;
    let mut improvement_scores = Vec::new();

    for arc_ep in arc_episodes {
        let episode = arc_ep.as_ref();
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

    let result = PatternAnalysisResult {
        pattern_id: pattern_id.clone(),
        analysis,
    };

    result.write(&mut std::io::stdout(), &format)?;
    Ok(())
}
