//! Integration tests for AgentMonitor storage integration

#[cfg(test)]
mod tests {
    use memory_core::memory::SelfLearningMemory;
    use memory_core::monitoring::{AgentMonitor, MonitoringConfig};
    use memory_core::monitoring::storage::SimpleMonitoringStorage;
    use memory_core::storage::StorageBackend;
    use memory_core::{TaskContext, TaskType};
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;

    /// Test that AgentMonitor can be created with storage backend
    #[tokio::test]
    async fn test_agent_monitor_with_storage_integration() {
        // Create a temporary directory for test storage
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_storage.redb");
        
        // Create a basic storage backend (redb for testing)
        let storage = memory_storage_redb::RedbStorage::new(&storage_path)
            .await
            .expect("Failed to create test storage");
        
        // Create monitoring storage wrapper
        let monitoring_storage = SimpleMonitoringStorage::new(Arc::new(storage));
        
        // Create AgentMonitor with storage backend
        let config = MonitoringConfig {
            enabled: true,
            enable_persistence: true,
            max_records: 100,
        };
        
        let agent_monitor = AgentMonitor::with_storage(
            config,
            Arc::new(monitoring_storage),
        );
        
        // Verify the monitor was created with storage
        assert!(agent_monitor.storage.is_some());
    }

    /// Test that AgentMonitor can record executions and persist them
    #[tokio::test]
    async fn test_agent_monitor_recording_with_storage() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_storage2.redb");
        
        let storage = memory_storage_redb::RedbStorage::new(&storage_path)
            .await
            .expect("Failed to create test storage");
        
        let monitoring_storage = SimpleMonitoringStorage::new(Arc::new(storage));
        
        let config = MonitoringConfig {
            enabled: true,
            enable_persistence: true,
            max_records: 100,
        };
        
        let agent_monitor = AgentMonitor::with_storage(
            config,
            Arc::new(monitoring_storage),
        );
        
        // Record some agent executions
        agent_monitor
            .record_execution("test-agent", true, Duration::from_secs(1))
            .await
            .expect("Failed to record execution");
            
        agent_monitor
            .record_execution("test-agent", false, Duration::from_secs(2))
            .await
            .expect("Failed to record execution");
        
        // Verify metrics are tracked
        let metrics = agent_monitor.get_agent_metrics("test-agent").await;
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_executions, 2);
        assert_eq!(metrics.successful_executions, 1);
        assert!((metrics.success_rate() - 0.5).abs() < 0.001);
    }

    /// Test integration with SelfLearningMemory system
    #[tokio::test]
    async fn test_self_learning_memory_agent_monitoring() {
        let temp_dir = tempdir().unwrap();
        let turso_path = temp_dir.path().join("test_turso.redb");
        let cache_path = temp_dir.path().join("test_cache.redb");
        
        // Create storage backends
        let turso_storage = memory_storage_redb::RedbStorage::new(&turso_path)
            .await
            .expect("Failed to create turso storage");
        let cache_storage = memory_storage_redb::RedbStorage::new(&cache_path)
            .await
            .expect("Failed to create cache storage");
        
        // Create SelfLearningMemory with storage
        let memory = SelfLearningMemory::with_storage(
            Default::default(),
            Arc::new(turso_storage),
            Arc::new(cache_storage),
        );
        
        // Verify agent monitoring is enabled
        assert!(memory.has_turso_storage());
        assert!(memory.has_cache_storage());
        
        // Record agent execution through memory system
        memory
            .record_agent_execution("feature-implementer", true, Duration::from_secs(5))
            .await
            .expect("Failed to record agent execution");
            
        memory
            .record_agent_execution("feature-implementer", false, Duration::from_secs(10))
            .await
            .expect("Failed to record agent execution");
        
        // Verify metrics are collected
        let metrics = memory.get_agent_metrics("feature-implementer").await;
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_executions, 2);
        assert_eq!(metrics.successful_executions, 1);
        
        // Verify summary statistics
        let summary = memory.get_monitoring_summary().await;
        assert_eq!(summary.total_agents, 1);
        assert_eq!(summary.total_executions, 2);
        assert_eq!(summary.successful_executions, 1);
        assert!((summary.success_rate - 0.5).abs() < 0.001);
    }

    /// Test detailed agent execution recording
    #[tokio::test]
    async fn test_detailed_agent_execution_recording() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_detailed.redb");
        
        let storage = memory_storage_redb::RedbStorage::new(&storage_path)
            .await
            .expect("Failed to create test storage");
        
        let monitoring_storage = SimpleMonitoringStorage::new(Arc::new(storage));
        
        let config = MonitoringConfig {
            enabled: true,
            enable_persistence: true,
            max_records: 100,
        };
        
        let agent_monitor = AgentMonitor::with_storage(
            config,
            Arc::new(monitoring_storage),
        );
        
        // Record detailed execution
        agent_monitor
            .record_execution_detailed(
                "code-analyzer",
                true,
                Duration::from_secs(3),
                Some("Analyze Rust code quality".to_string()),
                None,
            )
            .await
            .expect("Failed to record detailed execution");
            
        agent_monitor
            .record_execution_detailed(
                "code-analyzer",
                false,
                Duration::from_secs(8),
                Some("Analyze Python code".to_string()),
                Some("Failed to parse syntax".to_string()),
            )
            .await
            .expect("Failed to record failed execution");
        
        // Verify agent metrics
        let metrics = agent_monitor.get_agent_metrics("code-analyzer").await;
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_executions, 2);
        assert_eq!(metrics.successful_executions, 1);
        
        // Verify summary includes all agents
        let summary = agent_monitor.get_summary_stats().await;
        assert_eq!(summary.total_agents, 1);
        assert_eq!(summary.total_executions, 2);
    }

    /// Test monitoring can be disabled
    #[tokio::test]
    async fn test_monitoring_disabled() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_disabled.redb");
        
        let storage = memory_storage_redb::RedbStorage::new(&storage_path)
            .await
            .expect("Failed to create test storage");
        
        let monitoring_storage = SimpleMonitoringStorage::new(Arc::new(storage));
        
        // Create monitor with monitoring disabled
        let config = MonitoringConfig {
            enabled: false,
            enable_persistence: false,
            max_records: 100,
        };
        
        let agent_monitor = AgentMonitor::with_storage(
            config,
            Arc::new(monitoring_storage),
        );
        
        // Record execution (should be ignored when disabled)
        agent_monitor
            .record_execution("disabled-agent", true, Duration::from_secs(1))
            .await
            .expect("Failed to record execution");
        
        // Verify no metrics are tracked when disabled
        let metrics = agent_monitor.get_agent_metrics("disabled-agent").await;
        assert!(metrics.is_none());
        
        let summary = agent_monitor.get_summary_stats().await;
        assert_eq!(summary.total_agents, 0);
        assert_eq!(summary.total_executions, 0);
    }
}
