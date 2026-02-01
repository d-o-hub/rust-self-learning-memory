//! Episode relationship CLI commands
//!
//! This module provides commands for managing relationships between episodes:
//! - add-relationship: Create a new relationship between two episodes
//! - remove-relationship: Remove an existing relationship
//! - list-relationships: List all relationships for an episode
//! - find-related: Find episodes related to a given episode
//! - dependency-graph: Generate a dependency graph visualization
//! - validate-cycles: Check for cycles in relationships
//! - topological-sort: Get topological ordering of episodes

use crate::config::Config;
use crate::output::{Output, OutputFormat};
use clap::{Subcommand, ValueEnum};
use colored::Colorize;
use memory_core::episode::{
    Direction, EpisodeRelationship, RelationshipMetadata, RelationshipType,
};
use memory_core::memory::relationship_query::RelationshipFilter;
use memory_core::SelfLearningMemory;
use serde::Serialize;
use std::io::Write;
use uuid::Uuid;

/// Relationship-related subcommands
#[derive(Subcommand)]
pub enum RelationshipCommands {
    /// Add a relationship between two episodes
    #[command(name = "add-relationship")]
    AddRelationship {
        /// Source episode ID
        #[arg(value_name = "FROM_ID")]
        from_episode_id: String,

        /// Target episode ID
        #[arg(long, value_name = "TO_ID")]
        to: String,

        /// Relationship type
        #[arg(long, value_name = "TYPE")]
        r#type: RelationshipTypeArg,

        /// Optional reason for the relationship
        #[arg(long, value_name = "REASON")]
        reason: Option<String>,

        /// Priority (1-10)
        #[arg(long, value_name = "1-10")]
        priority: Option<u8>,

        /// Creator identifier
        #[arg(long, value_name = "NAME")]
        created_by: Option<String>,
    },

    /// Remove a relationship by its ID
    #[command(name = "remove-relationship")]
    RemoveRelationship {
        /// Relationship ID to remove
        #[arg(value_name = "RELATIONSHIP_ID")]
        relationship_id: String,
    },

    /// List relationships for an episode
    #[command(name = "list-relationships")]
    ListRelationships {
        /// Episode ID to list relationships for
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Direction filter (outgoing, incoming, both)
        #[arg(long, value_name = "DIRECTION", default_value = "both")]
        direction: DirectionArg,

        /// Filter by relationship type
        #[arg(long, value_name = "TYPE")]
        r#type: Option<RelationshipTypeArg>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: ListFormat,
    },

    /// Find episodes related to a given episode
    #[command(name = "find-related")]
    FindRelated {
        /// Episode ID to find related episodes for
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Filter by relationship type
        #[arg(long, value_name = "TYPE")]
        r#type: Option<RelationshipTypeArg>,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: ListFormat,
    },

    /// Generate a dependency graph for an episode
    #[command(name = "dependency-graph")]
    DependencyGraph {
        /// Root episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Maximum traversal depth
        #[arg(short, long, default_value = "3")]
        depth: usize,

        /// Output format (dot, json, ascii)
        #[arg(short, long, value_enum, default_value = "ascii")]
        format: GraphFormat,

        /// Output file (defaults to stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<std::path::PathBuf>,
    },

    /// Validate that no cycles exist in relationships
    #[command(name = "validate-cycles")]
    ValidateCycles {
        /// Episode ID to start validation from
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Relationship type to check (defaults to all acyclic types)
        #[arg(long, value_name = "TYPE")]
        r#type: Option<RelationshipTypeArg>,
    },

    /// Get topological ordering of episodes
    #[command(name = "topological-sort")]
    TopologicalSort {
        /// Episode IDs to sort
        #[arg(value_name = "EPISODE_IDS", required = true)]
        episode_ids: Vec<String>,
    },
}

/// Relationship type argument for CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RelationshipTypeArg {
    /// Parent-child hierarchical relationship
    ParentChild,
    /// Dependency relationship
    DependsOn,
    /// Sequential relationship
    Follows,
    /// Loose association
    RelatedTo,
    /// Blocking relationship
    Blocks,
    /// Duplicate relationship
    Duplicates,
    /// Reference relationship
    References,
}

