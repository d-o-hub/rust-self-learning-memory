//! Error handling and validation tests for episode tagging

use memory_core::{SelfLearningMemory, TaskContext, TaskType};
use memory_mcp::mcp::tools::episode_tags::{
    AddEpisodeTagsInput, EpisodeTagTools, GetEpisodeTagsInput, RemoveEpisodeTagsInput,
    SearchEpisodesByTagsInput, SetEpisodeTagsInput,
};
use std::sync::Arc;
use uuid::Uuid;

/// Test invalid UUID formats
#[tokio::test]
async fn test_invalid_uuid_formats() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let invalid_uuids = vec![
        "not-a-uuid",
        "12345",
        "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
        "",
        "almost-valid-but-not-quite-a-uuid-format",
    ];

    for invalid_id in invalid_uuids {
        let result = tools
            .add_tags(AddEpisodeTagsInput {
                episode_id: invalid_id.to_string(),
                tags: vec!["test".to_string()],
            })
            .await;

        assert!(result.is_err(), "Should fail with invalid UUID format");
    }
}

/// Test operations on non-existent episode
#[tokio::test]
async fn test_nonexistent_episode() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Use a valid UUID that doesn't exist
    let nonexistent_id = Uuid::new_v4();

    // Try to add tags to non-existent episode
    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: nonexistent_id.to_string(),
            tags: vec!["test".to_string()],
        })
        .await;

    assert!(result.is_err(), "Should fail with non-existent episode");
}

/// Test empty tag list handling
#[tokio::test]
async fn test_empty_tag_list() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Empty tags test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add empty tag list
    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec![],
        })
        .await
        .unwrap();

    assert!(!result.success);
    assert_eq!(result.tags_added, 0);
    assert_eq!(result.message, "No tags provided");
}

/// Test invalid tag characters
#[tokio::test]
async fn test_invalid_tag_characters() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Invalid tags test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Tags with spaces should fail validation
    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["invalid tag with spaces".to_string()],
        })
        .await;

    assert!(result.is_err(), "Should fail with spaces in tag");

    // Tags with special characters should fail
    let result2 = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["invalid@tag".to_string()],
        })
        .await;

    assert!(result2.is_err(), "Should fail with @ in tag");
}

/// Test tag length validation
#[tokio::test]
async fn test_tag_length_validation() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Length test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Tag too short (1 character)
    let result_short = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["a".to_string()],
        })
        .await;

    assert!(result_short.is_err(), "Should fail with 1-char tag");

    // Tag too long (101 characters)
    let result_long = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["a".repeat(101)],
        })
        .await;

    assert!(result_long.is_err(), "Should fail with 101-char tag");

    // Tag at minimum length (2 characters) should work
    let result_min = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["ab".to_string()],
        })
        .await;

    assert!(result_min.is_ok(), "Should succeed with 2-char tag");

    // Tag at maximum length (100 characters) should work
    let result_max = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["a".repeat(100)],
        })
        .await;

    assert!(result_max.is_ok(), "Should succeed with 100-char tag");
}

/// Test search with empty tag list
#[tokio::test]
async fn test_search_empty_tags() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let result = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec![],
            require_all: Some(false),
            limit: Some(10),
        })
        .await
        .unwrap();

    assert!(!result.success);
    assert_eq!(result.count, 0);
    assert_eq!(result.message, "No tags provided for search");
}

/// Test get tags for non-existent episode
#[tokio::test]
async fn test_get_tags_nonexistent() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let nonexistent_id = Uuid::new_v4();

    let result = tools
        .get_tags(GetEpisodeTagsInput {
            episode_id: nonexistent_id.to_string(),
        })
        .await;

    assert!(result.is_err(), "Should fail for non-existent episode");
}

/// Test remove tags with empty list
#[tokio::test]
async fn test_remove_empty_tags() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Remove test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let result = tools
        .remove_tags(RemoveEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec![],
        })
        .await
        .unwrap();

    assert!(!result.success);
    assert_eq!(result.tags_removed, 0);
    assert_eq!(result.message, "No tags provided");
}

/// Test whitespace-only tags
#[tokio::test]
async fn test_whitespace_tags() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Whitespace test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Tag with only spaces should fail
    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["   ".to_string()],
        })
        .await;

    assert!(result.is_err(), "Should fail with whitespace-only tag");
}

/// Test tag with leading/trailing whitespace (should be trimmed)
#[tokio::test]
async fn test_tag_trimming() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Trim test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add tag with whitespace (should be trimmed and normalized)
    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["  test-tag  ".to_string()],
        })
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.current_tags[0], "test-tag"); // Trimmed and lowercase
}

/// Test set tags with invalid tags
#[tokio::test]
async fn test_set_invalid_tags() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Set invalid test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Try to set invalid tags
    let result = tools
        .set_tags(SetEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["invalid tag!".to_string()],
        })
        .await;

    assert!(
        result.is_err(),
        "Should fail with invalid tag in set operation"
    );
}

/// Test search with large limit
#[tokio::test]
async fn test_search_large_limit() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create a few episodes
    for i in 0..5 {
        let episode_id = memory
            .start_episode(
                format!("Episode {}", i),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory
            .add_episode_tags(episode_id, vec!["common".to_string()])
            .await
            .unwrap();
    }

    // Search with very large limit
    let result = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["common".to_string()],
            require_all: Some(false),
            limit: Some(10000), // Very large limit
        })
        .await
        .unwrap();

    // Should still return all results (5)
    assert_eq!(result.count, 5);
}

/// Test mixed valid and invalid tags
#[tokio::test]
async fn test_mixed_valid_invalid_tags() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Mixed tags test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Try to add mix of valid and invalid tags
    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["valid-tag".to_string(), "invalid tag".to_string()],
        })
        .await;

    // Should fail because one tag is invalid
    assert!(result.is_err(), "Should fail with any invalid tag in list");
}

/// Test case variations don't create duplicates
#[tokio::test]
async fn test_case_variations_no_duplicates() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Case test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add tag in lowercase
    tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["test-tag".to_string()],
        })
        .await
        .unwrap();

    // Try to add same tag in different cases
    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["Test-Tag".to_string(), "TEST-TAG".to_string()],
        })
        .await
        .unwrap();

    // Should add 0 tags (all duplicates)
    assert_eq!(result.tags_added, 0);
    assert_eq!(result.current_tags.len(), 1);
}
