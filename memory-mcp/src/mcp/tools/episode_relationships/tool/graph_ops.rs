use super::*;

impl EpisodeRelationshipTools {
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id))]
    pub async fn get_dependency_graph(
        &self,
        input: DependencyGraphInput,
    ) -> Result<DependencyGraphOutput> {
        info!(
            "Building dependency graph for episode: {}",
            input.episode_id
        );

        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode_id: {}", e))?;
        let depth = input.depth.map(|d| d.clamp(1, 5)).unwrap_or(2);
        let format = input.format.as_deref().unwrap_or("json");

        let graph = self
            .memory
            .build_relationship_graph(episode_id, depth)
            .await?;

        let nodes: Vec<RelationshipNode> = graph
            .nodes
            .values()
            .map(|ep| RelationshipNode {
                id: ep.episode_id.to_string(),
                task_description: ep.task_description.clone(),
                task_type: format!("{:?}", ep.task_type),
                is_complete: ep.is_complete(),
            })
            .collect();

        let edges: Vec<RelationshipEdge> = graph.edges.iter().map(relationship_to_edge).collect();
        let dot = if format == "dot" {
            Some(graph.to_dot())
        } else {
            None
        };

        let node_count = nodes.len();
        let edge_count = edges.len();

        info!(
            "Built dependency graph with {} nodes and {} edges",
            node_count, edge_count
        );

        Ok(DependencyGraphOutput {
            success: true,
            root: input.episode_id,
            node_count,
            edge_count,
            nodes,
            edges,
            dot,
            message: format!(
                "Graph contains {} nodes and {} edges",
                node_count, edge_count
            ),
        })
    }

    #[instrument(skip(self, input), fields(from = %input.from_episode_id, to = %input.to_episode_id))]
    pub async fn validate_no_cycles(
        &self,
        input: ValidateNoCyclesInput,
    ) -> Result<ValidateNoCyclesOutput> {
        info!(
            "Validating no cycles for relationship from {} to {}",
            input.from_episode_id, input.to_episode_id
        );

        let from_id = Uuid::parse_str(&input.from_episode_id)
            .map_err(|e| anyhow!("Invalid from_episode_id: {}", e))?;
        let to_id = Uuid::parse_str(&input.to_episode_id)
            .map_err(|e| anyhow!("Invalid to_episode_id: {}", e))?;
        let rel_type = RelationshipType::parse(&input.relationship_type)
            .map_err(|e| anyhow!("Invalid relationship_type: {}", e))?;

        if !rel_type.requires_acyclic() {
            return Ok(ValidateNoCyclesOutput {
                success: true,
                would_create_cycle: false,
                is_valid: true,
                cycle_path: None,
                message: format!(
                    "Relationship type '{}' does not require acyclic validation",
                    input.relationship_type
                ),
            });
        }

        let mut adjacency_list: HashMap<Uuid, Vec<EpisodeRelationship>> = HashMap::new();
        let all_rels = self.get_all_relationships().await?;

        for rel in all_rels {
            adjacency_list
                .entry(rel.from_episode_id)
                .or_default()
                .push(rel);
        }

        let would_create_cycle =
            memory_core::episode::graph_algorithms::has_path_dfs(&adjacency_list, to_id, from_id)?;

        let cycle_path = if would_create_cycle {
            match memory_core::episode::graph_algorithms::find_path_dfs(
                &adjacency_list,
                to_id,
                from_id,
            ) {
                Ok(path) => Some(path.iter().map(|u| u.to_string()).collect()),
                Err(_) => None,
            }
        } else {
            None
        };

        Ok(ValidateNoCyclesOutput {
            success: true,
            would_create_cycle,
            is_valid: !would_create_cycle,
            cycle_path,
            message: if would_create_cycle {
                "Adding this relationship would create a cycle".to_string()
            } else {
                "No cycle would be created".to_string()
            },
        })
    }

    #[instrument(skip(self, input), fields(episode_count = input.episode_ids.len()))]
    pub async fn get_topological_order(
        &self,
        input: GetTopologicalOrderInput,
    ) -> Result<GetTopologicalOrderOutput> {
        info!(
            "Getting topological order for {} episodes",
            input.episode_ids.len()
        );

        if input.episode_ids.is_empty() {
            return Ok(GetTopologicalOrderOutput {
                success: true,
                order: vec![],
                count: 0,
                has_cycles: false,
                message: "No episodes provided".to_string(),
            });
        }

        let mut episode_ids = Vec::new();
        for id_str in &input.episode_ids {
            let id = Uuid::parse_str(id_str)
                .map_err(|e| anyhow!("Invalid episode_id '{}': {}", id_str, e))?;
            episode_ids.push(id);
        }

        let mut adjacency_list: HashMap<Uuid, Vec<EpisodeRelationship>> = HashMap::new();

        for episode_id in &episode_ids {
            if let Ok(rels) = self
                .memory
                .get_episode_relationships(*episode_id, Direction::Outgoing)
                .await
            {
                let filtered_rels: Vec<_> = rels
                    .into_iter()
                    .filter(|r| episode_ids.contains(&r.to_episode_id))
                    .collect();

                if !filtered_rels.is_empty() {
                    adjacency_list.insert(*episode_id, filtered_rels);
                }
            }
        }

        let has_cycles = memory_core::episode::graph_algorithms::has_cycle(&adjacency_list)?;
        if has_cycles {
            return Ok(GetTopologicalOrderOutput {
                success: true,
                order: vec![],
                count: 0,
                has_cycles: true,
                message: "Cannot compute topological order: graph contains cycles".to_string(),
            });
        }

        let sorted_ids = memory_core::episode::graph_algorithms::topological_sort(&adjacency_list)?;

        let mut order = Vec::new();
        for (position, id) in sorted_ids.iter().enumerate() {
            if let Ok(episode) = self.memory.get_episode(*id).await {
                order.push(TopologicalEpisode {
                    episode_id: id.to_string(),
                    task_description: episode.task_description.clone(),
                    position: position + 1,
                });
            }
        }

        for id in &episode_ids {
            if !sorted_ids.contains(id) {
                if let Ok(episode) = self.memory.get_episode(*id).await {
                    order.push(TopologicalEpisode {
                        episode_id: id.to_string(),
                        task_description: episode.task_description.clone(),
                        position: order.len() + 1,
                    });
                }
            }
        }

        let count = order.len();
        info!("Computed topological order for {} episodes", count);

        Ok(GetTopologicalOrderOutput {
            success: true,
            order,
            count,
            has_cycles: false,
            message: format!("Episodes in topological order ({} total)", count),
        })
    }

    async fn get_all_relationships(&self) -> Result<Vec<EpisodeRelationship>> {
        Ok(Vec::new())
    }
}

pub(super) fn relationship_to_edge(rel: &EpisodeRelationship) -> RelationshipEdge {
    RelationshipEdge {
        id: rel.id.to_string(),
        from: rel.from_episode_id.to_string(),
        to: rel.to_episode_id.to_string(),
        relationship_type: rel.relationship_type.as_str().to_string(),
        reason: rel.metadata.reason.clone(),
        priority: rel.metadata.priority,
        created_by: rel.metadata.created_by.clone(),
        created_at: rel.created_at.to_rfc3339(),
    }
}
