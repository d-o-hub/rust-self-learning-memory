//! External signal provider handler functions
//!
//! This module contains handlers for external signal provider tools
//! including AgentFS configuration and status checks.

use super::{Content, MemoryMCPServer, Value, get_client_id};
use do_memory_mcp::mcp::tools::external_signals::{
    ConfigureAgentFsInput, ExternalSignalStatusInput, TestAgentFsConnectionInput,
};
use serde_json::json;

/// Handle configure_agentfs tool
pub async fn handle_configure_agentfs(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);
    let mut input: ConfigureAgentFsInput = serde_json::from_value(args)?;

    // Clamp weight, min_samples, and db_path length (CWE-770) - defense-in-depth beyond schema bounds
    if input.db_path.len() > do_memory_mcp::constants::MAX_DB_PATH_LEN {
        // Manually find the nearest UTF-8 char boundary (floor_char_boundary not stable until Rust 1.91.0)
        let mut end = do_memory_mcp::constants::MAX_DB_PATH_LEN;
        while end > 0 && (input.db_path.as_bytes()[end] & 0xC0 == 0x80) {
            end -= 1;
        }
        input.db_path.truncate(end);
    }
    input.weight = input.weight.clamp(
        do_memory_mcp::constants::MIN_EXTERNAL_WEIGHT,
        do_memory_mcp::constants::MAX_EXTERNAL_WEIGHT,
    );
    input.min_samples = input.min_samples.clamp(
        do_memory_mcp::constants::MIN_EXTERNAL_SAMPLES,
        do_memory_mcp::constants::MAX_EXTERNAL_SAMPLES,
    );

    let db_path = input.db_path.clone();
    let enabled = input.enabled;

    let result = server.execute_configure_agentfs(input).await;

    // Audit log the configuration change
    let success = result.is_ok();
    server
        .audit_logger()
        .log_external_signal_config(&client_id, "agentfs", &db_path, enabled, success)
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];
    Ok(content)
}

/// Handle external_signal_status tool
pub async fn handle_external_signal_status(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let input: ExternalSignalStatusInput = serde_json::from_value(args)?;

    let result = server.execute_external_signal_status(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle test_agentfs_connection tool
pub async fn handle_test_agentfs_connection(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);
    let input: TestAgentFsConnectionInput = serde_json::from_value(args)?;

    let result = server.execute_test_agentfs_connection(input).await;

    // Audit log the connection test
    let success = result.is_ok();
    server
        .audit_logger()
        .log_external_signal_test(&client_id, "agentfs", success)
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];
    Ok(content)
}
