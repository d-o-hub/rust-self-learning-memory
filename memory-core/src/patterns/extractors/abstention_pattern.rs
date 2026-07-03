//! Extractor for AbstentionPattern — episodes where an agent correctly
//! identified environment-infeasibility and halted before wasting further
//! tool calls. Inspired by CONVOLVE (arXiv:2606.28733).

use crate::episode::structs::Episode;
use crate::types::enums::TaskOutcome;

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
        reason: _,
        stopped_at_step,
        infeasibility_signals,
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
    use crate::episode::structs::ExecutionStep;
    use crate::types::enums::ExecutionResult;
    use crate::types::enums::TaskType;
    use crate::types::structs::TaskContext;

    #[test]
    fn test_extract_abstention_pattern() {
        let mut ep = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        let mut step = ExecutionStep::new(1, "search".to_string(), "find results".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "empty".to_string(),
        });
        ep.add_step(step);

        ep.complete(TaskOutcome::Abstained {
            reason: "No results found".to_string(),
            stopped_at_step: 1,
            infeasibility_signals: vec!["empty_result".to_string()],
        });

        let pattern = extract_abstention_pattern(&ep).unwrap();
        assert_eq!(pattern.stopped_at_step, 1);
        assert_eq!(
            pattern.infeasibility_signals,
            vec!["empty_result".to_string()]
        );
        assert_eq!(
            pattern.trajectory_prefix,
            vec!["search: find results".to_string()]
        );
    }

    #[test]
    fn test_extract_abstention_pattern_none() {
        let mut ep = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        ep.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        assert!(extract_abstention_pattern(&ep).is_none());
    }
}
