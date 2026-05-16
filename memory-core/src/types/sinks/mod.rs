//! EventEmitter sink implementations for CloudEvents.
//!
//! Sinks provide concrete implementations of the `EventEmitter` trait:
//! - `LogEmitter`: Logs events via the `tracing` crate
//! - `NoOpEmitter`: Discards all events (default, zero overhead)
//! - `HttpEmitter`: Delivers events to an HTTP webhook (requires `http-emitter` feature)

pub mod log;
pub mod noop;

#[cfg(feature = "http-emitter")]
pub mod http;

pub use log::LogEmitter;
pub use noop::NoOpEmitter;

#[cfg(feature = "http-emitter")]
pub use http::HttpEmitter;
