//! Hybrid Storage Consistency Tests
//!
//! Verifies that data is consistently mirrored across Turso and redb backends
//! in a hybrid storage configuration.

use do_memory_core::{ExecutionStep, TaskContext, TaskOutcome, TaskType};
use do_memory_test_utils::{assert_episode_parity, hybrid_memory};

#[tokio::test]
async fn test_hybrid_storage_write_parity() {
    let (memory, _turso_dir, _redb_dir) = hybrid_memory().await;

    // 1. Create and complete an episode
    let episode_id = memory
        .start_episode(
            "Hybrid parity test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    memory
        .log_step(
            episode_id,
            ExecutionStep::new(1, "tool".to_string(), "action".to_string()),
        )
        .await;

    let outcome = TaskOutcome::Success {
        verdict: "Consistent".to_string(),
        artifacts: vec![],
    };

    memory
        .complete_episode(episode_id, outcome)
        .await
        .expect("Complete episode");

    // 2. Fetch from the memory orchestrator
    let orchestrator_episode = memory
        .get_episode(episode_id)
        .await
        .expect("Get from orchestrator");

    // 3. Directly inspect backends
    let (turso, redb) = memory.storage_backends();
    let turso = turso.expect("Turso backend missing");
    let redb = redb.expect("redb backend missing");

    let turso_episode = turso
        .get_episode(episode_id)
        .await
        .expect("Get from Turso")
        .expect("Episode not in Turso");
    let redb_episode = redb
        .get_episode(episode_id)
        .await
        .expect("Get from redb")
        .expect("Episode not in redb");

    // 4. Verify parity
    assert_episode_parity(&orchestrator_episode, &turso_episode);
    assert_episode_parity(&orchestrator_episode, &redb_episode);
    assert_episode_parity(&turso_episode, &redb_episode);
}

#[tokio::test]
async fn test_hybrid_storage_update_parity() {
    let (memory, _turso_dir, _redb_dir) = hybrid_memory().await;

    let episode_id = memory
        .start_episode(
            "Update parity test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // complete it first
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "ok".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // update it
    let mut new_metadata = std::collections::HashMap::new();
    new_metadata.insert("test_key".to_string(), "test_value".to_string());

    memory
        .update_episode(
            episode_id,
            Some("Updated description".to_string()),
            Some(new_metadata),
        )
        .await
        .expect("Update episode");

    // Verify both backends updated
    let (turso, redb) = memory.storage_backends();
    let turso_ep = turso
        .unwrap()
        .get_episode(episode_id)
        .await
        .unwrap()
        .unwrap();
    let redb_ep = redb
        .unwrap()
        .get_episode(episode_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(turso_ep.task_description, "Updated description");
    assert_eq!(redb_ep.task_description, "Updated description");
    assert_eq!(turso_ep.metadata.get("test_key").unwrap(), "test_value");
    assert_eq!(redb_ep.metadata.get("test_key").unwrap(), "test_value");
}

#[tokio::test]
async fn test_hybrid_storage_delete_parity() {
    let (memory, _turso_dir, _redb_dir) = hybrid_memory().await;

    let episode_id = memory
        .start_episode(
            "Delete parity test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "ok".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // delete it
    memory
        .delete_episode(episode_id)
        .await
        .expect("Delete episode");

    // Verify both backends deleted
    let (turso, redb) = memory.storage_backends();
    assert!(
        turso
            .unwrap()
            .get_episode(episode_id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        redb.unwrap()
            .get_episode(episode_id)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn test_hybrid_storage_query_parity() {
    let (memory, _turso_dir, _redb_dir) = hybrid_memory().await;

    for i in 0..3 {
        let episode_id = memory
            .start_episode(
                format!("Query parity test {i}"),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "ok".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    let all_episodes = memory.get_all_episodes().await.expect("Get all episodes");
    assert_eq!(all_episodes.len(), 3);

    let (turso, redb) = memory.storage_backends();
    let turso_episodes = turso
        .unwrap()
        .query_episodes_since(chrono::Utc::now() - chrono::Duration::hours(1), None)
        .await
        .unwrap();
    let redb_episodes = redb
        .unwrap()
        .query_episodes_since(chrono::Utc::now() - chrono::Duration::hours(1), None)
        .await
        .unwrap();

    assert_eq!(turso_episodes.len(), 3);
    assert_eq!(redb_episodes.len(), 3);
    assert_eq!(turso_episodes.len(), redb_episodes.len());
}
