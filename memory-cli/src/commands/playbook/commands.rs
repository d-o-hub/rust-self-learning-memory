//! Playbook command implementations (ADR-044 Feature 1)

use super::types::{PatternExplanation, PlaybookStepSummary, PlaybookSummary};
use crate::config::Config;
use crate::output::OutputFormat;
use anyhow::Result;
use do_memory_core::types::{ComplexityLevel, TaskContext, TaskType};

/// Generate a playbook recommendation for a task
#[allow(clippy::too_many_arguments)]
pub async fn recommend_playbook(
    task: &str,
    domain: &str,
    task_type: &str,
    max_steps: usize,
    language: Option<&str>,
    framework: Option<&str>,
    tags: Vec<String>,
    memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> Result<()> {
    let task_type_enum = match task_type {
        "code_generation" => TaskType::CodeGeneration,
        "debugging" => TaskType::Debugging,
        "refactoring" => TaskType::Refactoring,
        "testing" => TaskType::Testing,
        "analysis" => TaskType::Analysis,
        "documentation" => TaskType::Documentation,
        _ => TaskType::CodeGeneration,
    };

    let context = TaskContext {
        domain: domain.to_string(),
        language: language.map(|s| s.to_string()),
        framework: framework.map(|s| s.to_string()),
        complexity: ComplexityLevel::Moderate,
        tags,
    };

    let playbooks = memory
        .retrieve_playbooks(task, domain, task_type_enum, context, 1, max_steps)
        .await;

    if playbooks.is_empty() {
        println!("No playbook could be generated for this task.");
        println!("\nTip: Complete more episodes with similar tasks to build up pattern data.");
        return Ok(());
    }

    let playbook = &playbooks[0];

    let summary = PlaybookSummary {
        playbook_id: playbook.playbook_id.to_string(),
        task_match_score: playbook.task_match_score,
        confidence: playbook.confidence,
        why_relevant: playbook.why_relevant.clone(),
        step_count: playbook.ordered_steps.len(),
        steps: playbook
            .ordered_steps
            .iter()
            .map(|s| PlaybookStepSummary {
                order: s.order,
                action: s.action.clone(),
                tool_hint: s.tool_hint.clone(),
                expected_result: s.expected_result.clone(),
            })
            .collect(),
        pitfalls: playbook
            .pitfalls
            .iter()
            .map(|p| p.warning.clone())
            .collect(),
        when_to_apply: playbook.when_to_apply.clone(),
        when_not_to_apply: playbook.when_not_to_apply.clone(),
        expected_outcome: playbook.expected_outcome.clone(),
    };

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        OutputFormat::Human | OutputFormat::Yaml => {
            print_playbook_human(&summary);
        }
    }

    Ok(())
}

/// Explain a pattern in human-readable form
pub async fn explain_pattern(
    pattern_id: &str,
    memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> Result<()> {
    let pattern_uuid = uuid::Uuid::parse_str(pattern_id)
        .map_err(|e| anyhow::anyhow!("Invalid pattern ID: {}", e))?;

    let explanation = memory
        .explain_pattern(pattern_uuid)
        .await
        .ok_or_else(|| anyhow::anyhow!("Pattern not found: {}", pattern_id))?;

    let result = PatternExplanation {
        pattern_id: pattern_id.to_string(),
        explanation,
    };

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Human | OutputFormat::Yaml => {
            println!("Pattern: {}", result.pattern_id);
            println!("{}", "-".repeat(50));
            println!("{}", result.explanation);
        }
    }

    Ok(())
}

fn print_playbook_human(summary: &PlaybookSummary) {
    println!("PLAYBOOK RECOMMENDATION");
    println!("{}", "=".repeat(60));
    println!();
    println!("ID: {}", summary.playbook_id);
    println!("Task Match Score: {:.0}%", summary.task_match_score * 100.0);
    println!("Confidence: {:.0}%", summary.confidence * 100.0);
    println!();
    println!("WHY RELEVANT");
    println!("{}", "-".repeat(40));
    println!("{}", summary.why_relevant);
    println!();

    if !summary.steps.is_empty() {
        println!("STEPS");
        println!("{}", "-".repeat(40));
        for step in &summary.steps {
            println!("\n  {}. {}", step.order, step.action);
            if let Some(ref tool) = step.tool_hint {
                println!("     Tool hint: {}", tool);
            }
            if let Some(ref result) = step.expected_result {
                println!("     Expected: {}", result);
            }
        }
        println!();
    }

    if !summary.when_to_apply.is_empty() {
        println!("WHEN TO APPLY");
        println!("{}", "-".repeat(40));
        for condition in &summary.when_to_apply {
            println!("  - {}", condition);
        }
        println!();
    }

    if !summary.when_not_to_apply.is_empty() {
        println!("WHEN NOT TO APPLY");
        println!("{}", "-".repeat(40));
        for condition in &summary.when_not_to_apply {
            println!("  - {}", condition);
        }
        println!();
    }

    if !summary.pitfalls.is_empty() {
        println!("PITFALLS TO AVOID");
        println!("{}", "-".repeat(40));
        for pitfall in &summary.pitfalls {
            println!("  [!] {}", pitfall);
        }
        println!();
    }

    println!("EXPECTED OUTCOME");
    println!("{}", "-".repeat(40));
    println!("{}", summary.expected_outcome);
}
