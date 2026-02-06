//! Load tests for validating system behavior under high load
//!
//! This module contains tests that simulate heavy usage patterns to ensure
//! the memory system can handle production-level traffic.
//!
//! Test suites:
//! - `connection_pool_test.rs`: Connection pool stability under load
//! - `cache_load_test.rs`: Cache behavior under heavy query load
//! - `batch_operations_test.rs`: Batch operation throughput and safety

mod batch_operations_test;
mod cache_load_test;
mod connection_pool_test;
