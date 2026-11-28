#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! External-consumer example: adaptive source discovery with episodic memory.
//!
//! The example simulates a search application that must decide *how* to discover
//! sources for a query family. A static application always uses the same
//! strategy/provider; this example retrieves prior episodes first and only moves
//! to a bounded alternative when the evidence says the baseline is failing.
//!
//! ```text
//! cargo run -p do-memory-examples --example adaptive_source_discovery -- --phase all --require-evidence
//! ```
//!
//! Phases:
//!
//! - `seed` records the dataset's history as episodes (and optionally emits
//!   memory-harness traces). It is a replay: each attempt carries a stable
//!   identity, so re-running it resolves to the episodes that already record
//!   those attempts instead of adding a second observation for every row.
//! - `recommend` opens the same database in a fresh process, retrieves evidence,
//!   chooses a bounded plan per family, and scores both plans against the
//!   deterministic ground truth.
//! - `verify` is `recommend` plus a fingerprint comparison against a report
//!   written by an earlier process, which is the restart/replay check.
//!
//! Memory recommendations are advisory: a discovered source is only a candidate
//! and the consuming application must verify it before acting on it.

mod adaptive_discovery;

use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::path::PathBuf;

use adaptive_discovery::dataset::Dataset;
use adaptive_discovery::memory_store::{DiscoveryMemory, RecordState};
use adaptive_discovery::metrics::{self, Report};
use adaptive_discovery::planner;
use adaptive_discovery::trace::Trace;

/// Default dataset path, resolved against the crate so the example runs from any
/// working directory in a clean checkout.
const DEFAULT_DATASET: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/fixtures/adaptive-discovery/dataset.json"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Seed,
    Recommend,
    Verify,
    All,
}

impl Phase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Seed => "seed",
            Self::Recommend => "recommend",
            Self::Verify => "verify",
            Self::All => "all",
        }
    }
}

#[derive(Debug)]
struct Args {
    db: PathBuf,
    dataset: PathBuf,
    report_dir: PathBuf,
    emit_traces: Option<PathBuf>,
    phase: Phase,
    limit: usize,
    require_evidence: bool,
    expect: Option<PathBuf>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            db: PathBuf::from("data/adaptive-discovery/memory.db"),
            dataset: PathBuf::from(DEFAULT_DATASET),
            report_dir: PathBuf::from("data/adaptive-discovery/report"),
            emit_traces: None,
            phase: Phase::All,
            limit: 16,
            require_evidence: false,
            expect: None,
        }
    }
}

