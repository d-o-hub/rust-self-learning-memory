//! # Batch Heuristic Operations - Tests
//!
//! Tests for batch heuristic operations.

#[cfg(test)]
mod tests {
    use super::super::heuristic_types::{HeuristicBatchProgress, HeuristicBatchResult};
    use memory_core::{types::Evidence, Heuristic};
    use tempfile::TempDir;
    use uuid::Uuid;

    async fn create_test_storage() -> crate::Result<(crate::TursoStorage, TempDir)> {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path)
            .build()
            .await
            .map_err(|e| {
                memory_core::Error::Storage(format!("Failed to create test database: {}", e))
            })?;

        let storage = crate::TursoStorage::from_database(db)?;
        storage.initialize_schema().await?;

        Ok((storage, dir))
    }

    fn create_test_heuristic(suffix: &str) -> Heuristic {
        Heuristic {
            heuristic_id: Uuid::new_v4(),
            condition: format!("test condition {}", suffix),
            action: format!("test action {}", suffix),
            confidence: 0.85,
            evidence: Evidence {
                episode_ids: vec![],
                success_rate: 0.9,
                sample_size: 10,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_store_heuristics_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let result = storage.store_heuristics_batch(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_heuristics_batch_single() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristics = vec![create_test_heuristic("1")];
        let result = storage.store_heuristics_batch(heuristics).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_heuristics_batch_multiple() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristics = vec![
            create_test_heuristic("1"),
            create_test_heuristic("2"),
            create_test_heuristic("3"),
        ];
        let result = storage.store_heuristics_batch(heuristics.clone()).await;
        assert!(result.is_ok());

        for heuristic in &heuristics {
            let retrieved = storage.get_heuristic(heuristic.heuristic_id).await.unwrap();
            assert!(retrieved.is_some());
            let retrieved = retrieved.unwrap();
            assert_eq!(retrieved.condition, heuristic.condition);
            assert_eq!(retrieved.action, heuristic.action);
        }
    }

    #[tokio::test]
    async fn test_update_heuristics_batch() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristics = vec![create_test_heuristic("1"), create_test_heuristic("2")];
        storage
            .store_heuristics_batch(heuristics.clone())
            .await
            .unwrap();

        let mut updated_heuristics = heuristics.clone();
        for heuristic in &mut updated_heuristics {
            heuristic.condition = format!("{} updated", heuristic.condition);
            heuristic.action = format!("{} updated", heuristic.action);
            heuristic.confidence = 0.95;
            heuristic.updated_at = chrono::Utc::now();
        }

        let result = storage
            .update_heuristics_batch(updated_heuristics.clone())
            .await;
        assert!(result.is_ok());

        for heuristic in &updated_heuristics {
            let retrieved = storage.get_heuristic(heuristic.heuristic_id).await.unwrap();
            assert!(retrieved.is_some());
            let retrieved = retrieved.unwrap();
            assert!(retrieved.condition.contains("updated"));
            assert!(retrieved.action.contains("updated"));
            assert!((retrieved.confidence - 0.95).abs() < f32::EPSILON);
        }
    }

    #[tokio::test]
    async fn test_update_heuristics_batch_nonexistent() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristics = vec![create_test_heuristic("nonexistent")];
        let result = storage.update_heuristics_batch(heuristics).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_store_heuristics_batch_with_progress() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristics: Vec<Heuristic> = (0..25)
            .map(|i| create_test_heuristic(&i.to_string()))
            .collect();

        let result = storage
            .store_heuristics_batch_with_progress(heuristics, 10)
            .await
            .unwrap();

        assert_eq!(result.total_processed, 25);
        assert_eq!(result.succeeded, 25);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded);
    }

    #[tokio::test]
    async fn test_update_heuristics_batch_with_progress() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristics: Vec<Heuristic> = (0..25)
            .map(|i| create_test_heuristic(&i.to_string()))
            .collect();
        storage
            .store_heuristics_batch(heuristics.clone())
            .await
            .unwrap();

        let result = storage
            .update_heuristics_batch_with_progress(heuristics, 10)
            .await
            .unwrap();

        assert_eq!(result.total_processed, 25);
        assert_eq!(result.succeeded, 25);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded);
    }

    #[tokio::test]
    async fn test_heuristic_batch_progress_tracking() {
        let progress = HeuristicBatchProgress::new(100, 10);
        assert_eq!(progress.total, 100);
        assert_eq!(progress.total_batches, 10);
        assert_eq!(progress.processed, 0);
        assert!(!progress.is_complete());

        let mut progress = progress;
        progress.update(10, 10, 0);
        assert_eq!(progress.processed, 10);
        assert_eq!(progress.succeeded, 10);
        assert_eq!(progress.current_batch, 1);
        assert_eq!(progress.percent_complete(), 10.0);
    }

    #[tokio::test]
    async fn test_heuristic_batch_result_success() {
        let result = HeuristicBatchResult::success(10);
        assert_eq!(result.total_processed, 10);
        assert_eq!(result.succeeded, 10);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_heuristic_batch_result_failure() {
        let result = HeuristicBatchResult::failure("test error".to_string());
        assert_eq!(result.total_processed, 0);
        assert_eq!(result.succeeded, 0);
        assert!(!result.all_succeeded);
        assert_eq!(result.errors.len(), 1);
    }

    #[tokio::test]
    async fn test_heuristic_transaction_rollback() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let existing = create_test_heuristic("existing");
        storage
            .store_heuristics_batch(vec![existing.clone()])
            .await
            .unwrap();

        let non_existent = create_test_heuristic("nonexistent");
        let update_result = storage
            .update_heuristics_batch(vec![existing.clone(), non_existent])
            .await;

        assert!(update_result.is_err());

        let retrieved = storage.get_heuristic(existing.heuristic_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().condition, existing.condition);
    }

    #[tokio::test]
    async fn test_delete_heuristics_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let result = storage.delete_heuristics_batch(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_heuristics_batch_single() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristic = create_test_heuristic("1");
        storage
            .store_heuristics_batch(vec![heuristic.clone()])
            .await
            .unwrap();

        let result = storage
            .delete_heuristics_batch(vec![heuristic.heuristic_id])
            .await;
        assert!(result.is_ok());

        let retrieved = storage.get_heuristic(heuristic.heuristic_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_heuristics_batch_multiple() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristics = vec![
            create_test_heuristic("1"),
            create_test_heuristic("2"),
            create_test_heuristic("3"),
        ];
        let ids: Vec<Uuid> = heuristics.iter().map(|h| h.heuristic_id).collect();
        storage
            .store_heuristics_batch(heuristics.clone())
            .await
            .unwrap();

        let result = storage.delete_heuristics_batch(ids.clone()).await;
        assert!(result.is_ok());

        for id in ids {
            let retrieved = storage.get_heuristic(id).await.unwrap();
            assert!(retrieved.is_none());
        }
    }

    #[tokio::test]
    async fn test_delete_heuristics_batch_with_progress() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristics: Vec<Heuristic> = (0..25)
            .map(|i| create_test_heuristic(&i.to_string()))
            .collect();
        let ids: Vec<Uuid> = heuristics.iter().map(|h| h.heuristic_id).collect();
        storage.store_heuristics_batch(heuristics).await.unwrap();

        let result = storage
            .delete_heuristics_batch_with_progress(ids, 10)
            .await
            .unwrap();

        assert_eq!(result.total_processed, 25);
        assert_eq!(result.succeeded, 25);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded);
    }

    #[tokio::test]
    async fn test_delete_heuristic_single() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let heuristic = create_test_heuristic("1");
        storage.store_heuristic(&heuristic).await.unwrap();

        let result = storage.delete_heuristic(heuristic.heuristic_id).await;
        assert!(result.is_ok());

        let retrieved = storage.get_heuristic(heuristic.heuristic_id).await.unwrap();
        assert!(retrieved.is_none());
    }
}