impl RelationshipTypeArg {
    /// Convert to core RelationshipType
    fn to_core_type(self) -> RelationshipType {
        match self {
            Self::ParentChild => RelationshipType::ParentChild,
            Self::DependsOn => RelationshipType::DependsOn,
            Self::Follows => RelationshipType::Follows,
            Self::RelatedTo => RelationshipType::RelatedTo,
            Self::Blocks => RelationshipType::Blocks,
            Self::Duplicates => RelationshipType::Duplicates,
            Self::References => RelationshipType::References,
        }
    }
}

/// Direction argument for CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DirectionArg {
    /// Outgoing relationships only
    Outgoing,
    /// Incoming relationships only
    Incoming,
    /// Both directions
    Both,
}

impl DirectionArg {
    /// Convert to core Direction
    fn to_core_direction(self) -> Direction {
        match self {
            Self::Outgoing => Direction::Outgoing,
            Self::Incoming => Direction::Incoming,
            Self::Both => Direction::Both,
        }
    }
}

/// Output format for list commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListFormat {
    /// Table format
    Table,
    /// JSON format
    Json,
}

/// Output format for graph commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GraphFormat {
    /// DOT format for Graphviz
    Dot,
    /// JSON format
    Json,
    /// ASCII tree format
    Ascii,
}

/// Result structure for add-relationship command
#[derive(Debug, Serialize)]
pub struct AddRelationshipResult {
    pub relationship_id: String,
    pub from_episode_id: String,
    pub to_episode_id: String,
    pub relationship_type: String,
    pub success: bool,
}

impl Output for AddRelationshipResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(writer, "{}", "Relationship Created".green().bold())?;
        writeln!(writer, "  ID: {}", self.relationship_id.dimmed())?;
        writeln!(writer, "  From: {}", self.from_episode_id)?;
        writeln!(writer, "  To: {}", self.to_episode_id)?;
        writeln!(writer, "  Type: {}", self.relationship_type.cyan())?;
        Ok(())
    }
}

/// Result structure for remove-relationship command
#[derive(Debug, Serialize)]
pub struct RemoveRelationshipResult {
    pub relationship_id: String,
    pub success: bool,
}

impl Output for RemoveRelationshipResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        if self.success {
            writeln!(
                writer,
                "{} Relationship {} removed",
                "✓".green().bold(),
                self.relationship_id
            )?;
        } else {
            writeln!(
                writer,
                "{} Failed to remove relationship {}",
                "✗".red().bold(),
                self.relationship_id
            )?;
        }
        Ok(())
    }
}

/// Relationship list item for display
#[derive(Debug, Serialize)]
pub struct RelationshipListItem {
    pub id: String,
    pub relationship_type: String,
    pub from: String,
    pub to: String,
    pub priority: Option<u8>,
    pub reason: Option<String>,
}

/// Result structure for list-relationships command
#[derive(Debug, Serialize)]
pub struct ListRelationshipsResult {
    pub relationships: Vec<RelationshipListItem>,
    pub total_count: usize,
}

impl Output for ListRelationshipsResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        if self.relationships.is_empty() {
            writeln!(writer, "{}", "No relationships found".yellow())?;
            return Ok(());
        }

        writeln!(
            writer,
            "{} {} relationship(s)\n",
            "→".blue().bold(),
            self.total_count
        )?;

        // Print table header
        writeln!(
            writer,
            "{:<36} {:<15} {:<36} {:<36} {:<9} {}",
            "ID".bold(),
            "Type".bold(),
            "From".bold(),
            "To".bold(),
            "Priority".bold(),
            "Reason".bold()
        )?;
        writeln!(writer, "{}", "─".repeat(140).dimmed())?;

        // Print rows
        for rel in &self.relationships {
            let priority_str = rel
                .priority
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string());
            let reason_str = rel
                .reason
                .as_ref()
                .map(|r| {
                    if r.len() > 30 {
                        format!("{}...", &r[..27])
                    } else {
                        r.clone()
                    }
                })
                .unwrap_or_else(|| "-".to_string());

            writeln!(
                writer,
                "{:<36} {:<15} {:<36} {:<36} {:<9} {}",
                rel.id.dimmed(),
                rel.relationship_type.cyan(),
                rel.from,
                rel.to,
                priority_str,
                reason_str
            )?;
        }

        Ok(())
    }
}

