//! MCP types for tool definitions and execution results

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name (must be unique)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON schema for input validation
    pub input_schema: serde_json::Value,
}

impl Tool {
    /// Create a new tool definition
    pub fn new(name: String, description: String, input_schema: serde_json::Value) -> Self {
        Self {
            name,
            description,
            input_schema,
        }
    }
}

/// Result of code execution in sandbox
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionResult {
    /// Successful execution with output
    Success {
        output: String,
        stdout: String,
        stderr: String,
        execution_time_ms: u64,
    },
    /// Execution error (syntax, runtime, etc.)
    Error {
        message: String,
        error_type: ErrorType,
        stdout: String,
        stderr: String,
    },
    /// Execution timed out
    Timeout {
        elapsed_ms: u64,
        partial_output: Option<String>,
    },
    /// Security violation detected
    SecurityViolation {
        reason: String,
        violation_type: SecurityViolationType,
    },
}

/// Type of execution error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    Syntax,
    Runtime,
    Permission,
    Resource,
    Unknown,
}

/// Type of security violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityViolationType {
    FileSystemAccess,
    NetworkAccess,
    ProcessExecution,
    MemoryLimit,
    InfiniteLoop,
    MaliciousCode,
}

/// Enhanced resource limits for sandbox
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum CPU percentage (0-100)
    pub max_cpu_percent: f32,
    /// Maximum memory in megabytes
    pub max_memory_mb: usize,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum file operations (0 = deny all)
    pub max_file_operations: usize,
    /// Maximum network requests (0 = deny all)
    pub max_network_requests: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 50.0,
            max_memory_mb: 128,
            max_execution_time_ms: 5000,
            max_file_operations: 0,
            max_network_requests: 0,
        }
    }
}

impl ResourceLimits {
    /// Create restrictive resource limits
    pub fn restrictive() -> Self {
        Self {
            max_cpu_percent: 30.0,
            max_memory_mb: 64,
            max_execution_time_ms: 3000,
            max_file_operations: 0,
            max_network_requests: 0,
        }
    }

    /// Create permissive resource limits (for trusted code)
    pub fn permissive() -> Self {
        Self {
            max_cpu_percent: 80.0,
            max_memory_mb: 256,
            max_execution_time_ms: 10000,
            max_file_operations: 100,
            max_network_requests: 10,
        }
    }
}

/// Configuration for code sandbox
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum memory in megabytes
    pub max_memory_mb: usize,
    /// Maximum CPU percentage (0-100)
    pub max_cpu_percent: u8,
    /// Allowed file system paths (whitelist)
    pub allowed_paths: Vec<String>,
    /// Allowed network hosts (empty = deny all)
    pub allowed_network: Vec<String>,
    /// Enable network access
    pub allow_network: bool,
    /// Enable file system access (to allowed paths only)
    pub allow_filesystem: bool,
    /// Enable subprocess execution
    pub allow_subprocesses: bool,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Process UID for privilege dropping (None = no change)
    pub process_uid: Option<u32>,
    /// Read-only file system mode
    pub read_only_mode: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        let limits = ResourceLimits::default();
        Self {
            max_execution_time_ms: limits.max_execution_time_ms,
            max_memory_mb: limits.max_memory_mb,
            max_cpu_percent: limits.max_cpu_percent as u8,
            allowed_paths: vec![],
            allowed_network: vec![],
            allow_network: false,
            allow_filesystem: false,
            allow_subprocesses: false,
            resource_limits: limits,
            process_uid: None,
            read_only_mode: true,
        }
    }
}

impl SandboxConfig {
    /// Create a restrictive configuration for untrusted code
    pub fn restrictive() -> Self {
        let limits = ResourceLimits::restrictive();
        Self {
            max_execution_time_ms: limits.max_execution_time_ms,
            max_memory_mb: limits.max_memory_mb,
            max_cpu_percent: limits.max_cpu_percent as u8,
            allowed_paths: vec![],
            allowed_network: vec![],
            allow_network: false,
            allow_filesystem: false,
            allow_subprocesses: false,
            resource_limits: limits,
            process_uid: None,
            read_only_mode: true,
        }
    }

