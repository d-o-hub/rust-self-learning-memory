//! State DAG structure for context management (WG-134).
//!
//! The StateDag manages shared context nodes and edges,
//! enabling efficient context assembly with token reduction.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

use super::edge::{EdgeType, StateEdge};
use super::node::{NodeId, StateNode, StateNodeType};

use crate::episode::Episode;

/// The DAG structure managing shared context state.
///
/// Coordinates nodes and edges to enable efficient context
/// assembly with significant token reduction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateDag {
    /// All state nodes indexed by node ID.
    nodes: HashMap<NodeId, StateNode>,
    /// All edges indexed by source episode ID.
    edges_by_episode: HashMap<Uuid, Vec<StateEdge>>,
    /// Index: (node_type, value) -> node_id for fast lookup.
    node_index: HashMap<(StateNodeType, String), NodeId>,
    /// Statistics about the DAG.
    stats: DagStats,
}

/// Statistics about the StateDag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DagStats {
    /// Total number of nodes.
    pub total_nodes: u64,
    /// Total number of edges.
    pub total_edges: u64,
    /// Total episodes registered.
    pub total_episodes: u64,
    /// Estimated token savings from deduplication.
    pub token_savings: u64,
    /// Most referenced node (shared context).
    pub most_ref_node: Option<(NodeId, usize)>,
    /// Last update time.
    pub last_updated: DateTime<Utc>,
}

impl StateDag {
    /// Create a new empty StateDag.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an episode's context in the DAG.
    ///
    /// Creates or references nodes for shared context attributes.
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to register
    ///
    /// # Returns
    ///
    /// Number of new edges created
    pub fn register_episode(&mut self, episode: &Episode) -> usize {
        let episode_id = episode.episode_id;
        let mut edges_created = 0;

        // Register task type
        let task_type_str = episode.task_type.to_string();
        let node_id = self.get_or_create_node(StateNodeType::TaskType, task_type_str);
        self.add_edge(episode_id, node_id, EdgeType::HasAttribute, "task_type");
        edges_created += 1;

        // Register context attributes
        if let Some(ref language) = episode.context.language {
            let node_id = self.get_or_create_node(StateNodeType::Language, language.clone());
            self.add_edge(episode_id, node_id, EdgeType::HasAttribute, "language");
            edges_created += 1;
        }

        if let Some(ref framework) = episode.context.framework {
            let node_id = self.get_or_create_node(StateNodeType::Framework, framework.clone());
            self.add_edge(episode_id, node_id, EdgeType::HasAttribute, "framework");
            edges_created += 1;
        }

        let domain = episode.context.domain.clone();
        let node_id = self.get_or_create_node(StateNodeType::Domain, domain);
        self.add_edge(episode_id, node_id, EdgeType::HasAttribute, "domain");
        edges_created += 1;

        // Register complexity
        let complexity_str = format!("{:?}", episode.context.complexity);
        let node_id = self.get_or_create_node(StateNodeType::Complexity, complexity_str);
        self.add_edge(episode_id, node_id, EdgeType::HasAttribute, "complexity");
        edges_created += 1;

        // Register tags
        for tag in &episode.context.tags {
            let node_id = self.get_or_create_node(StateNodeType::Tag, tag.clone());
            self.add_edge(episode_id, node_id, EdgeType::HasAttribute, "tag");
            edges_created += 1;
        }

        // Update stats
        self.stats.total_episodes += 1;
        self.stats.last_updated = Utc::now();
        self.update_token_savings();

        debug!(
            episode_id = %episode_id,
            edges_created = edges_created,
            total_nodes = self.nodes.len(),
            "Registered episode in StateDag"
        );

        edges_created
    }

    /// Get or create a node for a shared context attribute.
    fn get_or_create_node(&mut self, node_type: StateNodeType, value: String) -> NodeId {
        let key = (node_type, value.clone());

        // Check if node already exists
        if let Some(&node_id) = self.node_index.get(&key) {
            // Update existing node
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.mark_accessed();
            }
            return node_id;
        }

        // Create new node
        let node = StateNode::new(node_type, value);
        let node_id = node.node_id;

        self.nodes.insert(node_id, node);
        self.node_index.insert(key, node_id);
        self.stats.total_nodes += 1;

