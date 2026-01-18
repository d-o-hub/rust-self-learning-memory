//! Step batching for efficient I/O
//!
//! Buffers execution steps in memory and flushes to storage when conditions are met,
//! reducing I/O overhead for episodes with many steps.

mod buffer;
mod config;
mod ops;

pub use buffer::StepBuffer;
pub use config::BatchConfig;
