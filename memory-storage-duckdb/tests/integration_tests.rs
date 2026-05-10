use chrono::{Duration, Utc};
use do_memory_core::monitoring::storage::MonitoringStorageBackend;
use do_memory_core::monitoring::types::{AgentMetrics, AgentType, ExecutionRecord, TaskMetrics};
use do_memory_core::types::RewardScore;
use do_memory_core::{Episode, StorageBackend, TaskContext, TaskType};
use do_memory_storage_duckdb::DuckDbStorage;
use std::collections::HashMap;
use tempfile::tempdir;

#[tokio::test]
async fn test_duckdb_storage_basic_ops() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    let episode = Episode::new(
        "Test task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );
    let episode_id = episode.episode_id;

    // Store
    storage.store_episode(&episode).await?;

    // Get
    let retrieved = storage.get_episode(episode_id).await?;
    assert!(retrieved.is_some(), "Episode should be retrieved");
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.episode_id, episode_id);
    assert_eq!(retrieved.task_description, "Test task");

    // Delete
    storage.delete_episode(episode_id).await?;
    let retrieved = storage.get_episode(episode_id).await?;
    assert!(retrieved.is_none(), "Episode should be deleted");

    Ok(())
}

#[tokio::test]
async fn test_duckdb_vss_search() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("test_vss.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    // Load VSS extension
    storage.load_vss_extension().await?;

    // Store some embeddings
    let vec1 = vec![1.0, 0.0, 0.0];
    let vec2 = vec![0.0, 1.0, 0.0];
    let vec3 = vec![0.9, 0.1, 0.0];

    storage.store_embedding("id1", vec1).await?;
    storage.store_embedding("id2", vec2).await?;
    storage.store_embedding("id3", vec3).await?;

    // Search for something similar to [1.0, 0.0, 0.0]
    let results = storage
        .search_embeddings_vss(vec![1.0, 0.0, 0.0], 2)
        .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["item_id"], "id1");
    assert_eq!(results[1]["item_id"], "id3");

    Ok(())
}

#[tokio::test]
async fn test_duckdb_monitoring_ops() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("test_monitoring.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    // 1. Execution Records
    let record = ExecutionRecord {
        agent_name: "test-agent".to_string(),
        agent_type: AgentType::GoapAgent,
        success: true,
        duration: std::time::Duration::from_millis(150),
        started_at: Utc::now(),
        task_description: Some("Test monitoring task".to_string()),
        error_message: None,
    };

    storage.store_execution_record(&record).await?;

    let records = storage
        .load_execution_records(Some("test-agent"), 10)
        .await?;
    assert_eq!(records.len(), 1, "Should have 1 execution record");
    assert_eq!(records[0].agent_name, "test-agent");

    // 2. Agent Metrics
    let metrics = AgentMetrics {
        agent_name: "test-agent".to_string(),
        agent_type: AgentType::GoapAgent,
        total_executions: 10,
        successful_executions: 9,
        total_duration: std::time::Duration::from_secs(2),
        avg_duration: std::time::Duration::from_millis(200),
        min_duration: std::time::Duration::from_millis(100),
        max_duration: std::time::Duration::from_millis(500),
        last_execution: Some(Utc::now()),
        current_streak: 5,
        longest_streak: 8,
    };

    storage.store_agent_metrics(&metrics).await?;

    let retrieved_metrics = storage.load_agent_metrics("test-agent").await?;
    assert!(retrieved_metrics.is_some());
    let retrieved_metrics = retrieved_metrics.unwrap();
    assert_eq!(retrieved_metrics.total_executions, 10);
    assert_eq!(retrieved_metrics.successful_executions, 9);

    // 3. Task Metrics
    let mut success_rates = HashMap::new();
    success_rates.insert(AgentType::GoapAgent, 0.9);

    let task_metrics = TaskMetrics {
        task_type: "CodeGeneration".to_string(),
        total_tasks: 5,
        completed_tasks: 4,
        avg_completion_time: std::time::Duration::from_secs(30),
        agent_success_rates: success_rates,
    };

    storage.store_task_metrics(&task_metrics).await?;

    let retrieved_task = storage.load_task_metrics("CodeGeneration").await?;
    assert!(retrieved_task.is_some());
    assert_eq!(retrieved_task.unwrap().total_tasks, 5);

    Ok(())
}

#[tokio::test]
async fn test_duckdb_analytics_queries() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("test_analytics.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    // Create a few episodes with specific data for analytics
    let mut ep1 = Episode::new(
        "Low reward task".to_string(),
        TaskContext {
            domain: "analytics".to_string(),
            ..Default::default()
        },
        TaskType::CodeGeneration,
    );
    ep1.reward = Some(RewardScore {
        total: 0.2,
        base: 0.2,
        efficiency: 1.0,
        complexity_bonus: 1.0,
        quality_multiplier: 1.0,
        learning_bonus: 0.0,
    });
    ep1.start_time = Utc::now() - Duration::hours(2);

    let mut ep2 = Episode::new(
        "High reward task".to_string(),
        TaskContext {
            domain: "analytics".to_string(),
            ..Default::default()
        },
        TaskType::CodeGeneration,
    );
    ep2.reward = Some(RewardScore {
        total: 0.9,
        base: 0.9,
        efficiency: 1.0,
        complexity_bonus: 1.0,
        quality_multiplier: 1.0,
        learning_bonus: 0.0,
    });
    ep2.start_time = Utc::now() - Duration::minutes(30);

    storage.store_episode(&ep1).await?;
    storage.store_episode(&ep2).await?;

    // 1. Session Windowing
    let window = storage.query_session_windowing(1).await?;
    assert!(!window.is_empty());
    assert_eq!(window[0]["domain"], "analytics");

    // 2. Temporal Decay
    let decay = storage.query_temporal_decay().await?;
    assert_eq!(decay.len(), 2);
    // ep2 (30 mins ago) should have higher score than ep1 (2 hours ago)
    let score1 = decay
        .iter()
        .find(|o| o["description"] == "Low reward task")
        .unwrap()["recency_score"]
        .as_f64()
        .unwrap();
    let score2 = decay
        .iter()
        .find(|o| o["description"] == "High reward task")
        .unwrap()["recency_score"]
        .as_f64()
        .unwrap();
    assert!(score2 > score1);

    // 3. Reward Distribution
    let dist = storage.query_reward_distribution().await?;
    assert!(dist["p50"].as_f64().is_some());
    // Median of 0.2 and 0.9 is around 0.55
    assert!(dist["p50"].as_f64().unwrap() > 0.4);

    Ok(())
}
