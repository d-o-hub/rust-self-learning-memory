//! Benchmark suite for the self-learning memory system

pub mod benchmark_helpers;

use criterion::async_executor::AsyncExecutor;
use std::future::Future;

/// Tokio-based executor for Criterion benchmarks
/// 
/// Unlike FuturesExecutor, this provides a full tokio runtime which is required
/// when code uses tokio::spawn, tokio::task::spawn_blocking, or other tokio features.
pub struct TokioExecutor;

impl AsyncExecutor for TokioExecutor {
    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
            .block_on(future)
    }
}
