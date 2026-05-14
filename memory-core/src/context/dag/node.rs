//! State nodes for DAG-based context management (WG-134).
//!
//! Each node represents a shared context attribute that can be
//! referenced by multiple episodes, reducing token duplication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// Unique identifier for a state node.
pub type NodeId = Uuid;

/// Type of shared context this node represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateNodeType {
    /// Programming language (e.g., "rust", "python")
    Language,
    /// Framework or library (e.g., "tokio", "axum")
    Framework,
    /// Domain or category (e.g., "web-api", "data-science")
    Domain,
    /// Task type (e.g., "Debugging", "Refactoring")
    TaskType,
    /// Complexity level
    Complexity,
    /// Shared tag (e.g., "async", "rest")
    Tag,
    /// Composite node combining multiple attributes
    Composite,
}

/// A node in the state DAG representing shared context.
///
/// Nodes store shared context attributes that can be referenced
/// by multiple episodes, avoiding token duplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNode {
    /// Unique identifier for this node.
    pub node_id: NodeId,
    /// Type of context this node represents.
    pub node_type: StateNodeType,
    /// The actual context value (e.g., "rust", "web-api").
    pub value: String,
    /// Episodes referencing this node.
    pub episode_refs: HashSet<Uuid>,
    /// When this node was created.
    pub created_at: DateTime<Utc>,
    /// When this node was last accessed.
    pub last_accessed: DateTime<Utc>,
    /// Number of times this node has been accessed.
    pub access_count: u64,
    /// Optional parent node (for composite nodes).
    pub parent: Option<NodeId>,
    /// Child nodes (for composite nodes).
    pub children: HashSet<NodeId>,
}

impl StateNode {
    /// Create a new state node for a shared context attribute.
    ///
    /// # Arguments
    ///
    /// * `node_type` - Type of context (Language, Domain, etc.)
    /// * `value` - The actual value (e.g., "rust")
    ///
    /// # Returns
    ///
    /// A new `StateNode` with generated UUID
    #[must_use]
    pub fn new(node_type: StateNodeType, value: String) -> Self {
        Self {
            node_id: Uuid::new_v4(),
            node_type,
            value,
            episode_refs: HashSet::new(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            parent: None,
            children: HashSet::new(),
        }
    }

    /// Create a node with a specific ID (for testing/deserialization).
    #[must_use]
    pub fn with_id(node_id: NodeId, node_type: StateNodeType, value: String) -> Self {
        Self {
            node_id,
            node_type,
            value,
            episode_refs: HashSet::new(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            parent: None,
            children: HashSet::new(),
        }
    }

    /// Add an episode reference to this node.
    pub fn add_episode_ref(&mut self, episode_id: Uuid) {
        self.episode_refs.insert(episode_id);
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Remove an episode reference from this node.
    pub fn remove_episode_ref(&mut self, episode_id: &Uuid) {
        self.episode_refs.remove(episode_id);
    }

    /// Get the number of episodes referencing this node.
    #[must_use]
    pub fn ref_count(&self) -> usize {
        self.episode_refs.len()
    }

    /// Check if this node has any episode references.
    #[must_use]
    pub fn has_refs(&self) -> bool {
        !self.episode_refs.is_empty()
    }

    /// Mark this node as accessed (update timestamp and count).
    pub fn mark_accessed(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Create a composite node combining multiple child nodes.
    #[must_use]
    pub fn composite(children: Vec<&StateNode>) -> Self {
        let mut node = Self::new(
            StateNodeType::Composite,
            Self::compute_composite_value(&children),
        );
        for child in children {
            node.children.insert(child.node_id);
        }
        node
    }

    /// Compute the value for a composite node from its children.
    fn compute_composite_value(children: &[&StateNode]) -> String {
        let parts: Vec<String> = children
            .iter()
            .map(|n| format!("{}={}", n.node_type_name(), n.value))
            .collect();
        parts.join(",")
    }

    /// Get a human-readable name for this node's type.
    #[must_use]
    pub fn node_type_name(&self) -> &'static str {
        match self.node_type {
            StateNodeType::Language => "lang",
            StateNodeType::Framework => "framework",
            StateNodeType::Domain => "domain",
            StateNodeType::TaskType => "task_type",
            StateNodeType::Complexity => "complexity",
            StateNodeType::Tag => "tag",
            StateNodeType::Composite => "composite",
        }
    }

    /// Estimate the token count for this node's value.
    #[must_use]
    pub fn estimated_tokens(&self) -> usize {
        // Rough estimate: 1 token per 4 characters, plus overhead
        (self.value.len() / 4).max(1) + 2
    }

    /// Calculate the token savings from using this node.
    ///
    /// If N episodes reference this node, savings = N × token_count - token_count
    #[must_use]
    pub fn token_savings(&self) -> usize {
        if self.ref_count() <= 1 {
            return 0;
        }
        let per_episode_tokens = self.estimated_tokens();
        // Old: N episodes × tokens each
        // New: 1 node + N refs (refs are cheap, ~1 token each)
        let old_total = self.ref_count() * per_episode_tokens;
        let new_total = per_episode_tokens + self.ref_count(); // 1 full + N refs
        old_total - new_total
    }
}

impl PartialEq for StateNode {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}

impl Eq for StateNode {}

impl std::hash::Hash for StateNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
    }
}
