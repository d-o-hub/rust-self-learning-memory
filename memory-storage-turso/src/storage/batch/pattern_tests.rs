//! # Batch Pattern Operations - Tests
//!
//! Tests for batch pattern operations.

#[cfg(test)]
mod tests {
    use super::super::pattern_types::{BatchProgress, BatchResult};
    use super::super::*;
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
}
