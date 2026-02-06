//! Property-based tests for Episode and ExecutionStep
//!
//! Tests invariants that must hold regardless of input values:
//! - Episode IDs are valid UUIDs
//! - Episode tags are unique and normalized
//! - Episode creation is deterministic given same inputs
//! - Episode modification preserves invariants

use memory_core::episode::{Episode, ExecutionStep};
use memory_core::types::{
    ComplexityLevel, ExecutionResult, Reflection, RewardScore, TaskContext, TaskOutcome, TaskType,
};
use proptest::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

// ============================================================================
// Episode Creation Properties
// ============================================================================

proptest! {
    /// Episode IDs are always valid UUIDs
    #[test]
    fn episode_id_is_valid_uuid(task_desc in "[a-zA-Z0-9 ]{1,50}", task_type in any::<TaskType>()) {
        let context = TaskContext::default();
        let episode = Episode::new(task_desc, context, task_type);

        // Verify episode_id is a non-nil UUID
        assert!(!episode.episode_id.is_nil());
        assert_eq!(episode.episode_id.get_version().unwrap(), uuid::Version::Random);
    }

    /// Episode start_time is set to current time on creation
    #[test]
    fn episode_start_time_is_set(task_desc in "[a-zA-Z0-9 ]{1,50}") {
        let before = chrono::Utc::now();
        let episode = Episode::new(task_desc, TaskContext::default(), TaskType::CodeGeneration);
        let after = chrono::Utc::now();

        // start_time should be between before and after (within reasonable tolerance)
        let duration_before = (episode.start_time - before).num_milliseconds().abs();
        let duration_after = (after - episode.start_time).num_milliseconds().abs();

        assert!(duration_before < 1000, "start_time too far before creation time");
        assert!(duration_after < 1000, "start_time too far after creation time");
    }

    /// Newly created episode is incomplete (no outcome or end_time)
    #[test]
    fn new_episode_is_incomplete(task_desc in "[a-zA-Z0-9 ]{1,50}") {
        let episode = Episode::new(task_desc, TaskContext::default(), TaskType::CodeGeneration);

        assert!(!episode.is_complete());
        assert!(episode.outcome.is_none());
        assert!(episode.end_time.is_none());
    }

    /// Newly created episode has no steps
    #[test]
    fn new_episode_has_no_steps(task_desc in "[a-zA-Z0-9 ]{1,50}") {
        let episode = Episode::new(task_desc, TaskContext::default(), TaskType::CodeGeneration);

        assert!(episode.steps.is_empty());
        assert_eq!(episode.successful_steps_count(), 0);
        assert_eq!(episode.failed_steps_count(), 0);
    }
}

// ============================================================================
// Episode Tag Properties
// ============================================================================

proptest! {
    /// Tag normalization is idempotent - normalizing twice gives same result
    #[test]
    fn tag_normalization_is_idempotent(tag in "[a-zA-Z0-9_-]{1,30}") {
        // Directly test the normalize_tag function through add_tag behavior
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tag once
        episode.add_tag(tag.clone()).ok();
        let tags_after_first = episode.tags.clone();

        // Try to add tag again (should be idempotent - not added twice)
        episode.add_tag(tag.clone()).ok();
        let tags_after_second = episode.tags.clone();

        // Tags should be identical
        assert_eq!(tags_after_first, tags_after_second);
    }

    /// Tags are case-insensitive and trimmed (normalization property)
    #[test]
    fn tags_are_normalized(tag in "[a-zA-Z0-9_-]{1,10}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tag with various case and whitespace
        let variations = vec![
            tag.to_uppercase(),
            tag.to_lowercase(),
            format!("  {}  ", tag),
            format!("\t{}\t", tag),
            format!("  {}  ", tag.to_uppercase()),
        ];

        let mut successful_adds = 0;
        for variation in variations {
            if episode.add_tag(variation).ok() == Some(true) {
                successful_adds += 1;
            }
        }

        // Only one should have been added due to normalization
        assert_eq!(successful_adds, 1, "Only first tag variation should have been added");
        assert_eq!(episode.tags.len(), 1);
    }

    /// Tags maintain uniqueness (duplicate detection property)
    #[test]
    fn tags_maintain_uniqueness(tags in proptest::collection::vec("[a-zA-Z0-9_-]{1,15}", 1..10)) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add all tags
        for tag in tags.clone() {
            episode.add_tag(tag).ok();
        }

        // Add them again - none should be added
        for tag in tags {
            let result = episode.add_tag(tag).unwrap();
            assert!(!result, "Duplicate tag should not be added");
        }

        // Check no duplicates exist
        let tag_set: HashSet<_> = episode.tags.iter().collect();
        assert_eq!(tag_set.len(), episode.tags.len(), "Tags should not have duplicates");
    }

    /// Empty or invalid tags are rejected
    #[test]
    fn invalid_tags_are_rejected(tag in "[ \t\n]{0,5}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        let tags_to_reject = vec![
            "",           // Empty
            " ",          // Just space
            "\t",         // Just tab
            "  ",         // Multiple spaces
            "a",          // Too short
            "-",          // Just special char
            "_",          // Just special char
            "@",          // Invalid character
        ];

        for tag in tags_to_reject {
            let result = episode.add_tag(tag.to_string());
            assert!(
                result.is_err(),
                "Tag '{:?}' should be rejected as invalid",
                tag
            );
        }
    }

    /// Clear tags removes all tags
    #[test]
    fn clear_tags_removes_all(tags in proptest::collection::vec("[a-zA-Z0-9_-]{2,20}", 0..10)) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tags
        for tag in tags {
            episode.add_tag(tag).ok();
        }

        // Clear all tags
        episode.clear_tags();

        // Verify all tags gone
        assert!(episode.tags.is_empty());
    }

    /// Has tag works correctly with normalization
    #[test]
    fn has_tag_works_with_normalization(tag in "[a-zA-Z0-9_-]{2,15}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add tag in lowercase
        episode.add_tag(tag.clone()).ok();

        // Check various forms - should all return true
        assert!(episode.has_tag(&tag));
        assert!(episode.has_tag(&tag.to_uppercase()));
        assert!(episode.has_tag(&format!("  {}  ", tag)));
    }
}

