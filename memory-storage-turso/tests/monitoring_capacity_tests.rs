//! Comprehensive tests for monitoring.rs and capacity.rs modules
//!
//! NOTE: All tests that create `TursoStorage` are ignored due to a memory corruption bug
//! in the libsql/turso native library that causes `malloc_consolidate(): unaligned fastbin chunk detected`
//! in CI environments. See ADR-027 for details.
//!
//! Coverage targets:
//! - monitoring.rs: 50% (~214 lines from 427 LOC)
//! - capacity.rs: 50% (~109 lines from 217 LOC)

#![allow(clippy::float_cmp)]
#![allow(clippy::doc_markdown)]
#![allow(missing_docs)]

use do_memory_core::monitoring::types::{AgentMetrics, AgentType, ExecutionRecord, TaskMetrics};
use do_memory_core::{Episode, TaskContext, TaskOutcome, TaskType};
use do_memory_storage_turso::{CapacityStatistics, TursoStorage};
use std::time::Duration;
use tempfile::TempDir;

// ========== Test Helpers ==========

/// Helper to create a test storage instance with initialized schema
async fn create_test_storage() -> anyhow::Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");
    let db_url = format!("file:{}", db_path.display());

    let storage = TursoStorage::new(&db_url, "").await?;
    storage.initialize_schema().await?;

    Ok((storage, dir))
}

/// Helper to create a test execution record
fn create_test_execution_record(
    agent_name: &str,
    agent_type: AgentType,
    success: bool,
) -> ExecutionRecord {
    ExecutionRecord::new(
        agent_name.to_string(),
        agent_type,
        success,
        Duration::from_millis(100),
        Some(format!("Test task for {agent_name}")),
        if success {
            None
        } else {
            Some("Test error message".to_string())
        },
    )
}

/// Helper to create a test agent metrics instance
fn create_test_agent_metrics(agent_name: &str, agent_type: AgentType) -> AgentMetrics {
    AgentMetrics {
        agent_name: agent_name.to_string(),
        agent_type,
        total_executions: 10,
        successful_executions: 8,
        total_duration: Duration::from_secs(1),
        avg_duration: Duration::from_millis(100),
        min_duration: Duration::from_millis(50),
        max_duration: Duration::from_millis(200),
        last_execution: Some(chrono::Utc::now()),
        current_streak: 3,
        longest_streak: 5,
    }
}

/// Helper to create a test task metrics instance
fn create_test_task_metrics(task_type: &str) -> TaskMetrics {
    TaskMetrics {
        task_type: task_type.to_string(),
        total_tasks: 20,
        completed_tasks: 18,
        avg_completion_time: Duration::from_millis(150),
        agent_success_rates: std::collections::HashMap::new(),
    }
}

