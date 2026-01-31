//! Query structures and graph visualization for episode relationships.
//!
//! This module provides:
//! - `RelationshipFilter` for advanced relationship queries
//! - `RelationshipGraph` for visualization and analysis
//! - Helper functions for querying episodes with their relationships

use crate::episode::{Direction, EpisodeRelationship, RelationshipType};
use crate::Episode;
use std::collections::HashMap;
use uuid::Uuid;

/// Filter options for finding related episodes.
#[derive(Debug, Clone, Default)]
pub struct RelationshipFilter {
    /// Filter by relationship type (None = all types)
    pub relationship_type: Option<RelationshipType>,
    /// Filter by direction (None = Both)
    pub direction: Option<Direction>,
    /// Maximum number of results (None = unlimited)
    pub limit: Option<usize>,
    /// Minimum priority (if relationships have priority)
    pub min_priority: Option<u8>,
}

impl RelationshipFilter {
    /// Create a new filter with default settings (no filtering)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by relationship type
    #[must_use]
    pub fn with_type(mut self, rel_type: RelationshipType) -> Self {
        self.relationship_type = Some(rel_type);
        self
    }

    /// Filter by direction
    #[must_use]
    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Set result limit
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set minimum priority
    #[must_use]
    pub fn with_min_priority(mut self, priority: u8) -> Self {
        self.min_priority = Some(priority);
        self
    }
}

/// Graph structure for visualization and analysis.
#[derive(Debug, Clone)]
pub struct RelationshipGraph {
    /// Root episode ID
    pub root: Uuid,
    /// All episodes in the graph
    pub nodes: HashMap<Uuid, Episode>,
    /// All relationships (edges)
    pub edges: Vec<EpisodeRelationship>,
}

