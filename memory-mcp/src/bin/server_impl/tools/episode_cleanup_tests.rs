use super::{Content, handle_cleanup_episodes};
use do_memory_core::SelfLearningMemory;
use do_memory_mcp::MemoryMCPServer;
use do_memory_mcp::SandboxConfig;
use serde_json::{Value, json};
use std::sync::Arc;

#[tokio::test]
async fn cleanup_handler_serializes_results_and_propagates_policy_errors() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = MemoryMCPServer::new(SandboxConfig::restrictive(), memory)
        .await
        .unwrap();

    let content = handle_cleanup_episodes(&mut server, None).await.unwrap();
    let text = match &content[0] {
        Content::Text { text } => text,
    };
    let result: Value = serde_json::from_str(text).unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["dry_run"], true);

    let error = handle_cleanup_episodes(&mut server, Some(json!({"preset": "invalid"})))
        .await
        .unwrap_err();
    assert!(error.to_string().contains("Invalid preset 'invalid'"));
}
