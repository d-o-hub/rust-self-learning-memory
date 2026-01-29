//! Tests for episode tagging tools

use crate::mcp::tools::episode_tags::{
    AddEpisodeTagsInput, EpisodeTagTools, GetEpisodeTagsInput, RemoveEpisodeTagsInput,
    SearchEpisodesByTagsInput, SetEpisodeTagsInput,
};
use memory_core::{SelfLearningMemory, TaskContext, TaskType};
use std::sync::Arc;

fn create_test_memory() -> Arc<SelfLearningMemory> {
    Arc::new(SelfLearningMemory::new())
}

#[tokio::test]
async fn test_add_episode_tags() {
    let memory = create_test_memory();
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create an episode
    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add tags
    let input = AddEpisodeTagsInput {
        episode_id: episode_id.to_string(),
        tags: vec!["bug-fix".to_string(), "critical".to_string()],
    };

    let output = tools.add_tags(input).await.unwrap();

    assert!(output.success);
    assert_eq!(output.tags_added, 2);
    assert_eq!(output.current_tags.len(), 2);
    assert!(output.current_tags.contains(&"bug-fix".to_string()));
    assert!(output.current_tags.contains(&"critical".to_string()));
}

#[tokio::test]
async fn test_add_duplicate_tags() {
    let memory = create_test_memory();
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add tags first time
    let input1 = AddEpisodeTagsInput {
        episode_id: episode_id.to_string(),
        tags: vec!["bug-fix".to_string()],
    };
    tools.add_tags(input1).await.unwrap();

    // Try adding same tag again
    let input2 = AddEpisodeTagsInput {
        episode_id: episode_id.to_string(),
        tags: vec!["bug-fix".to_string()],
    };
    let output = tools.add_tags(input2).await.unwrap();

    assert!(output.success);
    assert_eq!(output.tags_added, 0); // No new tags added
    assert_eq!(output.current_tags.len(), 1);
}

#[tokio::test]
async fn test_remove_episode_tags() {
    let memory = create_test_memory();
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add tags
    memory
        .add_episode_tags(
            episode_id,
            vec![
                "bug-fix".to_string(),
                "critical".to_string(),
                "feature".to_string(),
            ],
        )
        .await
        .unwrap();

    // Remove one tag
    let input = RemoveEpisodeTagsInput {
        episode_id: episode_id.to_string(),
        tags: vec!["critical".to_string()],
    };

    let output = tools.remove_tags(input).await.unwrap();

    assert!(output.success);
    assert_eq!(output.tags_removed, 1);
    assert_eq!(output.current_tags.len(), 2);
    assert!(!output.current_tags.contains(&"critical".to_string()));
}

#[tokio::test]
async fn test_set_episode_tags() {
    let memory = create_test_memory();
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add initial tags
    memory
        .add_episode_tags(episode_id, vec!["old-tag".to_string()])
        .await
        .unwrap();

    // Set new tags (replaces old)
    let input = SetEpisodeTagsInput {
        episode_id: episode_id.to_string(),
        tags: vec!["new-tag1".to_string(), "new-tag2".to_string()],
    };

    let output = tools.set_tags(input).await.unwrap();

    assert!(output.success);
    assert_eq!(output.tags_set, 2);
    assert_eq!(output.current_tags.len(), 2);
    assert!(!output.current_tags.contains(&"old-tag".to_string()));
    assert!(output.current_tags.contains(&"new-tag1".to_string()));
    assert!(output.current_tags.contains(&"new-tag2".to_string()));
}

#[tokio::test]
async fn test_get_episode_tags() {
    let memory = create_test_memory();
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add tags
    memory
        .add_episode_tags(episode_id, vec!["tag1".to_string(), "tag2".to_string()])
        .await
        .unwrap();

    // Get tags
    let input = GetEpisodeTagsInput {
        episode_id: episode_id.to_string(),
    };

    let output = tools.get_tags(input).await.unwrap();

    assert!(output.success);
    assert_eq!(output.tags.len(), 2);
    assert!(output.tags.contains(&"tag1".to_string()));
    assert!(output.tags.contains(&"tag2".to_string()));
}

#[tokio::test]
async fn test_search_episodes_by_tags_any() {
    let memory = create_test_memory();
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create episodes with different tags
    let episode1 = memory
        .start_episode(
            "Task 1".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    memory
        .add_episode_tags(
            episode1,
            vec!["bug-fix".to_string(), "critical".to_string()],
        )
        .await
        .unwrap();

    let episode2 = memory
        .start_episode(
            "Task 2".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    memory
        .add_episode_tags(
            episode2,
            vec!["feature".to_string(), "critical".to_string()],
        )
        .await
        .unwrap();

    let episode3 = memory
        .start_episode(
            "Task 3".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    memory
        .add_episode_tags(episode3, vec!["refactor".to_string()])
        .await
        .unwrap();

    // Search for episodes with "bug-fix" OR "feature"
    let input = SearchEpisodesByTagsInput {
        tags: vec!["bug-fix".to_string(), "feature".to_string()],
        require_all: Some(false), // OR search
        limit: Some(10),
    };

    let output = tools.search_by_tags(input).await.unwrap();

    assert!(output.success);
    assert_eq!(output.count, 2); // Episodes 1 and 2
    assert!(output
        .episodes
        .iter()
        .any(|e| e.episode_id == episode1.to_string()));
    assert!(output
        .episodes
        .iter()
        .any(|e| e.episode_id == episode2.to_string()));
}

#[tokio::test]
async fn test_search_episodes_by_tags_all() {
    let memory = create_test_memory();
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create episodes with different tags
    let episode1 = memory
        .start_episode(
            "Task 1".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    memory
        .add_episode_tags(
            episode1,
            vec!["bug-fix".to_string(), "critical".to_string()],
        )
        .await
        .unwrap();

    let episode2 = memory
        .start_episode(
            "Task 2".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    memory
        .add_episode_tags(episode2, vec!["bug-fix".to_string()])
        .await
        .unwrap();

    // Search for episodes with both "bug-fix" AND "critical"
    let input = SearchEpisodesByTagsInput {
        tags: vec!["bug-fix".to_string(), "critical".to_string()],
        require_all: Some(true), // AND search
        limit: Some(10),
    };

    let output = tools.search_by_tags(input).await.unwrap();

    assert!(output.success);
    assert_eq!(output.count, 1); // Only episode 1
    assert_eq!(output.episodes[0].episode_id, episode1.to_string());
}

#[tokio::test]
async fn test_invalid_episode_id() {
    let memory = create_test_memory();
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let input = AddEpisodeTagsInput {
        episode_id: "invalid-uuid".to_string(),
        tags: vec!["test".to_string()],
    };

    let result = tools.add_tags(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_empty_tags() {
    let memory = create_test_memory();
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let input = AddEpisodeTagsInput {
        episode_id: episode_id.to_string(),
        tags: vec![],
    };

    let output = tools.add_tags(input).await.unwrap();
    assert!(!output.success);
    assert_eq!(output.tags_added, 0);
}
