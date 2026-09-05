//! Compact handoff packs with byte budgets (issue #965).
//!
//! Full [`HandoffPack`](super::types::HandoffPack) payloads embed every
//! completed step, which inflates storage, retrieval payloads, and MCP
//! context usage. This module builds the default compact profile instead:
//!
//! - task objective, status, and progress counters
//! - verified findings, decisions, and pending actions (all count-capped)
//! - bounded evidence excerpts (most recent steps, addressable by
//!   `(episode_id, step_number)`)
//! - ID-only pattern/heuristic references (bodies stay fetchable by ID)
//! - explicit [`OmissionMetadata`] so receivers know what was left out and
//!   where the full-fidelity pack lives
//!
//! ## Budget contract
//!
//! 1. Per-section count caps apply first (omissions recorded exactly).
//! 2. The serialized JSON payload is then forced under
//!    [`HandoffBudget::max_bytes`] by dropping whole items in reverse
//!    priority (heuristic refs, pattern refs, oldest excerpts, pending
//!    actions, decisions, findings) and, as a last resort, truncating the
//!    goal. Every cut is recorded; string cuts are UTF-8 safe.
//! 3. Assembly is deterministic: identical inputs yield identical bytes
//!    (no map iteration, stable section order).
//!
//! Token counts are estimated as `bytes / 4` and documented as a heuristic,
//! not a tokenizer guarantee.
//!
//! ## Budget floor
//!
//! The empty skeleton serializes to ~590 bytes and truncation receipts grow
//! it (worst case ~950 bytes with a full receipt list), so budgets below
//! ~1 KB are best-effort: the assembler drops everything droppable, records
//! exact omissions, and returns the skeleton even if it exceeds the budget.
//! Callers that need a hard guarantee should enforce a 1024-byte minimum
//! (see `MIN_HANDOFF_BYTES` on the MCP input).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::episode::ExecutionStep;
use crate::error::Result;
use crate::memory::SelfLearningMemory;
use crate::types::ExecutionResult;
use tracing::{info, instrument};

/// Default payload ceiling in bytes (also ≈2k estimated tokens).
pub const DEFAULT_HANDOFF_MAX_BYTES: usize = 8192;
/// Default cap for verified findings.
pub const DEFAULT_MAX_FINDINGS: usize = 10;
/// Default cap for decisions.
pub const DEFAULT_MAX_DECISIONS: usize = 10;
/// Default cap for pending actions.
pub const DEFAULT_MAX_PENDING_ACTIONS: usize = 10;
/// Default cap for evidence excerpts.
pub const DEFAULT_MAX_EVIDENCE_EXCERPTS: usize = 5;
/// Default cap for pattern references.
pub const DEFAULT_MAX_PATTERN_REFS: usize = 5;
/// Default cap for heuristic references.
pub const DEFAULT_MAX_HEURISTIC_REFS: usize = 5;
/// Default cap for goal characters before the byte budget applies.
pub const DEFAULT_MAX_GOAL_CHARS: usize = 500;
/// Default cap for excerpt action/result characters.
pub const DEFAULT_MAX_EXCERPT_CHARS: usize = 300;

/// Size and count budget for a compact handoff pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffBudget {
    /// Serialized JSON ceiling in bytes.
    pub max_bytes: usize,
    /// Maximum verified findings (worked, then failed, then facts).
    pub max_findings: usize,
    /// Maximum decisions.
    pub max_decisions: usize,
    /// Maximum pending actions.
    pub max_pending_actions: usize,
    /// Maximum evidence excerpts (most recent steps win).
    pub max_evidence_excerpts: usize,
    /// Maximum pattern ID references.
    pub max_pattern_refs: usize,
    /// Maximum heuristic ID references.
    pub max_heuristic_refs: usize,
    /// Maximum goal characters (before byte-budget pressure).
    pub max_goal_chars: usize,
    /// Maximum characters per excerpt action/result summary.
    pub max_excerpt_chars: usize,
}

impl Default for HandoffBudget {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_HANDOFF_MAX_BYTES,
            max_findings: DEFAULT_MAX_FINDINGS,
            max_decisions: DEFAULT_MAX_DECISIONS,
            max_pending_actions: DEFAULT_MAX_PENDING_ACTIONS,
            max_evidence_excerpts: DEFAULT_MAX_EVIDENCE_EXCERPTS,
            max_pattern_refs: DEFAULT_MAX_PATTERN_REFS,
            max_heuristic_refs: DEFAULT_MAX_HEURISTIC_REFS,
            max_goal_chars: DEFAULT_MAX_GOAL_CHARS,
            max_excerpt_chars: DEFAULT_MAX_EXCERPT_CHARS,
        }
    }
}

