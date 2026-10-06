//! Incremental vs. full-rebuild ranking index benchmarks (issue #1078).
//!
//! Measures the ADR-082 derived `RankingIndex` at 1,000 / 10,000 / 100,000
//! history entries:
//! - `full_rebuild`: the canonical `RankingIndex::from_history` reduction over
//!   the whole history (cold start and fallback path),
//! - `incremental_update`: `RankingIndex::apply_feedback` for one accepted
//!   replacement, which must not scale with total history size.
//!
//! The incremental path additionally avoids re-materializing the whole index on
//! every accepted feedback; this bench measures the latency half of that win
//! (the rebuild's cost grows with the history, the incremental update's does
//! not), which is the observable issue #1078 asks for.
//!
//! Run with: `cargo bench -p do-memory-benches --bench ranking_incremental`

use std::hint::black_box;
use std::time::Duration;

use chrono::Utc;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use do_memory_core::TaskOutcome;
use do_memory_core::memory::attribution::{
    RankingIndex, RecommendationFeedback, RecommendationSession,
};

/// History sizes required by issue #1078.
const SIZES: [usize; 3] = [1_000, 10_000, 100_000];
/// Distinct patterns spread across the history.
const PATTERNS: usize = 512;
/// Rotating replacements for the updated session (each touches two patterns).
const ROTATION: usize = 16;

fn pattern_id(index: usize) -> String {
    format!("pattern-{:04}", index % PATTERNS)
}

fn session_id(index: usize) -> uuid::Uuid {
    uuid::Uuid::from_u128(index as u128)
}

fn success() -> TaskOutcome {
    TaskOutcome::Success {
        verdict: "ok".to_string(),
        artifacts: vec![],
    }
}

fn failure() -> TaskOutcome {
    TaskOutcome::Failure {
        reason: "no".to_string(),
        error_details: None,
    }
}

fn make_feedback(session: usize, applied: &[String], positive: bool) -> RecommendationFeedback {
    RecommendationFeedback {
        session_id: session_id(session),
        applied_pattern_ids: applied.to_vec(),
        consulted_episode_ids: vec![],
        outcome: if positive { success() } else { failure() },
        agent_rating: None,
    }
}

/// A reproducible history of `size` sessions, plus rotating replacements for the
/// session under test (`session 0`).
struct History {
    sessions: Vec<RecommendationSession>,
    feedback: Vec<RecommendationFeedback>,
    replacements: Vec<RecommendationFeedback>,
}

impl History {
    fn build(size: usize) -> Self {
        let sessions: Vec<RecommendationSession> = (0..size)
            .map(|i| RecommendationSession {
                session_id: session_id(i),
                episode_id: uuid::Uuid::from_u128(1_000_000 + i as u128),
                timestamp: Utc::now(),
                recommended_pattern_ids: vec![pattern_id(i)],
                recommended_playbook_ids: vec![],
            })
            .collect();

        let feedback: Vec<RecommendationFeedback> = (0..size)
            .map(|i| make_feedback(i, &[pattern_id(i), pattern_id(i + 1)], i % 3 != 0))
            .collect();

        let replacements: Vec<RecommendationFeedback> = (0..ROTATION)
            .map(|r| make_feedback(0, &[pattern_id(r), pattern_id(r + 1)], r % 2 == 0))
            .collect();

        Self {
            sessions,
            feedback,
            replacements,
        }
    }
}

fn bench_ranking_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("ranking_index");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    for size in SIZES {
        let history = History::build(size);

        group.bench_with_input(BenchmarkId::new("full_rebuild", size), &history, |b, h| {
            b.iter(|| black_box(RankingIndex::from_history(&h.sessions, &h.feedback)));
        });

        group.bench_with_input(
            BenchmarkId::new("incremental_update", size),
            &history,
            |b, h| {
                let mut index = RankingIndex::from_history(&h.sessions, &h.feedback);
                let mut next = 0usize;
                b.iter(|| {
                    let replacement = &h.replacements[next % h.replacements.len()];
                    next += 1;
                    // Replacing one session keeps the index size stable, so each
                    // iteration measures exactly one incremental update.
                    black_box(index.apply_feedback(replacement))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_ranking_index);
criterion_main!(benches);
