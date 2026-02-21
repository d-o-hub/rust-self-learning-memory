//! Episode relationship CLI commands

use colored::Colorize;
use memory_core::episode::{Direction, RelationshipMetadata};
use memory_core::memory::SelfLearningMemory;
use memory_core::memory::relationship_query::RelationshipFilter;
use uuid::Uuid;

use crate::config::Config;
use crate::output::OutputFormat;

mod helpers;
mod types;

use helpers::*;
pub use types::*;

#[allow(clippy::too_many_arguments)]
pub async fn add_relationship(
    from_episode_id: String,
    to: String,
    relationship_type: RelationshipTypeArg,
    reason: Option<String>,
    priority: Option<u8>,
    created_by: Option<String>,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    // Parse UUIDs
    let from_id = Uuid::parse_str(&from_episode_id)
        .map_err(|e| anyhow::anyhow!("Invalid from_episode_id UUID: {}", e))?;
    let to_id = Uuid::parse_str(&to).map_err(|e| anyhow::anyhow!("Invalid to UUID: {}", e))?;

    let rel_type = relationship_type.to_core_type();

    if dry_run {
        println!("Would create relationship:");
        println!("  From: {}", from_episode_id);
        println!("  To: {}", to);
        println!("  Type: {:?}", rel_type);
        if let Some(ref r) = reason {
            println!("  Reason: {}", r);
        }
        if let Some(p) = priority {
            println!("  Priority: {}", p);
        }
        if let Some(ref c) = created_by {
            println!("  Created by: {}", c);
        }
        return Ok(());
    }

    // Build metadata
    let metadata = RelationshipMetadata {
        reason,
        created_by,
        priority,
        custom_fields: std::collections::HashMap::new(),
    };

    // Create relationship
    let relationship_id = memory
        .add_episode_relationship(from_id, to_id, rel_type, metadata)
        .await?;

    let result = AddRelationshipResult {
        relationship_id: relationship_id.to_string(),
        from_episode_id,
        to_episode_id: to,
        relationship_type: format!("{:?}", rel_type),
        success: true,
    };

    format.print_output(&result)
}

/// Remove a relationship by its ID
pub async fn remove_relationship(
    relationship_id: String,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    let rel_id = Uuid::parse_str(&relationship_id)
        .map_err(|e| anyhow::anyhow!("Invalid relationship_id UUID: {}", e))?;

    if dry_run {
        println!("Would remove relationship: {}", relationship_id);
        return Ok(());
    }

    memory.remove_episode_relationship(rel_id).await?;

    let result = RemoveRelationshipResult {
        relationship_id,
        success: true,
    };

    format.print_output(&result)
}

/// List relationships for an episode
#[allow(clippy::too_many_arguments)]
pub async fn list_relationships(
    episode_id: String,
    direction: DirectionArg,
    relationship_type: Option<RelationshipTypeArg>,
    list_format: ListFormat,
    memory: &SelfLearningMemory,
    _config: &Config,
    output_format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    let ep_id = Uuid::parse_str(&episode_id)
        .map_err(|e| anyhow::anyhow!("Invalid episode_id UUID: {}", e))?;

    let dir = direction.to_core_direction();

    let relationships = memory.get_episode_relationships(ep_id, dir).await?;

    // Filter by type if specified
    let filtered: Vec<_> = relationships
        .into_iter()
        .filter(|rel| {
            if let Some(filter_type) = relationship_type {
                rel.relationship_type == filter_type.to_core_type()
            } else {
                true
            }
        })
        .collect();

    // Convert to display items
    let items: Vec<RelationshipListItem> = filtered
        .into_iter()
        .map(|rel| RelationshipListItem {
            id: rel.id.to_string(),
            relationship_type: format!("{:?}", rel.relationship_type),
            from: rel.from_episode_id.to_string(),
            to: rel.to_episode_id.to_string(),
            priority: rel.metadata.priority,
            reason: rel.metadata.reason,
        })
        .collect();

    let result = ListRelationshipsResult {
        total_count: items.len(),
        relationships: items,
    };

    match list_format {
        ListFormat::Table => output_format.print_output(&result),
        ListFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
    }
}

/// Find episodes related to a given episode
#[allow(clippy::too_many_arguments)]
pub async fn find_related(
    episode_id: String,
    relationship_type: Option<RelationshipTypeArg>,
    limit: usize,
    list_format: ListFormat,
    memory: &SelfLearningMemory,
    _config: &Config,
    output_format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    let ep_id = Uuid::parse_str(&episode_id)
        .map_err(|e| anyhow::anyhow!("Invalid episode_id UUID: {}", e))?;

    // Build filter
    let filter = RelationshipFilter {
        relationship_type: relationship_type.map(|t| t.to_core_type()),
        direction: None,
        limit: Some(limit),
        min_priority: None,
    };

    let related_ids = memory.find_related_episodes(ep_id, filter).await?;

    // Get episode details for each related episode
    let mut items = Vec::new();
    for related_id in &related_ids {
        let Ok(episode) = memory.get_episode(*related_id).await else {
            continue;
        };
        // Get the relationship to determine type and direction
        let rels = memory
            .get_episode_relationships(ep_id, Direction::Both)
            .await?;
        for rel in rels {
            let is_match = rel.from_episode_id == *related_id || rel.to_episode_id == *related_id;
            if !is_match {
                continue;
            }
            let direction = if rel.from_episode_id == ep_id {
                "outgoing"
            } else {
                "incoming"
            };
            items.push(RelatedEpisodeItem {
                episode_id: related_id.to_string(),
                task_description: episode.task_description.clone(),
                relationship_type: format!("{:?}", rel.relationship_type),
                direction: direction.to_string(),
            });
            break;
        }
    }

    let result = FindRelatedResult {
        total_count: items.len(),
        episodes: items,
    };

    match list_format {
        ListFormat::Table => output_format.print_output(&result),
        ListFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
    }
}

