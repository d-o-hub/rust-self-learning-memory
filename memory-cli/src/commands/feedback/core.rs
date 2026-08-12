//! Feedback command implementations

use super::types::FeedbackCommands;
use crate::config::Config;
use crate::output::{Output, OutputFormat};
use anyhow::{Result, anyhow};
use do_memory_core::PersistenceReceipt;
use do_memory_core::SelfLearningMemory;
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::types::TaskOutcome;
use serde::Serialize;
use std::io::Write;
use uuid::Uuid;

/// Result for session recording
#[derive(Debug, Serialize)]
pub struct RecordSessionResult {
    pub success: bool,
    pub session_id: String,
    pub episode_id: String,
    pub patterns_recommended: usize,
    pub playbooks_recommended: usize,
    pub receipt: PersistenceReceipt,
}

impl Output for RecordSessionResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(
            writer,
            "Recorded recommendation session: {}",
            self.session_id
        )?;
        writeln!(writer, "  Episode: {}", self.episode_id)?;
        writeln!(
            writer,
            "  Patterns recommended: {}",
            self.patterns_recommended
        )?;
        writeln!(
            writer,
            "  Playbooks recommended: {}",
            self.playbooks_recommended
        )?;
        crate::commands::attribution_output::write_durability_line(&mut writer, &self.receipt)?;
        Ok(())
    }
}

/// Result for feedback recording
#[derive(Debug, Serialize)]
pub struct RecordFeedbackResult {
    pub success: bool,
    pub session_id: String,
    pub patterns_applied: usize,
    pub episodes_consulted: usize,
    pub receipt: PersistenceReceipt,
}

impl Output for RecordFeedbackResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(writer, "Recorded feedback for session: {}", self.session_id)?;
        writeln!(writer, "  Patterns applied: {}", self.patterns_applied)?;
        writeln!(writer, "  Episodes consulted: {}", self.episodes_consulted)?;
        crate::commands::attribution_output::write_durability_line(&mut writer, &self.receipt)?;
        Ok(())
    }
}

/// Result for stats
#[derive(Debug, Serialize)]
pub struct StatsResult {
    pub total_sessions: usize,
    pub total_feedback: usize,
    pub patterns_applied: usize,
    pub patterns_ignored: usize,
    pub adoption_rate: f32,
    pub success_after_adoption_rate: f32,
    pub avg_agent_rating: Option<f32>,
}

impl Output for StatsResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(writer, "Recommendation Statistics")?;
        writeln!(writer, "=========================")?;
        writeln!(writer, "  Total sessions: {}", self.total_sessions)?;
        writeln!(writer, "  Total feedback records: {}", self.total_feedback)?;
        writeln!(writer, "  Patterns applied: {}", self.patterns_applied)?;
        writeln!(writer, "  Patterns ignored: {}", self.patterns_ignored)?;
        writeln!(
            writer,
            "  Adoption rate: {:.1}%",
            self.adoption_rate * 100.0
        )?;
        writeln!(
            writer,
            "  Success after adoption: {:.1}%",
            self.success_after_adoption_rate * 100.0
        )?;
        if let Some(rating) = self.avg_agent_rating {
            writeln!(writer, "  Average agent rating: {:.2}", rating)?;
        }
        Ok(())
    }
}

