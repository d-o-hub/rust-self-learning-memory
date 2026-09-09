//! Retrieval and cascade observability (issue #962).
//!
//! Dependency-free, label-bounded telemetry for the retrieval plane:
//! request counts and durations by operation/tier/outcome, candidate-set
//! sizes by stage, query-cache hits/misses, embedding calls by provider,
//! Tier-4 fallback reasons, and recommendation-feedback signals. Renders
//! both JSON snapshots (MCP `get_metrics`) and Prometheus text exposition
//! (see `MetricsRegistry::export_metrics`).
//!
//! ## Cardinality and redaction contract
//!
//! Every label is a fieldless enum with a fixed snake_case vocabulary
//! documented in [`labels`] — raw queries, episode IDs, tags, and provider
//! error strings can never become label values. Series are stored in
//! fixed-size arrays (no per-value map growth), so cardinality is
//! compile-time bounded: at most 2·11·2 request series plus the smaller
//! per-family tables.
//!
//! ## Wiring
//!
//! Call sites record into the process-global registry
//! ([`global_retrieval_metrics`]); tests use isolated [`RetrievalMetrics`]
//! instances or serial delta assertions on the global one.

mod exposition;
mod labels;
mod registry;

pub use labels::{
    CacheLayer, EmbeddingOutcome, EmbeddingProviderLabel, FallbackReason, FeedbackSignal,
    RetrievalOperation, RetrievalOutcome, RetrievalStage, RetrievalTier,
};
pub use registry::{
    RetrievalMetrics, cascade_tier, global_retrieval_metrics, provisional_fallback_reason,
};
