use anyhow::Result;
use chrono::Utc;
use do_memory_core::monitoring::storage::MonitoringStorageBackend;
use do_memory_core::monitoring::types::{AgentMetrics, AgentType, ExecutionRecord, TaskMetrics};
use do_memory_core::storage::StorageBackend;
use do_memory_storage_duckdb::DuckDbStorage;
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_duckdb_monitoring_storage() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    // Test execution records
    let record = ExecutionRecord {
        agent_name: "test-agent".to_string(),
        agent_type: AgentType::GoapAgent,
        success: true,
        duration: Duration::from_millis(150),
        started_at: Utc::now(),
        task_description: Some("test task".to_string()),
        error_message: None,
    };

    storage.store_execution_record(&record).await?;
    let records = storage
        .load_execution_records(Some("test-agent"), 10)
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].agent_name, "test-agent");

    // Test agent metrics
    let metrics = AgentMetrics {
        agent_name: "test-agent".to_string(),
        agent_type: AgentType::GoapAgent,
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
    success_rates.insert(AgentType::GoapAgent, 0.85);
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
async fn test_duckdb_episode_crud() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db_episodes.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    let episode_id = uuid::Uuid::new_v4();
    let mut episode = do_memory_core::Episode::new(
        "test episode".to_string(),
        do_memory_core::TaskContext {
            domain: "testing".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        do_memory_core::types::TaskType::CodeGeneration,
    );
    episode.episode_id = episode_id;
    episode
        .add_tag("test-tag".to_string())
        .map_err(anyhow::Error::msg)?;
    episode
        .add_tag("duckdb".to_string())
        .map_err(anyhow::Error::msg)?;

    // Store
    storage.store_episode(&episode).await?;

    // Retrieve
    let retrieved = storage.get_episode(episode_id).await?;
    let retrieved = retrieved.ok_or_else(|| anyhow::anyhow!("Episode not found"))?;
    assert_eq!(retrieved.task_description, "test episode");
    assert_eq!(retrieved.tags.len(), 2);
    assert!(retrieved.tags.contains(&"test-tag".to_string()));
    assert!(retrieved.tags.contains(&"duckdb".to_string()));

    // Query by metadata
    episode
        .metadata
        .insert("key".to_string(), "value".to_string());
    storage.store_episode(&episode).await?; // Update
    let results = storage
        .query_episodes_by_metadata("key", "value", None)
        .await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].episode_id, episode_id);

    // Delete
    storage.delete_episode(episode_id).await?;
    let deleted = storage.get_episode(episode_id).await?;
    assert!(deleted.is_none());

    Ok(())
}

#[tokio::test]
async fn test_duckdb_relationships() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db_relationships.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    let from_id = uuid::Uuid::new_v4();
    let to_id = uuid::Uuid::new_v4();
    let rel = do_memory_core::episode::EpisodeRelationship {
        id: uuid::Uuid::new_v4(),
        from_episode_id: from_id,
        to_episode_id: to_id,
        relationship_type: do_memory_core::episode::RelationshipType::DependsOn,
        created_at: Utc::now(),
        metadata: do_memory_core::episode::RelationshipMetadata {
            reason: Some("test reason".to_string()),
            created_by: Some("test-suite".to_string()),
            priority: Some(1),
            ..Default::default()
        },
    };

    storage.store_relationship(&rel).await?;
    assert!(
        storage
            .relationship_exists(
                from_id,
                to_id,
                do_memory_core::episode::RelationshipType::DependsOn
            )
            .await?
    );

    let rels = storage
        .get_relationships(from_id, do_memory_core::episode::Direction::Outgoing)
        .await?;
    assert_eq!(rels.len(), 1);
    assert_eq!(rels[0].metadata.reason, Some("test reason".to_string()));

    storage.remove_relationship(rel.id).await?;
    assert!(
        !storage
            .relationship_exists(
                from_id,
                to_id,
                do_memory_core::episode::RelationshipType::DependsOn
            )
            .await?
    );

    Ok(())
}

