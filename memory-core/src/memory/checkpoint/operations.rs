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
#[instrument(skip(memory), fields(episode_id = %episode_id))]
pub async fn checkpoint_episode(
    memory: &SelfLearningMemory,
    episode_id: Uuid,
    reason: String,
) -> Result<CheckpointMeta> {
    checkpoint_episode_with_note(memory, episode_id, reason, None).await
}

/// Create a checkpoint with an optional note.
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
    info!(checkpoint_id = %checkpoint.checkpoint_id, "Created checkpoint");
    Ok(checkpoint)
}

/// Generate a handoff pack from a checkpoint.
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

    Ok(HandoffPack {
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
    })
}

/// Resume work from a handoff pack.
#[instrument(skip(memory, handoff), fields(checkpoint_id = %handoff.checkpoint_id))]
pub async fn resume_from_handoff(
    memory: &SelfLearningMemory,
    handoff: HandoffPack,
) -> Result<Uuid> {
    info!(
        "Resuming from handoff pack: checkpoint_id={}",
        handoff.checkpoint_id
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
    info!(new_episode_id = %new_episode_id, "Created new episode for resumption");
    Ok(new_episode_id)
}

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
            let cp = checkpoint.clone();
            return Ok((episode, cp));
        }
    }
    Err(Error::NotFound(checkpoint_id))
}

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
                suggestions.push(format!("Consider: {} (then {})", condition, action));
            }
            _ => {}
        }
    }
    suggestions.truncate(5);
    suggestions
}

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

async fn get_relevant_heuristics(
    memory: &SelfLearningMemory,
    episode: &crate::episode::Episode,
) -> Vec<Heuristic> {
    let all_heuristics = memory.get_all_heuristics().await.unwrap_or_default();
    all_heuristics
        .into_iter()
        .filter(|h| {
            let cl = h.condition.to_lowercase();
            let tl = episode.task_description.to_lowercase();
            cl.split_whitespace().any(|word| tl.contains(word))
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
    async fn test_resume_from_handoff_context() {
        let memory = SelfLearningMemory::new();
        let handoff = HandoffPack {
            checkpoint_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            current_goal: "Test goal".to_string(),
            steps_completed: vec![],
            what_worked: vec!["worked".to_string()],
            what_failed: vec!["failed".to_string()],
            salient_facts: vec!["fact".to_string()],
            suggested_next_steps: vec!["next".to_string()],
            relevant_patterns: vec![],
            relevant_heuristics: vec![],
        };
        let new_id = resume_from_handoff(&memory, handoff).await.unwrap();
        let episode = memory.get_episode(new_id).await.unwrap();
        assert!(episode.metadata.contains_key("what_worked"));
        assert_eq!(episode.context.domain, "resumed");
    }

    #[tokio::test]
    async fn test_checkpoint_with_note() {
        let memory = SelfLearningMemory::new();
        let ep_id = memory
            .start_episode(
                "test".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        let cp = checkpoint_episode_with_note(
            &memory,
            ep_id,
            "reason".to_string(),
            Some("note".to_string()),
        )
        .await
        .unwrap();
        assert_eq!(cp.reason, "reason");
        assert_eq!(cp.note, Some("note".to_string()));
    }
}