        node_id
    }

    /// Add an edge from episode to node.
    fn add_edge(
        &mut self,
        episode_id: Uuid,
        node_id: NodeId,
        _edge_type: EdgeType,
        source_field: &str,
    ) {
        let edge = StateEdge::attribute(episode_id, node_id, source_field.to_string());

        // Add to episode index
        self.edges_by_episode
            .entry(episode_id)
            .or_default()
            .push(edge.clone());

        // Add episode ref to node
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.add_episode_ref(episode_id);
        }

        self.stats.total_edges += 1;
    }

    /// Remove an episode from the DAG.
    ///
    /// Removes edges and cleans up unreferenced nodes.
    /// Returns `true` if the episode was found and removed.
    #[must_use]
    pub fn remove_episode(&mut self, episode_id: &Uuid) -> bool {
        // Remove edges
        let removed = if let Some(edges) = self.edges_by_episode.remove(episode_id) {
            let edge_count = edges.len() as u64;
            for edge in edges {
                // Remove episode ref from node
                if let Some(node) = self.nodes.get_mut(&edge.target_node) {
                    node.remove_episode_ref(episode_id);
                }
            }
            // Decrement edge count now that edges are removed
            self.stats.total_edges = self.stats.total_edges.saturating_sub(edge_count);
            true
        } else {
            false
        };

        // Clean up unreferenced nodes
        self.cleanup_unreferenced_nodes();

        if removed {
            self.stats.total_episodes = self.stats.total_episodes.saturating_sub(1);
            self.stats.last_updated = Utc::now();
            self.update_token_savings();
        }

        removed
    }

    /// Remove nodes with no episode references.
    fn cleanup_unreferenced_nodes(&mut self) {
        let unreferenced: Vec<NodeId> = self
            .nodes
            .iter()
            .filter(|(_, node)| !node.has_refs())
            .map(|(id, _)| *id)
            .collect();

        for node_id in unreferenced {
            if let Some(node) = self.nodes.remove(&node_id) {
                let key = (node.node_type, node.value);
                self.node_index.remove(&key);
                self.stats.total_nodes = self.stats.total_nodes.saturating_sub(1);
            }
        }
    }

    /// Update token savings calculation.
    fn update_token_savings(&mut self) {
        let total_savings: usize = self.nodes.values().map(|n| n.token_savings()).sum();
        self.stats.token_savings = total_savings as u64;

        // Find most referenced node
        let most_ref = self
            .nodes
            .iter()
            .max_by_key(|(_, n)| n.ref_count())
            .map(|(id, n)| (*id, n.ref_count()));
        self.stats.most_ref_node = most_ref;
    }

    /// Get a node by ID.
    #[must_use]
    pub fn get_node(&self, node_id: &NodeId) -> Option<&StateNode> {
        self.nodes.get(node_id)
    }

    /// Get all edges for an episode.
    #[must_use]
    pub fn get_episode_edges(&self, episode_id: &Uuid) -> Option<&Vec<StateEdge>> {
        self.edges_by_episode.get(episode_id)
    }

    /// Get all nodes of a specific type.
    #[must_use]
    pub fn nodes_by_type(&self, node_type: StateNodeType) -> Vec<&StateNode> {
        self.nodes
            .values()
            .filter(|n| n.node_type == node_type)
            .collect()
    }

    /// Get nodes for an episode's context.
    #[must_use]
    pub fn get_episode_nodes(&self, episode_id: &Uuid) -> Vec<&StateNode> {
        self.edges_by_episode
            .get(episode_id)
            .map(|edges| {
                edges
                    .iter()
                    .filter_map(|e| self.nodes.get(&e.target_node))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get shared context between episodes.
    ///
    /// Returns nodes that are referenced by **all** given episodes.
    /// Uses frequency counting across all episodes (O(E × N) count +
    /// O(N) filter) rather than sequential narrowing from a single
    /// starting episode, which ensures the result is correct regardless
    /// of which episode happens to appear first in the slice.
    #[must_use]
    pub fn get_shared_context(&self, episode_ids: &[Uuid]) -> Vec<&StateNode> {
        if episode_ids.is_empty() {
            return Vec::new();
        }

        let target = episode_ids.len();

        // Count how many episodes reference each node.
        let mut freq: HashMap<NodeId, usize> = HashMap::with_capacity(self.nodes.len());
        for ep_id in episode_ids {
            for node in self.get_episode_nodes(ep_id) {
                *freq.entry(node.node_id).or_insert(0) += 1;
            }
        }

        // Return only nodes referenced by all episodes.
        freq.into_iter()
            .filter(|(_, count)| *count == target)
            .filter_map(|(id, _)| self.nodes.get(&id))
            .collect()
    }

    /// Calculate token reduction percentage.
    ///
    /// Uses the same formula as `StateNode::token_savings()`:
    /// - Old: N episodes × tokens each
    /// - New: 1 full copy + N refs (1 token each)
    #[must_use]
    pub fn reduction_percentage(&self) -> f32 {
        if self.stats.total_episodes == 0 {
            return 0.0;
        }

        // Estimate original token cost (without DAG): every episode stores full value
        let estimated_without_dag: u64 = self
            .nodes
            .values()
            .map(|n| n.estimated_tokens() as u64 * n.ref_count() as u64)
            .sum();

        if estimated_without_dag == 0 {
            return 0.0;
        }

        // DAG cost: 1 full copy per node + 1 token ref per episode that references it
        let dag_cost: u64 = self
            .nodes
            .values()
            .map(|n| n.estimated_tokens() as u64 + n.ref_count() as u64)
            .sum();

        let reduction =
            (estimated_without_dag.saturating_sub(dag_cost)) as f32 / estimated_without_dag as f32;
        reduction * 100.0
    }

    /// Get current statistics.
    #[must_use]
    pub fn stats(&self) -> &DagStats {
        &self.stats
    }

    /// Get total node count.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get total edge count.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges_by_episode.values().map(|v| v.len()).sum()
    }

    /// Clear all nodes and edges.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges_by_episode.clear();
        self.node_index.clear();
        self.stats = DagStats::default();
    }
}