impl Args {
    fn parse(mut argv: impl Iterator<Item = String>) -> Result<Self> {
        let mut parsed = Self::default();
        while let Some(flag) = argv.next() {
            let mut value = || -> Result<String> {
                argv.next()
                    .with_context(|| format!("{flag} requires a value"))
            };
            match flag.as_str() {
                "--db" => parsed.db = PathBuf::from(value()?),
                "--dataset" => parsed.dataset = PathBuf::from(value()?),
                "--report-dir" => parsed.report_dir = PathBuf::from(value()?),
                "--emit-traces" => parsed.emit_traces = Some(PathBuf::from(value()?)),
                "--limit" => parsed.limit = value()?.parse().context("--limit must be a number")?,
                "--phase" => {
                    parsed.phase = match value()?.as_str() {
                        "seed" => Phase::Seed,
                        "recommend" => Phase::Recommend,
                        "verify" => Phase::Verify,
                        "all" => Phase::All,
                        other => bail!("unknown phase `{other}` (seed|recommend|verify|all)"),
                    };
                }
                "--require-evidence" => parsed.require_evidence = true,
                "--expect" => parsed.expect = Some(PathBuf::from(value()?)),
                "--help" | "-h" => {
                    println!(
                        "adaptive_source_discovery [--db PATH] [--dataset PATH] \
                         [--report-dir DIR] [--emit-traces DIR] \
                         [--phase seed|recommend|verify|all] [--limit N] \
                         [--require-evidence] [--expect PATH]"
                    );
                    std::process::exit(0);
                }
                other => bail!("unknown argument `{other}` (try --help)"),
            }
        }
        if parsed.phase == Phase::Verify && parsed.expect.is_none() {
            bail!("the verify phase requires --expect <previous report.json>");
        }
        Ok(parsed)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // The only caller is the operator running the example locally: these values
    // select a fixture path and an output directory, and nothing downstream
    // treats them as trusted input for a privilege decision.
    // nosemgrep: rust.lang.security.args.args
    run(Args::parse(std::env::args().skip(1))?).await
}

async fn run(args: Args) -> Result<()> {
    let dataset = Dataset::load(&args.dataset)?;
    println!(
        "dataset: {} families, version {} ({})",
        dataset.families.len(),
        dataset.version,
        args.dataset.display()
    );

    match args.phase {
        Phase::Seed => seed(&args, &dataset).await,
        Phase::Recommend | Phase::Verify => recommend(&args, &dataset).await,
        Phase::All => {
            seed(&args, &dataset).await?;
            recommend(&args, &dataset).await
        }
    }
}

async fn seed(args: &Args, dataset: &Dataset) -> Result<()> {
    let memory = DiscoveryMemory::open(&args.db).await?;
    let mut recorded = 0;
    for family in &dataset.families {
        let legacy = memory.legacy_episodes(family, args.limit).await?;
        if legacy > 0 {
            bail!(
                "{} episode(s) for `{}` carry no attempt identity, so this database cannot be \
                 replayed; re-seed into a fresh --db ({}) instead of counting those rows twice",
                legacy,
                family.id,
                args.db.display()
            );
        }

        let mut created = 0;
        let mut replayed = 0;
        let mut retried = 0;
        for (index, attempt) in family.history.iter().enumerate() {
            let seeded = memory
                .record(family, attempt, &attempt.identity(&family.id, index))
                .await?;
            recorded += 1;
            match seeded.state {
                RecordState::Created => created += 1,
                RecordState::Replayed => replayed += 1,
                RecordState::Retried => retried += 1,
            }
            if let Some(dir) = &args.emit_traces {
                let trace = Trace::from_attempt(family, attempt, seeded.episode_id)?;
                trace.write(dir)?;
            }
        }
        println!(
            "seeded {}: {} prior attempts ({} created, {} replayed, {} retried)",
            family.id,
            family.history.len(),
            created,
            replayed,
            retried
        );
    }
    println!("recorded {recorded} attempts into {}", args.db.display());
    if let Some(dir) = &args.emit_traces {
        println!("emitted memory-harness traces into {}", dir.display());
    }
    Ok(())
}

async fn recommend(args: &Args, dataset: &Dataset) -> Result<()> {
    let memory = DiscoveryMemory::open(&args.db).await?;
    let mut baseline_plans = BTreeMap::new();
    let mut guided_plans = BTreeMap::new();
    let mut decisions = Vec::new();
    let mut advisory = BTreeMap::new();
    let mut missing_evidence = Vec::new();

    for family in &dataset.families {
        let (observations, ignored) = memory.observations(family, args.limit).await?;
        // A family whose dataset history is empty is *expected* to have no
        // evidence: that is the cold-start case the planner must answer with the
        // baseline. Only a family that declares history but retrieves none is a
        // seeding failure.
        if observations.is_empty() && !family.history.is_empty() {
            missing_evidence.push(family.id.clone());
        }
        let decision = planner::choose(family, &observations, ignored);
        println!(
            "{:<24} baseline {:<20} guided {:<20} {} ({} observations, {} unattributed)",
            family.id,
            family.baseline,
            decision.plan,
            decision.reason.as_str(),
            observations.len(),
            ignored
        );
        baseline_plans.insert(family.id.clone(), family.baseline.clone());
        guided_plans.insert(family.id.clone(), decision.plan.clone());
        advisory.insert(
            family.id.clone(),
            memory.advisory_patterns(family, 3).await?,
        );
        decisions.push(decision);
    }

    if args.require_evidence && !missing_evidence.is_empty() {
        bail!(
            "no episodic evidence retrieved for {}; seed the database at {} first",
            missing_evidence.join(", "),
            args.db.display()
        );
    }

    let baseline = metrics::evaluate(dataset, &baseline_plans)?;
    let guided = metrics::evaluate(dataset, &guided_plans)?;
    let report = metrics::build_report(dataset, baseline, guided, decisions, advisory);
    let report_path = args
        .report_dir
        .join(format!("report-{}.json", args.phase.as_str()));
    report.write(&report_path)?;
    println!();
    print!("{}", report.render_markdown());
    println!("report: {}", report_path.display());

    if let Some(expected_path) = &args.expect {
        compare_with_previous(expected_path, &report)?;
    }
    if !report.expectations.passed {
        bail!(
            "dataset expectations failed: {}",
            report.expectations.failures.join("; ")
        );
    }
    Ok(())
}

fn compare_with_previous(path: &std::path::Path, report: &Report) -> Result<()> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read previous report {}", path.display()))?;
    let previous: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("previous report is not valid JSON: {}", path.display()))?;
    let previous_fingerprint = previous
        .get("fingerprint")
        .and_then(serde_json::Value::as_str)
        .with_context(|| format!("previous report has no fingerprint: {}", path.display()))?;
    if previous_fingerprint != report.fingerprint {
        bail!(
            "recommendation fingerprint changed across processes: {} -> {} \
             (prior evidence was not recovered identically)",
            previous_fingerprint,
            report.fingerprint
        );
    }
    println!(
        "fingerprint {} matches the previous process ({})",
        report.fingerprint,
        path.display()
    );
    Ok(())
}
