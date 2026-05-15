//! EventEmitter sink implementations for CloudEvents.
//!
//! Sinks provide concrete implementations of the `EventEmitter` trait:
//! - `LogEmitter`: Logs events via the `tracing` crate
//! - `NoOpEmitter`: Discards all events (default, zero overhead)

pub mod log;
pub mod noop;

pub use log::LogEmitter;
pub use noop::NoOpEmitter;