/// One bounded step excerpt. Addressable via `(episode_id, step_number)`
/// on the owning [`CompactHandoff`]; full step bodies stay in the episode
/// and in the full-fidelity pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceExcerpt {
    /// 1-indexed step number within the episode.
    pub step_number: usize,
    /// Tool used.
    pub tool: String,
    /// Action description (truncated to budget).
    pub action: String,
    /// Bounded result summary, if the step produced one.
    pub result_summary: Option<String>,
}

/// Exact account of what the budget left out of a compact pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmissionMetadata {
    /// Completed steps without an excerpt.
    pub omitted_steps: usize,
    /// Findings dropped by the count cap or byte budget.
    pub omitted_findings: usize,
    /// Decisions dropped.
    pub omitted_decisions: usize,
    /// Pending actions dropped.
    pub omitted_pending_actions: usize,
    /// Pattern references dropped.
    pub omitted_patterns: usize,
    /// Heuristic references dropped.
    pub omitted_heuristics: usize,
    /// Fields cut mid-string (e.g. `"current_goal"`, `"evidence[2].action"`).
    pub truncated_fields: Vec<String>,
    /// Pointer to the full-fidelity pack (`get_handoff_pack`).
    pub full_available_via: String,
}

impl Default for OmissionMetadata {
    fn default() -> Self {
        Self {
            omitted_steps: 0,
            omitted_findings: 0,
            omitted_decisions: 0,
            omitted_pending_actions: 0,
            omitted_patterns: 0,
            omitted_heuristics: 0,
            truncated_fields: Vec::new(),
            full_available_via: "get_handoff_pack(checkpoint_id)".to_string(),
        }
    }
}

/// Default compact handoff profile: bounded context with omission receipts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompactHandoff {
    /// Checkpoint this pack was derived from.
    pub checkpoint_id: Uuid,
    /// Source episode.
    pub episode_id: Uuid,
    /// When the pack was built.
    pub timestamp: DateTime<Utc>,
    /// Task objective (byte-budgeted).
    pub current_goal: String,
    /// Episode phase at build time (`in_progress` or `completed`).
    pub status: String,
    /// Steps completed at the checkpoint.
    pub steps_done: usize,
    /// Total steps in the episode.
    pub steps_total: usize,
    /// Established findings, worked first.
    pub verified_findings: Vec<String>,
    /// Critical decisions with rationale.
    pub decisions: Vec<String>,
    /// Actions still pending.
    pub pending_actions: Vec<String>,
    /// Pattern IDs (fetch bodies by ID; full pack embeds them).
    pub pattern_refs: Vec<String>,
    /// Heuristic IDs (fetch bodies by ID).
    pub heuristic_refs: Vec<String>,
    /// Most recent step excerpts, chronological.
    pub evidence_excerpts: Vec<EvidenceExcerpt>,
    /// Exact omission account.
    pub omitted: OmissionMetadata,
    /// Heuristic token estimate (`payload_bytes / 4`).
    pub approx_tokens: usize,
}

impl CompactHandoff {
    /// Serialized JSON payload size in bytes (the budgeted unit).
    #[must_use]
    pub fn payload_bytes(&self) -> usize {
        serde_json::to_string(self)
            .map(|s| s.len())
            .unwrap_or(usize::MAX)
    }
}

/// Owned inputs for [`assemble_compact`]; keeps the assembler pure and
/// deterministic (unit-testable without a memory system).
#[derive(Debug, Clone)]
pub struct CompactInputs {
    /// Checkpoint this pack derives from.
    pub checkpoint_id: Uuid,
    /// Source episode.
    pub episode_id: Uuid,
    /// Build timestamp.
    pub timestamp: DateTime<Utc>,
    /// Task objective.
    pub current_goal: String,
    /// Episode phase (`in_progress` or `completed`).
    pub status: String,
    /// Steps completed at the checkpoint.
    pub steps_done: usize,
    /// Total steps in the episode.
    pub steps_total: usize,
    /// Completed steps up to the checkpoint, chronological.
    pub steps: Vec<ExecutionStep>,
    /// Observed successes.
    pub worked: Vec<String>,
    /// Observed failures.
    pub failed: Vec<String>,
    /// Extractor salient facts.
    pub salient_facts: Vec<String>,
    /// Critical decisions.
    pub decisions: Vec<String>,
    /// Suggested next steps.
    pub pending_actions: Vec<String>,
    /// Pattern IDs.
    pub pattern_ids: Vec<String>,
    /// Heuristic IDs.
    pub heuristic_ids: Vec<String>,
}

