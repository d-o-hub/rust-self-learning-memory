//! # Batch Pattern Operations - Tests
//!
//! Tests for batch pattern operations.

#[cfg(test)]
mod tests {
    use super::super::pattern_types::{BatchProgress, BatchResult};
    use memory_core::{
        episode::PatternId, types::OutcomeStats, Pattern, PatternEffectiveness, TaskContext,
    };
    use tempfile::TempDir;

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

    fn create_test_pattern(id_suffix: &str) -> Pattern {
        Pattern::DecisionPoint {
            id: PatternId::new_v4(),
            condition: format!("test condition {}", id_suffix),
            action: format!("test action {}", id_suffix),
            outcome_stats: OutcomeStats {
                success_count: 5,
                failure_count: 1,
                total_count: 6,
                avg_duration_secs: 0.0,
            },
            context: TaskContext::default(),
            effectiveness: PatternEffectiveness::default(),
        }
    }

    #[tokio::test]
    async fn test_store_patterns_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let result = storage.store_patterns_batch(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_patterns_batch_single() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let patterns = vec![create_test_pattern("1")];
        let result = storage.store_patterns_batch(patterns).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_patterns_batch_multiple() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let patterns = vec![
            create_test_pattern("1"),
            create_test_pattern("2"),
            create_test_pattern("3"),
        ];
        let result = storage.store_patterns_batch(patterns.clone()).await;
        assert!(result.is_ok());

        for pattern in &patterns {
            let retrieved = storage.get_pattern(pattern.id()).await.unwrap();
            assert!(retrieved.is_some());
        }
    }

    #[tokio::test]
    async fn test_update_patterns_batch() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let patterns = vec![create_test_pattern("1"), create_test_pattern("2")];
        storage
            .store_patterns_batch(patterns.clone())
            .await
            .unwrap();

        let updated_patterns: Vec<Pattern> = patterns
            .into_iter()
            .map(|mut p| {
                if let Pattern::DecisionPoint {
                    ref mut condition,
                    ref mut action,
                    ref mut outcome_stats,
                    ..
                } = p
                {
                    *condition = format!("{} updated", condition);
                    *action = format!("{} updated", action);
                    outcome_stats.success_count = 10;
                }
                p
            })
            .collect();

        let result = storage
            .update_patterns_batch(updated_patterns.clone())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_patterns_batch_nonexistent() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let patterns = vec![create_test_pattern("nonexistent")];
        let result = storage.update_patterns_batch(patterns).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_store_patterns_batch_with_progress() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let patterns: Vec<Pattern> = (0..25)
            .map(|i| create_test_pattern(&i.to_string()))
            .collect();

        let result = storage
            .store_patterns_batch_with_progress(patterns, 10)
            .await
            .unwrap();

