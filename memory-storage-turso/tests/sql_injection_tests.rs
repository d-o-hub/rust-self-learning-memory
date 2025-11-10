//! SQL injection prevention tests for Turso storage backend
//!
//! These tests verify that parameterized queries prevent SQL injection attacks.
//! All malicious input should be safely stored as literal text without executing
//! malicious SQL commands.

use memory_core::{Episode, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use memory_storage_turso::TursoStorage;
use tempfile::TempDir;

/// Helper to create a test episode with default values
fn create_test_episode() -> Episode {
    Episode::new(
        "Test episode".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    )
}

/// Helper to create test storage with a local file database
async fn create_test_storage() -> anyhow::Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");

    // Use Builder::new_local for file-based databases
    let db = libsql::Builder::new_local(&db_path).build().await?;

    let storage = TursoStorage::from_database(db)?;
    storage.initialize_schema().await?;
    Ok((storage, dir))
}

#[tokio::test]
async fn test_sql_injection_in_task_description() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Malicious input attempting SQL injection
    let mut episode = create_test_episode();
    episode.task_description = "'; DROP TABLE episodes; --".to_string();

    // Should store safely without executing malicious SQL
    storage.store_episode(&episode).await.unwrap();

    // Verify data stored correctly and table still exists
    let retrieved = storage.get_episode(episode.episode_id).await.unwrap();
    assert!(retrieved.is_some(), "Episode should be retrievable");
    assert_eq!(
        retrieved.unwrap().task_description,
        episode.task_description,
        "SQL injection attempt should be stored as literal text"
    );
}

#[tokio::test]
async fn test_sql_injection_with_union_select() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut episode = create_test_episode();
    episode.task_description = "' UNION SELECT * FROM episodes; --".to_string();

    storage.store_episode(&episode).await.unwrap();
    let retrieved = storage
        .get_episode(episode.episode_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved.task_description, episode.task_description);
}

#[tokio::test]
async fn test_sql_injection_with_or_condition() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut episode = create_test_episode();
    episode.task_description = "' OR '1'='1".to_string();

    storage.store_episode(&episode).await.unwrap();
    let retrieved = storage
        .get_episode(episode.episode_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved.task_description, episode.task_description);
}

#[tokio::test]
async fn test_sql_injection_with_comment_bypass() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut episode = create_test_episode();
    episode.task_description = "admin'--".to_string();

    storage.store_episode(&episode).await.unwrap();
    let retrieved = storage
        .get_episode(episode.episode_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved.task_description, episode.task_description);
}

#[tokio::test]
async fn test_sql_injection_in_metadata() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut episode = create_test_episode();
    episode.metadata.insert(
        "malicious".to_string(),
        "'; DROP TABLE episodes; --".to_string(),
    );

    storage.store_episode(&episode).await.unwrap();
    let retrieved = storage
        .get_episode(episode.episode_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        retrieved.metadata, episode.metadata,
        "Metadata with SQL injection attempt should be stored safely"
    );
}

#[tokio::test]
async fn test_multiple_sql_injection_attempts() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let payloads = vec![
        "'; DROP TABLE episodes; --",
        "' UNION SELECT * FROM episodes; --",
        "' OR '1'='1",
        "admin'--",
        "1'; UPDATE episodes SET task_description='hacked'; --",
    ];

    for payload in payloads {
        let mut episode = create_test_episode();
        episode.task_description = payload.to_string();

        storage.store_episode(&episode).await.unwrap();
        let retrieved = storage
            .get_episode(episode.episode_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            retrieved.task_description, payload,
            "Payload '{}' should be safely stored",
            payload
        );
    }
}

#[tokio::test]
async fn test_table_integrity_after_injection_attempts() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Try multiple injection attacks
    for i in 0..10 {
        let mut episode = create_test_episode();
        episode.task_description = format!("'; DROP TABLE episodes; -- {}", i);
        storage.store_episode(&episode).await.unwrap();
    }

    // Verify table still exists and all episodes are retrievable
    // If table was dropped, this would fail
    let all_episodes: Vec<_> = (0..10).map(|_| create_test_episode()).collect();
    for ep in &all_episodes {
        storage.store_episode(ep).await.unwrap();
    }

    for ep in &all_episodes {
        let retrieved = storage.get_episode(ep.episode_id).await.unwrap();
        assert!(
            retrieved.is_some(),
            "Table should still exist and be functional"
        );
    }
}

#[tokio::test]
async fn test_sql_injection_in_execution_steps() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut episode = create_test_episode();
    let mut step = ExecutionStep::new(
        1,
        "'; DROP TABLE episodes; --".to_string(),
        "Malicious action".to_string(),
    );
    step.parameters = serde_json::json!({
        "sql_injection": "' OR '1'='1"
    });
    episode.add_step(step);

    storage.store_episode(&episode).await.unwrap();
    let retrieved = storage
        .get_episode(episode.episode_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.steps.len(), 1);
    assert_eq!(retrieved.steps[0].tool, "'; DROP TABLE episodes; --");
}

#[tokio::test]
async fn test_sql_injection_in_outcome() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut episode = create_test_episode();
    episode.complete(TaskOutcome::Failure {
        reason: "'; DELETE FROM episodes; --".to_string(),
        error_details: Some("' UNION SELECT * FROM episodes; --".to_string()),
    });

    storage.store_episode(&episode).await.unwrap();
    let retrieved = storage
        .get_episode(episode.episode_id)
        .await
        .unwrap()
        .unwrap();

    match retrieved.outcome.unwrap() {
        TaskOutcome::Failure {
            reason,
            error_details,
        } => {
            assert_eq!(reason, "'; DELETE FROM episodes; --");
            assert_eq!(error_details.unwrap(), "' UNION SELECT * FROM episodes; --");
        }
        _ => panic!("Expected Failure outcome"),
    }
}

#[tokio::test]
async fn test_sql_injection_in_context() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let context = TaskContext {
        domain: "'; DROP TABLE patterns; --".to_string(),
        language: Some("' OR '1'='1".to_string()),
        tags: vec![
            "'; DELETE FROM heuristics; --".to_string(),
            "admin'--".to_string(),
        ],
        ..Default::default()
    };

    let episode = Episode::new(
        "Test with malicious context".to_string(),
        context,
        TaskType::CodeGeneration,
    );

    storage.store_episode(&episode).await.unwrap();
    let retrieved = storage
        .get_episode(episode.episode_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.context.domain, "'; DROP TABLE patterns; --");
    assert_eq!(retrieved.context.language, Some("' OR '1'='1".to_string()));
    assert_eq!(retrieved.context.tags.len(), 2);
    assert_eq!(retrieved.context.tags[0], "'; DELETE FROM heuristics; --");
}
