//! State edges for DAG-based context management (WG-134).
//!
//! Edges connect episodes to shared state nodes, representing
//! the relationship between episodes and their deduplicated context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::node::NodeId;

/// Type of relationship between episode and node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Episode has this context attribute (e.g., language="rust")
    HasAttribute,
    /// Episode inherits from parent episode's context
    InheritsFrom,
    /// Episode depends on another episode's context
    DependsOn,
    /// Episode is similar to another (for context sharing)
    SimilarTo,
}

/// An edge in the state DAG connecting episode to node.
///
/// Edges represent the relationship between episodes and
/// shared context nodes, enabling context deduplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEdge {
    /// Source episode ID.
    pub source_episode: Uuid,
    /// Target state node ID.
    pub target_node: NodeId,
    /// Type of relationship.
    pub edge_type: EdgeType,
    /// Strength of relationship (0.0-1.0).
    pub strength: f32,
    /// When this edge was created.
    pub created_at: DateTime<Utc>,
    /// Optional metadata about this relationship.
    pub metadata: EdgeMetadata,
}

/// Additional metadata about an edge.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// Source field name (e.g., "language", "domain").
    pub source_field: Option<String>,
    /// Whether this edge is primary (first choice) or secondary.
    pub is_primary: bool,
    /// Confidence level of this relationship.
    pub confidence: f32,
}

impl StateEdge {
    /// Create a new edge connecting an episode to a state node.
    ///
    /// # Arguments
    ///
    /// * `source_episode` - Episode ID
    /// * `target_node` - State node ID
    /// * `edge_type` - Type of relationship
    ///
    /// # Returns
    ///
    /// A new `StateEdge` with default strength and metadata
    #[must_use]
    pub fn new(source_episode: Uuid, target_node: NodeId, edge_type: EdgeType) -> Self {
        Self {
            source_episode,
            target_node,
            edge_type,
            strength: 1.0,
            created_at: Utc::now(),
            metadata: EdgeMetadata::default(),
        }
    }

    /// Create an attribute edge with source field info.
    #[must_use]
    pub fn attribute(source_episode: Uuid, target_node: NodeId, source_field: String) -> Self {
        Self {
            source_episode,
            target_node,
            edge_type: EdgeType::HasAttribute,
            strength: 1.0,
            created_at: Utc::now(),
            metadata: EdgeMetadata {
                source_field: Some(source_field),
                is_primary: true,
                confidence: 1.0,
            },
        }
    }

    /// Set the strength of this edge.
    pub fn set_strength(&mut self, strength: f32) {
        self.strength = strength.clamp(0.0, 1.0);
    }

    /// Set whether this is a primary edge.
    pub fn set_primary(&mut self, is_primary: bool) {
        self.metadata.is_primary = is_primary;
    }

    /// Check if this is a primary edge.
    #[must_use]
    pub fn is_primary(&self) -> bool {
        self.metadata.is_primary
    }

    /// Get the source field name (if set).
    #[must_use]
    pub fn source_field(&self) -> Option<&str> {
        self.metadata.source_field.as_deref()
    }

    /// Calculate token cost for representing this edge.
    ///
    /// Edges are cheap: just episode_id + node_id reference.
    #[must_use]
    pub fn estimated_tokens(&self) -> usize {
        // Episode ref: ~2 tokens, Node ref: ~1 token
        3
    }
}

impl PartialEq for StateEdge {
    fn eq(&self, other: &Self) -> bool {
        self.source_episode == other.source_episode
            && self.target_node == other.target_node
            && self.edge_type == other.edge_type
    }
}

impl Eq for StateEdge {}

impl std::hash::Hash for StateEdge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source_episode.hash(state);
        self.target_node.hash(state);
        self.edge_type.hash(state);
    }
}
