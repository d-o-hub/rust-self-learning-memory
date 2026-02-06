//! End-to-End (E2E) tests for embeddings and CLI/MCP integration
//!
//! This module contains comprehensive E2E tests for several systems:
//!
//! **Embbedding Tests** (existing):
//! - `embeddings_openai_test.rs`: OpenAI provider tests (with mock for CI)
//! - `embeddings_local_test.rs`: Local CPU-based provider tests
//! - `embeddings_cli_test.rs`: CLI command integration tests
//! - `embeddings_mcp_test.rs`: MCP server tool integration tests
//! - `embeddings_quality_test.rs`: Search quality and accuracy tests
//! - `embeddings_performance_test.rs`: Performance benchmarks and profiling
//!
//! **CLI Workflow Tests**:
//! - `cli_episode_workflow.rs`: Episode lifecycle workflow tests
//! - `cli_pattern_workflow.rs`: Pattern discovery workflow tests
//! - `cli_workflows.rs`: Comprehensive CLI workflow tests (NEW)
//!
//! **MCP Integration Tests**:
//! - `mcp_episode_chain.rs`: Episode management chain tests
//! - `mcp_relationship_chain.rs`: Relationship chain tests
//! - `mcp_tag_chain.rs`: Tag management chain tests
//! - `mcp_integration.rs`: Comprehensive MCP integration tests (NEW)
//!
//! **Error Handling**:
//! - `error_handling.rs`: Error scenario tests

#![allow(clippy::unwrap_used, clippy::expect_used)]

// Embedding tests
pub mod embeddings_cli_test;
pub mod embeddings_local_test;
pub mod embeddings_mcp_test;
pub mod embeddings_openai_test;
pub mod embeddings_quality_test;
pub mod embeddings_performance_test;

// CLI and MCP E2E tests (30+ scenarios for >95% coverage)
pub mod cli_episode_workflow;
pub mod cli_pattern_workflow;
pub mod cli_workflows;
pub mod mcp_episode_chain;
pub mod mcp_relationship_chain;
pub mod mcp_tag_chain;
pub mod mcp_integration;
pub mod error_handling;

// Common test utilities and helpers
pub mod common;

use std::time::{Duration, Instant};

/// Performance benchmark helper
pub struct Benchmark {
    name: String,
    iterations: usize,
}

impl Benchmark {
    pub fn new(name: String, iterations: usize) -> Self {
        Self { name, iterations }
    }

    pub fn run<F, Fut>(&self, f: F) -> BenchmarkResults
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let mut durations = Vec::with_capacity(self.iterations);

        for _ in 0..self.iterations {
            let start = Instant::now();
            tokio::runtime::Runtime::new().unwrap().block_on(f());
            durations.push(start.elapsed());
        }

        BenchmarkResults::new(self.name.clone(), durations)
    }
}

#[derive(Debug)]
pub struct BenchmarkResults {
    pub name: String,
    pub count: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub throughput: f64,
}

impl BenchmarkResults {
    fn new(name: String, durations: Vec<Duration>) -> Self {
        let count = durations.len();
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / count as u32;
        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();
        let throughput = count as f64 / total_duration.as_secs_f64();

        Self {
            name,
            count,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            throughput,
        }
    }

    pub fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("Benchmark: {}", self.name);
        println!("{}", "=".repeat(60));
        println!("  Iterations: {}", self.count);
        println!("  Total: {:?}", self.total_duration);
        println!("  Average: {:.2}ms", self.avg_duration.as_millis());
        println!("  Min: {:?}", self.min_duration);
        println!("  Max: {:?}", self.max_duration);
        println!("  Throughput: {:.2} ops/sec", self.throughput);
        println!("{}", "=".repeat(60));
    }
}

/// Quality metrics for search accuracy
pub struct QualityMetrics {
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub avg_rank: f64,
}

impl QualityMetrics {
    pub fn calculate(relevant: &[usize], retrieved: &[usize]) -> Self {
        let relevant_set: std::collections::HashSet<_> = relevant.iter().collect();
        let retrieved_set: std::collections::HashSet<_> = retrieved.iter().collect();

        let true_positives = relevant_set.intersection(&retrieved_set).count();
        let precision = if retrieved.is_empty() {
            0.0
        } else {
            true_positives as f64 / retrieved.len() as f64
        };
        let recall = if relevant.is_empty() {
            0.0
        } else {
            true_positives as f64 / relevant.len() as f64
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        // Calculate average rank of relevant items
        let mut avg_rank = 0.0;
        if !retrieved.is_empty() && !relevant.is_empty() {
            for rel in relevant {
                if let Some(rank) = retrieved.iter().position(|&x| x == *rel) {
                    avg_rank += rank as f64;
                }
            }
            avg_rank /= relevant.len() as f64;
        }

        Self {
            precision,
            recall,
            f1_score,
            avg_rank,
        }
    }

    pub fn print(&self) {
        println!("\nQuality Metrics:");
        println!("  Precision: {:.2}%", self.precision * 100.0);
        println!("  Recall: {:.2}%", self.recall * 100.0);
        println!("  F1 Score: {:.2}", self.f1_score);
        println!("  Avg Rank: {:.2}", self.avg_rank);
    }
}