/// Related episode item for display
#[derive(Debug, Serialize)]
pub struct RelatedEpisodeItem {
    pub episode_id: String,
    pub task_description: String,
    pub relationship_type: String,
    pub direction: String,
}

/// Result structure for find-related command
#[derive(Debug, Serialize)]
pub struct FindRelatedResult {
    pub episodes: Vec<RelatedEpisodeItem>,
    pub total_count: usize,
}

impl Output for FindRelatedResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        if self.episodes.is_empty() {
            writeln!(writer, "{}", "No related episodes found".yellow())?;
            return Ok(());
        }

        writeln!(
            writer,
            "{} {} related episode(s)\n",
            "→".blue().bold(),
            self.total_count
        )?;

        // Print table header
        writeln!(
            writer,
            "{:<36} {:<15} {:<12} {}",
            "Episode ID".bold(),
            "Type".bold(),
            "Direction".bold(),
            "Description".bold()
        )?;
        writeln!(writer, "{}", "─".repeat(120).dimmed())?;

        // Print rows
        for ep in &self.episodes {
            let desc = if ep.task_description.len() > 40 {
                format!("{}...", &ep.task_description[..37])
            } else {
                ep.task_description.clone()
            };

            writeln!(
                writer,
                "{:<36} {:<15} {:<12} {}",
                ep.episode_id.dimmed(),
                ep.relationship_type.cyan(),
                ep.direction,
                desc
            )?;
        }

        Ok(())
    }
}

/// Result structure for dependency-graph command
#[derive(Debug, Serialize)]
pub struct DependencyGraphResult {
    pub root_episode_id: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub output: String,
    pub format: String,
}

impl Output for DependencyGraphResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(
            writer,
            "{} Dependency Graph for {}\n",
            "→".blue().bold(),
            self.root_episode_id.dimmed()
        )?;
        writeln!(
            writer,
            "  Nodes: {} | Edges: {} | Format: {}\n",
            self.node_count, self.edge_count, self.format
        )?;
        writeln!(writer, "{}", self.output)?;
        Ok(())
    }
}

/// Result structure for validate-cycles command
#[derive(Debug, Serialize)]
pub struct ValidateCyclesResult {
    pub episode_id: String,
    pub has_cycle: bool,
    pub cycle_path: Option<Vec<String>>,
    pub message: String,
}

impl Output for ValidateCyclesResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        if self.has_cycle {
            writeln!(
                writer,
                "{} Cycle detected for episode {}",
                "✗".red().bold(),
                self.episode_id
            )?;
            if let Some(path) = &self.cycle_path {
                writeln!(writer, "  Path: {}", path.join(" → "))?;
            }
        } else {
            writeln!(
                writer,
                "{} No cycles detected for episode {}",
                "✓".green().bold(),
                self.episode_id
            )?;
        }
        Ok(())
    }
}

/// Result structure for topological-sort command
#[derive(Debug, Serialize)]
pub struct TopologicalSortResult {
    pub ordered_episodes: Vec<String>,
    pub has_cycle: bool,
}

impl Output for TopologicalSortResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        if self.has_cycle {
            writeln!(
                writer,
                "{} Cannot perform topological sort: cycle detected in relationships",
                "✗".red().bold(),
            )?;
            return Ok(());
        }

        writeln!(writer, "{}", "Topological Order:".blue().bold())?;
        for (i, episode_id) in self.ordered_episodes.iter().enumerate() {
            writeln!(writer, "  {}. {}", i + 1, episode_id)?;
        }
        Ok(())
    }
}

