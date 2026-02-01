//! Tag management commands for episodes
//!
//! This module provides commands for managing tags on episodes:
//! - tag add: Add tags to an episode
//! - tag remove: Remove tags from an episode
//! - tag set: Set/replace all tags on an episode
//! - tag list: List all tags for an episode
//! - tag search: Search episodes by tags
//! - tag show: Show episode with its tags

use clap::Subcommand;
use serde::Serialize;

pub mod core;
pub mod output;
pub mod types;

pub use core::*;
pub use output::*;
pub use types::*;

#[cfg(test)]
mod tests;
