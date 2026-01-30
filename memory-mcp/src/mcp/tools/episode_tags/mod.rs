//! # Episode Tagging MCP Tools
//!
//! This module provides MCP tools for managing episode tags.

#[cfg(test)]
mod tests;
mod tool;
mod types;

pub use tool::EpisodeTagTools;
pub use types::{
    AddEpisodeTagsInput, AddEpisodeTagsOutput, EpisodeTagResult, GetEpisodeTagsInput,
    GetEpisodeTagsOutput, RemoveEpisodeTagsInput, RemoveEpisodeTagsOutput,
    SearchEpisodesByTagsInput, SearchEpisodesByTagsOutput, SetEpisodeTagsInput,
    SetEpisodeTagsOutput,
};