/// Add a relationship between two episodes
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
fn detect_cycle_in_graph(
    graph: &memory_core::memory::relationship_query::RelationshipGraph,
    relationship_type: Option<RelationshipType>,
) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut rec_stack = std::collections::HashSet::new();

    // Check from root
    has_cycle_util(
        graph,
        graph.root,
        &mut visited,
        &mut rec_stack,
        relationship_type,
    )
}

fn has_cycle_util(
    graph: &memory_core::memory::relationship_query::RelationshipGraph,
    node_id: Uuid,
    visited: &mut std::collections::HashSet<Uuid>,
    rec_stack: &mut std::collections::HashSet<Uuid>,
    relationship_type: Option<RelationshipType>,
) -> bool {
    visited.insert(node_id);
    rec_stack.insert(node_id);

    // Find outgoing edges of specified type
    for edge in &graph.edges {
        let matches = edge.from_episode_id == node_id;
        if !matches {
            continue;
        }

        // Check relationship type filter
        if let Some(ref rel_type) = relationship_type {
            if edge.relationship_type != *rel_type {
                continue;
            }
        }

        let neighbor = edge.to_episode_id;
        if !visited.contains(&neighbor) {
            if has_cycle_util(graph, neighbor, visited, rec_stack, relationship_type) {
                return true;
            }
        } else if rec_stack.contains(&neighbor) {
            return true;
        }
    }

    rec_stack.remove(&node_id);
    false
}

/// Get topological ordering of episodes
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

