//! Output formatting for DAG context assembly (WG-134).
//!
//! Provides display/formatting functions for `AssembledContext`,
//! `SharedContextItem`, and `UniqueContextItem`.

use std::fmt::Write;

use super::assembler::{AssembledContext, AssemblyFormat, SharedContextItem};
use super::node::StateNodeType;

/// Format assembled context as a string, dispatching by format mode.
#[must_use]
pub fn format_for_prompt(format: AssemblyFormat, assembled: &AssembledContext) -> String {
    match format {
        AssemblyFormat::Compact => format_compact(assembled),
        AssemblyFormat::Full => format_full(assembled),
        AssemblyFormat::TokenOptimized => format_optimized(assembled),
    }
}

/// Format in compact mode (minimal — node refs only).
fn format_compact(assembled: &AssembledContext) -> String {
    let shared = assembled
        .shared_context
        .iter()
        .map(|s| format!("{}:{}", s.node_type_name(), s.node_id))
        .collect::<Vec<_>>()
        .join(",");

    let unique = assembled
        .unique_context
        .iter()
        .map(|u| {
            format!(
                "{}:{}",
                u.episode_id,
                u.task_description.chars().take(30).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join("|");

    format!("S:{shared}\nU:{unique}")
}

/// Format in full mode (verbose — includes section headers and descriptions).
fn format_full(assembled: &AssembledContext) -> String {
    let mut output = String::new();

    output.push_str("## Shared Context\n");
    for item in &assembled.shared_context {
        writeln!(
            output,
            "- {} = {} (shared by {} episodes)",
            item.node_type_name(),
            item.value,
            item.shared_count
        )
        .unwrap();
    }

    output.push_str("\n## Episode Context\n");
    for item in &assembled.unique_context {
        writeln!(
            output,
            "- Episode {}: {}",
            item.episode_id, item.task_description
        )
        .unwrap();
        if !item.unique_aspects.is_empty() {
            writeln!(output, "  Unique: {}", item.unique_aspects.join(", ")).unwrap();
        }
    }

    output
}

/// Format in token-optimized mode (minimum tokens for LLM context).
fn format_optimized(assembled: &AssembledContext) -> String {
    let mut output = String::new();

    // Shared block (one copy, referenced)
    if !assembled.shared_context.is_empty() {
        output.push_str("SHARED:\n");
        for item in &assembled.shared_context {
            writeln!(output, "{}={}", item.node_type_name(), item.value).unwrap();
        }
    }

    // Unique per episode
    for item in &assembled.unique_context {
        writeln!(
            output,
            "EP:{}|{}",
            item.episode_id,
            item.task_description.chars().take(50).collect::<String>()
        )
        .unwrap();
    }

    output
}

/// Calculate token reduction percentage from assembled context.
#[must_use]
pub fn reduction_percentage(assembled: &AssembledContext) -> f32 {
    if assembled.token_savings == 0 {
        return 0.0;
    }
    let original = assembled.estimated_tokens + assembled.token_savings;
    if original == 0 {
        return 0.0;
    }
    (assembled.token_savings as f32 / original as f32) * 100.0
}

impl SharedContextItem {
    /// Get human-readable type name (short form).
    #[must_use]
    pub fn node_type_name(&self) -> &'static str {
        match self.node_type {
            StateNodeType::Language => "lang",
            StateNodeType::Framework => "fw",
            StateNodeType::Domain => "domain",
            StateNodeType::TaskType => "type",
            StateNodeType::Complexity => "complexity",
            StateNodeType::Tag => "tag",
            StateNodeType::Composite => "composite",
        }
    }
}
