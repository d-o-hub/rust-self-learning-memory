//! Extractor for AbstentionPattern — episodes where an agent correctly
//! identified environment-infeasibility and halted before wasting further
//! tool calls. Inspired by CONVOLVE (arXiv:2606.28733).

use crate::episode::Episode;
use crate::types::TaskOutcome;

/// A captured stopping rule: the conditions under which the agent abstained.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AbstentionPattern {
    /// Episode from which this pattern was extracted
    pub episode_id: uuid::Uuid,
    /// Step at which the agent decided to stop
    pub stopped_at_step: usize,
    /// The environment signals that triggered abstention
    pub infeasibility_signals: Vec<String>,
    /// Tool calls that preceded the stop (the "trajectory prefix")
    pub trajectory_prefix: Vec<String>,
    /// Human-readable stopping rule summary
    pub stopping_rule: String,
}

/// Extract an `AbstentionPattern` from a completed episode, if applicable.
///
/// Returns `None` if the episode did not end in `TaskOutcome::Abstained`.
pub fn extract_abstention_pattern(episode: &Episode) -> Option<AbstentionPattern> {
    if let Some(TaskOutcome::Abstained {
        stopped_at_step,
        infeasibility_signals,
        ..
    }) = &episode.outcome
    {
        let trajectory_prefix = episode
            .steps
            .iter()
            .take(*stopped_at_step)
            .map(|s| format!("{}: {}", s.tool, s.action))
            .collect();

        let stopping_rule = format!(
            "Stop after step {} when signals match: {}",
            stopped_at_step,
            infeasibility_signals.join(", ")
        );

        Some(AbstentionPattern {
            episode_id: episode.episode_id,
            stopped_at_step: *stopped_at_step,
            infeasibility_signals: infeasibility_signals.clone(),
            trajectory_prefix,
            stopping_rule,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
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
    fn test_extract_abstention_pattern() {
        let mut ep = create_test_episode();

        // Add some steps
        ep.add_step(ExecutionStep::new(1, "search".to_string(), "query A".to_string()));
        ep.add_step(ExecutionStep::new(2, "search".to_string(), "query B".to_string()));

        ep.outcome = Some(TaskOutcome::Abstained {
            reason: "Empty search results after 2 attempts".to_string(),
            stopped_at_step: 2,
            infeasibility_signals: vec!["empty_result".to_string()],
        });

        let pattern = extract_abstention_pattern(&ep).expect("Should extract pattern");
        assert_eq!(pattern.stopped_at_step, 2);
        assert_eq!(pattern.infeasibility_signals, vec!["empty_result".to_string()]);
        assert_eq!(pattern.trajectory_prefix.len(), 2);
        assert_eq!(pattern.trajectory_prefix[0], "search: query A");
        assert!(pattern.stopping_rule.contains("Stop after step 2"));
    }

    #[test]
    fn test_extract_abstention_pattern_none() {
        let mut ep = create_test_episode();
        ep.outcome = Some(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let pattern = extract_abstention_pattern(&ep);
        assert!(pattern.is_none());
    }
}
