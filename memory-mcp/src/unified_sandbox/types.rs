//! Unified sandbox types and configuration.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Sandbox backend selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxBackend {
    /// Use only Node.js sandbox (current implementation)
    NodeJs,
    /// Use only WASM sandbox (new implementation)
    Wasm,
    /// Hybrid mode with intelligent routing
    Hybrid {
        /// Ratio of executions to route to WASM (0.0 to 1.0)
        wasm_ratio: f64,
        /// Route specific workloads based on heuristics
        intelligent_routing: bool,
    },
}

impl Default for SandboxBackend {
    fn default() -> Self {
        Self::Hybrid {
            wasm_ratio: 0.5, // Start with 50% WASM
            intelligent_routing: true,
        }
    }
}

/// Metrics for unified sandbox
#[derive(Debug, Default, Clone)]
pub struct UnifiedMetrics {
    pub total_executions: u64,
    pub node_executions: u64,
    pub wasm_executions: u64,
    pub node_success_rate: f64,
    pub wasm_success_rate: f64,
    pub node_avg_latency_ms: f64,
    pub wasm_avg_latency_ms: f64,
    pub routing_decisions: VecDeque<RoutingDecision>,
}

/// Routing decision tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub backend: String,
    pub reason: String,
    pub code_length: usize,
    pub has_async: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Backend choice for execution
#[derive(Debug, Clone, PartialEq)]
pub enum BackendChoice {
    NodeJs,
    Wasm,
}

/// Code heuristics for routing decisions
#[derive(Debug)]
#[allow(dead_code)]
pub struct CodeHeuristics {
    pub is_simple: bool,
    pub is_short: bool,
    pub is_complex: bool,
    pub has_external_deps: bool,
    pub has_async: bool,
    pub has_loops: bool,
}

/// Backend health status
#[derive(Debug, Default, Clone)]
pub struct BackendHealth {
    pub node_available: bool,
    pub wasm_available: bool,
    pub wasmtime_pool_stats: Option<crate::wasmtime_sandbox::WasmtimeMetrics>,
}
