//! Batch executor for managing parallel execution

use super::dependency_graph::DependencyGraph;
use super::types::{BatchMode, BatchOperation, BatchRequest, BatchResponse, OperationResult};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Batch executor for managing parallel execution
pub struct BatchExecutor {
    /// Completed operation results cache
    /// Reserved for future result aggregation and query capabilities
    #[allow(dead_code)]
    results: Arc<RwLock<HashMap<String, OperationResult>>>,
}

impl BatchExecutor {
    /// Create a new batch executor
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute a batch of operations with dependency management
    pub async fn execute<F, Fut>(
        &self,
        request: BatchRequest,
        executor_fn: F,
    ) -> Result<BatchResponse, String>
    where
        F: Fn(String, Value) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<Value, (i32, String)>> + Send,
    {
        let start_time = std::time::Instant::now();

        // Build dependency graph
        let graph = DependencyGraph::new(request.operations)?;
        let total_operations = graph.len();

        let mut completed = HashSet::new();
        let mut results = Vec::new();
        let mut parallel_count = 0;
        let mut sequential_count = 0;

        // Execute operations based on mode
        match request.mode {
            BatchMode::Sequential => {
                // Execute all operations sequentially in insertion order
                for op in graph.operations_in_order() {
                    let result = self.execute_operation(&op, &executor_fn).await;
                    completed.insert(op.id.clone());
                    results.push(result);
                    sequential_count += 1;
                }
            }
            BatchMode::FailFast => {
                // Execute operations sequentially in insertion order, stop on first failure
                for op in graph.operations_in_order() {
                    let result = self.execute_operation(&op, &executor_fn).await;
                    let success = result.success;
                    completed.insert(op.id.clone());
                    results.push(result);
                    sequential_count += 1;

                    if !success {
                        break;
                    }
                }
            }
            BatchMode::Parallel => {
                // Execute operations respecting dependencies
                while completed.len() < total_operations {
                    let ready = graph.get_ready_operations(&completed);

                    if ready.is_empty() {
                        break; // No more operations can be executed
                    }

                    // Execute ready operations in parallel (up to max_parallel)
                    let batch_size = ready.len().min(request.max_parallel);
                    let batch: Vec<_> = ready.into_iter().take(batch_size).collect();

                    let mut handles = Vec::new();
                    for op in batch {
                        let op_clone = op.clone();
                        let executor_fn_clone = executor_fn.clone();
                        let handle = tokio::spawn(async move {
                            Self::execute_single_operation(&op_clone, executor_fn_clone).await
                        });
                        handles.push((op.id.clone(), handle));
                    }

                    // Wait for all operations in this batch to complete
                    for (id, handle) in handles {
                        match handle.await {
                            Ok(result) => {
                                completed.insert(id);
                                results.push(result);
                                parallel_count += 1;
                            }
                            Err(e) => {
                                // Task panicked
                                results.push(OperationResult {
                                    id: id.clone(),
                                    success: false,
                                    result: None,
                                    error: Some(super::types::OperationError {
                                        code: -32603,
                                        message: format!("Operation panicked: {}", e),
                                        details: None,
                                    }),
                                    duration_ms: 0,
                                });
                                completed.insert(id);
                            }
                        }
                    }
                }
            }
        }

        let total_duration_ms = start_time.elapsed().as_millis() as u64;
        let success_count = results.iter().filter(|r| r.success).count();
        let failure_count = results.len() - success_count;

        let avg_duration_ms = if !results.is_empty() {
            results.iter().map(|r| r.duration_ms).sum::<u64>() as f64 / results.len() as f64
        } else {
            0.0
        };

        Ok(BatchResponse {
            results,
            total_duration_ms,
            success_count,
            failure_count,
            stats: super::types::BatchStats {
                total_operations,
                parallel_executed: parallel_count,
                sequential_executed: sequential_count,
                avg_duration_ms,
            },
        })
    }

