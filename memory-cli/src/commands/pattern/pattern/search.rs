//! Pattern search command

use crate::output::OutputFormat;
use anyhow::Result;
use memory_core::{ComplexityLevel, SelfLearningMemory, TaskContext};
use serde::{Deserialize, Serialize};

/// Search for patterns semantically similar to a query
#[allow(clippy::too_many_arguments)]
pub async fn search_patterns(
    memory: &SelfLearningMemory,
    query: &str,
    domain: &str,
    tags: Vec<String>,
    limit: usize,
    min_relevance: f32,
    filter_by_domain: bool,
    format: OutputFormat,
) -> Result<()> {
    // Build context
    let context = TaskContext {
        domain: domain.to_string(),
        language: None,
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags,
    };

    // Build config
    let config = memory_core::memory::SearchConfig {
        min_relevance,
        filter_by_domain,
        ..memory_core::memory::SearchConfig::default()
    };

    // Execute search
    let results = memory
        .search_patterns_with_config(query, context, config, limit)
        .await?;

    // Format output
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&results)?;
            println!("{}", json);
        }
        OutputFormat::Human => {
            if results.is_empty() {
                println!("No patterns found matching query: {}", query);
                return Ok(());
            }

            println!("Found {} patterns for query: \"{}\"", results.len(), query);
            println!();

            for (i, result) in results.iter().enumerate() {
                println!(
                    "{}. Pattern (relevance: {:.2})",
                    i + 1,
                    result.relevance_score
                );
                println!("   ID: {}", result.pattern.id());
                println!(
                    "   Success Rate: {:.1}%",
                    result.pattern.success_rate() * 100.0
                );

                if let Some(ctx) = result.pattern.context() {
                    println!("   Domain: {}", ctx.domain);
                }

                println!("   Score Breakdown:");
                println!(
                    "     - Semantic: {:.2}",
                    result.score_breakdown.semantic_similarity
                );
                println!(
                    "     - Context: {:.2}",
                    result.score_breakdown.context_match
                );
                println!(
                    "     - Effectiveness: {:.2}",
                    result.score_breakdown.effectiveness
                );
                println!("     - Recency: {:.2}", result.score_breakdown.recency);
                println!("     - Success: {:.2}", result.score_breakdown.success_rate);

                let eff = result.pattern.effectiveness();
                println!(
                    "   Usage: {} times applied, {} times retrieved",
                    eff.times_applied, eff.times_retrieved
                );
                println!();
            }
        }
        OutputFormat::Yaml => {
            let yaml_output: Vec<YamlPatternResult> = results
                .iter()
                .map(|r| YamlPatternResult {
                    id: r.pattern.id().to_string(),
                    relevance_score: r.relevance_score,
                    score_breakdown: YamlScoreBreakdown {
                        semantic_similarity: r.score_breakdown.semantic_similarity,
                        context_match: r.score_breakdown.context_match,
                        effectiveness: r.score_breakdown.effectiveness,
                        recency: r.score_breakdown.recency,
                        success_rate: r.score_breakdown.success_rate,
                    },
                })
                .collect();
            println!(
                "{}",
                serde_yaml::to_string(&yaml_output).unwrap_or_default()
            );
        }
    }

    Ok(())
}

/// Recommend patterns for a specific task
pub async fn recommend_patterns(
    memory: &SelfLearningMemory,
    task_description: &str,
    domain: &str,
    tags: Vec<String>,
    limit: usize,
    format: OutputFormat,
) -> Result<()> {
    // Build context
    let context = TaskContext {
        domain: domain.to_string(),
        language: None,
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags,
    };

    // Execute recommendation
    let results = memory
        .recommend_patterns_for_task(task_description, context, limit)
        .await?;

    // Format output
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&results)?;
            println!("{}", json);
        }
        OutputFormat::Human => {
            if results.is_empty() {
                println!(
                    "No pattern recommendations found for task: {}",
                    task_description
                );
                return Ok(());
            }

            println!("Pattern Recommendations for: \"{}\"", task_description);
            println!("Found {} recommendations", results.len());
            println!();

            for (i, result) in results.iter().enumerate() {
                println!(
                    "{}. Recommended Pattern (relevance: {:.2})",
                    i + 1,
                    result.relevance_score
                );
                println!("   ID: {}", result.pattern.id());
                println!(
                    "   Success Rate: {:.1}%",
                    result.pattern.success_rate() * 100.0
                );

                if let Some(ctx) = result.pattern.context() {
                    println!("   Domain: {}", ctx.domain);
                    if let Some(lang) = &ctx.language {
                        println!("   Language: {}", lang);
                    }
                    if let Some(fw) = &ctx.framework {
                        println!("   Framework: {}", fw);
                    }
                }

                // Show pattern details based on type
                match &result.pattern {
                    memory_core::Pattern::ToolSequence { tools, .. } => {
                        println!("   Type: Tool Sequence");
                        println!("   Tools: {}", tools.join(" → "));
                    }
                    memory_core::Pattern::DecisionPoint {
                        condition, action, ..
                    } => {
                        println!("   Type: Decision Point");
                        println!("   Condition: {}", condition);
                        println!("   Action: {}", action);
                    }
                    memory_core::Pattern::ErrorRecovery {
                        error_type,
                        recovery_steps,
                        ..
                    } => {
                        println!("   Type: Error Recovery");
                        println!("   Error: {}", error_type);
                        println!("   Recovery: {}", recovery_steps.join(", "));
                    }
                    memory_core::Pattern::ContextPattern {
                        recommended_approach,
                        ..
                    } => {
                        println!("   Type: Context Pattern");
                        println!("   Approach: {}", recommended_approach);
                    }
                }

                let eff = result.pattern.effectiveness();
                println!(
                    "   Applied {} times with {:.1}% success rate",
                    eff.times_applied,
                    eff.application_success_rate() * 100.0
                );
                println!();
            }

            println!("💡 Tip: Use these patterns as guidance for your current task!");
        }
        OutputFormat::Yaml => {
            let yaml_output: Vec<YamlPatternResult> = results
                .iter()
                .map(|r| YamlPatternResult {
                    id: r.pattern.id().to_string(),
                    relevance_score: r.relevance_score,
                    score_breakdown: YamlScoreBreakdown {
                        semantic_similarity: r.score_breakdown.semantic_similarity,
                        context_match: r.score_breakdown.context_match,
                        effectiveness: r.score_breakdown.effectiveness,
                        recency: r.score_breakdown.recency,
                        success_rate: r.score_breakdown.success_rate,
                    },
                })
                .collect();
            println!(
                "{}",
                serde_yaml::to_string(&yaml_output).unwrap_or_default()
            );
        }
    }

    Ok(())
}

/// YAML output types for pattern search results
#[derive(Debug, Serialize, Deserialize)]
struct YamlPatternResult {
    id: String,
    relevance_score: f32,
    score_breakdown: YamlScoreBreakdown,
}

#[derive(Debug, Serialize, Deserialize)]
struct YamlScoreBreakdown {
    semantic_similarity: f32,
    context_match: f32,
    effectiveness: f32,
    recency: f32,
    success_rate: f32,
}
