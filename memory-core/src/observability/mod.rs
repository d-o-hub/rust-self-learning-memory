//! Opt-in observability backends.
//!
//! The module is compiled only with the `otel` feature, and each backend is
//! inert until it is explicitly configured. See [`otel`] for the OpenTelemetry
//! OTLP pipeline used by R-F2.

#[cfg(feature = "otel")]
pub mod otel;

#[cfg(feature = "otel")]
pub use otel::{OtelConfig, OtelGuard, OtelInitError, init_otel, init_otel_from_env};
