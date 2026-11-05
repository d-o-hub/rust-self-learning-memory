//! Incremental synchronization must see updates to already-consumed episodes.

use chrono::{Duration, Utc};
use do_memory_core::{Episode, TaskContext, TaskType};
use do_memory_storage_redb::RedbStorage;

#[tokio::test]
async fn test_modified_since_update_after_reopen_advances_saved_cursor() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("sync.redb");
    let storage = RedbStorage::new(&path).await?;
    let since = Utc::now() - Duration::hours(1);
    let mut episode = Episode::new(
        "Original task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    episode.start_time = since - Duration::days(1);
    storage.store_episode(&episode).await?;
    let initial = storage
        .query_episodes_modified_since(since, None, Some(1))
        .await?;
    assert_eq!(initial.len(), 1);
    let cursor = (initial[0].1, initial[0].0.episode_id);
    assert_eq!(
        storage
            .query_episodes_modified_since(since, Some(cursor), None)
            .await?,
        Vec::<(Episode, chrono::DateTime<Utc>)>::new()
    );
    drop(storage);

    let storage = RedbStorage::new(&path).await?;
    assert_eq!(
        storage.get_episode(episode.episode_id).await?,
        Some(episode.clone())
    );
    let restored = storage
        .query_episodes_modified_since(since, None, Some(1))
        .await?;
    assert_eq!(restored.len(), 1);
    assert_eq!((restored[0].1, restored[0].0.episode_id), cursor);
    episode.task_description = "Updated after restart".to_string();
    episode
        .metadata
        .insert("status".to_string(), "reviewed".to_string());
    storage.store_episode(&episode).await?;

    let updates = storage
        .query_episodes_modified_since(since, Some(cursor), None)
        .await?;
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].0, episode);
    assert!(updates[0].1 > cursor.0);
    assert_eq!(
        storage
            .query_episodes_modified_since(
                since,
                Some((updates[0].1, updates[0].0.episode_id)),
                None,
            )
            .await?,
        Vec::<(Episode, chrono::DateTime<Utc>)>::new()
    );
    Ok(())
}