/// Helper to create a test episode
fn create_test_episode(task_desc: &str) -> Episode {
    let mut episode = Episode::new(
        task_desc.to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

    episode.complete(TaskOutcome::Success {
        verdict: "Task completed".to_string(),
        artifacts: vec![format!("{}.rs", task_desc)],
    });

    episode
}

// ========== Monitoring Storage Tests ==========

mod monitoring_tests {
    use super::*;

    // === Execution Record Tests ===

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_execution_record_success() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let record =
            create_test_execution_record("test_agent", AgentType::FeatureImplementer, true);

        // Act
        let result = storage.store_execution_record(&record).await;

        // Assert
        assert!(result.is_ok(), "Should successfully store execution record");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_execution_record_failure() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let record = create_test_execution_record("test_agent", AgentType::Debugger, false);

        // Act
        let result = storage.store_execution_record(&record).await;

        // Assert
        assert!(result.is_ok(), "Should store failed execution record");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_execution_record_all_agent_types() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let agent_types = [
            AgentType::FeatureImplementer,
            AgentType::CodeReviewer,
            AgentType::TestRunner,
            AgentType::ArchitectureValidator,
            AgentType::Debugger,
            AgentType::AnalysisSwarm,
            AgentType::GoapAgent,
            AgentType::Other,
        ];

        // Act & Assert
        for agent_type in agent_types {
            let record =
                create_test_execution_record(&format!("agent_{agent_type}"), agent_type, true);
            let result = storage.store_execution_record(&record).await;
            assert!(
                result.is_ok(),
                "Should store record for agent type {agent_type:?}"
            );
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_load_execution_records_by_agent() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let agent_name = "specific_agent";
        let record = create_test_execution_record(agent_name, AgentType::TestRunner, true);
        storage.store_execution_record(&record).await?;

        // Act
        let loaded = storage.load_execution_records(Some(agent_name), 10).await?;

        // Assert
        assert!(!loaded.is_empty(), "Should find stored execution records");
        assert_eq!(loaded[0].agent_name, agent_name, "Agent name should match");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_load_execution_records_all_agents() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let record1 = create_test_execution_record("agent1", AgentType::FeatureImplementer, true);
        let record2 = create_test_execution_record("agent2", AgentType::CodeReviewer, true);
        storage.store_execution_record(&record1).await?;
        storage.store_execution_record(&record2).await?;

        // Act
        let loaded = storage.load_execution_records(None, 10).await?;

        // Assert
        assert!(
            loaded.len() >= 2,
            "Should find at least 2 execution records"
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_load_execution_records_empty_result() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;

        // Act
        let loaded = storage
            .load_execution_records(Some("nonexistent_agent"), 10)
            .await?;

        // Assert
        assert!(
            loaded.is_empty(),
            "Should return empty list for nonexistent agent"
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_load_execution_records_with_limit() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        for i in 0..20 {
            let record =
                create_test_execution_record(&format!("agent_{i}"), AgentType::Other, true);
            storage.store_execution_record(&record).await?;
        }

        // Act
        let loaded = storage.load_execution_records(None, 5).await?;

        // Assert
        assert!(loaded.len() <= 5, "Should respect limit parameter");

        Ok(())
    }

    // === Agent Metrics Tests ===

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_agent_metrics() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let metrics = create_test_agent_metrics("test_metrics_agent", AgentType::GoapAgent);

        // Act
        let result = storage.store_agent_metrics(&metrics).await;

        // Assert
        assert!(result.is_ok(), "Should successfully store agent metrics");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_load_agent_metrics_existing() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let agent_name = "loadable_agent";
        let metrics = create_test_agent_metrics(agent_name, AgentType::FeatureImplementer);
        storage.store_agent_metrics(&metrics).await?;

        // Act
        let loaded = storage.load_agent_metrics(agent_name).await?;

        // Assert
        assert!(loaded.is_some(), "Should find stored agent metrics");
        let loaded_metrics = loaded.unwrap();
        assert_eq!(loaded_metrics.agent_name, agent_name);
        assert_eq!(loaded_metrics.agent_type, AgentType::FeatureImplementer);
        assert_eq!(loaded_metrics.total_executions, 10);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_load_agent_metrics_nonexistent() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;

        // Act
        let loaded = storage.load_agent_metrics("nonexistent_agent").await?;

        // Assert
        assert!(loaded.is_none(), "Should return None for nonexistent agent");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_agent_metrics_all_types() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let agent_types = [
            AgentType::FeatureImplementer,
            AgentType::CodeReviewer,
            AgentType::TestRunner,
            AgentType::ArchitectureValidator,
            AgentType::Debugger,
            AgentType::AnalysisSwarm,
            AgentType::GoapAgent,
            AgentType::Other,
        ];

        // Act & Assert
        for agent_type in agent_types {
            let metrics = create_test_agent_metrics(&format!("metrics_{agent_type}"), agent_type);
            let result = storage.store_agent_metrics(&metrics).await;
            assert!(
                result.is_ok(),
                "Should store metrics for agent type {agent_type:?}"
            );
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_update_agent_metrics() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let agent_name = "updateable_agent";
        let initial_metrics = create_test_agent_metrics(agent_name, AgentType::TestRunner);
        storage.store_agent_metrics(&initial_metrics).await?;

        // Act - Update with new metrics
        let updated_metrics = AgentMetrics {
            agent_name: agent_name.to_string(),
            agent_type: AgentType::TestRunner,
            total_executions: 20,
            successful_executions: 18,
            total_duration: Duration::from_secs(2),
            avg_duration: Duration::from_millis(100),
            min_duration: Duration::from_millis(30),
            max_duration: Duration::from_millis(300),
            last_execution: Some(chrono::Utc::now()),
            current_streak: 5,
            longest_streak: 10,
        };
        storage.store_agent_metrics(&updated_metrics).await?;

        // Assert
        let loaded = storage.load_agent_metrics(agent_name).await?;
        assert!(loaded.is_some());
        let loaded_metrics = loaded.unwrap();
        assert_eq!(
            loaded_metrics.total_executions, 20,
            "Metrics should be updated"
        );

        Ok(())
    }

    // === Task Metrics Tests ===

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_task_metrics() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let metrics = create_test_task_metrics("test_task_type");

        // Act
        let result = storage.store_task_metrics(&metrics).await;

        // Assert
        assert!(result.is_ok(), "Should successfully store task metrics");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_load_task_metrics_existing() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let task_type = "loadable_task";
        let metrics = create_test_task_metrics(task_type);
        storage.store_task_metrics(&metrics).await?;

        // Act
        let loaded = storage.load_task_metrics(task_type).await?;

        // Assert
        assert!(loaded.is_some(), "Should find stored task metrics");
        let loaded_metrics = loaded.unwrap();
        assert_eq!(loaded_metrics.task_type, task_type);
        assert_eq!(loaded_metrics.total_tasks, 20);
        assert_eq!(loaded_metrics.completed_tasks, 18);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_load_task_metrics_nonexistent() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;

        // Act
        let loaded = storage.load_task_metrics("nonexistent_task").await?;

        // Assert
        assert!(
            loaded.is_none(),
            "Should return None for nonexistent task type"
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_update_task_metrics() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let task_type = "updateable_task";
        let initial_metrics = create_test_task_metrics(task_type);
        storage.store_task_metrics(&initial_metrics).await?;

        // Act - Update with new metrics
        let updated_metrics = TaskMetrics {
            task_type: task_type.to_string(),
            total_tasks: 50,
            completed_tasks: 45,
            avg_completion_time: Duration::from_millis(200),
            agent_success_rates: std::collections::HashMap::new(),
        };
        storage.store_task_metrics(&updated_metrics).await?;

        // Assert
        let loaded = storage.load_task_metrics(task_type).await?;
        assert!(loaded.is_some());
        let loaded_metrics = loaded.unwrap();
        assert_eq!(
            loaded_metrics.total_tasks, 50,
            "Task metrics should be updated"
        );

        Ok(())
    }

    // === Edge Cases and Error Handling ===

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_execution_record_with_empty_description() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let record = ExecutionRecord::new(
            "no_desc_agent".to_string(),
            AgentType::Other,
            true,
            Duration::from_millis(50),
            None, // No description
            None,
        );

        // Act
        let result = storage.store_execution_record(&record).await;

        // Assert
        assert!(result.is_ok(), "Should store record without description");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_execution_record_with_zero_duration() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let record = ExecutionRecord::new(
            "instant_agent".to_string(),
            AgentType::Other,
            true,
            Duration::ZERO,
            Some("Instant execution".to_string()),
            None,
        );

        // Act
        let result = storage.store_execution_record(&record).await;

        // Assert
        assert!(result.is_ok(), "Should store record with zero duration");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_agent_metrics_with_zero_executions() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let metrics = AgentMetrics::default();

        // Act
        let result = storage.store_agent_metrics(&metrics).await;

        // Assert
        assert!(result.is_ok(), "Should store metrics with zero executions");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_task_metrics_with_zero_tasks() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let metrics = TaskMetrics::default();

        // Act
        let result = storage.store_task_metrics(&metrics).await;

        // Assert
        assert!(result.is_ok(), "Should store metrics with zero tasks");

        Ok(())
    }
}

// ========== Capacity Storage Tests ==========

mod capacity_tests {
    use super::*;

    // === CapacityStatistics Tests ===

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_get_capacity_statistics_empty() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;

        // Act
        let stats = storage.get_capacity_statistics().await?;

        // Assert
        assert_eq!(stats.episode_count, 0, "Episode count should be 0");
        assert_eq!(stats.pattern_count, 0, "Pattern count should be 0");
        assert_eq!(stats.heuristic_count, 0, "Heuristic count should be 0");
        assert_eq!(stats.embedding_count, 0, "Embedding count should be 0");
        assert_eq!(
            stats.execution_record_count, 0,
            "Execution record count should be 0"
        );
        assert_eq!(
            stats.agent_metrics_count, 0,
            "Agent metrics count should be 0"
        );
        assert_eq!(
            stats.task_metrics_count, 0,
            "Task metrics count should be 0"
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_get_capacity_statistics_with_episodes() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        for i in 0..5 {
            let episode = create_test_episode(&format!("task_{i}"));
            storage.store_episode(&episode).await?;
        }

        // Act
        let stats = storage.get_capacity_statistics().await?;

        // Assert
        assert_eq!(stats.episode_count, 5, "Episode count should be 5");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_get_capacity_statistics_with_monitoring_data() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let record = create_test_execution_record("agent", AgentType::Other, true);
        storage.store_execution_record(&record).await?;
        let metrics = create_test_agent_metrics("agent", AgentType::Other);
        storage.store_agent_metrics(&metrics).await?;
        let task_metrics = create_test_task_metrics("task");
        storage.store_task_metrics(&task_metrics).await?;

        // Act
        let stats = storage.get_capacity_statistics().await?;

        // Assert
        assert!(
            stats.execution_record_count >= 1,
            "Execution record count should be at least 1"
        );
        assert!(
            stats.agent_metrics_count >= 1,
            "Agent metrics count should be at least 1"
        );
        assert!(
            stats.task_metrics_count >= 1,
            "Task metrics count should be at least 1"
        );

        Ok(())
    }

    // === store_episode_with_capacity Tests ===

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_episode_with_capacity_under_limit() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let max_episodes = 10;

        // Act - Store 3 episodes (under capacity)
        for i in 0..3 {
            let episode = create_test_episode(&format!("under_limit_{i}"));
            storage
                .store_episode_with_capacity(&episode, max_episodes)
                .await?;
        }

        // Assert
        let stats = storage.get_capacity_statistics().await?;
        assert_eq!(stats.episode_count, 3, "All episodes should be stored");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_episode_with_capacity_at_limit() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let max_episodes = 3;

        // Act - Store exactly 3 episodes
        for i in 0..3 {
            let episode = create_test_episode(&format!("at_limit_{i}"));
            storage
                .store_episode_with_capacity(&episode, max_episodes)
                .await?;
        }

        // Assert
        let stats = storage.get_capacity_statistics().await?;
        assert_eq!(
            stats.episode_count, max_episodes,
            "Episode count should match limit"
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_episode_with_capacity_exceeds_limit() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let max_episodes = 3;

        // Act - Store 5 episodes (exceeds capacity by 2)
        for i in 0..5 {
            let episode = create_test_episode(&format!("exceeds_{i}"));
            storage
                .store_episode_with_capacity(&episode, max_episodes)
                .await?;
        }

        // Assert
        let stats = storage.get_capacity_statistics().await?;
        assert_eq!(
            stats.episode_count, max_episodes,
            "Should enforce capacity limit"
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_episode_with_capacity_large_limit() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let max_episodes = 100;

        // Act - Store 10 episodes with large limit
        for i in 0..10 {
            let episode = create_test_episode(&format!("large_limit_{i}"));
            storage
                .store_episode_with_capacity(&episode, max_episodes)
                .await?;
        }

        // Assert
        let stats = storage.get_capacity_statistics().await?;
        assert_eq!(
            stats.episode_count, 10,
            "All episodes should be stored with large limit"
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_store_episode_with_capacity_single_episode_limit() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let max_episodes = 1;

        // Act - Store multiple episodes with limit of 1
        for i in 0..5 {
            let episode = create_test_episode(&format!("single_limit_{i}"));
            storage
                .store_episode_with_capacity(&episode, max_episodes)
                .await?;
        }

        // Assert
        let stats = storage.get_capacity_statistics().await?;
        assert_eq!(stats.episode_count, 1, "Should keep only 1 episode");

        Ok(())
    }

    // === CapacityStatistics Display Test ===

    #[test]
    fn test_capacity_statistics_display() {
        // Arrange
        let stats = CapacityStatistics {
            episode_count: 10,
            pattern_count: 5,
            heuristic_count: 3,
            embedding_count: 20,
            execution_record_count: 100,
            agent_metrics_count: 8,
            task_metrics_count: 15,
        };

        // Act
        let display_str = stats.to_string();

        // Assert
        assert!(display_str.contains("episodes=10"));
        assert!(display_str.contains("patterns=5"));
        assert!(display_str.contains("heuristics=3"));
        assert!(display_str.contains("embeddings=20"));
        assert!(display_str.contains("CapacityStatistics"));
    }

    #[test]
    fn test_capacity_statistics_display_zero_counts() {
        // Arrange
        let stats = CapacityStatistics {
            episode_count: 0,
            pattern_count: 0,
            heuristic_count: 0,
            embedding_count: 0,
            execution_record_count: 0,
            agent_metrics_count: 0,
            task_metrics_count: 0,
        };

        // Act
        let display_str = stats.to_string();

        // Assert
        assert!(display_str.contains("episodes=0"));
        assert!(display_str.contains("patterns=0"));
    }

    // === Integration: Monitoring + Capacity ===

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_capacity_statistics_reflects_all_tables() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;

        // Store data in multiple tables
        let episode = create_test_episode("integration_episode");
        storage.store_episode(&episode).await?;

        let record = create_test_execution_record("integration_agent", AgentType::Other, true);
        storage.store_execution_record(&record).await?;

        let agent_metrics = create_test_agent_metrics("integration_agent", AgentType::Other);
        storage.store_agent_metrics(&agent_metrics).await?;

        let task_metrics = create_test_task_metrics("integration_task");
        storage.store_task_metrics(&task_metrics).await?;

        // Act
        let stats = storage.get_capacity_statistics().await?;

        // Assert - Verify all tables are counted
        assert!(stats.episode_count >= 1);
        assert!(stats.execution_record_count >= 1);
        assert!(stats.agent_metrics_count >= 1);
        assert!(stats.task_metrics_count >= 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Memory corruption bug in libsql native library - malloc_consolidate() unaligned fastbin chunk in CI"]
    async fn test_eviction_preserves_monitoring_data() -> anyhow::Result<()> {
        // Arrange
        let (storage, _dir) = create_test_storage().await?;
        let max_episodes = 2;

        // Store monitoring data
        let record = create_test_execution_record("eviction_test_agent", AgentType::Other, true);
        storage.store_execution_record(&record).await?;

        // Store episodes exceeding capacity
        for i in 0..5 {
            let episode = create_test_episode(&format!("eviction_episode_{i}"));
            storage
                .store_episode_with_capacity(&episode, max_episodes)
                .await?;
        }

        // Act
        let stats = storage.get_capacity_statistics().await?;

        // Assert - Episodes should be at capacity, but monitoring data preserved
        assert_eq!(
            stats.episode_count, max_episodes,
            "Episodes should be at capacity"
        );
        assert!(
            stats.execution_record_count >= 1,
            "Execution records should not be evicted"
        );

        Ok(())
    }
}

// ========== AgentType Tests ==========

mod agent_type_tests {
    use do_memory_core::monitoring::types::AgentType;

    #[test]
    fn test_agent_type_display() {
        assert_eq!(
            AgentType::FeatureImplementer.to_string(),
            "feature-implementer"
        );
        assert_eq!(AgentType::CodeReviewer.to_string(), "code-reviewer");
        assert_eq!(AgentType::TestRunner.to_string(), "test-runner");
        assert_eq!(
            AgentType::ArchitectureValidator.to_string(),
            "architecture-validator"
        );
        assert_eq!(AgentType::Debugger.to_string(), "debugger");
        assert_eq!(AgentType::AnalysisSwarm.to_string(), "analysis-swarm");
        assert_eq!(AgentType::GoapAgent.to_string(), "goap-agent");
        assert_eq!(AgentType::Other.to_string(), "other");
    }

    #[test]
    fn test_agent_type_from_str() {
        assert_eq!(
            AgentType::from("feature-implementer"),
            AgentType::FeatureImplementer
        );
        assert_eq!(AgentType::from("code-reviewer"), AgentType::CodeReviewer);
        assert_eq!(AgentType::from("test-runner"), AgentType::TestRunner);
        assert_eq!(
            AgentType::from("architecture-validator"),
            AgentType::ArchitectureValidator
        );
        assert_eq!(AgentType::from("debugger"), AgentType::Debugger);
        assert_eq!(AgentType::from("analysis-swarm"), AgentType::AnalysisSwarm);
        assert_eq!(AgentType::from("goap-agent"), AgentType::GoapAgent);
        assert_eq!(AgentType::from("unknown"), AgentType::Other);
        assert_eq!(AgentType::from(""), AgentType::Other);
    }

    #[test]
    fn test_agent_type_roundtrip() {
        for agent_type in [
            AgentType::FeatureImplementer,
            AgentType::CodeReviewer,
            AgentType::TestRunner,
            AgentType::ArchitectureValidator,
            AgentType::Debugger,
            AgentType::AnalysisSwarm,
            AgentType::GoapAgent,
            AgentType::Other,
        ] {
            let display = agent_type.to_string();
            let parsed = AgentType::from(display.as_str());
            assert_eq!(parsed, agent_type, "AgentType should roundtrip correctly");
        }
    }
}

// ========== ExecutionRecord Tests ==========

mod execution_record_tests {
    use do_memory_core::monitoring::types::{AgentType, ExecutionRecord};
    use std::time::Duration;

    #[test]
    fn test_execution_record_creation() {
        let record = ExecutionRecord::new(
            "test_agent".to_string(),
            AgentType::FeatureImplementer,
            true,
            Duration::from_millis(100),
            Some("Test task".to_string()),
            None,
        );

        assert_eq!(record.agent_name, "test_agent");
        assert_eq!(record.agent_type, AgentType::FeatureImplementer);
        assert!(record.success);
        assert_eq!(record.duration, Duration::from_millis(100));
        assert_eq!(record.task_description, Some("Test task".to_string()));
        assert!(record.error_message.is_none());
    }

    #[test]
    fn test_execution_record_failure() {
        let record = ExecutionRecord::new(
            "failed_agent".to_string(),
            AgentType::Debugger,
            false,
            Duration::from_millis(200),
            Some("Debugging task".to_string()),
            Some("Error: something went wrong".to_string()),
        );

        assert!(!record.success);
        assert!(record.error_message.is_some());
        assert!(record.error_message.unwrap().contains("Error"));
    }
}

// ========== AgentMetrics Tests ==========

mod agent_metrics_tests {
    use do_memory_core::monitoring::types::{AgentMetrics, AgentType, ExecutionRecord};
    use std::time::Duration;

    #[test]
    fn test_agent_metrics_default() {
        let metrics = AgentMetrics::default();

        assert!(metrics.agent_name.is_empty());
        assert_eq!(metrics.agent_type, AgentType::Other);
        assert_eq!(metrics.total_executions, 0);
        assert_eq!(metrics.successful_executions, 0);
        assert_eq!(metrics.total_duration, Duration::ZERO);
        assert_eq!(metrics.avg_duration, Duration::ZERO);
        assert_eq!(metrics.min_duration, Duration::MAX);
        assert_eq!(metrics.max_duration, Duration::ZERO);
        assert!(metrics.last_execution.is_none());
        assert_eq!(metrics.current_streak, 0);
        assert_eq!(metrics.longest_streak, 0);
    }

    #[test]
    fn test_agent_metrics_success_rate() {
        let metrics = AgentMetrics {
            agent_name: "test".to_string(),
            agent_type: AgentType::Other,
            total_executions: 10,
            successful_executions: 8,
            total_duration: Duration::from_secs(1),
            avg_duration: Duration::from_millis(100),
            min_duration: Duration::from_millis(50),
            max_duration: Duration::from_millis(200),
            last_execution: None,
            current_streak: 3,
            longest_streak: 5,
        };

        assert_eq!(metrics.success_rate(), 0.8);
    }

    #[test]
    fn test_agent_metrics_success_rate_zero_executions() {
        let metrics = AgentMetrics::default();
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_agent_metrics_avg_duration_secs() {
        let metrics = AgentMetrics {
            avg_duration: Duration::from_secs(5),
            ..AgentMetrics::default()
        };

        assert_eq!(metrics.avg_duration_secs(), 5.0);
    }

    #[test]
    fn test_agent_metrics_update_success() {
        let mut metrics = AgentMetrics::default();
        let record = ExecutionRecord::new(
            "test".to_string(),
            AgentType::Other,
            true,
            Duration::from_millis(100),
            None,
            None,
        );

        metrics.update(&record);

        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 1);
        assert_eq!(metrics.total_duration, Duration::from_millis(100));
        assert_eq!(metrics.min_duration, Duration::from_millis(100));
        assert_eq!(metrics.max_duration, Duration::from_millis(100));
        assert_eq!(metrics.current_streak, 1);
        assert_eq!(metrics.longest_streak, 1);
    }

    #[test]
    fn test_agent_metrics_update_failure() {
        let mut metrics = AgentMetrics {
            current_streak: 5,
            longest_streak: 5,
            ..AgentMetrics::default()
        };
        let record = ExecutionRecord::new(
            "test".to_string(),
            AgentType::Other,
            false,
            Duration::from_millis(100),
            None,
            Some("Error".to_string()),
        );

        metrics.update(&record);

        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 0);
        assert_eq!(metrics.current_streak, 0, "Streak should reset on failure");
        assert_eq!(
            metrics.longest_streak, 5,
            "Longest streak should not change"
        );
    }
}

// ========== TaskMetrics Tests ==========

mod task_metrics_tests {
    use do_memory_core::monitoring::types::{AgentType, ExecutionRecord, TaskMetrics};
    use std::time::Duration;

    #[test]
    fn test_task_metrics_default() {
        let metrics = TaskMetrics::default();

        assert!(metrics.task_type.is_empty());
        assert_eq!(metrics.total_tasks, 0);
        assert_eq!(metrics.completed_tasks, 0);
        assert_eq!(metrics.avg_completion_time, Duration::ZERO);
        assert!(metrics.agent_success_rates.is_empty());
    }

    #[test]
    fn test_task_metrics_success_rate() {
        let metrics = TaskMetrics {
            task_type: "test".to_string(),
            total_tasks: 20,
            completed_tasks: 15,
            avg_completion_time: Duration::from_millis(100),
            agent_success_rates: std::collections::HashMap::new(),
        };

        assert_eq!(metrics.success_rate(), 0.75);
    }

    #[test]
    fn test_task_metrics_success_rate_zero_tasks() {
        let metrics = TaskMetrics::default();
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_task_metrics_update() {
        let mut metrics = TaskMetrics::default();
        let record = ExecutionRecord::new(
            "test".to_string(),
            AgentType::FeatureImplementer,
            true,
            Duration::from_millis(100),
            None,
            None,
        );

        metrics.update(&record);

        assert_eq!(metrics.total_tasks, 1);
        assert_eq!(metrics.completed_tasks, 1);
        assert!(
            metrics
                .agent_success_rates
                .contains_key(&AgentType::FeatureImplementer)
        );
    }

    #[test]
    fn test_task_metrics_update_failure() {
        let mut metrics = TaskMetrics::default();
        let record = ExecutionRecord::new(
            "test".to_string(),
            AgentType::Debugger,
            false,
            Duration::from_millis(100),
            None,
            Some("Error".to_string()),
        );

        metrics.update(&record);

        assert_eq!(metrics.total_tasks, 1);
        assert_eq!(metrics.completed_tasks, 0);
        assert!(
            metrics
                .agent_success_rates
                .contains_key(&AgentType::Debugger)
        );
    }
}
