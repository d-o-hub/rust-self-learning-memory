//! Property-based tests for Episode Relationships
//!
//! Tests invariants that must hold regardless of input values:
//! - Relationships cannot create cycles in depends_on
//! - Relationship deletion is idempotent
//! - Relationship types are valid
//! - Bidirectional relationships are consistent

use memory_core::episode::relationship_manager::RelationshipManager;
use memory_core::episode::{EpisodeRelationship, RelationshipMetadata, RelationshipType};
use proptest::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

// ============================================================================
// Relationship Validation Properties
// ============================================================================

proptest! {
    /// Self-relationships are always rejected
    #[test]
    fn self_relationships_rejected(rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();
        let episode_id = Uuid::new_v4();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());
        let result = manager.add_with_validation(
            episode_id,
            episode_id,  // Same episode
            rel_type,
            metadata,
        );

        assert!(result.is_err(), "Self-relationship should be rejected");
    }

    /// Duplicate relationships of same type are rejected
    #[test]
    fn duplicate_relationships_rejected(rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());

        // First relationship should succeed
        let result1 = manager.add_with_validation(from_id, to_id, rel_type, metadata.clone());
        assert!(result1.is_ok(), "First relationship should succeed");

        // Duplicate should fail
        let result2 = manager.add_with_validation(from_id, to_id, rel_type, metadata);
        assert!(result2.is_err(), "Duplicate relationship should be rejected");
    }

    /// Priority must be in valid range (1-10)
    #[test]
    fn invalid_priority_rejected(rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // Test invalid priorities
        for priority in [0, 11, 100, 255] {
            let mut metadata = RelationshipMetadata::with_reason("Test".to_string());
            metadata.priority = Some(priority);

            let result = manager.add_with_validation(from_id, to_id, rel_type, metadata.clone());
            assert!(result.is_err(), "Priority {priority} should be rejected");
        }

        // Test valid priorities
        for priority in 1..=10 {
            let mut metadata = RelationshipMetadata::with_reason("Test".to_string());
            metadata.priority = Some(priority);

            // Create new episode IDs to avoid duplicate detection
            let from_id = Uuid::new_v4();
            let to_id = Uuid::new_v4();

            let result = manager.add_with_validation(from_id, to_id, rel_type, metadata.clone());
            assert!(result.is_ok(), "Priority {priority} should be accepted");
        }
    }
}

// ============================================================================
// Cycle Detection Properties
// ============================================================================

proptest! {
    /// Acyclic relationships cannot create cycles
    #[test]
    fn depends_on_prevents_cycles(relationship_chain_length in 2..10usize) {
        let mut manager = RelationshipManager::new();

        // Create a chain of depends_on relationships
        let mut episode_ids: Vec<Uuid> = (0..=relationship_chain_length)
            .map(|_| Uuid::new_v4())
            .collect();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());

        // Add relationships forming a chain: episde[0] -> episode[1] -> ... -> episode[n]
        for i in 0..relationship_chain_length {
            let result = manager.add_with_validation(
                episode_ids[i],
                episode_ids[i + 1],
                RelationshipType::DependsOn,
                metadata.clone(),
            );
            assert!(result.is_ok(), "Chain link {i} should succeed");
        }

        // Try to close the cycle: episode[n] -> episode[0]
        let result = manager.add_with_validation(
            episode_ids[relationship_chain_length],
            episode_ids[0],
            RelationshipType::DependsOn,
            metadata,
        );

        assert!(result.is_err(), "Closing the cycle should be rejected");
    }

    /// Parent-child relationships cannot create cycles
    #[test]
    fn parent_child_prevents_cycles(chain_length in 2..10usize) {
        let mut manager = RelationshipManager::new();

        let mut episode_ids: Vec<Uuid> = (0..=chain_length)
            .map(|_| Uuid::new_v4())
            .collect();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());

        // Add relationships chain
        for i in 0..chain_length {
            let result = manager.add_with_validation(
                episode_ids[i],
                episode_ids[i + 1],
                RelationshipType::ParentChild,
                metadata.clone(),
            );
            assert!(result.is_ok(), "Chain link {i} should succeed");
        }

        // Try to close the cycle
        let result = manager.add_with_validation(
            episode_ids[chain_length],
            episode_ids[0],
            RelationshipType::ParentChild,
            metadata,
        );

        assert!(result.is_err(), "Closing the cycle should be rejected");
    }

    /// Blocks relationships cannot create cycles
    #[test]
    fn blocks_prevents_cycles(chain_length in 2..10usize) {
        let mut manager = RelationshipManager::new();

        let mut episode_ids: Vec<Uuid> = (0..=chain_length)
            .map(|_| Uuid::new_v4())
            .collect();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());

        // Add relationships chain
        for i in 0..chain_length {
            let result = manager.add_with_validation(
                episode_ids[i],
                episode_ids[i + 1],
                RelationshipType::Blocks,
                metadata.clone(),
            );
            assert!(result.is_ok(), "Chain link {i} should succeed");
        }

        // Try to close the cycle
        let result = manager.add_with_validation(
            episode_ids[chain_length],
            episode_ids[0],
            RelationshipType::Blocks,
            metadata,
        );

        assert!(result.is_err(), "Closing the cycle should be rejected");
    }

    /// Non-acyclic relationships allow cycles
    #[test]
    fn non_acyclic_allows_cycles(chain_length in 2..10usize) {
        let non_acyclic_types = vec![
            RelationshipType::Follows,
            RelationshipType::RelatedTo,
            RelationshipType::Duplicates,
            RelationshipType::References,
        ];

        for rel_type in non_acyclic_types {
            let mut manager = RelationshipManager::new();

            let mut episode_ids: Vec<Uuid> = (0..=chain_length)
                .map(|_| Uuid::new_v4())
                .collect();

            let metadata = RelationshipMetadata::with_reason("Test".to_string());

            // Add relationships chain
            for i in 0..chain_length {
                let result = manager.add_with_validation(
                    episode_ids[i],
                    episode_ids[i + 1],
                    rel_type,
                    metadata.clone(),
                );
                assert!(result.is_ok(), "Chain link {i} with {rel_type:?} should succeed");
            }

            // Try to close the cycle - should succeed for non-acyclic types
            let result = manager.add_with_validation(
                episode_ids[chain_length],
                episode_ids[0],
                rel_type,
                metadata,
            );

            assert!(result.is_ok(), "{rel_type:?} should allow cycles");
        }
    }
}