/// Truncate to a character boundary, reporting whether a cut happened.
fn truncate_chars(text: &str, max_chars: usize) -> (String, bool) {
    if text.chars().count() <= max_chars {
        return (text.to_string(), false);
    }
    let cut: String = text.chars().take(max_chars).collect();
    (cut, true)
}

/// Bounded one-line summary of a step result, plus whether it was cut.
fn summarize_result(result: &Option<ExecutionResult>, max_chars: usize) -> (Option<String>, bool) {
    let text = match result {
        None => return (None, false),
        Some(ExecutionResult::Success { output }) => output.clone(),
        Some(ExecutionResult::Error { message }) => format!("error: {message}"),
        Some(ExecutionResult::Timeout) => "timeout".to_string(),
    };
    let (cut, was_cut) = truncate_chars(&text, max_chars);
    (Some(cut), was_cut)
}

/// Assemble a budget-compliant compact pack. See the module docs for the
/// exact contract (count caps, then byte budget, deterministic order).
#[must_use]
pub fn assemble_compact(inputs: CompactInputs, budget: &HandoffBudget) -> CompactHandoff {
    let mut truncated_fields: Vec<String> = Vec::new();

    let (current_goal, goal_cut) = truncate_chars(&inputs.current_goal, budget.max_goal_chars);
    if goal_cut {
        truncated_fields.push("current_goal".to_string());
    }

    // Findings priority: worked, then failed, then extractor facts.
    let mut findings: Vec<String> = Vec::new();
    findings.extend(inputs.worked);
    findings.extend(inputs.failed);
    findings.extend(inputs.salient_facts);
    let omitted_findings = findings.len().saturating_sub(budget.max_findings);
    findings.truncate(budget.max_findings);

    let mut decisions = inputs.decisions;
    let omitted_decisions = decisions.len().saturating_sub(budget.max_decisions);
    decisions.truncate(budget.max_decisions);

    let mut pending_actions = inputs.pending_actions;
    let omitted_pending = pending_actions
        .len()
        .saturating_sub(budget.max_pending_actions);
    pending_actions.truncate(budget.max_pending_actions);

    let mut pattern_refs = inputs.pattern_ids;
    let omitted_patterns = pattern_refs.len().saturating_sub(budget.max_pattern_refs);
    pattern_refs.truncate(budget.max_pattern_refs);

    let mut heuristic_refs = inputs.heuristic_ids;
    let omitted_heuristics = heuristic_refs
        .len()
        .saturating_sub(budget.max_heuristic_refs);
    heuristic_refs.truncate(budget.max_heuristic_refs);

    // Most recent steps win; excerpts stay chronological.
    let omitted_steps = inputs
        .steps
        .len()
        .saturating_sub(budget.max_evidence_excerpts);
    let kept_from = inputs
        .steps
        .len()
        .saturating_sub(budget.max_evidence_excerpts);
    let mut evidence_excerpts: Vec<EvidenceExcerpt> = Vec::new();
    for (position, step) in inputs.steps[kept_from..].iter().enumerate() {
        let (action, action_cut) = truncate_chars(&step.action, budget.max_excerpt_chars);
        if action_cut {
            truncated_fields.push(format!("evidence[{position}].action"));
        }
        let (summary, summary_cut) = summarize_result(&step.result, budget.max_excerpt_chars);
        if summary_cut {
            truncated_fields.push(format!("evidence[{position}].result_summary"));
        }
        evidence_excerpts.push(EvidenceExcerpt {
            step_number: step.step_number,
            tool: step.tool.clone(),
            action,
            result_summary: summary,
        });
    }

    let mut pack = CompactHandoff {
        checkpoint_id: inputs.checkpoint_id,
        episode_id: inputs.episode_id,
        timestamp: inputs.timestamp,
        current_goal,
        status: inputs.status,
        steps_done: inputs.steps_done,
        steps_total: inputs.steps_total,
        verified_findings: findings,
        decisions,
        pending_actions,
        pattern_refs,
        heuristic_refs,
        evidence_excerpts,
        omitted: OmissionMetadata {
            omitted_steps,
            omitted_findings,
            omitted_decisions,
            omitted_pending_actions: omitted_pending,
            omitted_patterns,
            omitted_heuristics,
            truncated_fields,
            ..OmissionMetadata::default()
        },
        approx_tokens: 0,
    };

    enforce_byte_budget(&mut pack, budget);
    let bytes = pack.payload_bytes();
    pack.approx_tokens = bytes / 4;
    pack
}

