use super::*;
use crate::types::ComplexityLevel;

#[test]
fn test_episode_creation() {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec!["async".to_string()],
    };

    let episode = Episode::new(
        "Test task".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );

    assert!(!episode.is_complete());
    assert_eq!(episode.task_description, "Test task");
    assert_eq!(episode.context.domain, "web-api");
    assert_eq!(episode.steps.len(), 0);
}

#[test]
fn test_episode_completion() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

    assert!(!episode.is_complete());

    let outcome = TaskOutcome::Success {
        verdict: "All tests passed".to_string(),
        artifacts: vec![],
    };

    episode.complete(outcome);

    assert!(episode.is_complete());
    assert!(episode.end_time.is_some());
    assert!(episode.duration().is_some());
}

#[test]
fn test_execution_step() {
    let mut step = ExecutionStep::new(1, "read_file".to_string(), "Read source file".to_string());

    assert!(!step.is_success());

    step.result = Some(ExecutionResult::Success {
        output: "File contents".to_string(),
    });

    assert!(step.is_success());
}

#[test]
fn test_add_steps() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);
    }

    assert_eq!(episode.steps.len(), 3);
    assert_eq!(episode.successful_steps_count(), 3);
    assert_eq!(episode.failed_steps_count(), 0);
}

#[test]
fn test_add_tag() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Add valid tag
    assert!(episode.add_tag("bug-fix".to_string()).unwrap());
    assert!(episode.has_tag("bug-fix"));
    assert_eq!(episode.tags.len(), 1);

    // Add duplicate (normalized)
    assert!(!episode.add_tag("BUG-FIX".to_string()).unwrap());
    assert_eq!(episode.tags.len(), 1);

    // Add another tag
    assert!(episode.add_tag("feature".to_string()).unwrap());
    assert_eq!(episode.tags.len(), 2);
}

#[test]
fn test_tag_normalization() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Test case normalization
    episode.add_tag("Feature-123".to_string()).unwrap();
    assert!(episode.has_tag("feature-123"));
    assert!(episode.has_tag("FEATURE-123"));
    assert_eq!(episode.tags[0], "feature-123");

    // Test whitespace trimming
    episode.add_tag("  refactor  ".to_string()).unwrap();
    assert!(episode.has_tag("refactor"));
    assert_eq!(episode.tags[1], "refactor");
}

#[test]
fn test_tag_validation() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Empty tag
    assert!(episode.add_tag(String::new()).is_err());
    assert!(episode.add_tag("   ".to_string()).is_err());

    // Invalid characters
    assert!(episode.add_tag("bug fix".to_string()).is_err());
    assert!(episode.add_tag("bug@fix".to_string()).is_err());
    assert!(episode.add_tag("bug/fix".to_string()).is_err());

    // Valid characters
    assert!(episode.add_tag("bug-fix".to_string()).is_ok());
    assert!(episode.add_tag("bug_fix".to_string()).is_ok());
    assert!(episode.add_tag("bugfix123".to_string()).is_ok());
    assert_eq!(episode.tags.len(), 3);

    // Too long (>100 chars)
    let long_tag = "a".repeat(101);
    assert!(episode.add_tag(long_tag).is_err());
}

#[test]
fn test_remove_tag() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    episode.add_tag("bug-fix".to_string()).unwrap();
    episode.add_tag("feature".to_string()).unwrap();
    episode.add_tag("refactor".to_string()).unwrap();
    assert_eq!(episode.tags.len(), 3);

    // Remove existing tag
    assert!(episode.remove_tag("feature"));
    assert_eq!(episode.tags.len(), 2);
    assert!(!episode.has_tag("feature"));

    // Remove with different case
    assert!(episode.remove_tag("BUG-FIX"));
    assert_eq!(episode.tags.len(), 1);
    assert!(!episode.has_tag("bug-fix"));

    // Remove non-existent tag
    assert!(!episode.remove_tag("nonexistent"));
    assert_eq!(episode.tags.len(), 1);
}

#[test]
fn test_clear_tags() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    episode.add_tag("bug-fix".to_string()).unwrap();
    episode.add_tag("feature".to_string()).unwrap();
    assert_eq!(episode.tags.len(), 2);

    episode.clear_tags();
    assert_eq!(episode.tags.len(), 0);
    assert!(!episode.has_tag("bug-fix"));
}

