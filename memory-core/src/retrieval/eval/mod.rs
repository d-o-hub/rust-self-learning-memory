//! Retrieval quality and cost evaluation benchmark harness.

mod regression;
mod rerank;
mod rerank_judge;
mod runner;
mod types;

pub use regression::{RegressionCheckResult, RegressionChecker, format_markdown_report};
pub use rerank_judge::{LOCAL_OVERLAP_JUDGE_CONFIDENCE, LocalOverlapJudge};
pub use runner::RetrievalEvaluator;
pub use types::{
    BenchmarkMetrics, BenchmarkQuery, BenchmarkReport, CostModel, FixtureCorpus, FixtureItem,
    LatencyStats, RERANK_COMPARISON_STRATEGY, RegressionThresholds, RetrievalStrategy,
    TierDistribution,
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
    /// vs `AlwaysEmbed`) with no recall regression. `LocalOnly` pins the
    /// third policy arm (`FallbackPolicy::LocalOnly`) and must report zero
    /// embedding calls.
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
        let local = evaluator
            .evaluate_strategy(RetrievalStrategy::LocalOnly)
            .expect("local-only evaluation should succeed");

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
        assert!(
            local.embedding_calls_per_query == 0.0,
            "local-only ({} calls/query) must never call Tier 4",
            local.embedding_calls_per_query
        );
    }

    /// Shipping baseline artifacts predate the rerank metrics (issue #1031):
    /// they must keep deserializing, with the new counters defaulting to `0.0`.
    #[test]
    fn test_shipped_baseline_artifact_still_loads() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/fixtures/retrieval_baseline.json"
        );
        let json = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("baseline fixture must exist at {path}: {e}"));

        let baseline: BenchmarkReport =
            serde_json::from_str(&json).expect("baseline JSON must keep deserializing");

        assert!(!baseline.strategies.is_empty());
        for (name, m) in &baseline.strategies {
            assert_eq!(m.judge_calls_per_query, 0.0, "{name} judge calls");
            assert_eq!(
                m.rerank_candidates_per_query, 0.0,
                "{name} rerank candidates"
            );
            assert_eq!(m.top1_changed_rate, 0.0, "{name} top-1 changed rate");
        }
    }

    /// The JSON report exposes the rerank counters even for un-reranked runs,
    /// where they are zero.
    #[test]
    fn test_metrics_serialize_rerank_counters() {
        let evaluator = RetrievalEvaluator::new(create_test_corpus());

        let metrics = evaluator
            .evaluate_strategy(RetrievalStrategy::LocalOnly)
            .expect("local-only evaluation should succeed");

        assert_eq!(metrics.judge_calls_per_query, 0.0);
        assert_eq!(metrics.rerank_candidates_per_query, 0.0);
        assert_eq!(metrics.top1_changed_rate, 0.0);

        let json = serde_json::to_value(&metrics).expect("metrics must serialize");
        for key in [
            "judge_calls_per_query",
            "rerank_candidates_per_query",
            "top1_changed_rate",
        ] {
            assert!(json.get(key).is_some(), "metrics JSON is missing {key}");
        }
    }

    /// Un-reranked reports stay byte-identical to the pre-#1031 format, and the
    /// rerank section appears as soon as the counters carry data.
    #[test]
    fn test_markdown_renders_rerank_section_only_for_reranked_runs() {
        let evaluator = RetrievalEvaluator::new(create_test_corpus());
        let mut report = evaluator.evaluate_all().expect("evaluation should succeed");

        let markdown = format_markdown_report(&report, None);
        assert!(
            !markdown.contains("Semantic Rerank Comparison"),
            "an un-reranked report must not gain a rerank section"
        );

        report
            .strategies
            .get_mut("local_only")
            .expect("local_only present")
            .judge_calls_per_query = 1.0;
        report
            .strategies
            .get_mut("local_only")
            .expect("local_only present")
            .top1_changed_rate = 0.5;

        let markdown = format_markdown_report(&report, None);
        assert!(markdown.contains("## Semantic Rerank Comparison"));
        assert!(markdown.contains("Judge Calls / Query"));
        assert!(markdown.contains("50.0%"), "top-1 changed rate is rendered");
    }

    /// The rerank arm exists on demand and is absent from shipped baselines, so
    /// regression checks must skip it instead of failing every `--rerank` run.
    #[test]
    fn test_regression_check_skips_rerank_comparison_arm() {
        let evaluator = RetrievalEvaluator::new(create_test_corpus());

        let baseline = evaluator.evaluate_all().unwrap();
        let mut current = baseline.clone();
        let rerank_metrics = current.strategies["local_only"].clone();
        current
            .strategies
            .insert(RERANK_COMPARISON_STRATEGY.to_string(), rerank_metrics);

        let checker = RegressionChecker::new(RegressionThresholds::default());
        let strict = checker.check(&current, &baseline);
        assert!(
            !strict.passed,
            "an unlisted strategy missing from the baseline is still a violation"
        );

        let ignoring = checker.check_ignoring(&current, &baseline, &[RERANK_COMPARISON_STRATEGY]);
        assert!(
            ignoring.passed,
            "comparison-only arm must not fail the check: {:?}",
            ignoring.violations
        );
    }

    /// Rerank comparison tests (issue #1031). They need the CSM cascade to
    /// produce multi-candidate local shortlists.
    #[cfg(feature = "csm")]
    mod rerank {
        use crate::retrieval::eval::{
            BenchmarkMetrics, BenchmarkQuery, FixtureCorpus, FixtureItem, LocalOverlapJudge,
            RERANK_COMPARISON_STRATEGY, RetrievalEvaluator, RetrievalStrategy,
        };
        use crate::retrieval::judgment::{
            AtomicScore, CandidateJudgment, JudgmentCandidate, JudgmentError, RetrievalJudge,
        };
        use crate::retrieval::rerank::SemanticRerankConfig;
        use std::sync::Arc;

        /// Every item and query share most tokens, so each query's BM25
        /// shortlist holds all three candidates and the rerank stage always
        /// reaches the judge.
        fn create_rerank_corpus() -> FixtureCorpus {
            let item = |id: &str, tail: &str| FixtureItem {
                id: id.to_string(),
                text: format!("alpha beta gamma retrieval {tail}"),
                context: None,
                tags: Vec::new(),
                is_successful: Some(true),
                reward_score: Some(1.0),
            };

            FixtureCorpus {
                version: "1.0.0".to_string(),
                description: "Rerank comparison corpus".to_string(),
                corpus: vec![
                    item("item-alpha", "pipeline"),
                    item("item-beta", "cache"),
                    item("item-gamma", "fusion"),
                ],
                queries: vec![
                    BenchmarkQuery {
                        id: "q-alpha".to_string(),
                        query: "alpha beta gamma retrieval pipeline".to_string(),
                        context: None,
                        expected_ids: vec!["item-alpha".to_string()],
                        tags: Vec::new(),
                        expected_accepted_id: Some("item-alpha".to_string()),
                    },
                    BenchmarkQuery {
                        id: "q-beta".to_string(),
                        query: "alpha beta gamma retrieval cache".to_string(),
                        context: None,
                        expected_ids: vec!["item-beta".to_string()],
                        tags: Vec::new(),
                        expected_accepted_id: Some("item-beta".to_string()),
                    },
                ],
            }
        }

        fn enabled_rerank_config() -> SemanticRerankConfig {
            SemanticRerankConfig {
                enabled: true,
                ..SemanticRerankConfig::default()
            }
        }

        /// Judge that marks only the last shortlist candidate relevant, which
        /// forces the fused order to promote it over the local top-1.
        struct LastCandidateJudge;

        impl RetrievalJudge for LastCandidateJudge {
            fn judge_candidates(
                &self,
                _query: &str,
                candidates: &[JudgmentCandidate<'_>],
            ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
                let promoted = candidates.len().saturating_sub(1);
                Ok(candidates
                    .iter()
                    .enumerate()
                    .map(|(idx, candidate)| CandidateJudgment {
                        id: candidate.id.to_string(),
                        relevance: AtomicScore::new(if idx == promoted { 1.0 } else { 0.0 }, 1.0),
                        useful_evidence: AtomicScore::new(0.0, 0.0),
                        contradiction: AtomicScore::new(0.0, 0.0),
                        instruction_like: AtomicScore::new(0.0, 0.0),
                    })
                    .collect())
            }
        }

        fn evaluate_offline_judge_arm() -> BenchmarkMetrics {
            RetrievalEvaluator::new(create_rerank_corpus())
                .evaluate_strategy_with_rerank(
                    RetrievalStrategy::LocalOnly,
                    Arc::new(LocalOverlapJudge::new()),
                    &enabled_rerank_config(),
                )
                .expect("reranked evaluation should succeed")
        }

        #[test]
        fn test_offline_judge_arm_reports_bounded_rerank_counters() {
            let metrics = evaluate_offline_judge_arm();

            assert_eq!(
                metrics.judge_calls_per_query, 1.0,
                "the batch judge is called once per query"
            );
            assert!(
                metrics.rerank_candidates_per_query >= 2.0,
                "each query shortlists more than one candidate, got {}",
                metrics.rerank_candidates_per_query
            );
            assert!(
                (0.0..=1.0).contains(&metrics.top1_changed_rate),
                "top-1 changed rate must be a rate, got {}",
                metrics.top1_changed_rate
            );
            assert_eq!(
                metrics.embedding_calls_per_query, 0.0,
                "the rerank arm stays local: no Tier 4 calls"
            );
            assert!(
                metrics.recall_at_5 > 0.0,
                "the offline judge must not destroy local retrieval quality"
            );
            assert_eq!(metrics.total_queries, 2);
        }

        #[test]
        fn test_top1_changed_rate_tracks_real_rerank_effects() {
            let metrics = RetrievalEvaluator::new(create_rerank_corpus())
                .evaluate_strategy_with_rerank(
                    RetrievalStrategy::LocalOnly,
                    Arc::new(LastCandidateJudge),
                    &enabled_rerank_config(),
                )
                .expect("reranked evaluation should succeed");

            assert_eq!(
                metrics.judge_calls_per_query, 1.0,
                "precondition: the judge sees every query's shortlist"
            );
            assert_eq!(
                metrics.top1_changed_rate, 1.0,
                "every query's top-1 must flip when the judge promotes the last candidate"
            );
        }

        /// A judge failure must not fail the run, move the top-1, or lose local
        /// results — the counters record the attempt and the local order stands.
        #[test]
        fn test_judge_failure_keeps_local_order_and_still_counts_the_call() {
            struct FailingJudge;

            impl RetrievalJudge for FailingJudge {
                fn judge_candidates(
                    &self,
                    _query: &str,
                    _candidates: &[JudgmentCandidate<'_>],
                ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
                    Err(JudgmentError::Unavailable)
                }
            }

            let metrics = RetrievalEvaluator::new(create_rerank_corpus())
                .evaluate_strategy_with_rerank(
                    RetrievalStrategy::LocalOnly,
                    Arc::new(FailingJudge),
                    &enabled_rerank_config(),
                )
                .expect("a provider failure must not fail the evaluation");

            assert_eq!(metrics.judge_calls_per_query, 1.0);
            assert_eq!(
                metrics.top1_changed_rate, 0.0,
                "the local order is preserved on provider failure"
            );
            assert!(metrics.rerank_candidates_per_query >= 2.0);
            assert!(metrics.recall_at_5 > 0.0);
        }

        /// The comparison arm reported by the CLI is the offline judge over
        /// `local_only`, and the report key is the documented one.
        #[test]
        fn test_offline_judge_arm_matches_documented_report_key() {
            let metrics = evaluate_offline_judge_arm();

            assert_eq!(metrics.strategy, RetrievalStrategy::LocalOnly);
            assert_eq!(RERANK_COMPARISON_STRATEGY, "local_only+rerank");
        }
    }
}
