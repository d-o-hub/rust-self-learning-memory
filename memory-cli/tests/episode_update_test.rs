//! Integration tests for episode update command

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::{TaskContext, TaskType};
    use uuid::Uuid;

    /// Test updating episode description
    #[tokio::test]
    async fn test_update_description() {
        let memory = memory_core::SelfLearningMemory::new();

        // Create an episode
        let episode_id = memory
            .start_episode(
                "Original description".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Update description
        memory
            .update_episode(episode_id, Some("Updated description".to_string()), None)
            .await
            .unwrap();

        // Verify update
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.task_description, "Updated description");
    }

    /// Test updating episode metadata
    #[tokio::test]
    async fn test_update_metadata() {
        let memory = memory_core::SelfLearningMemory::new();

        // Create an episode
        let episode_id = memory
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Update metadata
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        memory
            .update_episode(episode_id, None, Some(metadata))
            .await
            .unwrap();

        // Verify update
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(episode.metadata.get("key2"), Some(&"value2".to_string()));
    }

    /// Test updating both description and metadata
    #[tokio::test]
    async fn test_update_description_and_metadata() {
        let memory = memory_core::SelfLearningMemory::new();

        // Create an episode
        let episode_id = memory
            .start_episode(
                "Original description".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Update both description and metadata
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());

        memory
            .update_episode(
                episode_id,
                Some("Updated description".to_string()),
                Some(metadata),
            )
            .await
            .unwrap();

        // Verify updates
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.task_description, "Updated description");
        assert_eq!(episode.metadata.get("key1"), Some(&"value1".to_string()));
    }

    /// Test updating non-existent episode fails
    #[tokio::test]
    async fn test_update_nonexistent_episode() {
        let memory = memory_core::SelfLearningMemory::new();

        let fake_id = Uuid::new_v4();
        let result = memory
            .update_episode(fake_id, Some("New description".to_string()), None)
            .await;

        assert!(result.is_err());
    }

    /// Test updating episode with same description (no-op)
    #[tokio::test]
    async fn test_update_same_description() {
        let memory = memory_core::SelfLearningMemory::new();

        // Create an episode
        let episode_id = memory
            .start_episode(
                "Original description".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Update with same description
        memory
            .update_episode(episode_id, Some("Original description".to_string()), None)
            .await
            .unwrap();

        // Verify episode still exists and is unchanged
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.task_description, "Original description");
    }

    /// Test merging metadata with existing metadata
    #[tokio::test]
    async fn test_merge_metadata() {
        let memory = memory_core::SelfLearningMemory::new();

        // Create an episode
        let episode_id = memory
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Add initial metadata
        let mut initial_metadata = HashMap::new();
        initial_metadata.insert("initial_key".to_string(), "initial_value".to_string());
        memory
            .update_episode(episode_id, None, Some(initial_metadata))
            .await
            .unwrap();

        // Add more metadata (merging with existing)
        let mut metadata = HashMap::new();
        metadata.insert("new_key".to_string(), "new_value".to_string());

        memory
            .update_episode(episode_id, None, Some(metadata))
            .await
            .unwrap();

        // Verify both old and new metadata exist
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(
            episode.metadata.get("initial_key"),
            Some(&"initial_value".to_string())
        );
        assert_eq!(
            episode.metadata.get("new_key"),
            Some(&"new_value".to_string())
        );
    }

    /// Test updating tags (add, remove, set)
    #[tokio::test]
    async fn test_update_tags() {
        let memory = memory_core::SelfLearningMemory::new();

        // Create an episode
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

        let tags = memory.get_episode_tags(episode_id).await.unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"tag1".to_string()));
        assert!(tags.contains(&"tag2".to_string()));

        // Add more tags
        memory
            .add_episode_tags(episode_id, vec!["tag3".to_string()])
            .await
            .unwrap();

        let tags = memory.get_episode_tags(episode_id).await.unwrap();
        assert_eq!(tags.len(), 3);

        // Remove a tag
        memory
            .remove_episode_tags(episode_id, vec!["tag2".to_string()])
            .await
            .unwrap();

        let tags = memory.get_episode_tags(episode_id).await.unwrap();
        assert_eq!(tags.len(), 2);
        assert!(!tags.contains(&"tag2".to_string()));

        // Set all tags
        memory
            .set_episode_tags(
                episode_id,
                vec!["new_tag1".to_string(), "new_tag2".to_string()],
            )
            .await
            .unwrap();

        let tags = memory.get_episode_tags(episode_id).await.unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"new_tag1".to_string()));
        assert!(tags.contains(&"new_tag2".to_string()));
        assert!(!tags.contains(&"tag1".to_string()));
    }

    /// Test that tag operations persist across storage
    #[tokio::test]
    #[cfg(feature = "turso")]
    async fn test_update_persists_to_storage() {
        use memory_storage_turso::TursoClient;
        use std::sync::Arc;

        // This test requires actual Turso setup, so we'll skip if not configured
        let db_url = std::env::var("TURSO_DB_URL").ok();
        let auth_token = std::env::var("TURSO_AUTH_TOKEN").ok();

        if db_url.is_none() || auth_token.is_none() {
            println!(
                "Skipping storage persistence test - TURSO_DB_URL and TURSO_AUTH_TOKEN not set"
            );
            return;
        }

        let turso = TursoClient::new(db_url.unwrap(), auth_token.unwrap()).unwrap();
        let memory = memory_core::SelfLearningMemory::with_storage(Arc::new(turso), None, None);

        // Create episode
        let episode_id = memory
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Update
        memory
            .update_episode(episode_id, Some("Updated".to_string()), None)
            .await
            .unwrap();

        // Force reload from storage
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert_eq!(episode.task_description, "Updated");
    }
}