/// Drop whole items in reverse priority until the JSON fits `max_bytes`.
/// Counts flow back into the omission metadata; the goal truncates last.
/// Best-effort when even the skeleton exceeds the budget (documented).
fn enforce_byte_budget(pack: &mut CompactHandoff, budget: &HandoffBudget) {
    loop {
        if pack.payload_bytes() <= budget.max_bytes {
            return;
        }
        if pack.heuristic_refs.pop().is_some() {
            pack.omitted.omitted_heuristics += 1;
        } else if pack.pattern_refs.pop().is_some() {
            pack.omitted.omitted_patterns += 1;
        } else if !pack.evidence_excerpts.is_empty() {
            // Excerpts are chronological; the oldest goes first so the most
            // recent context (highest resume value) survives the longest.
            pack.evidence_excerpts.remove(0);
            pack.omitted.omitted_steps += 1;
        } else if pack.pending_actions.pop().is_some() {
            pack.omitted.omitted_pending_actions += 1;
        } else if pack.decisions.pop().is_some() {
            pack.omitted.omitted_decisions += 1;
        } else if pack.verified_findings.pop().is_some() {
            pack.omitted.omitted_findings += 1;
        } else if !pack.current_goal.is_empty() {
            // Last resort: halve the goal (UTF-8 safe) until it fits.
            let half = pack.current_goal.chars().count() / 2;
            let (cut, _) = truncate_chars(&pack.current_goal, half);
            pack.current_goal = cut;
            if !pack
                .omitted
                .truncated_fields
                .contains(&"current_goal".to_string())
            {
                pack.omitted
                    .truncated_fields
                    .push("current_goal".to_string());
            }
            if half == 0 {
                return;
            }
        } else {
            return;
        }
    }
}

#[cfg(test)]
#[path = "compact_tests.rs"]
mod tests;

/// Resume work from a compact handoff pack.
///
/// Creates a new episode carrying the compact goal, findings, decisions,
/// and pending actions as metadata, plus pointers back to the source
/// checkpoint for full-fidelity fetch.
#[instrument(skip(memory, handoff), fields(checkpoint_id = %handoff.checkpoint_id))]
pub async fn resume_from_compact(
    memory: &SelfLearningMemory,
    handoff: CompactHandoff,
) -> Result<Uuid> {
    info!(
        "Resuming from compact handoff: checkpoint_id={}, findings={}, pending={}",
        handoff.checkpoint_id,
        handoff.verified_findings.len(),
        handoff.pending_actions.len()
    );

    let context = crate::types::TaskContext {
        domain: "resumed".to_string(),
        language: None,
        framework: None,
        complexity: crate::types::ComplexityLevel::Moderate,
        tags: vec![
            "resumed".to_string(),
            "compact".to_string(),
            format!("from-{}", handoff.episode_id),
        ],
    };

    let new_episode_id = memory
        .start_episode(
            handoff.current_goal.clone(),
            context,
            crate::types::TaskType::Other,
        )
        .await;

    let mut episode = memory.get_episode(new_episode_id).await?;
    episode.metadata.insert(
        "resumed_from_checkpoint".to_string(),
        handoff.checkpoint_id.to_string(),
    );
    episode.metadata.insert(
        "resumed_from_episode".to_string(),
        handoff.episode_id.to_string(),
    );
    episode
        .metadata
        .insert("handoff_format".to_string(), "compact".to_string());
    episode.metadata.insert(
        "compact_findings".to_string(),
        serde_json::to_string(&handoff.verified_findings).unwrap_or_default(),
    );
    episode.metadata.insert(
        "compact_decisions".to_string(),
        serde_json::to_string(&handoff.decisions).unwrap_or_default(),
    );
    episode.metadata.insert(
        "compact_pending_actions".to_string(),
        serde_json::to_string(&handoff.pending_actions).unwrap_or_default(),
    );
    episode.metadata.insert(
        "compact_omitted_steps".to_string(),
        handoff.omitted.omitted_steps.to_string(),
    );
    memory.update_episode_full(&episode).await?;

    info!(new_episode_id = %new_episode_id, "Created new episode for compact resumption");

    Ok(new_episode_id)
}
