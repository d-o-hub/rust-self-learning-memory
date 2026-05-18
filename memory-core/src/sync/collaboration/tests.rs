#[cfg(test)]
mod tests {
    use crate::sync::collaboration::CollaborationManager;
    use crate::learning::distillation::TrajectoryRepresentation;
    use crate::types::TaskType;

    #[test]
    fn test_collaboration_bundling() {
        let manager = CollaborationManager::new();
        let t1 = TrajectoryRepresentation::Embedding(vec![1.0, 0.0]);
        let t2 = TrajectoryRepresentation::Embedding(vec![0.0, 1.0]);

        let bundled = manager.bundle_prototypes(TaskType::Debugging, &[t1, t2]).unwrap();

        if let TrajectoryRepresentation::Embedding(emb) = bundled {
            assert_eq!(emb.len(), 2);
            assert_eq!(emb[0], 0.5);
            assert_eq!(emb[1], 0.5);
        } else {
            panic!("Expected embedding representation");
        }
    }
}
