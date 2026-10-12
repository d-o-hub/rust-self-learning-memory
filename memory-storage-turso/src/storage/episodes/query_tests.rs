use super::*;
use do_memory_core::{Episode, TaskContext, TaskType};
use tempfile::TempDir;

async fn create_test_storage() -> Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| Error::Storage(format!("Failed to create test database: {}", e)))?;

    let storage = TursoStorage::from_database(db)?;
    storage.initialize_schema().await?;

    Ok((storage, dir))
}

#[tokio::test]
async fn test_query_episodes_empty() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let query = EpisodeQuery::default();
    let result = storage.query_episodes(&query).await.unwrap();
    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_query_episodes_with_limit() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Create multiple episodes
    for i in 0..5 {
        let episode = Episode::new(
            format!("Task {}", i),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        storage.store_episode(&episode).await.unwrap();
    }

    // Query with limit
    let query = EpisodeQuery {
        limit: Some(3),
        ..Default::default()
    };
    let result = storage.query_episodes(&query).await.unwrap();
    assert_eq!(result.len(), 3);
}

#[tokio::test]
async fn test_query_episodes_by_task_type() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Create episodes with different task types
    for i in 0..3 {
        let episode = Episode::new(
            format!("Code task {}", i),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        storage.store_episode(&episode).await.unwrap();
    }

    for i in 0..2 {
        let episode = Episode::new(
            format!("Debug task {}", i),
            TaskContext::default(),
            TaskType::Debugging,
        );
        storage.store_episode(&episode).await.unwrap();
    }

    // Query by task type
    let query = EpisodeQuery {
        task_type: Some(TaskType::CodeGeneration),
        ..Default::default()
    };
    let result = storage.query_episodes(&query).await.unwrap();
    assert_eq!(result.len(), 3);
}

#[tokio::test]
async fn test_query_episodes_modified_since_pages_equal_timestamps() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut ids = Vec::new();
    for i in 0..5 {
        let episode = Episode::new(
            format!("Task {i}"),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        ids.push(episode.episode_id);
        storage.store_episode(&episode).await.unwrap();
    }

    // Collapse every revision onto one millisecond so only the
    // (modified_at, episode_id) tuple cursor can keep pages disjoint.
    let fixed_ms = chrono::Utc::now().timestamp_millis();
    let conn = storage.get_connection().await.unwrap();
    conn.execute(
        "UPDATE episode_revisions SET modified_at_ms = ?",
        libsql::params![fixed_ms],
    )
    .await
    .unwrap();

    let since = chrono::DateTime::from_timestamp_millis(fixed_ms).unwrap();

    let mut seen = Vec::new();
    let mut cursor = None;
    for _ in 0..10 {
        let page = storage
            .query_episodes_modified_since(since, cursor, Some(2))
            .await
            .unwrap();
        if page.is_empty() {
            break;
        }
        let (last_episode, last_modified_at) = page.last().unwrap();
        cursor = Some((*last_modified_at, last_episode.episode_id));
        seen.extend(page.into_iter().map(|(episode, _)| episode.episode_id));
    }

    assert_eq!(seen.len(), 5, "no duplicates or omissions across pages");
    let unique: std::collections::HashSet<_> = seen.iter().copied().collect();
    assert_eq!(unique.len(), 5);
    for id in ids {
        assert!(
            unique.contains(&id),
            "episode {id} must appear exactly once"
        );
    }
}

#[tokio::test]
async fn test_query_episodes_by_metadata() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut episode1 = Episode::new(
        "Task with tag".to_string(),
        TaskContext::default(),
        TaskType::Refactoring,
    );
    episode1
        .metadata
        .insert("tag".to_string(), "important".to_string());
    storage.store_episode(&episode1).await.unwrap();

    let episode2 = Episode::new(
        "Task without tag".to_string(),
        TaskContext::default(),
        TaskType::Refactoring,
    );
    storage.store_episode(&episode2).await.unwrap();

    // Query by metadata
    let result = storage
        .query_episodes_by_metadata("tag", "important", None)
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].task_description, "Task with tag");
}

#[tokio::test]
async fn test_episode_revisions_are_strictly_monotonic() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Store rapidly so several writes share the same wall-clock millisecond.
    for i in 0..25 {
        let episode = Episode::new(
            format!("Task {i}"),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        storage.store_episode(&episode).await.unwrap();
    }

    let conn = storage.get_connection().await.unwrap();
    let mut rows = conn
        .query(
            "SELECT modified_at_ms FROM episode_revisions ORDER BY modified_at_ms ASC",
            (),
        )
        .await
        .unwrap();

    let mut values = Vec::new();
    while let Some(row) = rows.next().await.unwrap() {
        values.push(row.get::<i64>(0).unwrap());
    }
    assert_eq!(values.len(), 25);

    let mut previous = 0i64;
    for value in &values {
        assert!(
            *value > previous,
            "revision values must be strictly increasing; {values:?}"
        );
        previous = *value;
    }

    // A later write always sorts after every existing one, so an advanced
    // watermark/cursor can never skip it.
    let extra = Episode::new(
        "Extra".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );
    storage.store_episode(&extra).await.unwrap();

    let mut max_rows = conn
        .query("SELECT MAX(modified_at_ms) FROM episode_revisions", ())
        .await
        .unwrap();
    let max = max_rows
        .next()
        .await
        .unwrap()
        .unwrap()
        .get::<i64>(0)
        .unwrap();
    assert!(max > *values.last().unwrap());
}

#[tokio::test]
async fn test_episode_revisions_backfilled_for_pre_existing_rows() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut episode = Episode::new(
        "Legacy".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );
    episode.start_time = chrono::Utc::now() - chrono::Duration::hours(2);
    storage.store_episode(&episode).await.unwrap();

    // Simulate a database written before `episode_revisions` existed.
    let conn = storage.get_connection().await.unwrap();
    conn.execute("DELETE FROM episode_revisions", ())
        .await
        .unwrap();
    conn.execute(
        "DELETE FROM metadata WHERE key = 'episode_revisions_backfilled'",
        (),
    )
    .await
    .unwrap();
    drop(conn);

    // Re-initialising runs the guarded backfill from start_time.
    storage.initialize_schema().await.unwrap();

    let since = chrono::Utc::now() - chrono::Duration::hours(3);
    let found = storage
        .query_episodes_modified_since(since, None, None)
        .await
        .unwrap();
    assert_eq!(found.len(), 1, "pre-existing episode must be backfilled");
    assert_eq!(found[0].0.episode_id, episode.episode_id);
}
