//! Edge case and stress tests for episode tagging

use memory_core::{SelfLearningMemory, TaskContext, TaskType};
use memory_mcp::mcp::tools::episode_tags::{
    AddEpisodeTagsInput, EpisodeTagTools, RemoveEpisodeTagsInput, SearchEpisodesByTagsInput,
    SetEpisodeTagsInput,
};
use std::sync::Arc;

/// Test with many tags on a single episode
#[tokio::test]
async fn test_many_tags_single_episode() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test with many tags".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add 50 tags
    let tags: Vec<String> = (0..50).map(|i| format!("tag-{}", i)).collect();

    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: tags.clone(),
        })
        .await
        .unwrap();

    assert_eq!(result.tags_added, 50);
    assert_eq!(result.current_tags.len(), 50);
}

/// Test with very long tag names (at limit)
#[tokio::test]
async fn test_long_tag_names() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test long tags".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Create a 100-character tag (at the limit)
    let long_tag = "a".repeat(100);

    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec![long_tag.clone()],
        })
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.tags_added, 1);
}

/// Test searching across many episodes
#[tokio::test]
async fn test_search_many_episodes() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create 20 episodes
    for i in 0..20 {
        let episode_id = memory
            .start_episode(
                format!("Episode {}", i),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Every even episode gets "even" tag
        // Every multiple of 3 gets "three" tag
        let mut tags = vec![];
        if i % 2 == 0 {
            tags.push("even".to_string());
        }
        if i % 3 == 0 {
            tags.push("three".to_string());
        }
        if !tags.is_empty() {
            memory.add_episode_tags(episode_id, tags).await.unwrap();
        }
    }

    // Search for "even" tag - should find 10 episodes
    let result = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["even".to_string()],
            require_all: Some(false),
            limit: Some(100),
        })
        .await
        .unwrap();

    assert_eq!(result.count, 10);

    // Search for "even" AND "three" - should find episodes 0, 6, 12, 18 (4 episodes)
    let result_and = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["even".to_string(), "three".to_string()],
            require_all: Some(true),
            limit: Some(100),
        })
        .await
        .unwrap();

    assert_eq!(result_and.count, 4);
}

/// Test removing tags that don't exist
#[tokio::test]
async fn test_remove_nonexistent_tags() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test remove".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add some tags
    tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["existing".to_string()],
        })
        .await
        .unwrap();

    // Try to remove a tag that doesn't exist
    let result = tools
        .remove_tags(RemoveEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["nonexistent".to_string()],
        })
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.tags_removed, 0); // Nothing removed
    assert_eq!(result.current_tags.len(), 1); // Still has "existing"
}

/// Test setting empty tags (clearing all)
#[tokio::test]
async fn test_set_empty_tags() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test clear".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add some tags
    tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        })
        .await
        .unwrap();

    // Set empty tags (clear all)
    let result = tools
        .set_tags(SetEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec![],
        })
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.current_tags.len(), 0);
}

/// Test tag normalization (spaces, case)
#[tokio::test]
async fn test_tag_normalization() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test normalization".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add tags with different cases
    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["BUG-FIX".to_string(), "Bug-Fix".to_string()],
        })
        .await
        .unwrap();

    // Should only add one tag due to normalization
    assert_eq!(result.tags_added, 1);
    assert_eq!(result.current_tags.len(), 1);
    assert_eq!(result.current_tags[0], "bug-fix");
}

/// Test search with limit
#[tokio::test]
async fn test_search_with_limit() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create 10 episodes with same tag
    for i in 0..10 {
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

    // Search with limit of 5
    let result = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["common".to_string()],
            require_all: Some(false),
            limit: Some(5),
        })
        .await
        .unwrap();

    assert_eq!(result.count, 5); // Limited to 5
}

/// Test multiple sequential operations on same episode
#[tokio::test]
async fn test_sequential_operations() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Sequential ops".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Operation 1: Add tags
    tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        })
        .await
        .unwrap();

    // Operation 2: Add more tags
    tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["tag3".to_string()],
        })
        .await
        .unwrap();

    // Operation 3: Remove one tag
    tools
        .remove_tags(RemoveEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["tag2".to_string()],
        })
        .await
        .unwrap();

    // Operation 4: Add back the removed tag
    tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["tag2".to_string()],
        })
        .await
        .unwrap();

    // Final check
    let result = tools
        .get_tags(memory_mcp::mcp::tools::episode_tags::GetEpisodeTagsInput {
            episode_id: episode_id.to_string(),
        })
        .await
        .unwrap();

    assert_eq!(result.tags.len(), 3);
    assert!(result.tags.contains(&"tag1".to_string()));
    assert!(result.tags.contains(&"tag2".to_string()));
    assert!(result.tags.contains(&"tag3".to_string()));
}

/// Test search with no matching tags
#[tokio::test]
async fn test_search_no_matches() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create episode with tags
    let episode_id = memory
        .start_episode(
            "Episode".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    memory
        .add_episode_tags(episode_id, vec!["existing".to_string()])
        .await
        .unwrap();

    // Search for non-existent tag
    let result = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["nonexistent".to_string()],
            require_all: Some(false),
            limit: Some(10),
        })
        .await
        .unwrap();

    assert_eq!(result.count, 0);
    assert!(result.episodes.is_empty());
}

/// Test adding the same tag multiple times
#[tokio::test]
async fn test_add_same_tag_multiple_times() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Duplicate test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add tag first time
    let result1 = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["duplicate".to_string()],
        })
        .await
        .unwrap();
    assert_eq!(result1.tags_added, 1);

    // Add same tag second time
    let result2 = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["duplicate".to_string()],
        })
        .await
        .unwrap();
    assert_eq!(result2.tags_added, 0); // No new tags

    // Add same tag third time
    let result3 = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags: vec!["duplicate".to_string()],
        })
        .await
        .unwrap();
    assert_eq!(result3.tags_added, 0); // Still no new tags

    // Verify only one tag exists
    assert_eq!(result3.current_tags.len(), 1);
}
