//! Checkpoint and handoff operations.
//!
//! Core functions for creating checkpoints, generating handoff packs, and resuming work.

use crate::episode::ExecutionStep;
use crate::error::{Error, Result};
use crate::memory::SelfLearningMemory;
use crate::memory::pattern_search::PatternSearchResult;
use crate::pattern::Heuristic;
use chrono::Utc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use super::{CheckpointMeta, HandoffPack};

/// Create a checkpoint for an in-progress episode.
///
/// Saves the current state of an episode for handoffs or recovery.
/// Returns an error if the episode doesn't exist, is already completed, or storage fails.
#[instrument(skip(memory), fields(episode_id = %episode_id))]
pub async fn checkpoint_episode(
    memory: &SelfLearningMemory,
    episode_id: Uuid,
    reason: String,
) -> Result<CheckpointMeta> {
    checkpoint_episode_with_note(memory, episode_id, reason, None).await
}

/// Create a checkpoint with an optional note.
///
/// Same as [`checkpoint_episode`] but allows specifying a note.
#[instrument(skip(memory), fields(episode_id = %episode_id))]
pub async fn checkpoint_episode_with_note(
    memory: &SelfLearningMemory,
    episode_id: Uuid,
    reason: String,
    note: Option<String>,
) -> Result<CheckpointMeta> {
    info!("Creating checkpoint for episode: {}", episode_id);

    let mut episode = memory.get_episode(episode_id).await?;

    if episode.is_complete() {
        warn!(
            "Cannot create checkpoint for completed episode: {}",
            episode_id
        );
        return Err(Error::InvalidState(
            "Cannot create checkpoint for completed episode".to_string(),
        ));
    }

    let step_number = episode.steps.len();
    let checkpoint = CheckpointMeta::new(reason, step_number, note);
    episode.checkpoints.push(checkpoint.clone());
    memory.update_episode_full(&episode).await?;

    info!(
        checkpoint_id = %checkpoint.checkpoint_id,
        step_number = checkpoint.step_number,
        "Created checkpoint"
    );

    Ok(checkpoint)
}

/// Generate a handoff pack from a checkpoint.
///
/// Creates a comprehensive context package for transferring work between agents,
/// including current goal, lessons learned, relevant patterns, and suggested next steps.
/// Returns an error if the checkpoint or episode doesn't exist.
#[instrument(skip(memory), fields(checkpoint_id = %checkpoint_id))]
pub async fn get_handoff_pack(
    memory: &SelfLearningMemory,
    checkpoint_id: Uuid,
) -> Result<HandoffPack> {
    info!("Generating handoff pack for checkpoint: {}", checkpoint_id);

    let (episode, checkpoint) = find_checkpoint(memory, checkpoint_id).await?;

    let steps_completed: Vec<ExecutionStep> = episode
        .steps
        .iter()
        .take(checkpoint.step_number)
        .cloned()
        .collect();

    let (what_worked, what_failed, salient_facts) =
        extract_lessons(memory, &episode, checkpoint.step_number);

    let suggested_next_steps = generate_suggested_next_steps(memory, &episode).await;
    let relevant_patterns = get_relevant_patterns(memory, &episode).await;
    let relevant_heuristics = get_relevant_heuristics(memory, &episode).await;

    let handoff = HandoffPack {
        checkpoint_id: checkpoint.checkpoint_id,
        episode_id: episode.episode_id,
        timestamp: Utc::now(),
        current_goal: episode.task_description.clone(),
        steps_completed,
        what_worked,
        what_failed,
        salient_facts,
        suggested_next_steps,
        relevant_patterns,
        relevant_heuristics,
    };

    info!(
        step_count = handoff.step_count(),
        pattern_count = handoff.relevant_patterns.len(),
        heuristic_count = handoff.relevant_heuristics.len(),
        "Generated handoff pack"
    );

    Ok(handoff)
}