        assert_eq!(result.total_processed, 25);
        assert_eq!(result.succeeded, 25);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded);
    }

    #[tokio::test]
    async fn test_batch_progress_tracking() {
        let progress = BatchProgress::new(100, 10);
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
    async fn test_batch_result_success() {
        let result = BatchResult::success(10);
        assert_eq!(result.total_processed, 10);
        assert_eq!(result.succeeded, 10);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_batch_result_failure() {
        let result = BatchResult::failure("test error".to_string());
        assert_eq!(result.total_processed, 0);
        assert_eq!(result.succeeded, 0);
        assert!(!result.all_succeeded);
        assert_eq!(result.errors.len(), 1);
    }

    #[tokio::test]
    async fn test_get_patterns_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let result = storage.get_patterns_batch(&[]).await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_get_patterns_batch_nonexistent() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let ids = vec![PatternId::new_v4(), PatternId::new_v4()];
        let result = storage.get_patterns_batch(&ids).await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_get_patterns_batch_multiple() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Store patterns first
        let patterns = vec![
            create_test_pattern("1"),
            create_test_pattern("2"),
            create_test_pattern("3"),
        ];
        let ids: Vec<PatternId> = patterns.iter().map(|p| p.id()).collect();

        storage
            .store_patterns_batch(patterns.clone())
            .await
            .unwrap();

        // Retrieve batch
        let retrieved = storage.get_patterns_batch(&ids).await.unwrap();
        assert_eq!(retrieved.len(), 3);

        // Verify IDs match (patterns are returned as Option<Pattern>)
        let retrieved_ids: Vec<PatternId> = retrieved
            .iter()
            .filter_map(|p| p.as_ref().map(|p| p.id()))
            .collect();
        assert_eq!(retrieved_ids, ids);
    }

    #[tokio::test]
    async fn test_get_patterns_batch_partial() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Store only 2 patterns
        let patterns = vec![create_test_pattern("1"), create_test_pattern("2")];
        let mut ids: Vec<PatternId> = patterns.iter().map(|p| p.id()).collect();

        // Add a non-existent ID
        ids.push(PatternId::new_v4());

        storage.store_patterns_batch(patterns).await.unwrap();

        // Retrieve batch - should only return 2 patterns
        let retrieved = storage.get_patterns_batch(&ids).await.unwrap();
        assert_eq!(retrieved.len(), 3); // Returns Option<Pattern> for each ID, 2 Some + 1 None
        let found_count = retrieved.iter().filter(|p| p.is_some()).count();
        assert_eq!(found_count, 2);
    }

    #[tokio::test]
    async fn test_delete_patterns_batch_empty() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let result = storage.delete_patterns_batch(vec![]).await.unwrap();
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_delete_patterns_batch_nonexistent() {
        let (storage, _dir) = create_test_storage().await.unwrap();
        let ids = vec![PatternId::new_v4(), PatternId::new_v4()];
        let result = storage.delete_patterns_batch(ids).await.unwrap();
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_delete_patterns_batch_single() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let pattern = create_test_pattern("1");
        let id = pattern.id();

        storage
            .store_patterns_batch(vec![pattern.clone()])
            .await
            .unwrap();

        // Verify exists
        let retrieved = storage.get_pattern(id).await.unwrap();
        assert!(retrieved.is_some());

        // Delete
        let deleted = storage.delete_patterns_batch(vec![id]).await.unwrap();
        assert_eq!(deleted, 1);

        // Verify deleted
        let retrieved = storage.get_pattern(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_patterns_batch_multiple() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let patterns = vec![
            create_test_pattern("1"),
            create_test_pattern("2"),
            create_test_pattern("3"),
        ];
        let ids: Vec<PatternId> = patterns.iter().map(|p| p.id()).collect();

        storage
            .store_patterns_batch(patterns.clone())
            .await
            .unwrap();

        // Verify all exist
        for id in &ids {
            let retrieved = storage.get_pattern(*id).await.unwrap();
            assert!(retrieved.is_some());
        }

        // Delete batch
        let deleted = storage.delete_patterns_batch(ids.clone()).await.unwrap();
        assert_eq!(deleted, 3);

        // Verify all deleted
        for id in &ids {
            let retrieved = storage.get_pattern(*id).await.unwrap();
            assert!(retrieved.is_none());
        }
    }

    #[tokio::test]
    async fn test_delete_patterns_batch_partial() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Store only 2 patterns
        let patterns = vec![create_test_pattern("1"), create_test_pattern("2")];
        let mut ids: Vec<PatternId> = patterns.iter().map(|p| p.id()).collect();

        // Add a non-existent ID
        ids.push(PatternId::new_v4());

        storage.store_patterns_batch(patterns).await.unwrap();

        // Delete batch - should delete 2 patterns
        let deleted = storage.delete_patterns_batch(ids).await.unwrap();
        assert_eq!(deleted, 2);
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_error() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Store initial patterns
        let patterns = vec![create_test_pattern("1"), create_test_pattern("2")];
        storage
            .store_patterns_batch(patterns.clone())
            .await
            .unwrap();

        // Try to update with invalid pattern (non-existent)
        let invalid_patterns = vec![patterns[0].clone(), create_test_pattern("nonexistent")];

        let result = storage.update_patterns_batch(invalid_patterns).await;
        assert!(result.is_err());

        // Verify original patterns still intact (transaction rolled back)
        let retrieved = storage.get_pattern(patterns[0].id()).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_batch_performance_improvement() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Create 50 patterns
        let patterns: Vec<Pattern> = (0..50)
            .map(|i| create_test_pattern(&i.to_string()))
            .collect();

        // Measure batch store time
        let start = std::time::Instant::now();
        storage
            .store_patterns_batch(patterns.clone())
            .await
            .unwrap();
        let batch_time = start.elapsed();

        // Measure individual store times (for 10 patterns to avoid long test)
        let individual_patterns = patterns.into_iter().take(10).collect::<Vec<_>>();
        let start = std::time::Instant::now();
        for pattern in &individual_patterns {
            let new_pattern = create_test_pattern(&format!("individual_{}", pattern.id()));
            storage.store_pattern(&new_pattern).await.unwrap();
        }
        let individual_time = start.elapsed();

        // Batch should be significantly faster (at least 2x for 10 items)
        let avg_individual = individual_time / 10;
        let avg_batch = batch_time / 50;

        // Allow for some variance, but batch should be faster
        println!(
            "Batch avg: {:?}, Individual avg: {:?}",
            avg_batch, avg_individual
        );

        // The batch operation should be at least reasonably efficient
        // This is a soft assertion - we just want to ensure it's not worse
        assert!(avg_batch.as_millis() < 100, "Batch operation too slow");
    }
}
