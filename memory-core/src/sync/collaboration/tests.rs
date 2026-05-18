#![allow(clippy::module_inception)]

#[cfg(test)]
mod tests {
    use crate::learning::distillation::TrajectoryRepresentation;
    use crate::sync::collaboration::CollaborationManager;
    use crate::types::TaskType;

    #[test]
    #[allow(clippy::infallible_destructuring_match)]
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
    fn test_bundling_dimension_mismatch() {
        let manager = CollaborationManager::new();
        let t1 = TrajectoryRepresentation::Embedding(vec![1.0, 0.0]);
        let t2 = TrajectoryRepresentation::Embedding(vec![0.0, 1.0, 2.0]);

        // Bundling should return None when embedding dimensions differ
        let result = manager.bundle_prototypes(TaskType::Debugging, &[t1, t2]);
        assert!(
            result.is_none(),
            "Dimension mismatch should reject bundling"
        );
    }
}