/// Kahn's algorithm for topological sorting
fn topological_sort_kahn(nodes: &[Uuid], edges: &[(Uuid, Uuid)]) -> Vec<Uuid> {
    let mut in_degree: std::collections::HashMap<Uuid, usize> =
        nodes.iter().map(|&id| (id, 0)).collect();
    let mut adj_list: std::collections::HashMap<Uuid, Vec<Uuid>> =
        nodes.iter().map(|&id| (id, Vec::new())).collect();

    // Build adjacency list and calculate in-degrees
    for (from, to) in edges {
        adj_list.get_mut(from).unwrap().push(*to);
        *in_degree.get_mut(to).unwrap() += 1;
    }

    // Find all nodes with in-degree 0
    let mut queue: std::collections::VecDeque<Uuid> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut sorted = Vec::new();

    while let Some(node) = queue.pop_front() {
        sorted.push(node);

        for &neighbor in &adj_list[&node] {
            let deg = in_degree.get_mut(&neighbor).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_type_arg_conversion() {
        assert_eq!(
            RelationshipTypeArg::ParentChild.to_core_type(),
            RelationshipType::ParentChild
        );
        assert_eq!(
            RelationshipTypeArg::DependsOn.to_core_type(),
            RelationshipType::DependsOn
        );
        assert_eq!(
            RelationshipTypeArg::Follows.to_core_type(),
            RelationshipType::Follows
        );
        assert_eq!(
            RelationshipTypeArg::RelatedTo.to_core_type(),
            RelationshipType::RelatedTo
        );
        assert_eq!(
            RelationshipTypeArg::Blocks.to_core_type(),
            RelationshipType::Blocks
        );
        assert_eq!(
            RelationshipTypeArg::Duplicates.to_core_type(),
            RelationshipType::Duplicates
        );
        assert_eq!(
            RelationshipTypeArg::References.to_core_type(),
            RelationshipType::References
        );
    }

    #[test]
    fn test_direction_arg_conversion() {
        assert_eq!(
            DirectionArg::Outgoing.to_core_direction(),
            Direction::Outgoing
        );
        assert_eq!(
            DirectionArg::Incoming.to_core_direction(),
            Direction::Incoming
        );
        assert_eq!(DirectionArg::Both.to_core_direction(), Direction::Both);
    }

    #[test]
    fn test_add_relationship_result_output() {
        let result = AddRelationshipResult {
            relationship_id: "abc-123".to_string(),
            from_episode_id: "def-456".to_string(),
            to_episode_id: "ghi-789".to_string(),
            relationship_type: "DependsOn".to_string(),
            success: true,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Relationship Created"));
        assert!(output.contains("abc-123"));
        assert!(output.contains("def-456"));
        assert!(output.contains("ghi-789"));
        assert!(output.contains("DependsOn"));
    }

    #[test]
    fn test_remove_relationship_result_output() {
        let result = RemoveRelationshipResult {
            relationship_id: "abc-123".to_string(),
            success: true,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("✓"));
        assert!(output.contains("abc-123"));
    }

    #[test]
    fn test_list_relationships_result_output() {
        let result = ListRelationshipsResult {
            relationships: vec![RelationshipListItem {
                id: "rel-1".to_string(),
                relationship_type: "DependsOn".to_string(),
                from: "ep-1".to_string(),
                to: "ep-2".to_string(),
                priority: Some(8),
                reason: Some("Test reason".to_string()),
            }],
            total_count: 1,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("1 relationship(s)"));
        assert!(output.contains("rel-1"));
        assert!(output.contains("ep-1"));
        assert!(output.contains("ep-2"));
    }

    #[test]
    fn test_list_relationships_empty_output() {
        let result = ListRelationshipsResult {
            relationships: vec![],
            total_count: 0,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("No relationships found"));
    }

    #[test]
    fn test_find_related_result_output() {
        let result = FindRelatedResult {
            episodes: vec![RelatedEpisodeItem {
                episode_id: "ep-2".to_string(),
                task_description: "Related task".to_string(),
                relationship_type: "DependsOn".to_string(),
                direction: "outgoing".to_string(),
            }],
            total_count: 1,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("1 related episode(s)"));
        assert!(output.contains("ep-2"));
        assert!(output.contains("Related task"));
    }

    #[test]
    fn test_validate_cycles_result_no_cycle() {
        let result = ValidateCyclesResult {
            episode_id: "ep-1".to_string(),
            has_cycle: false,
            cycle_path: None,
            message: "No cycles detected".to_string(),
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("✓"));
        assert!(output.contains("No cycles detected"));
    }

    #[test]
    fn test_validate_cycles_result_with_cycle() {
        let result = ValidateCyclesResult {
            episode_id: "ep-1".to_string(),
            has_cycle: true,
            cycle_path: Some(vec![
                "ep-1".to_string(),
                "ep-2".to_string(),
                "ep-1".to_string(),
            ]),
            message: "Cycle detected".to_string(),
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("✗"));
        assert!(output.contains("Cycle detected"));
    }

    #[test]
    fn test_topological_sort_result() {
        let result = TopologicalSortResult {
            ordered_episodes: vec!["ep-1".to_string(), "ep-2".to_string(), "ep-3".to_string()],
            has_cycle: false,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Topological Order"));
        assert!(output.contains("1. ep-1"));
        assert!(output.contains("2. ep-2"));
        assert!(output.contains("3. ep-3"));
    }

    #[test]
    fn test_topological_sort_result_with_cycle() {
        let result = TopologicalSortResult {
            ordered_episodes: vec![],
            has_cycle: true,
        };

        let mut buffer = Vec::new();
        result.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("✗"));
        assert!(output.contains("cycle detected"));
    }

    #[test]
    fn test_topological_sort_kahn() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        let nodes = vec![id1, id2, id3];
        let edges = vec![(id1, id2), (id2, id3)];

        let sorted = topological_sort_kahn(&nodes, &edges);

        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], id1);
        assert_eq!(sorted[1], id2);
        assert_eq!(sorted[2], id3);
    }

    #[test]
    fn test_topological_sort_kahn_with_cycle() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        let nodes = vec![id1, id2, id3];
        // Create cycle: 1 -> 2 -> 3 -> 1
        let edges = vec![(id1, id2), (id2, id3), (id3, id1)];

        let sorted = topological_sort_kahn(&nodes, &edges);

        // Should not include all nodes due to cycle
        assert!(sorted.len() < 3);
    }
}
