//! Performance Metrics & Observability
//!
//! Collects and reports performance metrics for Turso storage operations.
//!
//! ## Features
//!
//! - **P50, P95, P99 Latency Tracking**: Per-operation latency percentiles
//! - **Cache Hit/Miss Statistics**: Cache layer performance monitoring
//! - **Connection Pool Statistics**: Pool utilization and health metrics
//! - **Export for Monitoring**: Structured metrics for monitoring integration
//!
//! ## Usage
//!
//! ```rust
//! use memory_storage_turso::metrics::{TursoMetrics, MetricsCollector};
//!
//! let metrics = TursoMetrics::new();
//! ```

// Submodules
pub mod collector;
pub mod core;
pub mod performance;
pub mod types;

// Re-exports from submodules
pub use collector::MetricsCollector;
pub use core::TursoMetrics;
pub use performance::{
    BatchingMetrics, CacheFirstMetrics, OptimizationMetrics, PerformanceMetrics,
    PreparedStatementMetrics, QueryOptimizationMetrics,
};
pub use types::{CacheStats, LatencyStats, OperationMetrics, OperationType, PoolStats};

// Convenience re-exports
// Note: CacheStats and PoolStats are private in collector, removed from public exports