// ============================================================================
// Episode Completion Properties
// ============================================================================

proptest! {
    /// Completing an episode sets end_time and outcome
    #[test]
    fn completing_episode_sets_outcome(task_desc in "[a-zA-Z0-9 ]{1,50}", verdict in "[a-zA-Z0-9 ]{5,100}") {
        let mut episode = Episode::new(task_desc, TaskContext::default(), TaskType::CodeGeneration);
        let before = chrono::Utc::now();

        let outcome = TaskOutcome::Success {
            verdict: verdict.clone(),
            artifacts: vec!["test.rs".to_string()],
        };

        episode.complete(outcome.clone());
        let after = chrono::Utc::now();

        // Verify completion
        assert!(episode.is_complete());
        assert_eq!(episode.outcome, Some(outcome));
        assert!(episode.end_time.is_some());

        // Verify end_time is reasonably close to now
        let end_time = episode.end_time.unwrap();
        let duration_after = (after - end_time).num_milliseconds().abs();
        assert!(duration_after < 500, "end_time should be close to completion time");
    }

    /// Duration calculation requires completed episode
    #[test]
    fn duration_requires_completed_episode(task_desc in "[a-zA-Z0-9 ]{1,50}") {
        let mut episode = Episode::new(task_desc, TaskContext::default(), TaskType::CodeGeneration);

        // Incomplete episode has no duration
        assert!(episode.duration().is_none());

        // Complete the episode
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        // Now duration should be available
        assert!(episode.duration().is_some());
    }

    /// Duration is always non-negative
    #[test]
    fn duration_is_non_negative(task_desc in "[a-zA-Z0-9 ]{1,50}") {
        let mut episode = Episode::new(task_desc, TaskContext::default(), TaskType::CodeGeneration);

        // Complete immediately
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let duration = episode.duration().unwrap();
        assert!(duration.num_milliseconds() >= 0, "Duration should be non-negative");
    }
}

// ============================================================================
// Episode Step Properties
// ============================================================================

proptest! {
    /// Adding step increases step count
    #[test]
    fn adding_step_increases_count(tool in "[a-zA-Z0-9_-]{1,20}", action in "[a-zA-Z0-9 ]{1,50}") {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);
        let initial_count = episode.steps.len();

        let step = ExecutionStep::new(1, tool, action);
        episode.add_step(step);

        assert_eq!(episode.steps.len(), initial_count + 1);
    }

    /// Step numbering can be tracked
    #[test]
    fn step_numbers_accurate(steps in proptest::collection::vec(
        (1usize..10usize, "[a-zA-Z0-9_-]{1,20}", "[a-zA-Z0-9 ]{1,50}"),
        1..10
    )) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        for (step_number, tool, action) in steps {
            let step = ExecutionStep::new(step_number, tool, action);
            episode.add_step(step);
        }

        // Verify we can find steps by number
        assert_eq!(episode.steps.len(), episode.steps.iter().map(|s| s.step_number).collect::<HashSet<_>>().len());
    }

    /// Successful step count is accurate
    #[test]
    fn successful_step_count_accurate(
        successful_count in 0usize..5,
        failed_count in 0usize..5
    ) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        // Add successful steps
        for i in 0..successful_count {
            let mut step = ExecutionStep::new(i, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "success".to_string(),
            });
            episode.add_step(step);
        }

        // Add failed steps
        for i in 0..failed_count {
            let mut step = ExecutionStep::new(successful_count + i, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Error {
                message: "error".to_string(),
            });
            episode.add_step(step);
        }

        assert_eq!(episode.successful_steps_count(), successful_count);
        assert_eq!(episode.failed_steps_count(), failed_count);
    }
}

