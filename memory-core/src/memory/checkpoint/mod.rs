//! Episode Checkpoints & Handoff Packs (ADR-044 Feature 3)
//!
//! This module provides checkpoint and handoff functionality for multi-agent workflows.
//! It enables agents to save mid-task progress and transfer context to other agents
//! without requiring episode completion.
//!
//! # Key Concepts
//!
//! - **Checkpoint**: A saved snapshot of episode progress at a point in time
//! - **HandoffPack**: A comprehensive context package for transferring work between agents
//! - **Resume**: Continue work from a previous checkpoint in a new or existing episode
//!
//! # Example
//!
//! ```no_run
//! use do_memory_core::memory::checkpoint::{checkpoint_episode, get_handoff_pack, resume_from_handoff};
//! use do_memory_core::SelfLearningMemory;
//! use uuid::Uuid;
//!
//! # async fn example(memory: SelfLearningMemory) -> anyhow::Result<()> {
//! // Create a checkpoint for an in-progress episode
//! let episode_id = Uuid::new_v4();
//! let checkpoint = checkpoint_episode(&memory, episode_id, "Agent switching".to_string()).await?;
//!
//! // Get a handoff pack to transfer to another agent
//! let handoff = get_handoff_pack(&memory, checkpoint.checkpoint_id).await?;
//!
//! // Resume work in a new episode using the handoff pack
//! let new_episode_id = resume_from_handoff(&memory, handoff).await?;
//! # Ok(())
//! # }
//! ```

mod operations;
mod types;

pub use operations::{
    checkpoint_episode, checkpoint_episode_with_note, get_handoff_pack, resume_from_handoff,
};
pub use types::{CheckpointMeta, HandoffPack};