#[test]
fn test_get_tags() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    episode.add_tag("bug-fix".to_string()).unwrap();
    episode.add_tag("critical".to_string()).unwrap();

    let tags = episode.get_tags();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"bug-fix".to_string()));
    assert!(tags.contains(&"critical".to_string()));
}

#[test]
fn test_tag_validation_error_messages() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Empty tag error message
    let result = episode.add_tag(String::new());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Tag cannot be empty");

    // Whitespace-only tag error message
    let result = episode.add_tag("   ".to_string());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Tag cannot be empty");

    // Invalid characters error message
    let result = episode.add_tag("bug fix".to_string());
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("invalid characters"));
    assert!(error_msg.contains("bug fix"));

    // Too long tag error message
    let long_tag = "a".repeat(101);
    let result = episode.add_tag(long_tag.clone());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Tag cannot exceed 100 characters");
}

#[test]
fn test_tag_minimum_length() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Single character should be invalid
    let result = episode.add_tag("a".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at least 2 characters"));

    // Two characters should be valid
    let result = episode.add_tag("ab".to_string());
    assert!(result.is_ok());
    assert!(episode.has_tag("ab"));
}

#[test]
fn test_tag_boundary_lengths() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Exactly 2 characters
    assert!(episode.add_tag("ab".to_string()).is_ok());
    assert_eq!(episode.tags[0], "ab");

    // Exactly 100 characters
    let tag_100 = "a".repeat(100);
    assert!(episode.add_tag(tag_100.clone()).is_ok());
    assert_eq!(episode.tags[1].len(), 100);

    // 101 characters should fail
    let tag_101 = "b".repeat(101);
    assert!(episode.add_tag(tag_101).is_err());
}

#[test]
fn test_add_tag_with_combined_normalization() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Test combined normalization: trim + lowercase
    let result = episode.add_tag("  FEATURE-123  ".to_string());
    assert!(result.is_ok());
    assert_eq!(episode.tags[0], "feature-123");

    // Verify case-insensitive duplicate detection
    let result = episode.add_tag("feature-123".to_string());
    assert!(result.is_ok());
    assert!(!result.unwrap(), "Should return false for duplicate");
    assert_eq!(episode.tags.len(), 1);

    let result = episode.add_tag("  FEATURE-123  ".to_string());
    assert!(result.is_ok());
    assert!(
        !result.unwrap(),
        "Should return false for duplicate with whitespace"
    );
    assert_eq!(episode.tags.len(), 1);
}

#[test]
fn test_tag_ordering() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Add tags in non-alphabetical order
    episode.add_tag("zebra".to_string()).unwrap();
    episode.add_tag("alpha".to_string()).unwrap();
    episode.add_tag("beta".to_string()).unwrap();

    // Tags should maintain insertion order (not sorted)
    assert_eq!(episode.tags, vec!["zebra", "alpha", "beta"]);
}

#[test]
fn test_remove_tag_with_invalid_input() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    episode.add_tag("bug-fix".to_string()).unwrap();
    episode.add_tag("feature".to_string()).unwrap();
    assert_eq!(episode.tags.len(), 2);

    // Try to remove invalid tag (should return false, not panic)
    assert!(!episode.remove_tag(""));
    assert!(!episode.remove_tag("   "));
    assert!(!episode.remove_tag("invalid tag"));
    assert!(!episode.remove_tag("tag@invalid"));
    assert_eq!(episode.tags.len(), 2);
}

#[test]
fn test_has_tag_with_invalid_input() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    episode.add_tag("bug-fix".to_string()).unwrap();

    // Invalid tags should return false, not panic
    assert!(!episode.has_tag(""));
    assert!(!episode.has_tag("   "));
    assert!(!episode.has_tag("invalid tag"));
    assert!(!episode.has_tag("tag@invalid"));
    assert!(!episode.has_tag("nonexistent"));
}

#[test]
fn test_get_tags_on_empty_episode() {
    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    let tags = episode.get_tags();
    assert!(tags.is_empty());
    assert_eq!(tags.len(), 0);
}