/// Resume work from a handoff pack.
///
/// Creates a new episode initialized with context from a handoff pack,
/// including the same goal, lessons learned, and suggested next steps.
#[instrument(skip(memory, handoff), fields(checkpoint_id = %handoff.checkpoint_id))]
pub async fn resume_from_handoff(
    memory: &SelfLearningMemory,
    handoff: HandoffPack,
) -> Result<Uuid> {
    info!(
        "Resuming from handoff pack: checkpoint_id={}, steps={}",
        handoff.checkpoint_id,
        handoff.step_count()
    );

    let context = crate::types::TaskContext {
        domain: "resumed".to_string(),
        language: None,
        framework: None,
        complexity: crate::types::ComplexityLevel::Moderate,
        tags: vec![
            "resumed".to_string(),
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

    // Store handoff context in episode metadata using normal update path
    let mut episode = memory.get_episode(new_episode_id).await?;
    episode.metadata.insert(
        "resumed_from_checkpoint".to_string(),
        handoff.checkpoint_id.to_string(),
    );
    episode.metadata.insert(
        "resumed_from_episode".to_string(),
        handoff.episode_id.to_string(),
    );
    episode.metadata.insert(
        "what_worked".to_string(),
        serde_json::to_string(&handoff.what_worked).unwrap_or_default(),
    );
    episode.metadata.insert(
        "what_failed".to_string(),
        serde_json::to_string(&handoff.what_failed).unwrap_or_default(),
    );
    episode.metadata.insert(
        "salient_facts".to_string(),
        serde_json::to_string(&handoff.salient_facts).unwrap_or_default(),
    );
    episode.metadata.insert(
        "suggested_next_steps".to_string(),
        serde_json::to_string(&handoff.suggested_next_steps).unwrap_or_default(),
    );
    memory.update_episode_full(&episode).await?;

    info!(new_episode_id = %new_episode_id, "Created new episode for resumption");

    Ok(new_episode_id)
}

/// Find an episode and checkpoint by checkpoint ID.
async fn find_checkpoint(
    memory: &SelfLearningMemory,
    checkpoint_id: Uuid,
) -> Result<(crate::episode::Episode, CheckpointMeta)> {
    let episodes = memory.get_all_episodes().await?;

    for episode in episodes {
        if let Some(checkpoint) = episode
            .checkpoints
            .iter()
            .find(|c| c.checkpoint_id == checkpoint_id)
        {
            let checkpoint = checkpoint.clone();
            return Ok((episode, checkpoint));
        }
    }

    Err(Error::NotFound(checkpoint_id))
}

/// Extract lessons learned from an episode up to a step.
fn extract_lessons(
    memory: &SelfLearningMemory,
    episode: &crate::episode::Episode,
    step_number: usize,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut what_worked = Vec::new();
    let mut what_failed = Vec::new();
    let mut salient_facts = Vec::new();

    let mut partial_episode = episode.clone();
    partial_episode.steps.truncate(step_number);

    let features = memory.salient_extractor.extract(&partial_episode);

    for step in &partial_episode.steps {
        if step.is_success() {
            what_worked.push(format!("{}: {}", step.tool, step.action));
        } else {
            what_failed.push(format!("{}: {}", step.tool, step.action));
        }
    }

    for decision in &features.critical_decisions {
        salient_facts.push(format!("Decision: {decision}"));
    }
    for insight in &features.key_insights {
        salient_facts.push(format!("Insight: {insight}"));
    }
    for recovery in &features.error_recovery_patterns {
        salient_facts.push(format!("Recovery: {recovery}"));
    }

    (what_worked, what_failed, salient_facts)
}

/// Generate suggested next steps based on episode context.
async fn generate_suggested_next_steps(
    memory: &SelfLearningMemory,
    episode: &crate::episode::Episode,
) -> Vec<String> {
    let mut suggestions = Vec::new();

    let patterns = memory.retrieve_relevant_patterns(&episode.context, 3).await;

    for pattern in patterns {
        match &pattern {
            crate::pattern::Pattern::ToolSequence { tools, .. } => {
                let remaining_tools: Vec<_> = tools
                    .iter()
                    .filter(|t| !episode.steps.iter().any(|s| &s.tool == *t))
                    .collect();
                if !remaining_tools.is_empty() {
                    suggestions.push(format!(
                        "Consider using tools: {}",
                        remaining_tools
                            .iter()
                            .map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
            crate::pattern::Pattern::DecisionPoint {
                condition, action, ..
            } => {
                suggestions.push(format!("Consider: {condition} (then {action})"));
            }
            _ => {}
        }
    }

    if episode.steps.len() > 5 && episode.successful_steps_count() < episode.steps.len() / 2 {
        suggestions.push("Consider breaking down the task into smaller steps".to_string());
        suggestions.push("Review the what_failed list to avoid repeating mistakes".to_string());
    }

    suggestions.truncate(5);
    suggestions
}

/// Get relevant patterns for an episode.
async fn get_relevant_patterns(
    memory: &SelfLearningMemory,
    episode: &crate::episode::Episode,
) -> Vec<PatternSearchResult> {
    let config = crate::memory::pattern_search::SearchConfig::default();
    memory
        .search_patterns(&episode.task_description, &episode.context, config)
        .await
        .unwrap_or_default()
        .into_iter()
        .take(5)
        .collect()
}

/// Get relevant heuristics for an episode.
async fn get_relevant_heuristics(
    memory: &SelfLearningMemory,
    episode: &crate::episode::Episode,
) -> Vec<Heuristic> {
    let all_heuristics = memory.get_all_heuristics().await.unwrap_or_default();

    all_heuristics
        .into_iter()
        .filter(|h| {
            let condition_lower = h.condition.to_lowercase();
            let task_lower = episode.task_description.to_lowercase();
            condition_lower
                .split_whitespace()
                .any(|word| task_lower.contains(word))
        })
        .take(5)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskContext, TaskType};

    #[tokio::test]
    async fn test_checkpoint_episode_not_found() {
        let memory = SelfLearningMemory::new();
        let result = checkpoint_episode(&memory, Uuid::new_v4(), "test".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_handoff_pack_not_found() {
        let memory = SelfLearningMemory::new();
        let result = get_handoff_pack(&memory, Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_checkpoint_completed_episode() {
        use crate::episode::ExecutionStep;
        use crate::memory::MemoryConfig;
        use crate::types::ExecutionResult;

        let test_config = MemoryConfig {
            quality_threshold: 0.3,
            ..Default::default()
        };
        let memory = SelfLearningMemory::with_config(test_config);

        let episode_id = memory
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        let mut step = ExecutionStep::new(1, "test_tool".to_string(), "test action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "test output".to_string(),
        });
        memory.log_step(episode_id, step).await;

        memory
            .complete_episode(
                episode_id,
                crate::types::TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

        let result = checkpoint_episode(&memory, episode_id, "test".to_string()).await;
        assert!(result.is_err());
    }
}
