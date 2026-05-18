#[cfg(test)]
mod tests {
    use crate::episode::Episode;
    use crate::learning::distillation::TrajectoryDistiller;
    use crate::types::{TaskContext, TaskOutcome, TaskType};

    #[test]
    fn test_trajectory_distillation_shape() {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        );
        episode.complete(TaskOutcome::Success {
            verdict: "Fixed".to_string(),
            artifacts: vec![],
        });

        let distiller = TrajectoryDistiller::new(false);
        let representation = distiller.distill(&episode);

        match representation {
            crate::learning::distillation::TrajectoryRepresentation::Embedding(emb) => {
                assert_eq!(emb.len(), 1536);
            }
            #[cfg(feature = "csm")]
            _ => panic!("Expected embedding representation"),
        }
    }

    #[test]
    fn test_distillation_is_content_sensitive() {
        // Arrange: Two episodes with substantially different content
        let mut ep1 = Episode::new(
            "Fix critical authentication bug in login flow".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        );
        ep1.complete(TaskOutcome::Success {
            verdict: "Auth fixed".to_string(),
            artifacts: vec![],
        });

        let mut ep2 = Episode::new(
            "Refactor zzzz database migration scripts for performance".to_string(),
            TaskContext::default(),
            TaskType::Refactoring,
        );
        ep2.complete(TaskOutcome::Success {
            verdict: "Migration optimized".to_string(),
            artifacts: vec![],
        });

        // Act: Distill both episodes
        let distiller = TrajectoryDistiller::new(false);
        let r1 = distiller.distill(&ep1);
        let r2 = distiller.distill(&ep2);

        // Assert: Different content produces distinct embeddings
        match (&r1, &r2) {
            (
                crate::learning::distillation::TrajectoryRepresentation::Embedding(e1),
                crate::learning::distillation::TrajectoryRepresentation::Embedding(e2),
            ) => {
                assert_eq!(e1.len(), 1536);
                assert_eq!(e2.len(), 1536);
                assert_ne!(
                    e1, e2,
                    "Distinct episodes should produce distinct embeddings"
                );
            }
            #[cfg(feature = "csm")]
            _ => panic!("Expected embedding representations"),
        }
    }
}
