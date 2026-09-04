//! Retrieval quality and cost evaluation benchmark harness.

mod regression;
mod runner;
mod types;

pub use regression::{RegressionCheckResult, RegressionChecker, format_markdown_report};
pub use runner::RetrievalEvaluator;
pub use types::{
    BenchmarkMetrics, BenchmarkQuery, BenchmarkReport, CostModel, FixtureCorpus, FixtureItem,
    LatencyStats, RegressionThresholds, RetrievalStrategy, TierDistribution,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_corpus() -> FixtureCorpus {
        FixtureCorpus {
            version: "1.0.0".to_string(),
            description: "Test fixture dataset".to_string(),
            corpus: vec![
                FixtureItem {
                    id: "item-1".to_string(),
                    text: "OAuth2 authentication JWT tokens in Rust".to_string(),
                    context: None,
                    tags: vec!["auth".to_string()],
                    is_successful: Some(true),
                    reward_score: Some(1.0),
                },
                FixtureItem {
                    id: "item-2".to_string(),
                    text: "Database query optimization in PostgreSQL".to_string(),
                    context: None,
                    tags: vec!["db".to_string()],
                    is_successful: Some(true),
                    reward_score: Some(0.9),
                },
            ],
            queries: vec![
                BenchmarkQuery {
                    id: "q-1".to_string(),
                    query: "How to handle auth with JWT tokens in Rust?".to_string(),
                    context: None,
                    expected_ids: vec!["item-1".to_string()],
                    tags: vec!["auth".to_string()],
                    expected_accepted_id: Some("item-1".to_string()),
                },
                BenchmarkQuery {
                    id: "q-2".to_string(),
                    query: "PostgreSQL query optimization techniques".to_string(),
                    context: None,
                    expected_ids: vec!["item-2".to_string()],
                    tags: vec!["db".to_string()],
                    expected_accepted_id: Some("item-2".to_string()),
                },
            ],
        }
    }

    #[test]
    fn test_evaluator_runs_all_strategies() {
        let corpus = create_test_corpus();
        let evaluator = RetrievalEvaluator::new(corpus);

        let report = evaluator.evaluate_all().expect("evaluation should succeed");

        assert_eq!(report.query_count, 2);
        assert_eq!(report.corpus_size, 2);
        assert!(report.strategies.contains_key("adaptive"));
        assert!(report.strategies.contains_key("local_only"));
        assert!(report.strategies.contains_key("always_embed"));

        let adaptive = &report.strategies["adaptive"];
        assert!(adaptive.recall_at_5 > 0.0);
        assert!(adaptive.mrr > 0.0);

        let markdown = format_markdown_report(&report, None);
        assert!(markdown.contains("Retrieval Evaluation Quality & Cost Report"));
        assert!(markdown.contains("adaptive"));
    }

    #[test]
    fn test_regression_checker_detects_pass_and_fail() {
        let corpus = create_test_corpus();
        let evaluator = RetrievalEvaluator::new(corpus);

        let report1 = evaluator.evaluate_all().unwrap();
        let mut report2 = report1.clone();

        // Check identical runs -> pass
        let checker = RegressionChecker::new(RegressionThresholds::default());
        let res1 = checker.check(&report1, &report2);
        assert!(res1.passed);

        // Inject artificial recall regression in report2
        if let Some(m) = report2.strategies.get_mut("adaptive") {
            m.recall_at_5 = 0.0;
        }

        let res2 = checker.check(&report2, &report1);
        assert!(!res2.passed);
        assert!(!res2.violations.is_empty());
        assert!(res2.violations[0].contains("Recall@5 dropped"));
    }

    /// Issue #968 acceptance: on a corpus where every query has an exact
    /// local match, `Adaptive` must eliminate Tier 4 calls (>=50% reduction
    /// vs `AlwaysEmbed`) with no recall regression.
    #[cfg(feature = "csm")]
    #[test]
    fn test_adaptive_halves_tier4_calls_without_quality_loss() {
        let evaluator = RetrievalEvaluator::new(create_test_corpus());

        let adaptive = evaluator
            .evaluate_strategy(RetrievalStrategy::Adaptive)
            .expect("adaptive evaluation should succeed");
        let always = evaluator
            .evaluate_strategy(RetrievalStrategy::AlwaysEmbed)
            .expect("always-embed evaluation should succeed");

        assert!(
            adaptive.embedding_calls_per_query * 2.0 <= always.embedding_calls_per_query,
            "adaptive ({} calls/query) must at least halve always-embed ({} calls/query)",
            adaptive.embedding_calls_per_query,
            always.embedding_calls_per_query
        );
        assert!(
            adaptive.recall_at_5 + f64::EPSILON >= always.recall_at_5,
            "adaptive recall ({}) must not regress vs always-embed ({})",
            adaptive.recall_at_5,
            always.recall_at_5
        );
    }
}
