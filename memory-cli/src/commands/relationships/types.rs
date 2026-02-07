//! Types and structures for relationship commands

use crate::output::Output;
use clap::ValueEnum;
use colored::Colorize;
use memory_core::episode::RelationshipType;
use serde::Serialize;
use std::io::Write;

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
    pub fn to_core_direction(self) -> memory_core::episode::Direction {
        match self {
            Self::Outgoing => memory_core::episode::Direction::Outgoing,
            Self::Incoming => memory_core::episode::Direction::Incoming,
            Self::Both => memory_core::episode::Direction::Both,
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
    /// Plain text format
    Text,
}

/// Result structure for add command
#[derive(Debug, Serialize)]
pub struct AddResult {
    pub relationship_id: String,
    pub source_episode_id: String,
    pub target_episode_id: String,
    pub relationship_type: String,
    pub success: bool,
}

impl Output for AddResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(writer, "{}", "Relationship Created".green().bold())?;
        writeln!(writer, "  ID: {}", self.relationship_id.dimmed())?;
        writeln!(writer, "  Source: {}", self.source_episode_id)?;
        writeln!(writer, "  Target: {}", self.target_episode_id)?;
        writeln!(writer, "  Type: {}", self.relationship_type.cyan())?;
        Ok(())
    }
}

/// Result structure for remove command
#[derive(Debug, Serialize)]
pub struct RemoveResult {
    pub relationship_id: String,
    pub success: bool,
}

impl Output for RemoveResult {
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
    pub source: String,
    pub target: String,
    pub direction: String,
    pub priority: Option<u8>,
    pub reason: Option<String>,
}

/// Result structure for list command
#[derive(Debug, Serialize)]
pub struct ListResult {
    pub relationships: Vec<RelationshipListItem>,
    pub total_count: usize,
}

impl Output for ListResult {
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
            "{:<36} {:<15} {:<10} {:<36} {:<36} {:<9} {}",
            "ID".bold(),
            "Type".bold(),
            "Direction".bold(),
            "Source".bold(),
            "Target".bold(),
            "Priority".bold(),
            "Reason".bold()
        )?;
        writeln!(writer, "{}", "─".repeat(160).dimmed())?;

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
                "{:<36} {:<15} {:<10} {:<36} {:<36} {:<9} {}",
                rel.id.dimmed(),
                rel.relationship_type.cyan(),
                rel.direction,
                rel.source,
                rel.target,
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
    pub depth: usize,
}

/// Result structure for find command
#[derive(Debug, Serialize)]
pub struct FindResult {
    pub episodes: Vec<RelatedEpisodeItem>,
    pub total_count: usize,
}

impl Output for FindResult {
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
            "{:<36} {:<15} {:<12} {:<8} {}",
            "Episode ID".bold(),
            "Type".bold(),
            "Direction".bold(),
            "Depth".bold(),
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
                "{:<36} {:<15} {:<12} {:<8} {}",
                ep.episode_id.dimmed(),
                ep.relationship_type.cyan(),
                ep.direction,
                ep.depth,
                desc
            )?;
        }

        Ok(())
    }
}

/// Result structure for info command
#[derive(Debug, Serialize)]
pub struct InfoResult {
    pub relationship_id: String,
    pub relationship_type: String,
    pub source_episode_id: String,
    pub target_episode_id: String,
    pub source_task: String,
    pub target_task: String,
    pub priority: Option<u8>,
    pub reason: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub custom_fields: Vec<(String, String)>,
}

/// Result structure for graph command
#[derive(Debug, Serialize)]
pub struct GraphResult {
    pub root_episode_id: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub output: String,
    pub format: String,
}

impl Output for GraphResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(
            writer,
            "{} Relationship Graph for {}\n",
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

/// Result structure for validate command
#[derive(Debug, Serialize)]
pub struct ValidateResult {
    pub episode_id: Option<String>,
    pub has_cycle: bool,
    pub cycle_path: Option<Vec<String>>,
    pub message: String,
    pub checked_relationships: usize,
}

impl Output for ValidateResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        if self.has_cycle {
            writeln!(
                writer,
                "{} Cycle detected {}",
                "✗".red().bold(),
                self.episode_id
                    .as_ref()
                    .map(|id| format!("for episode {}", id))
                    .unwrap_or_default()
            )?;
            if let Some(path) = &self.cycle_path {
                writeln!(writer, "  Path: {}", path.join(" → "))?;
            }
        } else {
            writeln!(
                writer,
                "{} No cycles detected {}",
                "✓".green().bold(),
                self.episode_id
                    .as_ref()
                    .map(|id| format!("for episode {}", id))
                    .unwrap_or_default()
            )?;
            writeln!(
                writer,
                "  Checked {} relationships",
                self.checked_relationships
            )?;
        }
        Ok(())
    }
}
