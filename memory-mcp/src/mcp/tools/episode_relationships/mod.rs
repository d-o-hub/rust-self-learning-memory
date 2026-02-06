//! # Episode Relationship MCP Tools
//!
//! This module provides MCP tools for managing relationships between episodes.

#[cfg(test)]
mod tests;
mod tool;
mod types;

pub use tool::EpisodeRelationshipTools;
pub use types::{
    AddEpisodeRelationshipInput, AddEpisodeRelationshipOutput, CheckRelationshipExistsInput,
    CheckRelationshipExistsOutput, DependencyGraphInput, DependencyGraphOutput,
    FindRelatedEpisodesInput, FindRelatedEpisodesOutput, GetEpisodeRelationshipsInput,
    GetEpisodeRelationshipsOutput, GetTopologicalOrderInput, GetTopologicalOrderOutput,
    RelatedEpisode, RelationshipEdge, RelationshipNode, RemoveEpisodeRelationshipInput,
    RemoveEpisodeRelationshipOutput, ValidateNoCyclesInput, ValidateNoCyclesOutput,
};
