//! List patterns command implementation

use super::types::{PatternList, PatternSummary};
use crate::config::Config;
use crate::output::{Output, OutputFormat};
use chrono::{DateTime, Utc};
use memory_core::SelfLearningMemory;

pub async fn list_patterns(
    min_confidence: f32,
    pattern_type: Option<super::types::PatternType>,
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
                super::types::PatternType::ToolSequence => "ToolSequence",
                super::types::PatternType::DecisionPoint => "DecisionPoint",
                super::types::PatternType::ErrorRecovery => "ErrorRecovery",
                super::types::PatternType::ContextPattern => "ContextPattern",
            };

            if pattern_type_str != requested_type {
                continue;
            }
        }

        // Get effectiveness data from the pattern
        let effectiveness_data = pattern.effectiveness();
        let effectiveness = effectiveness_data.effectiveness_score();
        let use_count = effectiveness_data.times_applied;
        let last_used = format_relative_time(effectiveness_data.last_used);

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

fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}
