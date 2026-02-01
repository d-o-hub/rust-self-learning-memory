//! Episode relationship tool types and input/output structures.

use serde::{Deserialize, Serialize};

/// Input parameters for adding a relationship between episodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEpisodeRelationshipInput {
    /// Source episode UUID
    pub from_episode_id: String,
    /// Target episode UUID
    pub to_episode_id: String,
    /// Type of relationship (parent_child, depends_on, follows, related_to, blocks, duplicates, references)
    pub relationship_type: String,
    /// Optional explanation for the relationship
    pub reason: Option<String>,
    /// Optional priority (1-10, higher is more important)
    pub priority: Option<u8>,
    /// Optional creator identifier
    pub created_by: Option<String>,
}

/// Output from adding a relationship between episodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEpisodeRelationshipOutput {
    /// Whether operation was successful
    pub success: bool,
    /// UUID of the created relationship
    pub relationship_id: String,
    /// Source episode ID
    pub from_episode_id: String,
    /// Target episode ID
    pub to_episode_id: String,
    /// Type of relationship
    pub relationship_type: String,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for removing a relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveEpisodeRelationshipInput {
    /// Relationship UUID to remove
    pub relationship_id: String,
}

/// Output from removing a relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveEpisodeRelationshipOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Relationship ID that was removed
    pub relationship_id: String,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for getting episode relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEpisodeRelationshipsInput {
    /// Episode UUID to query
    pub episode_id: String,
    /// Direction filter (outgoing, incoming, or both)
    pub direction: Option<String>,
    /// Optional relationship type filter
    pub relationship_type: Option<String>,
}

/// A relationship edge in the output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEdge {
    /// Relationship ID
    pub id: String,
    /// Source episode ID
    pub from: String,
    /// Target episode ID
    pub to: String,
    /// Type of relationship
    pub relationship_type: String,
    /// Reason for the relationship
    pub reason: Option<String>,
    /// Priority
    pub priority: Option<u8>,
    /// Created by
    pub created_by: Option<String>,
    /// Created at timestamp
    pub created_at: String,
}

/// Output from getting episode relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEpisodeRelationshipsOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Episode ID queried
    pub episode_id: String,
    /// Outgoing relationships (this episode -> others)
    pub outgoing: Vec<RelationshipEdge>,
    /// Incoming relationships (others -> this episode)
    pub incoming: Vec<RelationshipEdge>,
    /// Total number of relationships
    pub total_count: usize,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for finding related episodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindRelatedEpisodesInput {
    /// Episode UUID to find relationships for
    pub episode_id: String,
    /// Optional relationship type filter
    pub relationship_type: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Whether to include relationship metadata
    pub include_metadata: Option<bool>,
}

/// A related episode result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEpisode {
    /// Episode ID
    pub episode_id: String,
    /// Task description
    pub task_description: String,
    /// Task type
    pub task_type: String,
    /// Relationship type
    pub relationship_type: String,
    /// Direction (outgoing or incoming)
    pub direction: String,
    /// Relationship metadata
    pub reason: Option<String>,
    /// Priority
    pub priority: Option<u8>,
}

/// Output from finding related episodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindRelatedEpisodesOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Episode ID queried
    pub episode_id: String,
    /// Related episodes found
    pub related_episodes: Vec<RelatedEpisode>,
    /// Total count
    pub count: usize,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for checking if a relationship exists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRelationshipExistsInput {
    /// Source episode UUID
    pub from_episode_id: String,
    /// Target episode UUID
    pub to_episode_id: String,
    /// Type of relationship
    pub relationship_type: String,
}

/// Output from checking if a relationship exists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRelationshipExistsOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Whether the relationship exists
    pub exists: bool,
    /// Source episode ID
    pub from_episode_id: String,
    /// Target episode ID
    pub to_episode_id: String,
    /// Type of relationship
    pub relationship_type: String,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for getting dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphInput {
    /// Root episode UUID
    pub episode_id: String,
    /// Maximum traversal depth (1-5)
    pub depth: Option<usize>,
    /// Output format (json or dot)
    pub format: Option<String>,
}

/// A node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNode {
    /// Episode ID
    pub id: String,
    /// Task description
    pub task_description: String,
    /// Task type
    pub task_type: String,
    /// Whether episode is complete
    pub is_complete: bool,
}

/// Output from getting dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Root episode ID
    pub root: String,
    /// Number of nodes in the graph
    pub node_count: usize,
    /// Number of edges in the graph
    pub edge_count: usize,
    /// Nodes (episodes) in the graph
    pub nodes: Vec<RelationshipNode>,
    /// Edges (relationships) in the graph
    pub edges: Vec<RelationshipEdge>,
    /// DOT format representation (if requested)
    pub dot: Option<String>,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for validating no cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateNoCyclesInput {
    /// Source episode UUID (proposed from)
    pub from_episode_id: String,
    /// Target episode UUID (proposed to)
    pub to_episode_id: String,
    /// Type of relationship being added
    pub relationship_type: String,
}

/// Output from validating no cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateNoCyclesOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Whether adding this relationship would create a cycle
    pub would_create_cycle: bool,
    /// Whether the relationship is valid (no cycle)
    pub is_valid: bool,
    /// Path of the cycle if one would be created
    pub cycle_path: Option<Vec<String>>,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for getting topological order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTopologicalOrderInput {
    /// Array of episode UUIDs to sort
    pub episode_ids: Vec<String>,
}

/// An episode in topological order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologicalEpisode {
    /// Episode ID
    pub episode_id: String,
    /// Task description
    pub task_description: String,
    /// Position in order (1-based)
    pub position: usize,
}

/// Output from getting topological order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTopologicalOrderOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Episodes in topological order
    pub order: Vec<TopologicalEpisode>,
    /// Total count
    pub count: usize,
    /// Whether the graph has cycles (would prevent topological sort)
    pub has_cycles: bool,
    /// Message describing the result
    pub message: String,
}