impl RelationshipGraph {
    /// Create a new relationship graph with the given root episode.
    #[must_use]
    pub fn new(root: Uuid) -> Self {
        Self {
            root,
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Add an episode node to the graph.
    pub fn add_node(&mut self, episode: Episode) {
        self.nodes.insert(episode.episode_id, episode);
    }

    /// Add a relationship edge to the graph.
    pub fn add_edge(&mut self, relationship: EpisodeRelationship) {
        self.edges.push(relationship);
    }

    /// Get the number of nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges in the graph.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if the graph contains a specific episode.
    #[must_use]
    pub fn contains_node(&self, episode_id: Uuid) -> bool {
        self.nodes.contains_key(&episode_id)
    }

    /// Get all relationships for a specific episode.
    #[must_use]
    pub fn get_relationships_for(&self, episode_id: Uuid) -> Vec<&EpisodeRelationship> {
        self.edges
            .iter()
            .filter(|rel| rel.from_episode_id == episode_id || rel.to_episode_id == episode_id)
            .collect()
    }

    /// Export to DOT format for visualization.
    ///
    /// The DOT format can be rendered by Graphviz or other graph visualization tools.
    #[must_use]
    pub fn to_dot(&self) -> String {
        use std::fmt::Write;
        let mut dot = String::from("digraph RelationshipGraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Add nodes
        for (id, episode) in &self.nodes {
            let label = episode
                .task_description
                .chars()
                .take(30)
                .collect::<String>()
                .replace('"', "\\\"");
            let truncated_label = if episode.task_description.len() > 30 {
                format!("{label}...")
            } else {
                label
            };
            let _ = writeln!(dot, "  \"{id}\" [label=\"{truncated_label}\"];");
        }

        dot.push('\n');

        // Add edges
        for edge in &self.edges {
            let label = format!("{:?}", edge.relationship_type);
            let _ = writeln!(
                dot,
                "  \"{}\" -> \"{}\" [label=\"{label}\"];",
                edge.from_episode_id, edge.to_episode_id
            );
        }

        dot.push_str("}\n");
        dot
    }

    /// Export to JSON format for programmatic use.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        let nodes: Vec<serde_json::Value> = self
            .nodes
            .values()
            .map(|ep| {
                serde_json::json!({
                    "id": ep.episode_id.to_string(),
                    "task_description": ep.task_description,
                    "task_type": format!("{:?}", ep.task_type),
                    "is_complete": ep.is_complete(),
                })
            })
            .collect();

        let edges: Vec<serde_json::Value> = self
            .edges
            .iter()
            .map(|rel| {
                serde_json::json!({
                    "id": rel.id.to_string(),
                    "from": rel.from_episode_id.to_string(),
                    "to": rel.to_episode_id.to_string(),
                    "type": format!("{:?}", rel.relationship_type),
                    "metadata": {
                        "reason": rel.metadata.reason,
                        "priority": rel.metadata.priority,
                    }
                })
            })
            .collect();

        serde_json::json!({
            "root": self.root.to_string(),
            "node_count": self.node_count(),
            "edge_count": self.edge_count(),
            "nodes": nodes,
            "edges": edges,
        })
    }
}

/// Query result containing an episode and its relationships.
#[derive(Debug, Clone)]
pub struct EpisodeWithRelationships {
    /// The episode
    pub episode: Episode,
    /// Outgoing relationships (this episode -> others)
    pub outgoing: Vec<EpisodeRelationship>,
    /// Incoming relationships (others -> this episode)
    pub incoming: Vec<EpisodeRelationship>,
}

impl EpisodeWithRelationships {
    /// Get total relationship count.
    #[must_use]
    pub fn total_relationships(&self) -> usize {
        self.outgoing.len() + self.incoming.len()
    }

    /// Get relationships of a specific type.
    #[must_use]
    pub fn get_by_type(&self, rel_type: RelationshipType) -> Vec<&EpisodeRelationship> {
        self.outgoing
            .iter()
            .chain(self.incoming.iter())
            .filter(|rel| rel.relationship_type == rel_type)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskContext, TaskType};
    use crate::Episode;

    fn create_test_episode(_id: Uuid, description: &str) -> Episode {
        Episode::new(
            description.to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
    }

    #[test]
    fn test_relationship_filter_default() {
        let filter = RelationshipFilter::default();
        assert!(filter.relationship_type.is_none());
        assert!(filter.direction.is_none());
        assert!(filter.limit.is_none());
        assert!(filter.min_priority.is_none());
    }

    #[test]
    fn test_relationship_filter_builder() {
        let filter = RelationshipFilter::new()
            .with_type(RelationshipType::DependsOn)
            .with_direction(Direction::Outgoing)
            .with_limit(10)
            .with_min_priority(5);

        assert_eq!(filter.relationship_type, Some(RelationshipType::DependsOn));
        assert_eq!(filter.direction, Some(Direction::Outgoing));
        assert_eq!(filter.limit, Some(10));
        assert_eq!(filter.min_priority, Some(5));
    }

    #[test]
    fn test_relationship_graph_new() {
        let root = Uuid::new_v4();
        let graph = RelationshipGraph::new(root);

        assert_eq!(graph.root, root);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_relationship_graph_add_node() {
        let root = Uuid::new_v4();
        let mut graph = RelationshipGraph::new(root);

        let episode = create_test_episode(Uuid::new_v4(), "Test episode");
        graph.add_node(episode.clone());

        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node(episode.episode_id));
    }

    #[test]
    fn test_relationship_graph_add_edge() {
        let root = Uuid::new_v4();
        let mut graph = RelationshipGraph::new(root);

        let rel = EpisodeRelationship::with_reason(
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationshipType::DependsOn,
            "Test reason".to_string(),
        );

        graph.add_edge(rel);

        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_relationship_graph_to_dot() {
        let root = Uuid::new_v4();
        let mut graph = RelationshipGraph::new(root);

        let ep1 = create_test_episode(Uuid::new_v4(), "Episode 1");
        let ep2 = create_test_episode(Uuid::new_v4(), "Episode 2");

        graph.add_node(ep1.clone());
        graph.add_node(ep2.clone());

        let rel = EpisodeRelationship::with_reason(
            ep1.episode_id,
            ep2.episode_id,
            RelationshipType::DependsOn,
            "Depends on".to_string(),
        );
        graph.add_edge(rel);

        let dot = graph.to_dot();

        assert!(dot.contains("digraph RelationshipGraph"));
        assert!(dot.contains(&ep1.episode_id.to_string()));
        assert!(dot.contains(&ep2.episode_id.to_string()));
        assert!(dot.contains("DependsOn"));
    }

    #[test]
    fn test_relationship_graph_to_json() {
        let root = Uuid::new_v4();
        let mut graph = RelationshipGraph::new(root);

        let ep1 = create_test_episode(Uuid::new_v4(), "Episode 1");
        let ep2 = create_test_episode(Uuid::new_v4(), "Episode 2");

        graph.add_node(ep1.clone());
        graph.add_node(ep2.clone());

        let rel = EpisodeRelationship::with_reason(
            ep1.episode_id,
            ep2.episode_id,
            RelationshipType::DependsOn,
            "Depends on".to_string(),
        );
        graph.add_edge(rel);

        let json = graph.to_json();

        assert_eq!(json["node_count"], 2);
        assert_eq!(json["edge_count"], 1);
        assert!(json["nodes"].as_array().unwrap().len() == 2);
        assert!(json["edges"].as_array().unwrap().len() == 1);
    }

    #[test]
    fn test_episode_with_relationships() {
        let episode = create_test_episode(Uuid::new_v4(), "Test");
        let rel1 = EpisodeRelationship::with_reason(
            episode.episode_id,
            Uuid::new_v4(),
            RelationshipType::DependsOn,
            "Reason 1".to_string(),
        );
        let rel2 = EpisodeRelationship::with_reason(
            Uuid::new_v4(),
            episode.episode_id,
            RelationshipType::RelatedTo,
            "Reason 2".to_string(),
        );

        let ewr = EpisodeWithRelationships {
            episode,
            outgoing: vec![rel1],
            incoming: vec![rel2],
        };

        assert_eq!(ewr.total_relationships(), 2);
        assert_eq!(ewr.get_by_type(RelationshipType::DependsOn).len(), 1);
        assert_eq!(ewr.get_by_type(RelationshipType::RelatedTo).len(), 1);
    }
}
