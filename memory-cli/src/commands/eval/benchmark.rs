//! Retrieval quality and cost benchmark command (`eval benchmark`).

use std::path::PathBuf;
use std::sync::Arc;

use do_memory_core::retrieval::eval::{
    EVIDENCE_COMPARISON_STRATEGY, EvidenceFixtureJudge, LocalOverlapJudge,
    RERANK_COMPARISON_STRATEGY,
};
use do_memory_core::retrieval::{
    BenchmarkReport, EvidencePolicy, FixtureCorpus, RegressionChecker, RegressionThresholds,
    RetrievalEvaluator, RetrievalStrategy, SemanticRerankConfig, format_markdown_report,
};

use crate::output::OutputFormat;

/// Strategies that exist on demand and are never tracked by baseline artifacts.
///
/// Regression checks skip these when the baseline has no counterpart, so a
/// `--rerank` or `--evidence` run stays comparable against
/// `benches/fixtures/retrieval_baseline.json`.
const COMPARISON_ONLY_STRATEGIES: [&str; 2] =
    [RERANK_COMPARISON_STRATEGY, EVIDENCE_COMPARISON_STRATEGY];

#[allow(clippy::too_many_arguments)]
pub async fn benchmark(
    fixture_path: Option<PathBuf>,
    strategy_str: String,
    rerank: bool,
    evidence: bool,
    baseline_path: Option<PathBuf>,
    fail_on_regression: bool,
    max_recall_drop: f64,
    max_mrr_drop: f64,
    max_ndcg_drop: f64,
    max_latency_increase: f64,
    max_cost_increase: f64,
    output_json: Option<PathBuf>,
    output_markdown: Option<PathBuf>,
    _remote: bool,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    // 1. Resolve fixture path
    let default_fixture = PathBuf::from("benches/fixtures/retrieval_benchmark_corpus.json");
    let target_fixture = fixture_path.unwrap_or(default_fixture);

    if !target_fixture.exists() {
        anyhow::bail!(
            "Fixture corpus file not found at path: {}",
            target_fixture.display()
        );
    }

    let fixture_content = tokio::fs::read_to_string(&target_fixture).await?;
    let corpus = if target_fixture.extension().and_then(|s| s.to_str()) == Some("jsonl") {
        FixtureCorpus::from_jsonl_str(&fixture_content)?
    } else {
        FixtureCorpus::from_json_str(&fixture_content)?
    };

    // The fixture judge replays each query's own labels, so it must capture
    // them before the corpus moves into the evaluator.
    let evidence_judge = evidence.then(|| EvidenceFixtureJudge::from_corpus(&corpus));
    let evaluator = RetrievalEvaluator::new(corpus);

    // 2. Execute benchmark
    let mut report: BenchmarkReport = if strategy_str.eq_ignore_ascii_case("all") {
        evaluator.evaluate_all()?
    } else {
        let strategy: RetrievalStrategy = strategy_str.parse().map_err(anyhow::Error::msg)?;
        let metrics = evaluator.evaluate_strategy(strategy)?;
        let mut strategies = std::collections::HashMap::new();
        strategies.insert(strategy.to_string(), metrics);
        BenchmarkReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            corpus_version: evaluator.corpus_version().to_string(),
            corpus_size: evaluator.corpus_size(),
            query_count: evaluator.query_count(),
            strategies,
        }
    };

    // 2b. Optional semantic rerank comparison arm (issue #1031). The baseline
    // local strategy runs exactly as above; this entry shows the same queries
    // after the offline judge and deterministic score fusion.
    if rerank {
        let rerank_config = SemanticRerankConfig {
            enabled: true,
            ..SemanticRerankConfig::default()
        };
        let reranked = evaluator.evaluate_strategy_with_rerank(
            RetrievalStrategy::LocalOnly,
            Arc::new(LocalOverlapJudge::new()),
            &rerank_config,
        )?;
        report
            .strategies
            .insert(RERANK_COMPARISON_STRATEGY.to_string(), reranked);
    }

    // 2c. Optional evidence-classification comparison arm (issue #1032). The
    // baseline local strategy runs exactly as above; this entry shows the same
    // queries after per-candidate evidence classification under the default
    // policy, which never drops anything.
    if let Some(judge) = evidence_judge {
        let classified = evaluator.evaluate_with_evidence(
            RetrievalStrategy::LocalOnly,
            Arc::new(judge),
            &EvidencePolicy::default(),
        )?;
        report
            .strategies
            .insert(EVIDENCE_COMPARISON_STRATEGY.to_string(), classified);
    }

    // 3. Baseline comparison
    let default_baseline = PathBuf::from("benches/fixtures/retrieval_baseline.json");
    let active_baseline_path = baseline_path.or_else(|| {
        if default_baseline.exists() {
            Some(default_baseline)
        } else {
            None
        }
    });

    let check_result = if let Some(ref base_path) = active_baseline_path {
        if base_path.exists() {
            let base_content = tokio::fs::read_to_string(base_path).await?;
            let baseline_report: BenchmarkReport = serde_json::from_str(&base_content)?;
            let thresholds = RegressionThresholds {
                max_recall_drop,
                max_mrr_drop,
                max_ndcg_drop,
                max_latency_increase_ratio: max_latency_increase,
                max_cost_increase_ratio: max_cost_increase,
            };
            let checker = RegressionChecker::new(thresholds);
            Some(checker.check_ignoring(&report, &baseline_report, &COMPARISON_ONLY_STRATEGIES))
        } else {
            None
        }
    } else {
        None
    };

    // 4. Output artifacts
    let markdown_str = format_markdown_report(&report, check_result.as_ref());

    if let Some(ref md_out) = output_markdown {
        tokio::fs::write(md_out, &markdown_str).await?;
    }

    if let Some(ref json_out) = output_json {
        let json_str = serde_json::to_string_pretty(&report)?;
        tokio::fs::write(json_out, json_str).await?;
    }

    // Print summary report to stdout
    println!("{markdown_str}");

    // 5. Fail on regression if configured
    if fail_on_regression {
        if let Some(ref res) = check_result {
            if !res.passed {
                anyhow::bail!(
                    "Retrieval benchmark failed regression checks: {}",
                    res.summary
                );
            }
        }
    }

    Ok(())
}

