//! Compact handoff CLI presentation (issue #965).
//!
//! Serializable views and human/JSON/YAML rendering for the byte-budgeted
//! compact handoff profile. Full-pack rendering stays in `checkpoint`.

use super::checkpoint::Output;
use crate::output::OutputFormat;
use anyhow::Result;
use do_memory_core::memory::checkpoint::CompactHandoff;
use serde::Serialize;

/// Result of compact handoff pack retrieval
#[derive(Debug, Serialize)]
pub struct CompactHandoffResult {
    /// Checkpoint ID
    pub checkpoint_id: String,
    /// Episode ID
    pub episode_id: String,
    /// Current goal
    pub current_goal: String,
    /// Episode phase at build time
    pub status: String,
    /// Steps completed at the checkpoint
    pub steps_done: usize,
    /// Total steps in the episode
    pub steps_total: usize,
    /// Verified findings
    pub verified_findings: Vec<String>,
    /// Critical decisions
    pub decisions: Vec<String>,
    /// Pending actions
    pub pending_actions: Vec<String>,
    /// Pattern ID references
    pub pattern_refs: Vec<String>,
    /// Heuristic ID references
    pub heuristic_refs: Vec<String>,
    /// Evidence excerpts (most recent steps)
    pub evidence_excerpts: Vec<EvidenceExcerptView>,
    /// Omitted steps / findings / decisions / actions / refs
    pub omitted: OmittedView,
    /// Serialized JSON payload size in bytes
    pub payload_bytes: usize,
    /// Heuristic token estimate
    pub approx_tokens: usize,
}

/// Serializable evidence excerpt for CLI output
#[derive(Debug, Serialize)]
pub struct EvidenceExcerptView {
    /// 1-indexed step number
    pub step_number: usize,
    /// Tool used
    pub tool: String,
    /// Action description
    pub action: String,
    /// Bounded result summary
    pub result_summary: Option<String>,
}

/// Serializable omission account for CLI output
#[derive(Debug, Serialize)]
pub struct OmittedView {
    /// Steps without an excerpt
    pub omitted_steps: usize,
    /// Findings dropped
    pub omitted_findings: usize,
    /// Decisions dropped
    pub omitted_decisions: usize,
    /// Pending actions dropped
    pub omitted_pending_actions: usize,
    /// Pattern references dropped
    pub omitted_patterns: usize,
    /// Heuristic references dropped
    pub omitted_heuristics: usize,
    /// Fields cut mid-string
    pub truncated_fields: Vec<String>,
    /// Pointer to the full-fidelity pack
    pub full_available_via: String,
}

impl From<CompactHandoff> for CompactHandoffResult {
    fn from(pack: CompactHandoff) -> Self {
        let payload_bytes = pack.payload_bytes();
        Self {
            checkpoint_id: pack.checkpoint_id.to_string(),
            episode_id: pack.episode_id.to_string(),
            current_goal: pack.current_goal,
            status: pack.status,
            steps_done: pack.steps_done,
            steps_total: pack.steps_total,
            verified_findings: pack.verified_findings,
            decisions: pack.decisions,
            pending_actions: pack.pending_actions,
            pattern_refs: pack.pattern_refs,
            heuristic_refs: pack.heuristic_refs,
            evidence_excerpts: pack
                .evidence_excerpts
                .into_iter()
                .map(|e| EvidenceExcerptView {
                    step_number: e.step_number,
                    tool: e.tool,
                    action: e.action,
                    result_summary: e.result_summary,
                })
                .collect(),
            omitted: OmittedView {
                omitted_steps: pack.omitted.omitted_steps,
                omitted_findings: pack.omitted.omitted_findings,
                omitted_decisions: pack.omitted.omitted_decisions,
                omitted_pending_actions: pack.omitted.omitted_pending_actions,
                omitted_patterns: pack.omitted.omitted_patterns,
                omitted_heuristics: pack.omitted.omitted_heuristics,
                truncated_fields: pack.omitted.truncated_fields,
                full_available_via: pack.omitted.full_available_via,
            },
            payload_bytes,
            approx_tokens: pack.approx_tokens,
        }
    }
}

