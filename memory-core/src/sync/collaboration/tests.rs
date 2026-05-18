#[cfg(test)]
mod tests {
    use crate::learning::distillation::TrajectoryRepresentation;
    use crate::sync::collaboration::CollaborationManager;
    use crate::types::TaskType;

    #[test]
    fn test_collaboration_bundling() {
        let manager = CollaborationManager::new();
        let t1 = TrajectoryRepresentation::Embedding(vec![1.0, 0.0]);
        let t2 = TrajectoryRepresentation::Embedding(vec![0.0, 1.0]);

        let bundled = manager
            .bundle_prototypes(TaskType::Debugging, &[t1, t2])
            .unwrap();

        let emb = match bundled {
            TrajectoryRepresentation::Embedding(e) => e,
            #[cfg(feature = "csm")]
            _ => panic!("Expected embedding representation"),
        };
        assert_eq!(emb.len(), 2);
        assert_eq!(emb[0], 0.5);
        assert_eq!(emb[1], 0.5);
    }

    #[test]
    fn test_bundling_dimension_mismatch_rejected() {
        let manager = CollaborationManager::new();
        let t1 = TrajectoryRepresentation::Embedding(vec![1.0, 0.0]);
        let t2 = TrajectoryRepresentation::Embedding(vec![0.0, 1.0, 0.0]); // Different dimension

        // Bundling should return None when embedding dimensions differ
        let result = manager.bundle_prototypes(TaskType::Debugging, &[t1, t2]);
        assert!(result.is_none(), "Dimension mismatch should reject bundling");
    }

    #[test]
    fn test_bundling_empty_trajectories() {
        let manager = CollaborationManager::new();
        let result = manager.bundle_prototypes(TaskType::Refactoring, &[]);
        assert!(result.is_none(), "Empty trajectories should return None");
    }

    #[test]
    fn test_single_embedding_self_average() {
        let manager = CollaborationManager::new();
        let t1 = TrajectoryRepresentation::Embedding(vec![3.0, 7.0]);

        let bundled = manager
            .bundle_prototypes(TaskType::Debugging, &[t1])
            .unwrap();

        let emb = match bundled {
            TrajectoryRepresentation::Embedding(e) => e,
            #[cfg(feature = "csm")]
            _ => panic!("Expected embedding representation"),
        };
        assert_eq!(emb, vec![3.0, 7.0]);
    }
}
