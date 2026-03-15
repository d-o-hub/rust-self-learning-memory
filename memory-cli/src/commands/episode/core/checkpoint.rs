//! Episode checkpoint CLI commands (ADR-044 Feature 3)

use crate::config::Config;
use crate::output::OutputFormat;
use anyhow::{Result, anyhow};
use memory_core::SelfLearningMemory;
use memory_core::memory::checkpoint::{
    checkpoint_episode, checkpoint_episode_with_note, get_handoff_pack, resume_from_handoff,
};
use serde::Serialize;
use uuid::Uuid;

/// Create a checkpoint for an episode
pub async fn checkpoint(
    episode_id: String,
    reason: String,
    note: Option<String>,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    _dry_run: bool,
) -> Result<()> {
    // Parse episode ID
    let episode_uuid =
        Uuid::parse_str(&episode_id).map_err(|e| anyhow!("Invalid episode ID: {}", e))?;

    // Create checkpoint
    let checkpoint = if let Some(n) = note {
        checkpoint_episode_with_note(memory, episode_uuid, reason.clone(), Some(n)).await
    } else {
        checkpoint_episode(memory, episode_uuid, reason.clone()).await
    }
    .map_err(|e| anyhow!("Failed to create checkpoint: {}", e))?;

    let result = CheckpointResult {
        checkpoint_id: checkpoint.checkpoint_id.to_string(),
        episode_id: episode_id.clone(),
        reason,
        step_number: checkpoint.step_number,
        created_at: checkpoint.created_at.to_rfc3339(),
    };

    result.write(format)?;
    Ok(())
}

/// Get a handoff pack from a checkpoint
pub async fn handoff(
    checkpoint_id: String,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    _dry_run: bool,
) -> Result<()> {
    // Parse checkpoint ID
    let checkpoint_uuid =
        Uuid::parse_str(&checkpoint_id).map_err(|e| anyhow!("Invalid checkpoint ID: {}", e))?;

    // Get handoff pack
    let handoff = get_handoff_pack(memory, checkpoint_uuid)
        .await
        .map_err(|e| anyhow!("Failed to get handoff pack: {}", e))?;

    let result = HandoffResult {
        checkpoint_id: handoff.checkpoint_id.to_string(),
        episode_id: handoff.episode_id.to_string(),
        current_goal: handoff.current_goal,
        timestamp: handoff.timestamp.to_rfc3339(),
        steps_completed_count: handoff.steps_completed.len(),
        what_worked: handoff.what_worked,
        what_failed: handoff.what_failed,
        salient_facts: handoff.salient_facts,
        suggested_next_steps: handoff.suggested_next_steps,
        pattern_count: handoff.relevant_patterns.len(),
        heuristic_count: handoff.relevant_heuristics.len(),
    };

    result.write(format)?;
    Ok(())
}

/// Resume work from a handoff pack
pub async fn resume(
    checkpoint_id: String,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    _dry_run: bool,
) -> Result<()> {
    // Parse checkpoint ID
    let checkpoint_uuid =
        Uuid::parse_str(&checkpoint_id).map_err(|e| anyhow!("Invalid checkpoint ID: {}", e))?;

    // Get handoff pack first
    let handoff = get_handoff_pack(memory, checkpoint_uuid)
        .await
        .map_err(|e| anyhow!("Failed to get handoff pack: {}", e))?;

    // Resume from handoff
    let new_episode_id = resume_from_handoff(memory, handoff)
        .await
        .map_err(|e| anyhow!("Failed to resume from handoff: {}", e))?;

    let result = ResumeResult {
        new_episode_id: new_episode_id.to_string(),
        checkpoint_id: checkpoint_id.clone(),
    };

    result.write(format)?;
    Ok(())
}

/// Result of checkpoint creation
#[derive(Debug, Serialize)]
pub struct CheckpointResult {
    /// Checkpoint ID
    pub checkpoint_id: String,
    /// Episode ID
    pub episode_id: String,
    /// Reason for checkpoint
    pub reason: String,
    /// Step number at checkpoint
    pub step_number: usize,
    /// When checkpoint was created
    pub created_at: String,
}

/// Result of handoff pack retrieval
#[derive(Debug, Serialize)]
pub struct HandoffResult {
    /// Checkpoint ID
    pub checkpoint_id: String,
    /// Episode ID
    pub episode_id: String,
    /// Current goal
    pub current_goal: String,
    /// Timestamp
    pub timestamp: String,
    /// Number of steps completed
    pub steps_completed_count: usize,
    /// What worked
    pub what_worked: Vec<String>,
    /// What failed
    pub what_failed: Vec<String>,
    /// Salient facts
    pub salient_facts: Vec<String>,
    /// Suggested next steps
    pub suggested_next_steps: Vec<String>,
    /// Pattern count
    pub pattern_count: usize,
    /// Heuristic count
    pub heuristic_count: usize,
}

/// Result of resume operation
#[derive(Debug, Serialize)]
pub struct ResumeResult {
    /// New episode ID created
    pub new_episode_id: String,
    /// Original checkpoint ID
    pub checkpoint_id: String,
}

trait Output: Serialize {
    fn write(&self, format: OutputFormat) -> Result<()>;
}

impl Output for CheckpointResult {
    fn write(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(self)?);
            }
            OutputFormat::Human => {
                println!("Checkpoint created successfully!");
                println!("  Checkpoint ID: {}", self.checkpoint_id);
                println!("  Episode ID:    {}", self.episode_id);
                println!("  Reason:        {}", self.reason);
                println!("  Step Number:   {}", self.step_number);
                println!("  Created At:    {}", self.created_at);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(self)?);
            }
        }
        Ok(())
    }
}

impl Output for HandoffResult {
    fn write(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(self)?);
            }
            OutputFormat::Human => {
                println!("Handoff Pack for Checkpoint: {}", self.checkpoint_id);
                println!("  Episode ID:      {}", self.episode_id);
                println!("  Goal:            {}", self.current_goal);
                println!("  Steps Completed: {}", self.steps_completed_count);
                println!("  Patterns:        {}", self.pattern_count);
                println!("  Heuristics:      {}", self.heuristic_count);
                println!();

                if !self.what_worked.is_empty() {
                    println!("What Worked:");
                    for item in &self.what_worked {
                        println!("  + {}", item);
                    }
                    println!();
                }

                if !self.what_failed.is_empty() {
                    println!("What Failed:");
                    for item in &self.what_failed {
                        println!("  - {}", item);
                    }
                    println!();
                }

                if !self.suggested_next_steps.is_empty() {
                    println!("Suggested Next Steps:");
                    for (i, step) in self.suggested_next_steps.iter().enumerate() {
                        println!("  {}. {}", i + 1, step);
                    }
                    println!();
                }

                if !self.salient_facts.is_empty() {
                    println!("Salient Facts:");
                    for fact in &self.salient_facts {
                        println!("  * {}", fact);
                    }
                }
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(self)?);
            }
        }
        Ok(())
    }
}

impl Output for ResumeResult {
    fn write(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(self)?);
            }
            OutputFormat::Human => {
                println!("Resumed work successfully!");
                println!("  New Episode ID:    {}", self.new_episode_id);
                println!("  From Checkpoint:   {}", self.checkpoint_id);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(self)?);
            }
        }
        Ok(())
    }
}
