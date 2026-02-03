//! Types and output structures for relationship commands
//!
//! This module provides commands for managing relationships between episodes:
//! - add-relationship: Create a new relationship between two episodes
//! - remove-relationship: Remove an existing relationship
//! - list-relationships: List all relationships for an episode
//! - find-related: Find episodes related to a given episode
//! - dependency-graph: Generate a dependency graph visualization
//! - validate-cycles: Check for cycles in relationships
//! - topological-sort: Get topological ordering of episodes

use crate::output::Output;
use clap::{Subcommand, ValueEnum};
use colored::Colorize;
use memory_core::episode::{Direction, RelationshipType};
use serde::Serialize;
use std::io::Write;

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
    pub fn to_core_type(self) -> RelationshipType {
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
    pub fn to_core_direction(self) -> Direction {
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
