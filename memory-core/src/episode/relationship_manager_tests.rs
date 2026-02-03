//! Tests for RelationshipManager

#[cfg(test)]
mod tests {
    use super::super::relationship_manager::RelationshipManager;
    use super::super::{RelationshipMetadata, RelationshipType};
    use uuid::Uuid;

    #[test]
    fn test_new_manager() {
        let manager = RelationshipManager::new();
        assert_eq!(manager.relationship_count(), 0);
        assert_eq!(manager.episode_count(), 0);
    }

    #[test]
    fn test_default_manager() {
        let manager = RelationshipManager::default();
        assert_eq!(manager.relationship_count(), 0);
    }

    #[test]
    fn test_add_valid_relationship() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let result = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(result.is_ok());
        assert_eq!(manager.relationship_count(), 1);
        assert!(manager.relationship_exists(from_id, to_id, RelationshipType::DependsOn));
    }

    #[test]
    fn test_prevent_self_relationship() {
        let mut manager = RelationshipManager::new();
        let episode_id = Uuid::new_v4();

        let result = manager.add_with_validation(
            episode_id,
            episode_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(result.is_err());
        assert_eq!(manager.relationship_count(), 0);
    }

    #[test]
    fn test_prevent_duplicate_relationship() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // Add first relationship
        let result1 = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );
        assert!(result1.is_ok());

        // Try to add duplicate
        let result2 = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(result2.is_err());
        assert_eq!(manager.relationship_count(), 1);
    }

    #[test]
    fn test_allow_different_types_same_pair() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let result1 = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );
        assert!(result1.is_ok());

        let result2 = manager.add_with_validation(
            from_id,
            to_id,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        );
        assert!(result2.is_ok());

        assert_eq!(manager.relationship_count(), 2);
    }

    #[test]
    fn test_detect_cycle() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        // Create chain: id1 -> id2 -> id3
        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                id2,
                id3,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Try to create cycle: id3 -> id1
        let result = manager.add_with_validation(
            id3,
            id1,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(result.is_err());
        assert_eq!(manager.relationship_count(), 2);
    }

    #[test]
    fn test_allow_non_cyclic_relationships() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let id4 = Uuid::new_v4();

        // Create a diamond graph that doesn't cycle back
        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                id1,
                id3,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let result = manager.add_with_validation(
            id2,
            id4,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );

        assert!(result.is_ok());
        assert_eq!(manager.relationship_count(), 3);
    }

    #[test]
    fn test_no_cycle_check_for_non_acyclic_types() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // RelatedTo allows cycles
        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let result = manager.add_with_validation(
            id2,
            id1,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        );

        // Should succeed because RelatedTo doesn't require acyclicity
        assert!(result.is_ok());
        assert_eq!(manager.relationship_count(), 2);
    }

    #[test]
    fn test_priority_validation() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // Test valid priorities (1-10)
        for priority in 1..=10 {
            let metadata = RelationshipMetadata {
                priority: Some(priority),
                ..Default::default()
            };

            let result = manager.add_with_validation(
                from_id,
                Uuid::new_v4(),
                RelationshipType::DependsOn,
                metadata,
            );
            assert!(result.is_ok());
        }

        // Test invalid priority (0)
        let metadata = RelationshipMetadata {
            priority: Some(0),
            ..Default::default()
        };

        let result = manager.add_with_validation(
            from_id,
            Uuid::new_v4(),
            RelationshipType::DependsOn,
            metadata,
        );
        assert!(result.is_err());

        // Test invalid priority (11)
        let metadata = RelationshipMetadata {
            priority: Some(11),
            ..Default::default()
        };

        let result =
            manager.add_with_validation(from_id, to_id, RelationshipType::DependsOn, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_relationship() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let rel = manager
            .add_with_validation(
                from_id,
                to_id,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        assert_eq!(manager.relationship_count(), 1);

        let result = manager.remove_relationship(rel.id);
        assert!(result.is_ok());
        assert_eq!(manager.relationship_count(), 0);
        assert!(!manager.relationship_exists(from_id, to_id, RelationshipType::DependsOn));
    }

    #[test]
    fn test_remove_nonexistent_relationship() {
        let mut manager = RelationshipManager::new();
        let result = manager.remove_relationship(Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_outgoing() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id1 = Uuid::new_v4();
        let to_id2 = Uuid::new_v4();

        manager
            .add_with_validation(
                from_id,
                to_id1,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                from_id,
                to_id2,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let outgoing = manager.get_outgoing(from_id);
        assert_eq!(outgoing.len(), 2);
        assert!(outgoing.iter().any(|r| r.to_episode_id == to_id1));
        assert!(outgoing.iter().any(|r| r.to_episode_id == to_id2));
    }

    #[test]
    fn test_get_incoming() {
        let mut manager = RelationshipManager::new();
        let to_id = Uuid::new_v4();
        let from_id1 = Uuid::new_v4();
        let from_id2 = Uuid::new_v4();

        manager
            .add_with_validation(
                from_id1,
                to_id,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                from_id2,
                to_id,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let incoming = manager.get_incoming(to_id);
        assert_eq!(incoming.len(), 2);
        assert!(incoming.iter().any(|r| r.from_episode_id == from_id1));
        assert!(incoming.iter().any(|r| r.from_episode_id == from_id2));
    }

    #[test]
    fn test_get_by_type() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                id2,
                id3,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let depends_on = manager.get_by_type(id1, RelationshipType::DependsOn);
        assert_eq!(depends_on.len(), 1);
        assert_eq!(depends_on[0].to_episode_id, id2);

        let related_to = manager.get_by_type(id2, RelationshipType::RelatedTo);
        assert_eq!(related_to.len(), 1);
        assert_eq!(related_to[0].to_episode_id, id3);
    }

    #[test]
    fn test_would_create_cycle() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                id2,
                id3,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Check if adding id3 -> id1 would create a cycle
        let would_cycle = manager.would_create_cycle(id3, id1).unwrap();
        assert!(would_cycle);

        // Check if adding id3 -> new_id would NOT create a cycle
        let new_id = Uuid::new_v4();
        let would_cycle = manager.would_create_cycle(id3, new_id).unwrap();
        assert!(!would_cycle);
    }

    #[test]
    fn test_find_cycle_path() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                id2,
                id3,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let path = manager.find_cycle_path(id3, id1).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], id1);
        assert_eq!(path[1], id2);
        assert_eq!(path[2], id3);
    }

    #[test]
    fn test_load_relationships() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let rel = manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Simulate loading from storage
        let relationships = vec![rel];
        manager.load_relationships(relationships);

        assert_eq!(manager.relationship_count(), 1);
        assert!(manager.relationship_exists(id1, id2, RelationshipType::DependsOn));
    }

    #[test]
    fn test_load_relationships_clears_existing() {
        let mut manager = RelationshipManager::new();

        // Add some relationships
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        assert_eq!(manager.relationship_count(), 1);

        // Load empty relationships (should clear existing)
        manager.load_relationships(vec![]);
        assert_eq!(manager.relationship_count(), 0);
    }

    #[test]
    fn test_get_all_relationships() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                id2,
                id3,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let all_rels = manager.get_all_relationships();
        assert_eq!(all_rels.len(), 2);
    }

    #[test]
    fn test_episode_count() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        assert_eq!(manager.episode_count(), 0);

        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        assert_eq!(manager.episode_count(), 2);

        manager
            .add_with_validation(
                id2,
                id3,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        assert_eq!(manager.episode_count(), 3);
    }

    #[test]
    fn test_topological_order() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                id2,
                id3,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let order = manager.topological_order().unwrap();
        assert_eq!(order.len(), 3);

        let pos1 = order.iter().position(|&id| id == id1).unwrap();
        let pos2 = order.iter().position(|&id| id == id2).unwrap();
        let pos3 = order.iter().position(|&id| id == id3).unwrap();

        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
    }

    #[test]
    fn test_topological_order_fails_with_cycle() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // Create a cycle using RelatedTo (which allows cycles)
        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                id2,
                id1,
                RelationshipType::RelatedTo,
                RelationshipMetadata::default(),
            )
            .unwrap();

        let result = manager.topological_order();
        assert!(result.is_err());
    }

    #[test]
    fn test_clear() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        assert_eq!(manager.relationship_count(), 1);

        manager.clear();
        assert_eq!(manager.relationship_count(), 0);
        assert_eq!(manager.episode_count(), 0);
    }

    #[test]
    fn test_complex_graph_operations() {
        let mut manager = RelationshipManager::new();

        // Create a complex graph
        let ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

        // Create relationships: 0->1, 0->2, 1->3, 2->3, 3->4
        manager
            .add_with_validation(
                ids[0],
                ids[1],
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                ids[0],
                ids[2],
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                ids[1],
                ids[3],
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                ids[2],
                ids[3],
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        manager
            .add_with_validation(
                ids[3],
                ids[4],
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Test various operations
        assert_eq!(manager.relationship_count(), 5);
        assert_eq!(manager.episode_count(), 5);

        let outgoing_0 = manager.get_outgoing(ids[0]);
        assert_eq!(outgoing_0.len(), 2);

        let incoming_3 = manager.get_incoming(ids[3]);
        assert_eq!(incoming_3.len(), 2);

        // Test topological order
        let order = manager.topological_order().unwrap();
        assert_eq!(order.len(), 5);

        let pos_0 = order.iter().position(|&id| id == ids[0]).unwrap();
        let pos_4 = order.iter().position(|&id| id == ids[4]).unwrap();
        assert!(pos_0 < pos_4);
    }

    #[test]
    fn test_all_relationship_types() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();

        // Test all relationship types
        let types = vec![
            RelationshipType::DependsOn,
            RelationshipType::Follows,
            RelationshipType::RelatedTo,
            RelationshipType::Blocks,
            RelationshipType::ParentChild,
            RelationshipType::Duplicates,
            RelationshipType::References,
        ];

        for rel_type in types {
            let to_id = Uuid::new_v4();
            let result = manager.add_with_validation(
                from_id,
                to_id,
                rel_type,
                RelationshipMetadata::default(),
            );
            assert!(result.is_ok());
        }

        assert_eq!(manager.relationship_count(), 7);
    }

    #[test]
    fn test_cycle_detection_with_different_types() {
        let mut manager = RelationshipManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // Add DependsOn relationship (requires acyclic)
        manager
            .add_with_validation(
                id1,
                id2,
                RelationshipType::DependsOn,
                RelationshipMetadata::default(),
            )
            .unwrap();

        // Adding reverse DependsOn should fail (would create cycle)
        let result = manager.add_with_validation(
            id2,
            id1,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );
        assert!(result.is_err());

        // But adding reverse RelatedTo should succeed (allows cycles)
        let result = manager.add_with_validation(
            id2,
            id1,
            RelationshipType::RelatedTo,
            RelationshipMetadata::default(),
        );
        assert!(result.is_ok());

        // Now we have both: id1 -DependsOn-> id2 and id2 -RelatedTo-> id1
        assert_eq!(manager.relationship_count(), 2);

        // Adding another DependsOn from id2 to id1 should still fail
        let result = manager.add_with_validation(
            id2,
            id1,
            RelationshipType::DependsOn,
            RelationshipMetadata::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_queries() {
        let manager = RelationshipManager::new();
        let id = Uuid::new_v4();

        assert!(manager.get_outgoing(id).is_empty());
        assert!(manager.get_incoming(id).is_empty());
        assert!(manager
            .get_by_type(id, RelationshipType::DependsOn)
            .is_empty());
    }

    #[test]
    fn test_relationship_metadata_preservation() {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let metadata = RelationshipMetadata {
            priority: Some(5),
            reason: Some("Test relationship".to_string()),
            ..Default::default()
        };

        let rel = manager
            .add_with_validation(
                from_id,
                to_id,
                RelationshipType::DependsOn,
                metadata.clone(),
            )
            .unwrap();

        assert_eq!(rel.metadata.priority, Some(5));
        assert_eq!(
            rel.metadata.reason,
            Some("Test relationship".to_string())
        );

        // Verify metadata is preserved in queries
        let outgoing = manager.get_outgoing(from_id);
        assert_eq!(outgoing[0].metadata.priority, Some(5));
    }

    #[test]
    fn test_large_graph_performance() {
        let mut manager = RelationshipManager::new();

        // Create a large linear chain
        let mut prev_id = Uuid::new_v4();
        for _ in 0..100 {
            let next_id = Uuid::new_v4();
            manager
                .add_with_validation(
                    prev_id,
                    next_id,
                    RelationshipType::Follows,
                    RelationshipMetadata::default(),
                )
                .unwrap();
            prev_id = next_id;
        }

        assert_eq!(manager.relationship_count(), 100);
        assert_eq!(manager.episode_count(), 101);
    }
}