#[tokio::test]
async fn test_duckdb_embeddings() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db_embeddings.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    let id = "test-item";
    let embedding = vec![0.1f32, 0.2, 0.3];

    storage.store_embedding(id, embedding.clone()).await?;
    let retrieved = storage.get_embedding(id).await?;
    assert_eq!(retrieved, Some(embedding));

    storage.delete_embedding(id).await?;
    let deleted = storage.get_embedding(id).await?;
    assert!(deleted.is_none());

    Ok(())
}

#[tokio::test]
async fn test_duckdb_analytics_full() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db_analytics_full.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    // Add some test episodes for analytics
    let episode = do_memory_core::Episode::new(
        "analytics test".to_string(),
        do_memory_core::TaskContext {
            domain: "coding".to_string(),
            ..Default::default()
        },
        do_memory_core::types::TaskType::CodeGeneration,
    );
    storage.store_episode(&episode).await?;

    // Add a pattern for trends
    let pattern = do_memory_core::Pattern::ToolSequence {
        id: uuid::Uuid::new_v4(),
        tools: vec!["tool1".to_string()],
        context: do_memory_core::TaskContext {
            domain: "coding".to_string(),
            ..Default::default()
        },
        success_rate: 1.0,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 1,
        effectiveness: Default::default(),
    };
    storage.store_pattern(&pattern).await?;

    // Test session windowing
    let results = storage.query_session_windowing(1).await?;
    assert!(!results.is_empty());

    // Test temporal decay
    let results = storage.query_temporal_decay().await?;
    assert!(!results.is_empty());

    // Test reward distribution
    let result = storage.query_reward_distribution().await?;
    assert!(!result.is_null());

    // Test pattern trends
    let result = storage.query_pattern_trends().await?;
    assert!(!result.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_duckdb_heuristics() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db_heuristics.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    let now = Utc::now();
    let heuristic = do_memory_core::Heuristic {
        heuristic_id: uuid::Uuid::new_v4(),
        condition: "test condition".to_string(),
        action: "test action".to_string(),
        confidence: 0.8,
        evidence: do_memory_core::types::Evidence {
            episode_ids: vec![uuid::Uuid::new_v4()],
            success_rate: 0.8,
            sample_size: 1,
        },
        created_at: now,
        updated_at: now,
    };

    storage.store_heuristic(&heuristic).await?;
    let retrieved = storage.get_heuristic(heuristic.heuristic_id).await?;
    assert_eq!(
        retrieved.map(|h| h.heuristic_id),
        Some(heuristic.heuristic_id)
    );

    Ok(())
}

#[tokio::test]
async fn test_duckdb_recommendations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db_recs.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    let episode_id = uuid::Uuid::new_v4();
    let session_id = uuid::Uuid::new_v4();

    let session = do_memory_core::memory::attribution::RecommendationSession {
        session_id,
        episode_id,
        timestamp: Utc::now(),
        recommended_pattern_ids: vec!["pattern-1".to_string()],
        recommended_playbook_ids: vec![uuid::Uuid::new_v4()],
    };

    storage.store_recommendation_session(&session).await?;
    let retrieved = storage.get_recommendation_session(session_id).await?;
    assert_eq!(retrieved, Some(session.clone()));

    let feedback = do_memory_core::memory::attribution::RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["pattern-1".to_string()],
        consulted_episode_ids: vec![episode_id],
        outcome: do_memory_core::types::TaskOutcome::Success {
            verdict: "test".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(1.0),
    };

    storage.store_recommendation_feedback(&feedback).await?;
    let retrieved_feedback = storage.get_recommendation_feedback(session_id).await?;
    assert_eq!(retrieved_feedback, Some(feedback));

    Ok(())
}

#[tokio::test]
async fn test_duckdb_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db_patterns.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    let pattern = do_memory_core::Pattern::ToolSequence {
        id: uuid::Uuid::new_v4(),
        tools: vec!["tool1".to_string(), "tool2".to_string()],
        context: do_memory_core::TaskContext {
            domain: "coding".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        success_rate: 0.9,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: Default::default(),
    };

    storage.store_pattern(&pattern).await?;
    let retrieved = storage.get_pattern(pattern.id()).await?;
    assert_eq!(retrieved.map(|p| p.id()), Some(pattern.id()));

    Ok(())
}
