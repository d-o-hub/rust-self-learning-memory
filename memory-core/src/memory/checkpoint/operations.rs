//! Checkpoint and handoff operations.
//!
//! Core functions for creating checkpoints, generating handoff packs, and resuming work.

use crate::episode::ExecutionStep;
use crate::error::{Error, Result};
use crate::memory::SelfLearningMemory;
use crate::memory::pattern_search::PatternSearchResult;
use crate::pattern::Heuristic;
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use super::{CheckpointMeta, HandoffPack};

/// Create a checkpoint for an in-progress episode.
///
/// Saves the current state of an episode as a checkpoint that can be used
/// for handoffs or recovery. The checkpoint is stored in the episode's
/// checkpoint list.
///
/// # Arguments
///
/// * `memory` - The memory system instance
/// * `episode_id` - ID of the episode to checkpoint
/// * `reason` - Why the checkpoint is being created
/// * `note` - Optional additional context about the checkpoint
///
/// # Returns
///
/// The created checkpoint metadata on success.
///
/// # Errors
///
/// Returns an error if:
/// - The episode doesn't exist
/// - The episode is already completed
/// - Storage operations fail
///
/// # Example
///
/// ```no_run
/// use memory_core::memory::checkpoint::checkpoint_episode;
/// use memory_core::SelfLearningMemory;
/// use uuid::Uuid;
///
/// # async fn example(memory: SelfLearningMemory) -> anyhow::Result<()> {
/// let episode_id = Uuid::new_v4();
/// let checkpoint = checkpoint_episode(&memory, episode_id, "Agent switch".to_string()).await?;
/// println!("Created checkpoint: {}", checkpoint.checkpoint_id);
/// # Ok(())
/// # }
/// ```
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

    // Get the episode
    let mut episode = memory.get_episode(episode_id).await?;

    // Check if episode is already completed
    if episode.is_complete() {
        warn!(
            "Cannot create checkpoint for completed episode: {}",
            episode_id
        );
        return Err(Error::InvalidState(
            "Cannot create checkpoint for completed episode".to_string(),
        ));
    }

    // Create checkpoint metadata
    let step_number = episode.steps.len();
    let checkpoint = CheckpointMeta::new(reason, step_number, note);

    // Add checkpoint to episode
    episode.checkpoints.push(checkpoint.clone());

    // Update episode in storage
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
/// Creates a comprehensive context package for transferring work between agents.
/// The handoff pack includes:
/// - Current goal and progress
/// - Lessons learned (what worked, what failed)
/// - Salient facts discovered
/// - Relevant patterns and heuristics
/// - Suggested next steps
///
/// # Arguments
///
/// * `memory` - The memory system instance
/// * `checkpoint_id` - ID of the checkpoint to generate handoff from
///
/// # Returns
///
/// The generated handoff pack on success.
///
/// # Errors
///
/// Returns an error if:
/// - The checkpoint doesn't exist
/// - The episode doesn't exist
/// - Retrieval operations fail
///
/// # Example
///
/// ```no_run
/// use memory_core::memory::checkpoint::{checkpoint_episode, get_handoff_pack};
/// use memory_core::SelfLearningMemory;
/// use uuid::Uuid;
///
/// # async fn example(memory: SelfLearningMemory) -> anyhow::Result<()> {
/// let episode_id = Uuid::new_v4();
/// let checkpoint = checkpoint_episode(&memory, episode_id, "Agent switch".to_string()).await?;
/// let handoff = get_handoff_pack(&memory, checkpoint.checkpoint_id).await?;
/// println!("Handoff has {} steps", handoff.step_count());
/// # Ok(())
/// # }
/// ```
#[instrument(skip(memory), fields(checkpoint_id = %checkpoint_id))]
pub async fn get_handoff_pack(
    memory: &SelfLearningMemory,
    checkpoint_id: Uuid,
) -> Result<HandoffPack> {
    info!("Generating handoff pack for checkpoint: {}", checkpoint_id);

    // Find the episode containing this checkpoint
    let (episode, checkpoint) = find_checkpoint(memory, checkpoint_id).await?;

    // Get steps up to checkpoint
    let steps_completed: Vec<ExecutionStep> = episode
        .steps
        .iter()
        .take(checkpoint.step_number)
        .cloned()
        .collect();

    // Extract lessons learned using salient extractor
    let (what_worked, what_failed, salient_facts) =
        extract_lessons(memory, &episode, checkpoint.step_number);

    // Generate suggested next steps based on patterns
    let suggested_next_steps = generate_suggested_next_steps(memory, &episode).await;

    // Get relevant patterns
    let relevant_patterns = get_relevant_patterns(memory, &episode).await;

    // Get relevant heuristics
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
/// Creates a new episode initialized with context from a handoff pack.
/// The new episode will have:
/// - The same goal/task description
/// - Access to the lessons learned
/// - Context from suggested next steps
///
/// # Arguments
///
/// * `memory` - The memory system instance
/// * `handoff` - The handoff pack to resume from
///
/// # Returns
///
/// The ID of the new episode created for resumption.
///
/// # Example
///
/// ```no_run
/// use memory_core::memory::checkpoint::{checkpoint_episode, get_handoff_pack, resume_from_handoff};
/// use memory_core::SelfLearningMemory;
/// use uuid::Uuid;
///
/// # async fn example(memory: SelfLearningMemory) -> anyhow::Result<()> {
/// let episode_id = Uuid::new_v4();
/// let checkpoint = checkpoint_episode(&memory, episode_id, "Agent switch".to_string()).await?;
/// let handoff = get_handoff_pack(&memory, checkpoint.checkpoint_id).await?;
///
/// // Resume in a new episode
/// let new_episode_id = resume_from_handoff(&memory, handoff).await?;
/// println!("Resumed in episode: {}", new_episode_id);
/// # Ok(())
/// # }
/// ```
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

    // Create a new episode for resumption
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

    // Store handoff context in episode metadata
    {
        let mut episodes = memory.episodes_fallback.write().await;
        if let Some(episode_arc) = episodes.get(&new_episode_id) {
            let mut episode = (**episode_arc).clone();
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
            episodes.insert(new_episode_id, Arc::new(episode));
        }
    }

    info!(
        new_episode_id = %new_episode_id,
        "Created new episode for resumption"
    );

    Ok(new_episode_id)
}

