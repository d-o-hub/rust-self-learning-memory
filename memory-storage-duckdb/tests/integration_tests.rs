use anyhow::Result;
use chrono::Utc;
use do_memory_core::monitoring::types::{AgentMetrics, AgentType, ExecutionRecord, TaskMetrics};
use do_memory_core::storage::{MonitoringStorageBackend, StorageBackend};
use do_memory_core::types::learning::Pattern;
use do_memory_storage_duckdb::DuckDbStorage;
use std::collections::HashMap;
use std::time::Duration;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_duckdb_monitoring_storage() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let storage = DuckDbStorage::new(db_file.path()).await?;

    // Test execution records
    let record = ExecutionRecord {
        agent_name: "test-agent".to_string(),
        agent_type: AgentType::Orchestrator,
        success: true,
        duration: Duration::from_millis(150),
        started_at: Utc::now(),
        task_description: Some("test task".to_string()),
        error_message: None,
    };

    storage.store_execution_record(&record).await?;
    let records = storage.load_execution_records(Some("test-agent"), 10).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].agent_name, "test-agent");

    // Test agent metrics
    let metrics = AgentMetrics {
        agent_name: "test-agent".to_string(),
        agent_type: AgentType::Orchestrator,
        total_executions: 10,
        successful_executions: 8,
        total_duration: Duration::from_secs(2),
        avg_duration: Duration::from_millis(200),
        min_duration: Duration::from_millis(100),
        max_duration: Duration::from_millis(500),
        last_execution: Some(Utc::now()),
        current_streak: 3,
        longest_streak: 5,
    };

    storage.store_agent_metrics(&metrics).await?;
    let retrieved = storage.load_agent_metrics("test-agent").await?;
    let retrieved = retrieved.ok_or_else(|| anyhow::anyhow!("Metrics not found"))?;
    assert_eq!(retrieved.total_executions, 10);
    assert_eq!(retrieved.current_streak, 3);
    assert_eq!(retrieved.longest_streak, 5);

    // Test task metrics
    let mut success_rates = HashMap::new();
    success_rates.insert("agent-1".to_string(), 0.85);
    let task_metrics = TaskMetrics {
        task_type: "translation".to_string(),
        total_tasks: 5,
        completed_tasks: 4,
        avg_completion_time: Duration::from_secs(1),
        agent_success_rates: success_rates,
    };

    storage.store_task_metrics(&task_metrics).await?;
    let retrieved_task = storage.load_task_metrics("translation").await?;
    assert_eq!(
        retrieved_task
            .ok_or_else(|| anyhow::anyhow!("Task metrics not found"))?
            .total_tasks,
        5
    );

    Ok(())
}

#[tokio::test]
async fn test_duckdb_analytics() -> Result<()> {
    let db_file = NamedTempFile::new()?;
    let storage = DuckDbStorage::new(db_file.path()).await?;

    // Add some test data for analytics
    // 1. Episodes with rewards
    // Since DuckDbStorage implements StorageBackend, we can use store_episode
    // But for a quick test, let's just use execute on the connection if needed,
    // or better, use the public API.

    // 2. Patterns
    let pattern = Pattern {
        id: "p1".to_string(),
        success_rate: 0.95,
        metadata: serde_json::json!({"domain": "coding"}),
        ..Default::default()
    };
    storage.store_pattern(&pattern).await?;

    // Test analytics query
    let results = storage
        .run_analytics_query("SELECT COUNT(*) as count FROM patterns")
        .await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["count"].as_i64(), Some(1));

    Ok(())
}
