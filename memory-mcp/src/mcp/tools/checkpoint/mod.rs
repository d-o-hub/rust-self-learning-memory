//! Episode Checkpoint MCP Tools (ADR-044 Feature 3)
//!
//! This module provides MCP tools for creating checkpoints, generating handoff packs,
//! and resuming work from handoff packs.

mod tool;
mod types;

pub use tool::CheckpointTools;
pub use types::{
    CheckpointEpisodeInput, CheckpointEpisodeOutput, GetHandoffPackInput, GetHandoffPackOutput,
    ResumeFromHandoffInput, ResumeFromHandoffOutput,
};