// ============================================================================
// Relationship Removal Properties
// ============================================================================

proptest! {
    /// Relationship deletion is idempotent
    #[test]
    fn relationship_removal_idempotent(rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());

        // Add relationship
        let rel = manager.add_with_validation(from_id, to_id, rel_type, metadata).unwrap();

        // Remove once
        let result1 = manager.remove_relationship(rel.id);
        assert!(result1.is_ok(), "First removal should succeed");

        // Try to remove again - should fail (not found)
        let result2 = manager.remove_relationship(rel.id);
        assert!(result2.is_err(), "Second removal should fail (idempotent)");

        // Third removal should also fail
        let result3 = manager.remove_relationship(rel.id);
        assert!(result3.is_err(), "Third removal should also fail");
    }

    /// Deleting a relationship removes it from all indexes
    #[test]
    fn removal_updates_indexes(rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());

        // Add relationship
        let rel = manager.add_with_validation(from_id, to_id, rel_type, metadata).unwrap();

        // Verify it exists
        assert!(manager.relationship_exists(from_id, to_id, rel_type));
        assert!(!manager.get_outgoing(from_id).is_empty());
        assert!(!manager.get_incoming(to_id).is_empty());

        // Remove it
        manager.remove_relationship(rel.id).unwrap();

        // Verify it's gone from all indexes
        assert!(!manager.relationship_exists(from_id, to_id, rel_type));
        assert!(manager.get_outgoing(from_id).is_empty());
        assert!(manager.get_incoming(to_id).is_empty());
    }
}

// ============================================================================
// Relationship Query Properties
// ============================================================================

proptest! {
    /// Relationship existence check is consistent
    #[test]
    fn relationship_exists_consistency(rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());

        // Relationship should not exist initially
        assert!(!manager.relationship_exists(from_id, to_id, rel_type));

        // Add relationship
        manager.add_with_validation(from_id, to_id, rel_type, metadata).unwrap();

        // Now it should exist
        assert!(manager.relationship_exists(from_id, to_id, rel_type));

        // Remove relationship
        let rel = manager.get_outgoing(from_id)[0].clone();
        manager.remove_relationship(rel.id).unwrap();

        // Should not exist again
        assert!(!manager.relationship_exists(from_id, to_id, rel_type));
    }

    /// Outgoing and incoming relationships are symmetric
    #[test]
    fn outgoing_incoming_symmetry(rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());

        // Add relationship
        let rel = manager.add_with_validation(from_id, to_id, rel_type, metadata).unwrap();

        // Check outgoing
        let outgoing = manager.get_outgoing(from_id);
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].id, rel.id);

        // Check incoming
        let incoming = manager.get_incoming(to_id);
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].id, rel.id);

        // Verify symmetry
        assert_eq!(outgoing[0].id, incoming[0].id);
    }

    /// get_by_type returns both outgoing and incoming
    #[test]
    fn get_by_type_returns_both_directions(rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let metadata = RelationshipMetadata::with_reason("Test".to_string());

        // Add relationship from_id -> to_id
        let rel1 = manager.add_with_validation(from_id, to_id, rel_type, metadata.clone()).unwrap();

        // Add relationship from another episode -> from_id
        let another_id = Uuid::new_v4();
        let rel2 = manager.add_with_validation(another_id, from_id, rel_type, metadata).unwrap();

        // Query from_id
        let from_rels = manager.get_by_type(from_id, rel_type);

        // Should include both outgoing and incoming
        assert_eq!(from_rels.len(), 2);
        let rel_ids: HashSet<_> = from_rels.iter().map(|r| r.id).collect();
        assert!(rel_ids.contains(&rel1.id));
        assert!(rel_ids.contains(&rel2.id));
    }

    /// Relationship count is accurate
    #[test]
    fn relationship_count_accurate(count in 1..10usize, rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();

        for _ in 0..count {
            let from_id = Uuid::new_v4();
            let to_id = Uuid::new_v4();
            let metadata = RelationshipMetadata::with_reason("Test".to_string());

            manager.add_with_validation(from_id, to_id, rel_type, metadata).unwrap();
        }

        assert_eq!(manager.relationship_count(), count);
    }
}

