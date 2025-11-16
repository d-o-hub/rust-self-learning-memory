//! # Agent Monitoring
//!
//! Basic monitoring system for tracking agent utilization, performance, and task completion rates.
//!
//! This module provides lightweight analytics collection for agent systems, tracking:
//! - Agent invocation counts and success rates
//! - Execution duration metrics
//! - Task completion rates by agent type
//! - Performance metrics for agent coordination
//!
//! ## Example
//!
//! ```
//! use memory_core::monitoring::{AgentMonitor, AgentMetrics, TaskMetrics};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let monitor = AgentMonitor::new();
//!
//!     // Track agent execution
//!     let start = std::time::Instant::now();
//!     // ... agent work ...
//!     let duration = start.elapsed();
//!
//!     monitor.record_execution("feature-implementer", true, duration).await;
//!
//!     // Get metrics
//!     let metrics = monitor.get_agent_metrics("feature-implementer").await
//!         .expect("Agent metrics should exist after recording execution");
//!     println!("Success rate: {:.2}", metrics.success_rate());
//!
//!     Ok(())
//! }
//! ```

mod core;
mod storage;
mod types;

pub use core::{AgentMonitor, MonitoringSummary};
pub use storage::{MonitoringAnalytics, MonitoringStorage};
pub use types::{AgentMetrics, ExecutionRecord, MonitoringConfig, TaskMetrics};

// Re-export for convenience
pub use types::AgentType;
