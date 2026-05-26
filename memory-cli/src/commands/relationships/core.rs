//! Core implementations for relationship commands

use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::memory::relationship_query::RelationshipFilter;
use uuid::Uuid;

use crate::config::Config;
use crate::output::OutputFormat;

use super::types::*;

mod graph_utils;
use graph_utils::render_text_tree;

mod validation;
use validation::validate_relationships;

/// Add a relationship between two episodes
#[allow(clippy::too_many_arguments)]
pub async fn add_relationship(
    source: String,
    target: String,
    relationship_type: RelationshipTypeArg,
    reason: Option<String>,
    priority: Option<u8>,
    created_by: Option<String>,
    metadata: Vec<String>,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    // Parse UUIDs
    let source_id =
        Uuid::parse_str(&source).map_err(|e| anyhow::anyhow!("Invalid source UUID: {}", e))?;
    let target_id =
        Uuid::parse_str(&target).map_err(|e| anyhow::anyhow!("Invalid target UUID: {}", e))?;

    let rel_type = relationship_type.to_core_type();

    // Parse metadata key=value pairs
    let mut custom_fields = std::collections::HashMap::new();
    for meta in metadata {
        if let Some((key, value)) = meta.split_once('=') {
            custom_fields.insert(key.to_string(), value.to_string());
        }
    }

    if dry_run {
        println!("Would create relationship:");
        println!("  Source: {}", source);
        println!("  Target: {}", target);
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
        if !custom_fields.is_empty() {
            println!("  Custom fields:");
            for (key, value) in &custom_fields {
                println!("    {}: {}", key, value);
            }
        }
        return Ok(());
    }

    // Build metadata
    let relationship_metadata = do_memory_core::episode::RelationshipMetadata {
        reason,
        created_by,
        priority,
        weight: None,
        custom_fields,
    };

    // Create relationship
    let relationship_id = memory
        .add_episode_relationship(source_id, target_id, rel_type, relationship_metadata)
        .await?;

    let result = AddResult {
        relationship_id: relationship_id.to_string(),
        source_episode_id: source,
        target_episode_id: target,
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
        .map_err(|e| anyhow::anyhow!("Invalid relationship UUID: {}", e))?;

    if dry_run {
        println!("Would remove relationship: {}", relationship_id);
        return Ok(());
    }

    memory.remove_episode_relationship(rel_id).await?;

    let result = RemoveResult {
        relationship_id,
        success: true,
    };

    format.print_output(&result)
}

/// List relationships for an episode
#[allow(clippy::too_many_arguments)]
pub async fn list_relationships(
    episode: String,
    direction: DirectionArg,
    relationship_type: Option<RelationshipTypeArg>,
    list_format: ListFormat,
    memory: &SelfLearningMemory,
    _config: &Config,
    output_format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    let ep_id =
        Uuid::parse_str(&episode).map_err(|e| anyhow::anyhow!("Invalid episode UUID: {}", e))?;

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
        .map(|rel| {
            let direction = if rel.from_episode_id == ep_id {
                "outgoing"
            } else {
                "incoming"
            };
            RelationshipListItem {
                id: rel.id.to_string(),
                relationship_type: format!("{:?}", rel.relationship_type),
                source: rel.from_episode_id.to_string(),
                target: rel.to_episode_id.to_string(),
                direction: direction.to_string(),
                priority: rel.metadata.priority,
                reason: rel.metadata.reason,
            }
        })
        .collect();

    let result = ListResult {
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
    episode: String,
    types: Vec<RelationshipTypeArg>,
    max_depth: usize,
    limit: usize,
    list_format: ListFormat,
    memory: &SelfLearningMemory,
    _config: &Config,
    output_format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    let ep_id =
        Uuid::parse_str(&episode).map_err(|e| anyhow::anyhow!("Invalid episode UUID: {}", e))?;

    // Build filter based on types
    let rel_type = if types.is_empty() {
        None
    } else {
        // Use first type for filtering, query all and filter later
        Some(types[0].to_core_type())
    };

    let filter = RelationshipFilter {
        relationship_type: rel_type,
        direction: None,
        limit: Some(limit),
        min_priority: None,
    };

    // Build graph and traverse
    let graph = memory.build_relationship_graph(ep_id, max_depth).await?;

    // Collect related episodes with their details
    let mut items = Vec::new();

    for (node_id, episode_info) in &graph.nodes {
        if *node_id == ep_id {
            continue; // Skip the root episode
        }

        // Find the relationship that connects to this node
        for edge in &graph.edges {
            let is_connected = (edge.from_episode_id == ep_id && edge.to_episode_id == *node_id)
                || (edge.to_episode_id == ep_id && edge.from_episode_id == *node_id);

            if is_connected {
                // Check if type filter matches
                if !types.is_empty()
                    && !types
                        .iter()
                        .any(|t| t.to_core_type() == edge.relationship_type)
                {
                    continue;
                }

                let direction = if edge.from_episode_id == ep_id {
                    "outgoing"
                } else {
                    "incoming"
                };

                items.push(RelatedEpisodeItem {
                    episode_id: node_id.to_string(),
                    task_description: episode_info.task_description.clone(),
                    relationship_type: format!("{:?}", edge.relationship_type),
                    direction: direction.to_string(),
                    depth: 1, // Simplified depth calculation
                });
                break;
            }
        }
    }

    let result = FindResult {
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

/// Get detailed information about a relationship (WG-150, ADR-055).
pub async fn get_relationship_info(
    relationship_id: String,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let rel_id = Uuid::parse_str(&relationship_id)
        .map_err(|e| anyhow::anyhow!("Invalid relationship UUID: {}", e))?;

    let rel = memory
        .get_relationship_by_id(rel_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Relationship {} not found", relationship_id))?;

    // Look up source/target task descriptions (best-effort).
    let source_task = memory
        .get_episode(rel.from_episode_id)
        .await
        .ok()
        .map(|e| e.task_description.clone())
        .unwrap_or_default();
    let target_task = memory
        .get_episode(rel.to_episode_id)
        .await
        .ok()
        .map(|e| e.task_description.clone())
        .unwrap_or_default();

    let result = InfoResult {
        relationship_id: rel.id.to_string(),
        relationship_type: format!("{:?}", rel.relationship_type),
        source_episode_id: rel.from_episode_id.to_string(),
        target_episode_id: rel.to_episode_id.to_string(),
        source_task,
        target_task,
        priority: rel.metadata.priority,
        reason: rel.metadata.reason.clone(),
        created_by: rel.metadata.created_by.clone(),
        created_at: Some(rel.created_at.to_rfc3339()),
        custom_fields: rel
            .metadata
            .custom_fields
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    };

    format.print_output(&result)
}

/// Generate a dependency graph for an episode
#[allow(clippy::too_many_arguments)]
pub async fn generate_graph(
    episode: String,
    max_depth: usize,
    graph_format: GraphFormat,
    output: Option<std::path::PathBuf>,
    memory: &SelfLearningMemory,
    _config: &Config,
    output_format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    let ep_id =
        Uuid::parse_str(&episode).map_err(|e| anyhow::anyhow!("Invalid episode UUID: {}", e))?;

    let graph = memory.build_relationship_graph(ep_id, max_depth).await?;

    let (output_str, format_name) = match graph_format {
        GraphFormat::Dot => (graph.to_dot(), "dot"),
        GraphFormat::Json => (graph.to_json().to_string(), "json"),
        GraphFormat::Text => (render_text_tree(&graph, ep_id), "text"),
    };

    // Write to file if specified
    if let Some(path) = output {
        std::fs::write(&path, &output_str)?;
        println!("Graph written to {}", path.display());
    }

    let result = GraphResult {
        root_episode_id: episode,
        node_count: graph.node_count(),
        edge_count: graph.edge_count(),
        output: output_str,
        format: format_name.to_string(),
    };

    output_format.print_output(&result)
}

/// Handle relationship commands
pub async fn handle_relationship_command(
    command: super::StandaloneRelationshipCommands,
    memory: &SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    use super::StandaloneRelationshipCommands;

    match command {
        StandaloneRelationshipCommands::Add {
            source,
            target,
            r#type,
            reason,
            priority,
            created_by,
            metadata,
        } => {
            add_relationship(
                source, target, r#type, reason, priority, created_by, metadata, memory, config,
                format, dry_run,
            )
            .await
        }
        StandaloneRelationshipCommands::Remove { relationship_id } => {
            remove_relationship(relationship_id, memory, config, format, dry_run).await
        }
        StandaloneRelationshipCommands::List {
            episode,
            direction,
            r#type,
            format: list_format,
        } => {
            list_relationships(
                episode,
                direction,
                r#type,
                list_format,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        StandaloneRelationshipCommands::Find {
            episode,
            types,
            max_depth,
            limit,
            format: list_format,
        } => {
            find_related(
                episode,
                types,
                max_depth,
                limit,
                list_format,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        StandaloneRelationshipCommands::Info { relationship_id } => {
            get_relationship_info(relationship_id, memory, config, format).await
        }
        StandaloneRelationshipCommands::Graph {
            episode,
            max_depth,
            format: graph_format,
            output,
        } => {
            generate_graph(
                episode,
                max_depth,
                graph_format,
                output,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        StandaloneRelationshipCommands::Validate { episode, r#type } => {
            validate_relationships(episode, r#type, memory, config, format, dry_run).await
        }
    }
}
