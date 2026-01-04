// Code execution tool handlers
//!
//! This module contains the execute_agent_code tool handler.

use crate::types::{ErrorType, ExecutionContext, ExecutionResult};
use anyhow::Result;
use tracing::{debug, info, warn};

impl crate::server::MemoryMCPServer {
    /// Execute the execute_agent_code tool
    ///
    /// # Arguments
    ///
    /// * `code` - TypeScript/JavaScript code to execute
    /// * `context` - Execution context
    ///
    /// # Returns
    ///
    /// Returns execution result from the sandbox
    ///
    /// # Security
    ///
    /// This method executes code in a secure sandbox with:
    /// - Timeout enforcement
    /// - Resource limits
    /// - No network access (by default)
    /// - No filesystem access (by default)
    /// - Malicious code detection
    pub async fn execute_agent_code(
        &self,
        code: String,
        context: ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.track_tool_usage("execute_agent_code").await;

        // Start monitoring request
        let request_id = format!(
            "execute_agent_code_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        self.monitoring
            .start_request(request_id.clone(), "execute_agent_code".to_string())
            .await;

        info!(
            "Executing agent code: task='{}', code_length={}",
            context.task,
            code.len()
        );

        let start = std::time::Instant::now();

        // Execute in sandbox
        let result = match self.sandbox.execute(&code, context).await {
            Ok(r) => r,
            Err(e) => {
                // Even on error, we should track the execution attempt
                let duration_ms = start.elapsed().as_millis() as u64;
                let mut stats = self.stats.write();
                stats.record_execution(
                    &ExecutionResult::Error {
                        message: e.to_string(),
                        error_type: ErrorType::Runtime,
                        stdout: "".to_string(),
                        stderr: "".to_string(),
                    },
                    duration_ms,
                );
                return Err(e);
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.record_execution(&result, duration_ms);
        }

        // End monitoring request
        let success = matches!(result, ExecutionResult::Success { .. });
        let error_message = match &result {
            ExecutionResult::Error { message, .. } => Some(message.clone()),
            ExecutionResult::SecurityViolation { reason, .. } => Some(reason.clone()),
            _ => None,
        };
        self.monitoring
            .end_request(&request_id, success, error_message)
            .await;

        // Log result
        match &result {
            ExecutionResult::Success { .. } => {
                debug!("Code execution succeeded in {}ms", duration_ms);
            }
            ExecutionResult::Error { error_type, .. } => {
                warn!(
                    "Code execution failed: {:?} in {}ms",
                    error_type, duration_ms
                );
            }
            ExecutionResult::Timeout { elapsed_ms, .. } => {
                warn!("Code execution timed out after {}ms", elapsed_ms);
            }
            ExecutionResult::SecurityViolation { violation_type, .. } => {
                warn!("Security violation detected: {:?}", violation_type);
            }
        }

        Ok(result)
    }
}