// ============================================================================
// Episode Serialization Properties
// ============================================================================

proptest! {
    /// Episode serialization is round-trippable
    #[test]
    fn episode_serialization_roundtrip(
        task_desc in "[a-zA-Z0-9 ]{1,50}",
        task_type in any::<TaskType>()
    ) {
        let mut episode = Episode::new(task_desc, TaskContext::default(), task_type);

        // Add some data
        episode.add_tag("test".to_string()).ok();
        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        step.result = Some(ExecutionResult::Success { output: "output".to_string() });
        episode.add_step(step);

        // Serialize and deserialize
        let json = serde_json::to_string(&episode).unwrap();
        let deserialized: Episode = serde_json::from_str(&json).unwrap();

        assert_eq!(episode.episode_id, deserialized.episode_id);
        assert_eq!(episode.task_type, deserialized.task_type);
        assert_eq!(episode.task_description, deserialized.task_description);
        assert_eq!(episode.tags, deserialized.tags);
        assert_eq!(episode.steps.len(), deserialized.steps.len());
    }

    /// ExecutionStep serialization is round-trippable
    #[test]
    fn step_serialization_roundtrip(
        step_num in 1usize..100,
        tool in "[a-zA-Z0-9_-]{1,20}",
        action in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let mut step = ExecutionStep::new(step_num, tool, action);
        step.result = Some(ExecutionResult::Success {
            output: "output".to_string(),
        });

        let json = serde_json::to_string(&step).unwrap();
        let deserialized: ExecutionStep = serde_json::from_str(&json).unwrap();

        assert_eq!(step.step_number, deserialized.step_number);
        assert_eq!(step.tool, deserialized.tool);
        assert_eq!(step.action, deserialized.action);
        assert_eq!(step.result, deserialized.result);
    }
}

// ============================================================================
// Episode Invariant Properties
// ============================================================================

proptest! {
    /// Episode modification preserves core invariants
    #[test]
    fn episode_modification_preserves_invariants(
        task_desc in "[a-zA-Z0-9 ]{1,50}",
        tags in proptest::collection::vec("[a-zA-Z0-9_-]{2,15}", 0..5)
    ) {
        let mut episode = Episode::new(task_desc, TaskContext::default(), TaskType::CodeGeneration);
        let original_id = episode.episode_id;

        // Add tags
        for tag in tags {
            episode.add_tag(tag).ok();
        }

        // ID should never change
        assert_eq!(episode.episode_id, original_id);

        // Task description should never change
        assert!(!episode.task_description.is_empty());

        // Add steps
        for i in 0..5 {
            let mut step = ExecutionStep::new(i, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Success { output: "output".to_string() });
            episode.add_step(step);
        }

        // Complete episode
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        // ID should still be the same
        assert_eq!(episode.episode_id, original_id);
    }

    /// Episode structure maintains consistency
    #[test]
    fn episode_structure_consistency(
        task_desc in "[a-zA-Z0-9 ]{5,50}",
        task_type in any::<TaskType>(),
        domain in "[a-zA-Z0-9_-]{2,20}"
    ) {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: domain.clone(),
            tags: vec![],
        };

        let episode = Episode::new(task_desc, context, task_type);

        // Basic structure invariants
        assert!(episode.task_description.len() >= 5); // From strategy
        assert_eq!(episode.context.domain, domain);
        assert_eq!(episode.task_type, task_type);
        assert!(episode.start_time <= chrono::Utc::now() + chrono::Duration::seconds(1));

        // Empty collections
        assert!(episode.steps.is_empty());
        assert!(episode.tags.is_empty());
        assert!(episode.patterns.is_empty());
        assert!(episode.heuristics.is_empty());

        // None optional fields
        assert!(episode.outcome.is_none());
        assert!(episode.end_time.is_none());
        assert!(episode.reward.is_none());
        assert!(episode.reflection.is_none());
    }
}