/// Generate a dependency graph for an episode
#[allow(clippy::too_many_arguments)]
pub async fn dependency_graph(
    episode_id: String,
    depth: usize,
    graph_format: GraphFormat,
    output: Option<std::path::PathBuf>,
    memory: &SelfLearningMemory,
    _config: &Config,
    output_format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    let ep_id = Uuid::parse_str(&episode_id)
        .map_err(|e| anyhow::anyhow!("Invalid episode_id UUID: {}", e))?;

    let graph = memory.build_relationship_graph(ep_id, depth).await?;

    let (output_str, format_name) = match graph_format {
        GraphFormat::Dot => (graph.to_dot(), "dot"),
        GraphFormat::Json => (graph.to_json().to_string(), "json"),
        GraphFormat::Ascii => (render_ascii_tree(&graph, ep_id), "ascii"),
    };

    // Write to file or stdout
    if let Some(path) = output {
        std::fs::write(&path, &output_str)?;
        println!("Graph written to {}", path.display());
    }

    let result = DependencyGraphResult {
        root_episode_id: episode_id,
        node_count: graph.node_count(),
        edge_count: graph.edge_count(),
        output: output_str,
        format: format_name.to_string(),
    };

    output_format.print_output(&result)
}

/// Render graph as ASCII tree
fn render_ascii_tree(
    graph: &memory_core::memory::relationship_query::RelationshipGraph,
    root_id: Uuid,
) -> String {
    let mut output = String::new();
    let mut visited = std::collections::HashSet::new();

    fn render_node(
        graph: &memory_core::memory::relationship_query::RelationshipGraph,
        node_id: Uuid,
        prefix: &str,
        is_last: bool,
        output: &mut String,
        visited: &mut std::collections::HashSet<Uuid>,
    ) {
        if visited.contains(&node_id) {
            output.push_str(&format!(
                "{}[{}] (cycle)\n",
                prefix,
                node_id.to_string().dimmed()
            ));
            return;
        }
        visited.insert(node_id);

        // Get episode info
        let label = if let Some(ep) = graph.nodes.get(&node_id) {
            let desc = if ep.task_description.len() > 30 {
                format!("{}...", &ep.task_description[..27])
            } else {
                ep.task_description.clone()
            };
            format!("{} ({})", desc, node_id.to_string().dimmed())
        } else {
            node_id.to_string()
        };

        let branch = if is_last { "└── " } else { "├── " };
        output.push_str(&format!("{}{}{}\n", prefix, branch, label));

        // Find outgoing relationships
        let outgoing: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.from_episode_id == node_id)
            .collect();

        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        for (i, edge) in outgoing.iter().enumerate() {
            let child_is_last = i == outgoing.len() - 1;
            let rel_label = format!("{:?}", edge.relationship_type).cyan();
            output.push_str(&format!(
                "{}{}── {} → ",
                child_prefix,
                if child_is_last { "└" } else { "├" },
                rel_label
            ));
            render_node(
                graph,
                edge.to_episode_id,
                &child_prefix,
                child_is_last,
                output,
                visited,
            );
        }
    }

    render_node(graph, root_id, "", true, &mut output, &mut visited);
    output
}

/// Validate that no cycles exist in relationships
pub async fn validate_cycles(
    episode_id: String,
    relationship_type: Option<RelationshipTypeArg>,
    memory: &SelfLearningMemory,
    _config: &Config,
    output_format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    let ep_id = Uuid::parse_str(&episode_id)
        .map_err(|e| anyhow::anyhow!("Invalid episode_id UUID: {}", e))?;

    // Build graph and detect cycles
    let graph = memory.build_relationship_graph(ep_id, 10).await?;

    // Simple cycle detection using DFS
    let has_cycle = detect_cycle_in_graph(&graph, relationship_type.map(|t| t.to_core_type()));

    let result = ValidateCyclesResult {
        episode_id,
        has_cycle,
        cycle_path: None, // Could be enhanced to return actual cycle path
        message: if has_cycle {
            "Cycle detected".to_string()
        } else {
            "No cycles detected".to_string()
        },
    };

    output_format.print_output(&result)
}

/// Detect cycle in graph using DFS
pub async fn topological_sort(
    episode_ids: Vec<String>,
    memory: &SelfLearningMemory,
    _config: &Config,
    output_format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    // Parse all episode IDs
    let mut ids = Vec::new();
    for id_str in &episode_ids {
        let id = Uuid::parse_str(id_str)
            .map_err(|e| anyhow::anyhow!("Invalid episode_id UUID '{}': {}", id_str, e))?;
        ids.push(id);
    }

    // Build combined graph from all episodes
    let mut all_edges: Vec<(Uuid, Uuid)> = Vec::new();
    for id in &ids {
        let graph = memory.build_relationship_graph(*id, 10).await?;
        for edge in &graph.edges {
            // Only include edges between specified episodes
            if ids.contains(&edge.from_episode_id) && ids.contains(&edge.to_episode_id) {
                // Only include acyclic relationship types
                if edge.relationship_type.requires_acyclic() {
                    all_edges.push((edge.from_episode_id, edge.to_episode_id));
                }
            }
        }
    }

    // Perform topological sort
    let sorted = topological_sort_kahn(&ids, &all_edges);

    let has_cycle = sorted.len() != ids.len();

    let ordered_strings: Vec<String> = sorted.iter().map(|id| id.to_string()).collect();

    let result = TopologicalSortResult {
        ordered_episodes: ordered_strings,
        has_cycle,
    };

    output_format.print_output(&result)
}