/// Render a relative "N minutes/days/weeks ago" timestamp label.
pub(super) fn format_time(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now - dt;

    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{} minutes ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{} hours ago", diff.num_hours())
    } else if diff.num_days() < 7 {
        format!("{} days ago", diff.num_days())
    } else {
        format!("{} weeks ago", diff.num_weeks())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    /// Two-item corpus with one query per item, mirroring the eval fixtures.
    const TINY_CORPUS: &str = r#"{
  "version": "1.0.0",
  "description": "CLI benchmark test fixture",
  "corpus": [
    {"id": "item-1", "text": "OAuth2 authentication JWT tokens in Rust", "context": null, "tags": ["auth"], "is_successful": true, "reward_score": 1.0},
    {"id": "item-2", "text": "PostgreSQL query optimization", "context": null, "tags": ["db"], "is_successful": true, "reward_score": 0.9}
  ],
  "queries": [
    {"id": "q-1", "query": "auth JWT tokens Rust", "context": null, "expected_ids": ["item-1"], "tags": ["auth"], "expected_accepted_id": "item-1"},
    {"id": "q-2", "query": "PostgreSQL optimization", "context": null, "expected_ids": ["item-2"], "tags": ["db"], "expected_accepted_id": "item-2"}
  ]
}"#;

    fn write_tiny_corpus(dir: &Path) -> PathBuf {
        let fixture = dir.join("corpus.json");
        std::fs::write(&fixture, TINY_CORPUS).expect("fixture written");
        fixture
    }

    /// Same corpus with per-candidate evidence labels, so the fixture-backed
    /// judge has ground truth to replay.
    const TINY_LABELLED_CORPUS: &str = r#"{
  "version": "1.0.0",
  "description": "CLI evidence-arm fixture",
  "corpus": [
    {"id": "item-1", "text": "OAuth2 authentication JWT tokens in Rust", "context": null, "tags": ["auth"], "is_successful": true, "reward_score": 1.0},
    {"id": "item-2", "text": "PostgreSQL query optimization", "context": null, "tags": ["db"], "is_successful": true, "reward_score": 0.9}
  ],
  "queries": [
    {"id": "q-1", "query": "auth JWT tokens Rust", "context": null, "expected_ids": ["item-1"], "tags": ["auth"],
     "expected_accepted_id": "item-1", "evidence_labels": {"item-1": {"relevance": true, "useful_evidence": true,
      "contradiction": false, "instruction_like": false, "expected_disposition": "keep"}}},
    {"id": "q-2", "query": "PostgreSQL optimization", "context": null, "expected_ids": ["item-2"], "tags": ["db"],
     "expected_accepted_id": "item-2", "evidence_labels": {"item-2": {"relevance": true, "useful_evidence": true,
      "contradiction": false, "instruction_like": false, "expected_disposition": "keep"}}}
  ]
}"#;

    fn write_labelled_corpus(dir: &Path) -> PathBuf {
        let fixture = dir.join("labelled.json");
        std::fs::write(&fixture, TINY_LABELLED_CORPUS).expect("fixture written");
        fixture
    }

    #[tokio::test]
    async fn test_benchmark_writes_report_with_rerank_arm() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fixture = write_tiny_corpus(dir.path());
        let json_out = dir.path().join("report.json");
        let md_out = dir.path().join("report.md");

        benchmark(
            Some(fixture),
            "all".to_string(),
            true,
            false,
            None,
            false,
            0.05,
            0.05,
            0.05,
            0.50,
            0.20,
            Some(json_out.clone()),
            Some(md_out.clone()),
            false,
            OutputFormat::Json,
        )
        .await
        .expect("benchmark runs with the rerank comparison arm");

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&json_out).expect("json written"))
                .expect("valid report json");
        let strategies = json["strategies"].as_object().expect("strategies map");
        assert!(
            strategies.contains_key(RERANK_COMPARISON_STRATEGY),
            "the rerank comparison arm must be reported: {:?}",
            strategies.keys().collect::<Vec<_>>()
        );
        let arm = &strategies[RERANK_COMPARISON_STRATEGY];
        for key in [
            "judge_calls_per_query",
            "rerank_candidates_per_query",
            "top1_changed_rate",
        ] {
            assert!(arm.get(key).is_some(), "missing counter {key}");
        }

        let markdown = std::fs::read_to_string(&md_out).expect("markdown written");
        assert!(
            markdown.contains(RERANK_COMPARISON_STRATEGY),
            "markdown report must list the comparison arm"
        );
    }

    #[tokio::test]
    async fn test_benchmark_writes_report_with_evidence_arm() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fixture = write_labelled_corpus(dir.path());
        let json_out = dir.path().join("evidence.json");
        let md_out = dir.path().join("evidence.md");

        benchmark(
            Some(fixture),
            "local_only".to_string(),
            false,
            true,
            None,
            false,
            0.05,
            0.05,
            0.05,
            0.50,
            0.20,
            Some(json_out.clone()),
            Some(md_out.clone()),
            false,
            OutputFormat::Json,
        )
        .await
        .expect("benchmark runs with the evidence comparison arm");

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&json_out).expect("json written"))
                .expect("valid report json");
        let strategies = json["strategies"].as_object().expect("strategies map");
        assert!(
            strategies.contains_key(EVIDENCE_COMPARISON_STRATEGY),
            "the evidence comparison arm must be reported: {:?}",
            strategies.keys().collect::<Vec<_>>()
        );
        let arm = &strategies[EVIDENCE_COMPARISON_STRATEGY];
        let evidence = arm
            .get("evidence_metrics")
            .expect("evidence block reported");
        for key in [
            "per_dimension",
            "dispositions",
            "false_drop_count",
            "false_drop_rate",
            "judge_calls_per_query",
            "added_latency_p50_us",
            "added_latency_p95_us",
        ] {
            assert!(evidence.get(key).is_some(), "missing evidence key {key}");
        }
        assert_eq!(
            evidence["false_drop_count"].as_u64(),
            Some(0),
            "the default policy must not drop a labelled keep candidate"
        );

        let markdown = std::fs::read_to_string(&md_out).expect("markdown written");
        assert!(
            markdown.contains("## Evidence Classification"),
            "markdown report must render the evidence section"
        );
        assert!(
            markdown.contains(EVIDENCE_COMPARISON_STRATEGY),
            "markdown report must list the evidence arm"
        );
    }

    #[tokio::test]
    async fn test_benchmark_single_strategy_and_missing_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fixture = write_tiny_corpus(dir.path());
        let json_out = dir.path().join("single.json");

        benchmark(
            Some(fixture),
            "local_only".to_string(),
            false,
            false,
            None,
            false,
            0.05,
            0.05,
            0.05,
            0.50,
            0.20,
            Some(json_out.clone()),
            None,
            false,
            OutputFormat::Json,
        )
        .await
        .expect("single-strategy benchmark runs");

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&json_out).expect("json written"))
                .expect("valid report json");
        assert!(
            !json["strategies"]
                .as_object()
                .expect("strategies map")
                .contains_key(RERANK_COMPARISON_STRATEGY),
            "the comparison arm is opt-in"
        );
        assert!(
            json["strategies"]["local_only"]["evidence_metrics"].is_null(),
            "the evidence block is opt-in"
        );

        let missing = dir.path().join("does-not-exist.json");
        let error = benchmark(
            Some(missing),
            "all".to_string(),
            false,
            false,
            None,
            false,
            0.05,
            0.05,
            0.05,
            0.50,
            0.20,
            None,
            None,
            false,
            OutputFormat::Json,
        )
        .await
        .expect_err("a missing fixture must fail the command");
        assert!(
            error.to_string().contains("does-not-exist.json"),
            "error must name the missing fixture: {error}"
        );
    }
}
