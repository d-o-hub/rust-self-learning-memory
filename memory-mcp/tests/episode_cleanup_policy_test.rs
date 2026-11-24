//! Validation tests for MCP episode retention policy arguments.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use do_memory_core::{MemoryConfig, SelfLearningMemory};
use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
use serde_json::json;
use std::sync::Arc;

async fn test_server() -> MemoryMCPServer {
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0,
        ..Default::default()
    }));
    MemoryMCPServer::new(SandboxConfig::restrictive(), memory)
        .await
        .unwrap()
}

#[tokio::test]
async fn cleanup_policy_accepts_presets_criteria_and_overrides() {
    let server = test_server().await;

    for preset in [
        "default",
        "storage_limited",
        "storage-limited",
        "aggressive",
        "archival",
    ] {
        let result = server
            .cleanup_episodes_tool(json!({"preset": preset, "count_only": true}))
            .await
            .unwrap();
        assert_eq!(result["candidate_count"], 0, "preset {preset}");
    }

    for criterion in [
        "combined",
        "age_based",
        "age-based",
        "reward_based",
        "reward-based",
        "unreferenced",
        "failed_only",
        "failed-only",
        "keep_all",
        "keep-all",
    ] {
        let result = server
            .cleanup_episodes_tool(json!({"criterion": criterion, "count_only": true}))
            .await
            .unwrap();
        assert_eq!(result["candidate_count"], 0, "criterion {criterion}");
    }

    let result = server
        .cleanup_episodes_tool(json!({
            "criterion": "keep_all",
            "max_age_days": 14,
            "min_reward_threshold": 0.5,
            "max_episodes": 100,
            "cleanup_batch_size": 25,
            "keep_pattern_sources": false,
            "keep_heuristic_sources": false,
            "count_only": true
        }))
        .await
        .unwrap();
    assert_eq!(result["candidate_count"], 0);
}

#[tokio::test]
async fn cleanup_policy_rejects_invalid_preset_criterion_and_limits() {
    let server = test_server().await;

    let invalid_preset = server
        .cleanup_episodes_tool(json!({"preset": "unknown", "count_only": true}))
        .await
        .unwrap_err();
    assert!(
        invalid_preset
            .to_string()
            .contains("Invalid preset 'unknown'")
    );

    let invalid_criterion = server
        .cleanup_episodes_tool(json!({"criterion": "unknown", "count_only": true}))
        .await
        .unwrap_err();
    assert!(
        invalid_criterion
            .to_string()
            .contains("Invalid criterion 'unknown'")
    );

    let invalid_threshold = server
        .cleanup_episodes_tool(json!({"min_reward_threshold": 1.5, "count_only": true}))
        .await
        .unwrap_err();
    assert!(
        invalid_threshold
            .to_string()
            .contains("min_reward_threshold must be between 0.0 and 1.0")
    );

    let invalid_batch = server
        .cleanup_episodes_tool(json!({"cleanup_batch_size": 0, "count_only": true}))
        .await
        .unwrap_err();
    assert!(
        invalid_batch
            .to_string()
            .contains("cleanup_batch_size must be greater than 0")
    );
}
