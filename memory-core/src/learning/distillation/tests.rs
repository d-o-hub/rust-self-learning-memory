#[cfg(test)]
mod tests {
    use crate::learning::distillation::TrajectoryDistiller;
    use crate::episode::Episode;
    use crate::types::{TaskContext, TaskType, TaskOutcome};

    #[test]
    fn test_trajectory_distillation() {
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
}
