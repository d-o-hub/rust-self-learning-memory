//! Base reward calculation based on task outcome

use crate::episode::Episode;
use crate::types::TaskOutcome;

/// Calculate base reward from episode outcome
///
/// Returns:
/// - 1.0 for complete success
/// - Proportional value (0.0-1.0) for partial success
/// - 0.0 for failure or incomplete episodes
pub fn calculate_base_reward(episode: &Episode) -> f32 {
    match &episode.outcome {
        Some(TaskOutcome::Success { .. }) => 1.0,
        Some(TaskOutcome::PartialSuccess {
            completed, failed, ..
        }) => {
            // Proportional reward based on completion ratio
            let total = completed.len() + failed.len();
            if total == 0 {
                0.5 // Default for partial success with no specifics
            } else {
                completed.len() as f32 / total as f32
            }
        }
        Some(TaskOutcome::Failure { .. }) => 0.0,
        None => 0.0, // Not completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityLevel, TaskContext, TaskType};

    fn create_test_episode() -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Simple,
            domain: "testing".to_string(),
            tags: vec![],
        };
        Episode::new("Test task".to_string(), context, TaskType::Testing)
    }

    #[test]
    fn test_base_reward_success() {
        let mut episode = create_test_episode();
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });
        assert_eq!(calculate_base_reward(&episode), 1.0);
    }

    #[test]
    fn test_base_reward_failure() {
        let mut episode = create_test_episode();
        episode.complete(TaskOutcome::Failure {
            reason: "Failed".to_string(),
            error_details: None,
        });
        assert_eq!(calculate_base_reward(&episode), 0.0);
    }

    #[test]
    fn test_base_reward_partial_success() {
        let mut episode = create_test_episode();
        episode.complete(TaskOutcome::PartialSuccess {
            verdict: "Partial".to_string(),
            completed: vec!["a".to_string(), "b".to_string()],
            failed: vec!["c".to_string()],
        });
        // 2 out of 3 = 0.667
        assert!((calculate_base_reward(&episode) - 0.667).abs() < 0.01);
    }

    #[test]
    fn test_base_reward_incomplete() {
        let episode = create_test_episode();
        assert_eq!(calculate_base_reward(&episode), 0.0);
    }
}
