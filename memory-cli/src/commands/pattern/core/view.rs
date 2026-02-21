//! View pattern command implementation

use super::types::PatternDetail;
use crate::config::Config;
use crate::errors::{EnhancedError, helpers};
use crate::output::{Output, OutputFormat};
use uuid::Uuid;

pub async fn view_pattern(
    pattern_id: String,
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let pattern_uuid = Uuid::parse_str(&pattern_id).context_with_help(
        &format!("Invalid pattern ID format: {}", pattern_id),
        helpers::INVALID_INPUT_HELP,
    )?;

    let pattern = memory
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
        context: serde_json::to_value(&details)?,
        effectiveness_data: serde_json::json!({
            "success_rate": pattern.success_rate(),
            "sample_size": pattern.sample_size()
        }),
        extracted_at: chrono::Utc::now().to_rfc3339(),
    };

    // For human format, create a custom display
    if format == OutputFormat::Human {
        println!("Pattern Details");
        println!("===============");
        println!("ID: {}", detail.id);
        println!("Type: {}", detail.pattern_type);
        println!("Confidence: {:.2}", detail.confidence);
        println!();
        println!("Details:");
        match &pattern {
            memory_core::pattern::Pattern::ToolSequence { tools, context, .. } => {
                println!("  Tools: {}", tools.join(" → "));
                println!("  Context Domain: {}", context.domain);
            }
            memory_core::pattern::Pattern::DecisionPoint {
                condition, action, ..
            } => {
                println!("  Condition: {}", condition);
                println!("  Action: {}", action);
            }
            memory_core::pattern::Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                ..
            } => {
                println!("  Error Type: {}", error_type);
                println!("  Recovery Steps:");
                for (i, step) in recovery_steps.iter().enumerate() {
                    println!("    {}. {}", i + 1, step);
                }
            }
            memory_core::pattern::Pattern::ContextPattern {
                context_features,
                recommended_approach,
                ..
            } => {
                println!("  Context Features:");
                for feature in context_features {
                    println!("    - {}", feature);
                }
                println!("  Recommended Approach: {}", recommended_approach);
            }
        }
    } else {
        detail.write(&mut std::io::stdout(), &format)?;
    }

    Ok(())
}
