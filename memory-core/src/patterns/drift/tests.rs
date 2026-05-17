#[cfg(test)]
mod tests {
    use crate::episode::Episode;
    use crate::patterns::drift::DriftAnalyzer;
    use crate::types::{RewardScore, TaskContext, TaskType};

    #[test]
    fn test_drift_analyzer_insufficient_data() {
        let mut analyzer = DriftAnalyzer::new();
        let episodes = vec![];
        let results = analyzer.analyze_drift(&episodes).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_drift_analyzer_no_drift() {
        let mut analyzer = DriftAnalyzer::new();
        let mut episodes = Vec::new();
        for _ in 0..10 {
            let mut ep = Episode::new("task".to_string(), TaskContext::default(), TaskType::Analysis);
            ep.reward = Some(RewardScore {
                total: 1.0,
                base: 1.0,
                efficiency: 1.0,
                complexity_bonus: 1.0,
                quality_multiplier: 1.0,
                learning_bonus: 0.0,
            });
            episodes.push(ep);
        }
        let results = analyzer.analyze_drift(&episodes).unwrap();
        assert!(results.is_empty() || results.iter().all(|r| r.probability < 0.5));
    }
}
