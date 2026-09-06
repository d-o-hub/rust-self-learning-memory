//! Retrieval quality and cost benchmark command (`eval benchmark`).

use std::path::PathBuf;

use do_memory_core::retrieval::{
    BenchmarkReport, FixtureCorpus, RegressionChecker, RegressionThresholds, RetrievalEvaluator,
    RetrievalStrategy, format_markdown_report,
};

use crate::output::OutputFormat;

#[allow(clippy::too_many_arguments)]
pub async fn benchmark(
    fixture_path: Option<PathBuf>,
    strategy_str: String,
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

    let evaluator = RetrievalEvaluator::new(corpus);

    // 2. Execute benchmark
    let report: BenchmarkReport = if strategy_str.eq_ignore_ascii_case("all") {
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
            Some(checker.check(&report, &baseline_report))
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
