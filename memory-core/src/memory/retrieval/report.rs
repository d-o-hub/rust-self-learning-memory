//! Execution-backed retrieval report types (issue #1079).
//!
//! Split out of `execution.rs` to honor the 500-LOC-per-file invariant. These
//! types describe what actually ran rather than what was requested; unknown
//! values stay `None` instead of being inferred from the result count.

use crate::episode::Episode;
use std::sync::Arc;

/// Execution-backed metadata for one retrieval operation (issue #1079).
///
/// Describes what actually ran rather than what was requested. Unknown values
/// stay `None` instead of being inferred from the result count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalExecution {
    /// Whether the query cache served this result.
    pub cache_hit: bool,
    /// Whether the retrieval pipeline executed (`!cache_hit`).
    pub executed: bool,
    /// Serving tier label (see [`RetrievalTier::as_str`](crate::monitoring::metrics::RetrievalTier::as_str)).
    pub tier: String,
    /// Whether a higher tier was attempted and this path is a fallback.
    pub fallback: bool,
    /// Candidate count before limit truncation, when the path exposes it.
    pub candidate_count: Option<usize>,
    /// Final result count returned to the caller.
    pub result_count: usize,
    /// Retrieval-pipeline latency in milliseconds (bounded to `u64`).
    pub latency_ms: u64,
}

impl RetrievalExecution {
    /// Cache-served execution: one lookup, no pipeline run, no candidates scored.
    pub(super) fn from_cache_hit(result_count: usize, start: std::time::Instant) -> Self {
        Self {
            cache_hit: true,
            executed: false,
            tier: crate::monitoring::metrics::RetrievalTier::Cache
                .as_str()
                .to_string(),
            fallback: false,
            candidate_count: None,
            result_count,
            latency_ms: elapsed_ms(start),
        }
    }

    /// Executed pipeline path with its measured metadata.
    pub(super) fn from_execution(
        tier: crate::monitoring::metrics::RetrievalTier,
        candidate_count: Option<usize>,
        result_count: usize,
        fallback: bool,
        start: std::time::Instant,
    ) -> Self {
        Self {
            cache_hit: false,
            executed: true,
            tier: tier.as_str().to_string(),
            fallback,
            candidate_count,
            result_count,
            latency_ms: elapsed_ms(start),
        }
    }
}

/// Result of one retrieval branch: episodes plus the tier and candidate count
/// that produced them.
pub(super) struct BranchOutcome {
    /// Episodes served by this branch.
    pub episodes: Vec<Arc<Episode>>,
    /// Serving tier of this branch.
    pub tier: crate::monitoring::metrics::RetrievalTier,
    /// Pre-truncation candidate count when the branch exposes it.
    pub candidate_count: Option<usize>,
}

/// Bounded milliseconds elapsed since `start`.
fn elapsed_ms(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}
