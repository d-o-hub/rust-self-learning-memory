//! Playbook CLI commands (ADR-044 Feature 1)
//!
//! Provides commands for generating and explaining actionable playbooks.

mod commands;
mod dispatch;
mod types;

pub use commands::*;
pub use dispatch::*;
pub use types::*;
