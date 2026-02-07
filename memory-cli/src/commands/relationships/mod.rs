//! Relationship management commands for episodes
//!
//! This module provides standalone commands for managing relationships between episodes:
//! - relationship add: Create a new relationship between two episodes
//! - relationship remove: Remove an existing relationship by ID
//! - relationship list: List all relationships for an episode
//! - relationship find: Find episodes related to a given episode
//! - relationship info: Get detailed information about a relationship
//! - relationship graph: Generate a dependency graph visualization
//! - relationship validate: Check for cycles in relationships

use clap::Subcommand;

mod core;
mod types;

pub use core::*;
pub use types::*;

/// Relationship-related subcommands (standalone)
#[derive(Subcommand)]
pub enum StandaloneRelationshipCommands {
    /// Add a relationship between two episodes
    #[command(alias = "create")]
    Add {
        /// Source episode ID
        #[arg(short = 's', long, value_name = "ID")]
        source: String,

        /// Target episode ID
        #[arg(short = 't', long, value_name = "ID")]
        target: String,

        /// Relationship type
        #[arg(short = 'y', long, value_name = "TYPE")]
        r#type: RelationshipTypeArg,

        /// Optional reason for the relationship
        #[arg(short = 'r', long, value_name = "REASON")]
        reason: Option<String>,

        /// Priority (1-10)
        #[arg(short = 'p', long, value_name = "1-10")]
        priority: Option<u8>,

        /// Creator identifier
        #[arg(short = 'c', long, value_name = "NAME")]
        created_by: Option<String>,

        /// Custom metadata fields (key=value format)
        #[arg(short = 'm', long, value_name = "KEY=VALUE")]
        metadata: Vec<String>,
    },

    /// Remove a relationship by its ID
    #[command(alias = "delete")]
    Remove {
        /// Relationship ID to remove
        #[arg(value_name = "RELATIONSHIP_ID")]
        relationship_id: String,
    },

    /// List relationships for an episode
    #[command(alias = "ls")]
    List {
        /// Episode ID to list relationships for
        #[arg(short = 'e', long, value_name = "ID")]
        episode: String,

        /// Direction filter (outgoing, incoming, both)
        #[arg(short = 'd', long, value_enum, default_value = "both")]
        direction: DirectionArg,

        /// Filter by relationship type
        #[arg(short = 't', long, value_name = "TYPE")]
        r#type: Option<RelationshipTypeArg>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "table")]
        format: ListFormat,
    },

    /// Find episodes related to a given episode
    #[command(alias = "search")]
    Find {
        /// Episode ID to find related episodes for
        #[arg(short = 'e', long, value_name = "ID")]
        episode: String,

        /// Filter by relationship types
        #[arg(short = 't', long, value_name = "TYPE")]
        types: Vec<RelationshipTypeArg>,

        /// Maximum traversal depth
        #[arg(short = 'm', long, default_value = "3")]
        max_depth: usize,

        /// Maximum number of results
        #[arg(short = 'l', long, default_value = "50")]
        limit: usize,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "table")]
        format: ListFormat,
    },

    /// Get detailed information about a relationship
    #[command(alias = "show")]
    Info {
        /// Relationship ID to get information for
        #[arg(value_name = "RELATIONSHIP_ID")]
        relationship_id: String,
    },

    /// Generate a dependency graph for an episode
    #[command(alias = "viz")]
    Graph {
        /// Root episode ID
        #[arg(short = 'e', long, value_name = "ID")]
        episode: String,

        /// Maximum traversal depth
        #[arg(short = 'm', long, default_value = "3")]
        max_depth: usize,

        /// Output format (dot, json, text)
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: GraphFormat,

        /// Output file (defaults to stdout)
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<std::path::PathBuf>,
    },

    /// Validate relationships for cycles
    #[command(alias = "check")]
    Validate {
        /// Episode ID to start validation from (optional, validates all if not provided)
        #[arg(short = 'e', long, value_name = "ID")]
        episode: Option<String>,

        /// Relationship type to check (defaults to all acyclic types)
        #[arg(short = 't', long, value_name = "TYPE")]
        r#type: Option<RelationshipTypeArg>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::episode::{Direction, RelationshipType};

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
}