    /// Create a permissive configuration for trusted code
    pub fn permissive() -> Self {
        let limits = ResourceLimits::permissive();
        Self {
            max_execution_time_ms: limits.max_execution_time_ms,
            max_memory_mb: limits.max_memory_mb,
            max_cpu_percent: limits.max_cpu_percent as u8,
            allowed_paths: vec!["/tmp".to_string()],
            allowed_network: vec![],
            allow_network: false,
            allow_filesystem: true,
            allow_subprocesses: false,
            resource_limits: limits,
            process_uid: None,
            read_only_mode: false,
        }
    }
}

/// Execution context passed to sandboxed code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Task description
    pub task: String,
    /// Input data (as JSON)
    pub input: serde_json::Value,
    /// Additional environment variables
    pub env: HashMap<String, String>,
    /// Execution metadata
    pub metadata: HashMap<String, String>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(task: String, input: serde_json::Value) -> Self {
        Self {
            task,
            input,
            env: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Statistics about code execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub timeout_count: u64,
    pub security_violations: u64,
    pub avg_execution_time_ms: f64,
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            timeout_count: 0,
            security_violations: 0,
            avg_execution_time_ms: 0.0,
        }
    }
}

impl ExecutionStats {
    /// Update statistics with a new execution result
    pub fn record_execution(&mut self, result: &ExecutionResult, duration_ms: u64) {
        self.total_executions += 1;

        match result {
            ExecutionResult::Success { .. } => {
                self.successful_executions += 1;
            }
            ExecutionResult::Error { .. } => {
                self.failed_executions += 1;
            }
            ExecutionResult::Timeout { .. } => {
                self.timeout_count += 1;
                self.failed_executions += 1;
            }
            ExecutionResult::SecurityViolation { .. } => {
                self.security_violations += 1;
                self.failed_executions += 1;
            }
        }

        // Update running average
        let total = self.total_executions as f64;
        self.avg_execution_time_ms =
            (self.avg_execution_time_ms * (total - 1.0) + duration_ms as f64) / total;
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.successful_executions as f64 / self.total_executions as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_stats() {
        let mut stats = ExecutionStats::default();
        assert_eq!(stats.success_rate(), 0.0);

        let success = ExecutionResult::Success {
            output: "result".to_string(),
            stdout: "out".to_string(),
            stderr: "".to_string(),
            execution_time_ms: 100,
        };

        stats.record_execution(&success, 100);
        assert_eq!(stats.total_executions, 1);
        assert_eq!(stats.successful_executions, 1);
        assert_eq!(stats.success_rate(), 100.0);

        let error = ExecutionResult::Error {
            message: "error".to_string(),
            error_type: ErrorType::Runtime,
            stdout: "".to_string(),
            stderr: "err".to_string(),
        };

        stats.record_execution(&error, 50);
        assert_eq!(stats.total_executions, 2);
        assert_eq!(stats.successful_executions, 1);
        assert_eq!(stats.failed_executions, 1);
        assert_eq!(stats.success_rate(), 50.0);
    }

    #[test]
    fn test_sandbox_config_defaults() {
        let default = SandboxConfig::default();
        assert_eq!(default.max_execution_time_ms, 5000);
        assert!(!default.allow_network);
        assert!(!default.allow_filesystem);

        let restrictive = SandboxConfig::restrictive();
        assert_eq!(restrictive.max_execution_time_ms, 3000);
        assert_eq!(restrictive.max_memory_mb, 64);

        let permissive = SandboxConfig::permissive();
        assert!(permissive.allow_filesystem);
        assert_eq!(permissive.allowed_paths.len(), 1);
    }

    #[test]
    fn test_execution_context() {
        let ctx =
            ExecutionContext::new("test task".to_string(), serde_json::json!({"key": "value"}));

        assert_eq!(ctx.task, "test task");
        assert_eq!(ctx.input["key"], "value");
        assert!(ctx.env.is_empty());
    }
}