/// Find an episode and checkpoint by checkpoint ID.
async fn find_checkpoint(
    memory: &SelfLearningMemory,
    checkpoint_id: Uuid,
) -> Result<(crate::episode::Episode, CheckpointMeta)> {
    // Get all episodes and search for the checkpoint
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

    // Create a partial episode for extraction
    let mut partial_episode = episode.clone();
    partial_episode.steps.truncate(step_number);

    // Use salient extractor if available
    let features = memory.salient_extractor.extract(&partial_episode);

    // Extract what worked from successful steps
    for step in &partial_episode.steps {
        if step.is_success() {
            what_worked.push(format!("{}: {}", step.tool, step.action));
        } else {
            what_failed.push(format!("{}: {}", step.tool, step.action));
        }
    }

    // Convert salient features to facts
    for decision in &features.critical_decisions {
        salient_facts.push(format!("Decision: {}", decision));
    }
    for insight in &features.key_insights {
        salient_facts.push(format!("Insight: {}", insight));
    }
    for recovery in &features.error_recovery_patterns {
        salient_facts.push(format!("Recovery: {}", recovery));
    }

    (what_worked, what_failed, salient_facts)
}

/// Generate suggested next steps based on episode context.
async fn generate_suggested_next_steps(
    memory: &SelfLearningMemory,
    episode: &crate::episode::Episode,
) -> Vec<String> {
    let mut suggestions = Vec::new();

    // Check if there are patterns available
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
                suggestions.push(format!("Consider: {} (then {})", condition, action));
            }
            _ => {}
        }
    }

    // Add generic suggestions if episode is stuck
    if episode.steps.len() > 5 && episode.successful_steps_count() < episode.steps.len() / 2 {
        suggestions.push("Consider breaking down the task into smaller steps".to_string());
        suggestions.push("Review the what_failed list to avoid repeating mistakes".to_string());
    }

    suggestions.truncate(5); // Limit to 5 suggestions
    suggestions
}

/// Get relevant patterns for an episode.
async fn get_relevant_patterns(
    memory: &SelfLearningMemory,
    episode: &crate::episode::Episode,
) -> Vec<PatternSearchResult> {
    // Use pattern search to find relevant patterns
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
    // Get heuristics that match the episode context
    let all_heuristics = memory.get_all_heuristics().await.unwrap_or_default();

    all_heuristics
        .into_iter()
        .filter(|h| {
            // Simple relevance check: does the condition relate to the task?
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
        let memory = SelfLearningMemory::new();

        // Create and complete an episode
        let episode_id = memory
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

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

        // Try to checkpoint completed episode
        let result = checkpoint_episode(&memory, episode_id, "test".to_string()).await;
        assert!(result.is_err());
    }
}
