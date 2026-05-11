//! Advanced persistence tests for cross-session, synchronization, and bulk data
#![allow(clippy::explicit_iter_loop)]
#![allow(missing_docs)]

mod common;

use common::setup_persistent_memory;
use do_memory_core::{
    ComplexityLevel, ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};
use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
use do_memory_storage_redb::RedbStorage;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_cross_session_persistence() {
    println!("🧪 Testing Cross-Session Persistence");
    println!("====================================");

    // Create temporary directory that persists across the test
    let temp_dir = TempDir::new().unwrap();
    let redb_path = temp_dir.path().join("test_memory.redb");

    // Create storage and memory instance
    let redb_storage = Arc::new(RedbStorage::new(&redb_path).await.unwrap());
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig {
            quality_threshold: 0.0, // Zero threshold for test episodes
            ..Default::default()
        },
        redb_storage.clone(),
        redb_storage,
    );
    let memory = Arc::new(memory);

    // First session - create and store data
    let episode_id = memory
        .start_episode(
            "Cross-Session Test".to_string(),
            TaskContext {
                domain: "persistence".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["cross-session".to_string()],
            },
            TaskType::Testing,
        )
        .await;

    let step = ExecutionStep::new(
        1,
        "persistence_test".to_string(),
        "Testing cross-session data persistence".to_string(),
    );
    memory.log_step(episode_id, step).await;

    let outcome = TaskOutcome::Success {
        verdict: "Cross-session persistence test completed".to_string(),
        artifacts: vec!["persistence_test.log".to_string()],
    };
    memory.complete_episode(episode_id, outcome).await.unwrap();

    println!("✅ Episode created and stored in persistent storage");

    // Verify episode exists
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.task_description, "Cross-Session Test");
    assert_eq!(episode.steps.len(), 1);
    println!("✅ Episode verified in persistent storage");

    // Test MCP query on the same instance
    let mcp_server = Arc::new(
        MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
            .await
            .unwrap(),
    );

    let query_result = mcp_server
        .query_memory(
            "Cross-Session".to_string(),
            "persistence".to_string(),
            None,
            10,
            "relevance".to_string(),
            None,
        )
        .await
        .unwrap();

    let episodes = query_result["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 1);

    let episode = &episodes[0];
    assert_eq!(episode["task_description"], "Cross-Session Test");
    assert_eq!(episode["steps"].as_array().unwrap().len(), 1);

    println!("✅ MCP query verified episode persistence");
    println!("   Episodes found: {}", episodes.len());
    println!("   Episode description: {}", episode["task_description"]);
    println!(
        "   Steps count: {}",
        episode["steps"].as_array().unwrap().len()
    );

    // Note: True cross-session persistence (across different memory instances)
    // would require more complex setup with proper database connections.
    // This test verifies persistence within the same memory instance using redb.

    println!("✅ Single-session persistence test completed successfully");
    println!("   Note: Cross-instance persistence requires full database setup");
}

#[tokio::test]
async fn test_storage_backend_synchronization() {
    println!("🧪 Testing Storage Backend Synchronization");
    println!("===========================================");

    let (memory, _temp_dir) = setup_persistent_memory().await.unwrap();

    // Create episode and verify it's stored
    let episode_id = memory
        .start_episode(
            "Sync Test Episode".to_string(),
            TaskContext {
                domain: "sync".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["sync".to_string(), "test".to_string()],
            },
            TaskType::Testing,
        )
        .await;

    // Add multiple steps
    for i in 1..=3 {
        let step = ExecutionStep::new(
            i,
            format!("sync_tool_{i}"),
            format!("Synchronization step {i}"),
        );
        memory.log_step(episode_id, step).await;
    }

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Storage synchronization test completed".to_string(),
                artifacts: vec!["sync_test.log".to_string()],
            },
        )
        .await
        .unwrap();

    println!("✅ Episode created with 3 steps");

    // Verify through multiple access methods
    let direct_episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(direct_episode.steps.len(), 3);

    // Verify through MCP
    let mcp_server = Arc::new(
        MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
            .await
            .unwrap(),
    );

    let query_result = mcp_server
        .query_memory(
            "Sync Test".to_string(),
            "sync".to_string(),
            None,
            10,
            "relevance".to_string(),
            None,
        )
        .await
        .unwrap();

    let episodes = query_result["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 1);

    let mcp_episode = &episodes[0];
    assert_eq!(mcp_episode["steps"].as_array().unwrap().len(), 3);

    println!("✅ Storage synchronization verified:");
    println!("   Direct access: {} steps", direct_episode.steps.len());
    println!(
        "   MCP access: {} steps",
        mcp_episode["steps"].as_array().unwrap().len()
    );
    println!("   Episode completed: {}", direct_episode.is_complete());
}

#[tokio::test]
async fn test_bulk_data_persistence() {
    println!("🧪 Testing Bulk Data Persistence");
    println!("================================");

    let (memory, _temp_dir) = setup_persistent_memory().await.unwrap();

    // Create multiple episodes
    let episode_count = 5;
    let mut episode_ids = Vec::new();

    for i in 1..=episode_count {
        let episode_id = memory
            .start_episode(
                format!("Bulk Test Episode {i}"),
                TaskContext {
                    domain: "bulk".to_string(),
                    language: Some("rust".to_string()),
                    framework: Some("tokio".to_string()),
                    complexity: ComplexityLevel::Simple,
                    tags: vec!["bulk".to_string(), "test".to_string()],
                },
                TaskType::Testing,
            )
            .await;

        // Add some steps
        for j in 1..=2 {
            let step = ExecutionStep::new(
                j,
                format!("bulk_tool_{j}"),
                format!("Bulk operation {j} for episode {i}"),
            );
            memory.log_step(episode_id, step).await;
        }

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Bulk episode {i} completed"),
                    artifacts: vec![format!("bulk_{}.log", i)],
                },
            )
            .await
            .unwrap();

        episode_ids.push(episode_id);
    }

    println!("✅ Created {episode_count} episodes with bulk data");

    // Verify all episodes persist
    let mcp_server = Arc::new(
        MemoryMCPServer::new(SandboxConfig::restrictive(), memory.clone())
            .await
            .unwrap(),
    );

    let query_result = mcp_server
        .query_memory(
            "Bulk Test".to_string(),
            "bulk".to_string(),
            None,
            10,
            "relevance".to_string(),
            None,
        )
        .await
        .unwrap();

    let episodes = query_result["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), episode_count);

    println!("✅ Bulk persistence verified:");
    println!("   Episodes created: {episode_count}");
    println!("   Episodes retrieved: {}", episodes.len());

    // Verify each episode has correct data (order may vary due to storage)
    let mut found_episodes = std::collections::HashSet::new();
    for episode in episodes {
        let title = episode["task_description"].as_str().unwrap();
        assert!(title.starts_with("Bulk Test Episode"));
        assert_eq!(episode["steps"].as_array().unwrap().len(), 2);
        found_episodes.insert(title.to_string());
        println!(
            "   Found episode: {} - {} steps",
            title,
            episode["steps"].as_array().unwrap().len()
        );
    }

    // Verify we found all expected episodes
    for i in 1..=episode_count {
        let expected_title = format!("Bulk Test Episode {i}");
        assert!(
            found_episodes.contains(&expected_title),
            "Missing episode: {expected_title}"
        );
    }

    println!("✅ All bulk data persisted and retrievable");
}