impl Output for CompactHandoffResult {
    fn write(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(self)?);
            }
            OutputFormat::Human => {
                println!("Compact Handoff for Checkpoint: {}", self.checkpoint_id);
                println!("  Episode ID:      {}", self.episode_id);
                println!("  Goal:            {}", self.current_goal);
                println!(
                    "  Status:          {} (step {}/{})",
                    self.status, self.steps_done, self.steps_total
                );
                println!(
                    "  Budget:          {} bytes (~{} tokens)",
                    self.payload_bytes, self.approx_tokens
                );
                println!();

                if !self.verified_findings.is_empty() {
                    println!("Verified Findings:");
                    for finding in &self.verified_findings {
                        println!("  * {finding}");
                    }
                    println!();
                }

                if !self.decisions.is_empty() {
                    println!("Decisions:");
                    for decision in &self.decisions {
                        println!("  # {decision}");
                    }
                    println!();
                }

                if !self.pending_actions.is_empty() {
                    println!("Pending Actions:");
                    for (i, action) in self.pending_actions.iter().enumerate() {
                        println!("  {}. {action}", i + 1);
                    }
                    println!();
                }

                if !self.evidence_excerpts.is_empty() {
                    println!("Evidence (most recent steps):");
                    for excerpt in &self.evidence_excerpts {
                        println!(
                            "  [step {}] {}: {}",
                            excerpt.step_number, excerpt.tool, excerpt.action
                        );
                    }
                    println!();
                }

                let omitted = &self.omitted;
                if omitted.omitted_steps > 0
                    || omitted.omitted_findings > 0
                    || omitted.omitted_decisions > 0
                    || omitted.omitted_pending_actions > 0
                    || omitted.omitted_patterns > 0
                    || omitted.omitted_heuristics > 0
                {
                    println!(
                        "Omitted: {} steps, {} findings, {} decisions, {} actions, {} patterns, {} heuristics",
                        omitted.omitted_steps,
                        omitted.omitted_findings,
                        omitted.omitted_decisions,
                        omitted.omitted_pending_actions,
                        omitted.omitted_patterns,
                        omitted.omitted_heuristics
                    );
                    println!("  Full pack: {}", omitted.full_available_via);
                }
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(self)?);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_result_maps_all_sections() {
        use do_memory_core::memory::checkpoint::{
            CompactHandoff, EvidenceExcerpt, OmissionMetadata,
        };

        let pack = CompactHandoff {
            checkpoint_id: uuid::Uuid::nil(),
            episode_id: uuid::Uuid::nil(),
            timestamp: chrono::Utc::now(),
            current_goal: "goal".to_string(),
            status: "in_progress".to_string(),
            steps_done: 3,
            steps_total: 5,
            verified_findings: vec!["finding".to_string()],
            decisions: vec!["decision".to_string()],
            pending_actions: vec!["action".to_string()],
            pattern_refs: vec!["pattern-1".to_string()],
            heuristic_refs: vec![],
            evidence_excerpts: vec![EvidenceExcerpt {
                step_number: 3,
                tool: "tool".to_string(),
                action: "did".to_string(),
                result_summary: None,
            }],
            omitted: OmissionMetadata {
                omitted_steps: 2,
                ..OmissionMetadata::default()
            },
            approx_tokens: 100,
        };

        let result = CompactHandoffResult::from(pack);

        assert_eq!(result.status, "in_progress");
        assert_eq!((result.steps_done, result.steps_total), (3, 5));
        assert_eq!(result.verified_findings, vec!["finding"]);
        assert_eq!(result.pattern_refs, vec!["pattern-1"]);
        assert_eq!(result.evidence_excerpts.len(), 1);
        assert_eq!(result.evidence_excerpts[0].step_number, 3);
        assert_eq!(result.omitted.omitted_steps, 2);
        assert!(result.payload_bytes > 0);
        assert_eq!(result.approx_tokens, 100);
    }
}
