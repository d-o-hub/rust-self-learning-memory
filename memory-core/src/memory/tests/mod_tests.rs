//! Tests for SelfLearningMemory constructors and accessors in memory/mod.rs + init.rs
use crate::memory::SelfLearningMemory;
use crate::memory::init::default_db_path;
use crate::types::MemoryConfig;

#[test]
fn test_default_db_path_is_non_empty() {
    let path = default_db_path();
    assert!(path.to_string_lossy().contains("memory.db"));
}

#[test]
fn test_new_default_has_no_storage_backends() {
    let memory = SelfLearningMemory::new();
    let (primary, cache) = memory.storage_backends();
    assert!(primary.is_none());
    assert!(cache.is_none());
}

#[test]
fn test_with_config_respects_quality_threshold() {
    let config = MemoryConfig {
        quality_threshold: 0.42,
        ..MemoryConfig::default()
    };
    let memory = SelfLearningMemory::with_config(config);
    assert!((memory.quality_threshold() - 0.42).abs() < f32::EPSILON);
}

#[test]
fn test_with_semantic_config_does_not_panic() {
    let config = MemoryConfig::default();
    let sem_config = crate::embeddings::EmbeddingConfig::default();
    let _ = SelfLearningMemory::with_semantic_config(config, sem_config);
}

#[test]
fn test_batch_config_some_by_default() {
    let memory = SelfLearningMemory::new();
    assert!(memory.batch_config().is_some());
}

#[test]
fn test_default_impl() {
    // Exercises the Default trait (calls new())
    let _: SelfLearningMemory = SelfLearningMemory::default();
}

#[tokio::test]
async fn test_get_episode_patterns_returns_recorded_patterns() {
    use crate::patterns::{Pattern, PatternEffectiveness};
    use crate::types::{TaskContext, TaskType};
    use uuid::Uuid;

    let memory = SelfLearningMemory::new();
    let episode_id = memory
        .start_episode(
            "Resolve recorded patterns".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let stored = Pattern::ContextPattern {
        id: Uuid::new_v4(),
        context_features: vec!["domain:testing".to_string()],
        recommended_approach: "Reuse the stored pattern".to_string(),
        evidence: vec![],
        success_rate: 1.0,
        effectiveness: PatternEffectiveness::new(),
    };
    let stored_id = stored.id();

    memory
        .store_patterns(episode_id, vec![stored])
        .await
        .expect("store patterns");

    // Dangling id stands in for a pattern removed by decay.
    let dangling_id = Uuid::new_v4();
    let mut episode = memory.get_episode(episode_id).await.expect("episode");
    assert_eq!(episode.patterns, vec![stored_id]);
    episode.patterns = vec![stored_id, dangling_id];
    memory
        .update_episode_full(&episode)
        .await
        .expect("update episode");

    let patterns = memory
        .get_episode_patterns(episode_id)
        .await
        .expect("get_episode_patterns");
    assert_eq!(patterns.len(), 1, "dangling pattern ids must be omitted");
    assert_eq!(patterns[0].id(), stored_id);

    let missing_id = Uuid::new_v4();
    let err = memory
        .get_episode_patterns(missing_id)
        .await
        .expect_err("missing episode must be an error");
    assert!(matches!(err, crate::Error::NotFound(id) if id == missing_id));
}

#[tokio::test]
async fn test_delete_pattern_removes_in_memory_record_and_episode_link() {
    use crate::patterns::{Pattern, PatternEffectiveness};
    use crate::types::{TaskContext, TaskType};
    use uuid::Uuid;

    let memory = SelfLearningMemory::new();
    let episode_id = memory
        .start_episode(
            "Delete a stored pattern".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let pattern = Pattern::ContextPattern {
        id: Uuid::new_v4(),
        context_features: vec!["domain:testing".to_string()],
        recommended_approach: "Delete me".to_string(),
        evidence: vec![],
        success_rate: 0.1,
        effectiveness: PatternEffectiveness::new(),
    };
    let pattern_id = pattern.id();

    memory
        .store_patterns(episode_id, vec![pattern])
        .await
        .expect("store pattern");
    assert!(memory.get_pattern(pattern_id).await.expect("get").is_some());

    memory
        .delete_pattern(pattern_id)
        .await
        .expect("delete pattern");

    assert!(
        memory
            .get_pattern(pattern_id)
            .await
            .expect("get after delete")
            .is_none()
    );
    assert_eq!(memory.get_all_patterns().await.expect("list").len(), 0);

    // The historical episode keeps the ID; resolution omits the dangling link.
    let patterns = memory
        .get_episode_patterns(episode_id)
        .await
        .expect("episode patterns");
    assert_eq!(patterns.len(), 0);

    // Idempotent: a retried decay must not fail.
    memory
        .delete_pattern(pattern_id)
        .await
        .expect("idempotent delete");
}
