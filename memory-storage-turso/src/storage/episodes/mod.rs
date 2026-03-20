//! # Episode Storage Module
//!
//! Episode CRUD operations and query functionality.

pub mod compression;
pub mod crud;
pub mod query;
pub mod row;

pub use compression::{compress_json_field, decompress_json_field};
pub use row::row_to_episode;

use memory_core::TaskType;

/// Query builder for episodes
#[derive(Debug, Clone, Default)]
pub struct EpisodeQuery {
    pub task_type: Option<TaskType>,
    pub domain: Option<String>,
    pub language: Option<String>,
    pub limit: Option<usize>,
    pub completed_only: bool,
}
