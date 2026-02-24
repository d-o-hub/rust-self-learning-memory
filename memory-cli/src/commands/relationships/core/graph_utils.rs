use colored::Colorize;
use uuid::Uuid;

pub(super) fn render_text_tree(
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

pub(super) fn detect_cycle(
    graph: &memory_core::memory::relationship_query::RelationshipGraph,
    relationship_type: Option<memory_core::episode::RelationshipType>,
) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut rec_stack = std::collections::HashSet::new();

    has_cycle_dfs(
        graph,
        graph.root,
        &mut visited,
        &mut rec_stack,
        relationship_type,
    )
}

fn has_cycle_dfs(
    graph: &memory_core::memory::relationship_query::RelationshipGraph,
    node_id: Uuid,
    visited: &mut std::collections::HashSet<Uuid>,
    rec_stack: &mut std::collections::HashSet<Uuid>,
    relationship_type: Option<memory_core::episode::RelationshipType>,
) -> bool {
    visited.insert(node_id);
    rec_stack.insert(node_id);

    for edge in &graph.edges {
        if edge.from_episode_id != node_id {
            continue;
        }

        if let Some(ref rel_type) = relationship_type {
            if edge.relationship_type != *rel_type {
                continue;
            }
        }

        let neighbor = edge.to_episode_id;
        if !visited.contains(&neighbor) {
            if has_cycle_dfs(graph, neighbor, visited, rec_stack, relationship_type) {
                return true;
            }
        } else if rec_stack.contains(&neighbor) {
            return true;
        }
    }

    rec_stack.remove(&node_id);
    false
}