/// Handle feedback subcommands
pub async fn handle_feedback_command(
    command: FeedbackCommands,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> Result<()> {
    match command {
        FeedbackCommands::RecordSession {
            episode_id,
            patterns,
            playbooks,
        } => record_session(episode_id, patterns, playbooks, memory, format, dry_run).await,
        FeedbackCommands::RecordFeedback {
            session,
            applied,
            consulted,
            outcome,
            message,
            rating,
        } => {
            record_feedback(
                session, applied, consulted, outcome, message, rating, memory, format, dry_run,
            )
            .await
        }
        FeedbackCommands::Stats => show_stats(memory, format, dry_run).await,
    }
}

/// Record a recommendation session
async fn record_session(
    episode_id: String,
    patterns: Vec<String>,
    playbooks: Vec<String>,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!(
            "Would record recommendation session with {} patterns and {} playbooks",
            patterns.len(),
            playbooks.len()
        );
        return Ok(());
    }

    // Parse episode ID
    let episode_uuid =
        Uuid::parse_str(&episode_id).map_err(|e| anyhow!("Invalid episode ID: {}", e))?;

    // Parse playbook IDs
    let playbook_uuids: Vec<Uuid> = playbooks
        .iter()
        .filter_map(|id| Uuid::parse_str(id).ok())
        .collect();

    // Create session
    let session_id = Uuid::new_v4();
    let session = RecommendationSession {
        session_id,
        episode_id: episode_uuid,
        timestamp: chrono::Utc::now(),
        recommended_pattern_ids: patterns.clone(),
        recommended_playbook_ids: playbook_uuids,
    };

    // Record session (checked: returns a truthful persistence receipt)
    let receipt = memory.record_recommendation_session_checked(session).await;

    let result = RecordSessionResult {
        success: receipt.is_durable(),
        session_id: session_id.to_string(),
        episode_id,
        patterns_recommended: patterns.len(),
        playbooks_recommended: playbooks.len(),
        receipt,
    };

    format.print_output(&result)
}

/// Record feedback about a recommendation session
#[expect(clippy::too_many_arguments)]
async fn record_feedback(
    session_id: String,
    applied_patterns: Vec<String>,
    consulted_episodes: Vec<String>,
    outcome_str: String,
    message: String,
    rating: Option<f32>,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!(
            "Would record feedback with {} applied patterns",
            applied_patterns.len()
        );
        return Ok(());
    }

    // Parse session ID
    let session_uuid =
        Uuid::parse_str(&session_id).map_err(|e| anyhow!("Invalid session ID: {}", e))?;

    // Parse consulted episode IDs
    let consulted_uuids: Vec<Uuid> = consulted_episodes
        .iter()
        .filter_map(|id| Uuid::parse_str(id).ok())
        .collect();

    // Parse outcome
    let outcome = match outcome_str.to_lowercase().as_str() {
        "success" => TaskOutcome::Success {
            verdict: message,
            artifacts: vec![],
        },
        "partial" => TaskOutcome::PartialSuccess {
            verdict: message,
            completed: vec![],
            failed: vec![],
        },
        "failure" => TaskOutcome::Failure {
            reason: message,
            error_details: None,
        },
        _ => {
            return Err(anyhow!(
                "Invalid outcome '{}'. Must be: success, partial, or failure",
                outcome_str
            ));
        }
    };

    // Create feedback
    let feedback = RecommendationFeedback {
        session_id: session_uuid,
        applied_pattern_ids: applied_patterns.clone(),
        consulted_episode_ids: consulted_uuids,
        outcome,
        agent_rating: rating,
    };

    // Record feedback (checked: preserves all rejection errors and returns a
    // truthful persistence receipt)
    let receipt = memory
        .record_recommendation_feedback_checked(feedback)
        .await?;

    let result = RecordFeedbackResult {
        success: receipt.is_durable(),
        session_id,
        patterns_applied: applied_patterns.len(),
        episodes_consulted: consulted_episodes.len(),
        receipt,
    };

    format.print_output(&result)
}

