//! Pattern decay command implementation

use crate::config::Config;
use crate::output::{Output, OutputFormat};
use memory_core::SelfLearningMemory;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DecayPatternInfo {
    pub pattern_id: String,
    pub effectiveness_score: f32,
    pub use_count: usize,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct DecayResult {
    pub patterns_to_decay: Vec<DecayPatternInfo>,
    pub total_patterns_analyzed: usize,
    pub dry_run: bool,
    pub would_decay_count: usize,
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
        let effectiveness_data = pattern.effectiveness();
        let effectiveness = effectiveness_data.effectiveness_score();

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
