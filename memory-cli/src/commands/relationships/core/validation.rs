use do_memory_core::memory::SelfLearningMemory;
use uuid::Uuid;

use crate::config::Config;
use crate::output::OutputFormat;

use super::super::types::*;
use super::graph_utils::detect_cycle;

/// Validate relationships for cycles (WG-151, ADR-055).
///
/// - With `--episode <id>`: builds a graph rooted at that episode and runs DFS.
/// - Without `--episode`: performs **global** validation across every stored
///   relationship by iterating all relationships and running DFS from every
///   distinct node. O(V+E) per starting node, bounded by visited-set reuse.
pub(super) async fn validate_relationships(
    episode: Option<String>,
    relationship_type: Option<RelationshipTypeArg>,
    memory: &SelfLearningMemory,
    _config: &Config,
    output_format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    let filter_type = relationship_type.map(|t| t.to_core_type());

    if let Some(ep) = episode {
        // Scoped validation: existing behavior.
        let ep_id =
            Uuid::parse_str(&ep).map_err(|e| anyhow::anyhow!("Invalid episode UUID: {}", e))?;
        let graph = memory.build_relationship_graph(ep_id, 10).await?;
        let has_cycle = detect_cycle(&graph, filter_type);
        let result = ValidateResult {
            episode_id: Some(ep),
            has_cycle,
            cycle_path: None,
            message: if has_cycle {
                "Cycle detected in relationships".to_string()
            } else {
                "No cycles detected".to_string()
            },
            checked_relationships: graph.edge_count(),
        };
        return output_format.print_output(&result);
    }

    // Global validation: iterate every relationship, build adjacency, DFS from
    // every distinct node with a shared visited set so each node is processed once.
    let all_rels = memory.get_all_relationships().await?;
    let mut adjacency: std::collections::HashMap<Uuid, Vec<Uuid>> =
        std::collections::HashMap::new();
    let mut nodes: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    let total = all_rels.len();
    for rel in all_rels {
        if let Some(ref ft) = filter_type
            && rel.relationship_type != *ft
        {
            continue;
        }
        adjacency
            .entry(rel.from_episode_id)
            .or_default()
            .push(rel.to_episode_id);
        nodes.insert(rel.from_episode_id);
        nodes.insert(rel.to_episode_id);
    }

    let mut visited: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    let mut has_cycle = false;
    for start in &nodes {
        if visited.contains(start) {
            continue;
        }
        let mut rec_stack: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        if dfs_has_cycle(*start, &adjacency, &mut visited, &mut rec_stack) {
            has_cycle = true;
            break;
        }
    }

    let result = ValidateResult {
        episode_id: None,
        has_cycle,
        cycle_path: None,
        message: if has_cycle {
            format!("Cycle detected across {} relationships", total)
        } else {
            format!("No cycles detected across {} relationships", total)
        },
        checked_relationships: total,
    };
    output_format.print_output(&result)
}

/// DFS helper for global cycle detection (WG-151, ADR-055).
fn dfs_has_cycle(
    node: Uuid,
    adjacency: &std::collections::HashMap<Uuid, Vec<Uuid>>,
    visited: &mut std::collections::HashSet<Uuid>,
    rec_stack: &mut std::collections::HashSet<Uuid>,
) -> bool {
    visited.insert(node);
    rec_stack.insert(node);
    if let Some(neighbors) = adjacency.get(&node) {
        for &n in neighbors {
            if !visited.contains(&n) {
                if dfs_has_cycle(n, adjacency, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&n) {
                return true;
            }
        }
    }
    rec_stack.remove(&node);
    false
}