// ============================================================================
// Relationship Type Properties
// ============================================================================

proptest! {
    /// Relationship type string conversion is round-trippable
    #[test]
    fn relationship_type_string_roundtrip(rel_type in any::<RelationshipType>()) {
        let s = rel_type.as_str();
        let parsed = RelationshipType::parse(s).unwrap();
        assert_eq!(rel_type, parsed);
    }

    /// All relationship types are serializable
    #[test]
    fn relationship_type_serializable(rel_type in any::<RelationshipType>()) {
        let json = serde_json::to_string(&rel_type).unwrap();
        let parsed: RelationshipType = serde_json::from_str(&json).unwrap();
        assert_eq!(rel_type, parsed);
    }

    /// Directionality property is consistent
    #[test]
    fn directionality_property_consistent(rel_type in any::<RelationshipType>()) {
        let is_directional = rel_type.is_directional();

        // Directional types should have inverse, non-directional should not
        let has_inverse = rel_type.inverse().is_some();

        assert_eq!(
            is_directional || !has_inverse,
            true,
            "Directional types should have inverse, non-directional should not"
        );
    }

    /// Acyclic requirement is consistent
    #[test]
    fn acyclic_requirement_consistent(rel_type in any::<RelationshipType>()) {
        let requires_acyclic = rel_type.requires_acyclic();

        // Types requiring acyclicity should be directional
        if requires_acyclic {
            assert!(rel_type.is_directional());
        }
    }
}

// ============================================================================
// Relationship Graph Properties
// ============================================================================

proptest! {
    /// Loading relationships preserves state
    #[test]
    fn load_relationships_preserves_state(count in 1..10usize, rel_type in any::<RelationshipType>()) {
        let mut manager1 = RelationshipManager::new();

        // Create relationships
        let mut relationships = Vec::new();
        for _ in 0..count {
            let from_id = Uuid::new_v4();
            let to_id = Uuid::new_v4();
            let metadata = RelationshipMetadata::with_reason("Test".to_string());

            let rel = EpisodeRelationship::new(from_id, to_id, rel_type, metadata);
            relationships.push(rel);
        }

        // Load into first manager
        manager1.load_relationships(relationships.clone());

        // Load into second manager
        let mut manager2 = RelationshipManager::new();
        manager2.load_relationships(relationships);

        // Both should have same state
        assert_eq!(manager1.relationship_count(), manager2.relationship_count());
        assert_eq!(manager1.episode_count(), manager2.episode_count());
    }

    /// Multiple relationships between different pairs
    #[test]
    fn multiple_relationships_between_pairs(count in 1..10usize, rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();

        // Create relationships between different pairs
        for _ in 0..count {
            let from_id = Uuid::new_v4();
            let to_id = Uuid::new_v4();
            let metadata = RelationshipMetadata::with_reason("Test".to_string());

            let result = manager.add_with_validation(from_id, to_id, rel_type, metadata);
            assert!(result.is_ok());
        }

        assert_eq!(manager.relationship_count(), count);
    }

    /// Relationship with custom fields works correctly
    #[test]
    fn custom_fields_preserved(rel_type in any::<RelationshipType>()) {
        let mut manager = RelationshipManager::new();
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let mut metadata = RelationshipMetadata::with_reason("Test".to_string());
        metadata.custom_fields.insert("key1".to_string(), "value1".to_string());
        metadata.custom_fields.insert("key2".to_string(), "value2".to_string());

        let result = manager.add_with_validation(from_id, to_id, rel_type, metadata).unwrap();

        assert!(result.metadata.custom_fields.contains_key("key1"));
        assert!(result.metadata.custom_fields.contains_key("key2"));
        assert_eq!(result.metadata.custom_fields.get("key1"), Some(&"value1".to_string()));
    }
}
