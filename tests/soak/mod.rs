//! Soak tests for validating long-term system stability
//!
//! This module contains long-running tests to validate system reliability
//! over extended periods, including memory leak detection and performance stability.
//!
//! Test suites:
//! - `stability_test.rs`: 24-hour stability test with memory monitoring
//! - `rate_limiter_test.rs`: Rate limiter accuracy and recovery tests

#[cfg(feature = "sysinfo")]
extern crate sysinfo;

mod stability_test;
mod rate_limiter_test;