#[test]
fn test_has_tag_on_empty_episode() {
    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Should return false for any tag when episode has no tags
    assert!(!episode.has_tag("bug-fix"));
    assert!(!episode.has_tag("feature"));
    assert!(!episode.has_tag("anything"));
}

#[test]
fn test_multiple_tags_with_special_characters_valid() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Test all valid character combinations
    assert!(episode.add_tag("bug-fix".to_string()).is_ok());
    assert!(episode.add_tag("bug_fix".to_string()).is_ok());
    assert!(episode.add_tag("bug123".to_string()).is_ok());
    assert!(episode.add_tag("123bug".to_string()).is_ok());
    assert!(episode.add_tag("priority_high".to_string()).is_ok());
    assert!(episode.add_tag("test-123".to_string()).is_ok());
    assert!(episode.add_tag("A1B2_C3-D4".to_string()).is_ok());

    assert_eq!(episode.tags.len(), 7);
}

#[test]
fn test_tags_with_only_numbers() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Tags with only numbers should be valid
    assert!(episode.add_tag("123".to_string()).is_ok());
    assert!(episode.add_tag("456".to_string()).is_ok());
    assert!(episode.has_tag("123"));
    assert!(episode.has_tag("456"));
    assert_eq!(episode.tags.len(), 2);
}

#[test]
fn test_tag_serialization() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    episode.add_tag("bug-fix".to_string()).unwrap();
    episode.add_tag("feature".to_string()).unwrap();
    episode.add_tag("priority_high".to_string()).unwrap();

    // Serialize to JSON
    let json = serde_json::to_string(&episode).unwrap();
    assert!(json.contains("bug-fix"));
    assert!(json.contains("feature"));
    assert!(json.contains("priority_high"));

    // Deserialize from JSON
    let deserialized: Episode = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tags.len(), 3);
    assert!(deserialized.has_tag("bug-fix"));
    assert!(deserialized.has_tag("feature"));
    assert!(deserialized.has_tag("priority_high"));
}

#[test]
fn test_tag_operations_chain() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Add multiple tags
    episode.add_tag("tag1".to_string()).unwrap();
    episode.add_tag("tag2".to_string()).unwrap();
    episode.add_tag("tag3".to_string()).unwrap();
    assert_eq!(episode.tags.len(), 3);

    // Check presence
    assert!(episode.has_tag("tag1"));
    assert!(episode.has_tag("tag2"));
    assert!(episode.has_tag("tag3"));

    // Remove middle tag
    assert!(episode.remove_tag("tag2"));
    assert_eq!(episode.tags.len(), 2);
    assert!(!episode.has_tag("tag2"));

    // Add new tag
    episode.add_tag("tag4".to_string()).unwrap();
    assert_eq!(episode.tags.len(), 3);

    // Clear all
    episode.clear_tags();
    assert_eq!(episode.tags.len(), 0);
    assert!(!episode.has_tag("tag1"));
    assert!(!episode.has_tag("tag3"));
    assert!(!episode.has_tag("tag4"));
}

#[test]
fn test_tag_persistence_after_operations() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Add tags with various operations
    let mut added = episode.add_tag("persistent".to_string()).unwrap();
    assert!(added);

    added = episode.add_tag("PERSISTENT".to_string()).unwrap();
    assert!(!added, "Duplicate should return false");

    // Verify tag is still there after failed add
    assert!(episode.has_tag("persistent"));

    // Try to remove with wrong case (should still work due to normalization)
    let removed = episode.remove_tag("PERSISTENT");
    assert!(removed);
    assert!(!episode.has_tag("persistent"));
}

#[test]
fn test_tags_are_case_insensitive() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Add tag in uppercase
    episode.add_tag("BUG-FIX".to_string()).unwrap();

    // Should find it with various cases
    assert!(episode.has_tag("bug-fix"));
    assert!(episode.has_tag("BUG-FIX"));
    assert!(episode.has_tag("Bug-Fix"));
    assert!(episode.has_tag("  bug-fix  "));

    // Should be able to remove with different case
    assert!(episode.remove_tag("BuG-FiX"));
    assert!(!episode.has_tag("bug-fix"));
}
