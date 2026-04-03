//! Pattern Affinity Classification for DyMoE Routing-Drift Protection
//!
//! Inspired by LLaVA-DyMoE (CVPR 2026), this module implements routing-drift
//! protection to prevent ambiguous episodes from corrupting established
//! high-success-rate pattern clusters during pattern extraction.
//!
//! Key concepts:
//! - **Drel (relative affinity)**: Measures how ambiguous an episode is relative
//!   to old vs new pattern clusters. Drel ≈ 0 → ambiguous episode.
//! - **Episode Assignment Guard**: Two-dimensional gate combining success_rate
//!   and affinity_clarity to control pattern mutation.
//!
//! Reference: <https://zhaoc5.github.io/DyMoE/> (Section 3.1-3.2)

mod classifier;
pub(crate) mod computation;
#[cfg(test)]
mod tests;
mod types;

pub use classifier::PatternAffinityClassifier;
pub use types::{
    DEFAULT_AFFINITY_THRESHOLD, DEFAULT_MIN_SUCCESS_RATE, EpisodeAssignmentGuard, RejectionReason,
    RelativeAffinity,
};