/// Show recommendation statistics
async fn show_stats(
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!("Would show recommendation statistics");
        return Ok(());
    }

    let stats = memory.get_recommendation_stats().await;

    let result = StatsResult {
        total_sessions: stats.total_sessions,
        total_feedback: stats.total_feedback,
        patterns_applied: stats.patterns_applied,
        patterns_ignored: stats.patterns_ignored,
        adoption_rate: stats.adoption_rate,
        success_after_adoption_rate: stats.success_after_adoption_rate,
        avg_agent_rating: stats.avg_agent_rating,
    };

    format.print_output(&result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_only_receipt() -> PersistenceReceipt {
        PersistenceReceipt::MemoryOnly {
            session_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
        }
    }

    fn persistence_failed_receipt() -> PersistenceReceipt {
        PersistenceReceipt::PersistenceFailed {
            session_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            failed_backends: vec!["turso".to_string(), "redb".to_string()],
        }
    }

    #[test]
    fn record_session_result_success_maps_to_receipt_durability() {
        for receipt in [memory_only_receipt(), persistence_failed_receipt()] {
            let result = RecordSessionResult {
                success: receipt.is_durable(),
                session_id: receipt.session_id().to_string(),
                episode_id: receipt.episode_id().to_string(),
                patterns_recommended: 2,
                playbooks_recommended: 1,
                receipt: receipt.clone(),
            };
            assert_eq!(result.success, result.receipt.is_durable());
            assert!(
                !result.success,
                "non-durable receipt must yield success=false"
            );
        }
    }

    #[test]
    fn record_feedback_result_success_maps_to_receipt_durability() {
        for receipt in [memory_only_receipt(), persistence_failed_receipt()] {
            let result = RecordFeedbackResult {
                success: receipt.is_durable(),
                session_id: receipt.session_id().to_string(),
                patterns_applied: 1,
                episodes_consulted: 0,
                receipt: receipt.clone(),
            };
            assert_eq!(result.success, result.receipt.is_durable());
            assert!(
                !result.success,
                "non-durable receipt must yield success=false"
            );
        }
    }

    #[test]
    fn record_session_human_output_renders_memory_only_durability_line() {
        let receipt = memory_only_receipt();
        let result = RecordSessionResult {
            success: receipt.is_durable(),
            session_id: receipt.session_id().to_string(),
            episode_id: receipt.episode_id().to_string(),
            patterns_recommended: 2,
            playbooks_recommended: 1,
            receipt: receipt.clone(),
        };
        let mut buf = Vec::new();
        result.write_human(&mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains(&format!(
            "Recorded recommendation session: {}",
            result.session_id
        )));
        assert!(
            out.contains("⚠️ Durability: Memory-only (process-local, will be lost on restart)")
        );
    }

    #[test]
    fn record_feedback_human_output_renders_persistence_failed_durability_line() {
        let receipt = persistence_failed_receipt();
        let result = RecordFeedbackResult {
            success: receipt.is_durable(),
            session_id: receipt.session_id().to_string(),
            patterns_applied: 1,
            episodes_consulted: 0,
            receipt: receipt.clone(),
        };
        let mut buf = Vec::new();
        result.write_human(&mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains(&format!(
            "Recorded feedback for session: {}",
            result.session_id
        )));
        assert!(out.contains("❌ Durability: Persistence Failed (failed backends: turso, redb)"));
    }

    #[test]
    fn record_session_result_serializes_receipt_field() {
        let receipt = memory_only_receipt();
        let result = RecordSessionResult {
            success: receipt.is_durable(),
            session_id: receipt.session_id().to_string(),
            episode_id: receipt.episode_id().to_string(),
            patterns_recommended: 0,
            playbooks_recommended: 0,
            receipt: receipt.clone(),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], serde_json::Value::Bool(false));
        assert_eq!(json["receipt"]["state"], "memory_only");
        assert_eq!(
            json["receipt"]["session_id"],
            serde_json::Value::String(receipt.session_id().to_string())
        );
        assert_eq!(
            json["receipt"]["episode_id"],
            serde_json::Value::String(receipt.episode_id().to_string())
        );
    }

    #[test]
    fn record_feedback_result_serializes_receipt_field() {
        let receipt = persistence_failed_receipt();
        let result = RecordFeedbackResult {
            success: receipt.is_durable(),
            session_id: receipt.session_id().to_string(),
            patterns_applied: 1,
            episodes_consulted: 0,
            receipt: receipt.clone(),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], serde_json::Value::Bool(false));
        assert_eq!(json["receipt"]["state"], "persistence_failed");
        let failed_backends = json["receipt"]["failed_backends"]
            .as_array()
            .map(|v| v.iter().filter_map(|b| b.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();
        assert_eq!(failed_backends, vec!["turso", "redb"]);
    }
}
