//! Batch operation types

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single operation in a batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    /// Unique identifier for this operation
    pub id: String,
    /// Tool name to execute
    pub tool: String,
    /// Tool arguments
    pub arguments: Value,
    /// Optional list of operation IDs this depends on
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Batch execution mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum BatchMode {
    /// Execute independent operations in parallel (default)
    #[default]
    Parallel,
    /// Execute all operations sequentially
    Sequential,
    /// Stop on first error
    FailFast,
}

/// Batch request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Operations to execute
    pub operations: Vec<BatchOperation>,
    /// Execution mode
    #[serde(default)]
    pub mode: BatchMode,
    /// Maximum parallel operations (default: 10)
    #[serde(default = "default_max_parallel")]
    pub max_parallel: usize,
}

fn default_max_parallel() -> usize {
    10
}

/// Result of a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    /// Operation ID
    pub id: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Result value if successful
    pub result: Option<Value>,
    /// Error information if failed
    pub error: Option<OperationError>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Operation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error details
    pub details: Option<Value>,
}

/// Batch response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// Individual operation results
    pub results: Vec<OperationResult>,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Number of successful operations
    pub success_count: usize,
    /// Number of failed operations
    pub failure_count: usize,
    /// Execution statistics
    pub stats: BatchStats,
}

/// Batch execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStats {
    /// Total number of operations
    pub total_operations: usize,
    /// Number of operations executed in parallel
    pub parallel_executed: usize,
    /// Number of operations executed sequentially
    pub sequential_executed: usize,
    /// Average operation duration in milliseconds
    pub avg_duration_ms: f64,
}