    /// Execute a single operation
    async fn execute_single_operation<F, Fut>(
        op: &BatchOperation,
        executor_fn: F,
    ) -> OperationResult
    where
        F: Fn(String, Value) -> Fut,
        Fut: std::future::Future<Output = Result<Value, (i32, String)>>,
    {
        let start = std::time::Instant::now();

        match executor_fn(op.tool.clone(), op.arguments.clone()).await {
            Ok(result) => OperationResult {
                id: op.id.clone(),
                success: true,
                result: Some(result),
                error: None,
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Err((code, message)) => OperationResult {
                id: op.id.clone(),
                success: false,
                result: None,
                error: Some(super::types::OperationError {
                    code,
                    message,
                    details: None,
                }),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Execute a single operation (instance method for backward compatibility)
    async fn execute_operation<F, Fut>(
        &self,
        op: &BatchOperation,
        executor_fn: F,
    ) -> OperationResult
    where
        F: Fn(String, Value) -> Fut,
        Fut: std::future::Future<Output = Result<Value, (i32, String)>>,
    {
        Self::execute_single_operation(op, executor_fn).await
    }
}

impl Default for BatchExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[tokio::test]
    async fn test_execute_empty_batch() {
        let executor = BatchExecutor::new();
        let request = BatchRequest {
            operations: vec![],
            mode: BatchMode::Parallel,
            max_parallel: 10,
        };

        let result = executor
            .execute(request, |_, _| async { Ok(Value::Null) })
            .await
            .unwrap();

        assert_eq!(result.success_count, 0);
        assert_eq!(result.failure_count, 0);
    }

    #[tokio::test]
    async fn test_execute_sequential_batch() {
        let executor = BatchExecutor::new();
        let operations = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "tool2".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
        ];

        let request = BatchRequest {
            operations,
            mode: BatchMode::Sequential,
            max_parallel: 10,
        };

        let call_count = Arc::new(std::sync::Mutex::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let result = executor
            .execute(request, move |_, _| {
                let count = *call_count_clone.lock().unwrap();
                *call_count_clone.lock().unwrap() = count + 1;
                async move { Ok(Value::Null) }
            })
            .await
            .unwrap();

        assert_eq!(result.success_count, 2);
        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_execute_parallel_batch() {
        let executor = BatchExecutor::new();
        let operations = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "tool2".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
        ];

        let request = BatchRequest {
            operations,
            mode: BatchMode::Parallel,
            max_parallel: 10,
        };

        let start = std::time::Instant::now();
        let result = executor
            .execute(request, |_, _| async { Ok(Value::Null) })
            .await
            .unwrap();
        let duration = start.elapsed();

        assert_eq!(result.success_count, 2);
        // Should complete in roughly 0ms since parallel
        assert!(duration.as_millis() < 100);
    }

    #[tokio::test]
    async fn test_execute_with_dependency() {
        let executor = BatchExecutor::new();
        let operations = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "tool2".to_string(),
                arguments: Value::Null,
                depends_on: vec!["op1".to_string()],
            },
        ];

        let request = BatchRequest {
            operations,
            mode: BatchMode::Parallel,
            max_parallel: 10,
        };

        let result = executor
            .execute(request, |id, _| async move {
                if id == "op1" {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
                Ok(Value::Null)
            })
            .await
            .unwrap();

        // Both operations should succeed
        assert_eq!(result.success_count, 2);
        // Verify total matches
        assert_eq!(result.success_count + result.failure_count, 2);
    }

    #[tokio::test]
    async fn test_fail_fast_mode() {
        let executor = BatchExecutor::new();
        let operations = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "tool1".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "tool2".to_string(),
                arguments: Value::Null,
                depends_on: vec![],
            },
        ];

        let request = BatchRequest {
            operations,
            mode: BatchMode::FailFast,
            max_parallel: 10,
        };

        let result = executor
            .execute(request, |id, _| async move {
                Err((-32600, format!("Operation {} failed", id)))
            })
            .await
            .unwrap();

        // Only first operation should be executed in fail_fast mode
        assert_eq!(result.results.len(), 1);
        assert!(!result.results[0].success);
    }
}
