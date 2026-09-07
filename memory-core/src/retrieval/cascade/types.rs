//! Configuration and result types for the cascading retrieval pipeline.
//!
//! Split out of `mod.rs` to keep individual source files under the 500 LOC
//! quality gate (WG-185).

/// Configuration for the cascading retrieval pipeline.
#[derive(Debug, Clone)]
pub struct CascadeConfig {
    /// Number of results to return from each tier.
    pub top_k: usize,
    /// Minimum score threshold for BM25 results (0.0-1.0).
    pub bm25_threshold: f32,
    /// Minimum similarity threshold for HDC results (0.0-1.0).
    pub hdc_threshold: f32,
    /// Minimum confidence threshold for ConceptGraph results (0.0-1.0).
    pub concept_graph_threshold: f32,
    /// Whether to merge results across tiers.
    pub merge_results: bool,
    /// Minimum results before escalating to next tier.
    pub min_results: usize,
    /// Enable/disable ConceptGraph expansion.
    pub enable_concept_expansion: bool,
    /// Tier 4 fallback policy (issue #968).
    ///
    /// Governs the fallback decision when CPU-local tiers do not already
    /// satisfy the query. See [`FallbackPolicy`].
    pub fallback_policy: FallbackPolicy,
    /// Minimum top local score for an `Adaptive` fallback to stay local.
    ///
    /// A local result set with top score below this threshold is treated as
    /// insufficiently confident and escalates to Tier 4. Default `0.78`.
    pub local_confidence_threshold: f32,
    /// Minimum margin between the top two local scores for `Adaptive`.
    ///
    /// A result set whose winner does not beat the runner-up by at least this
    /// margin is treated as ambiguous and escalates to Tier 4. With fewer
    /// than two results the absent score counts as `0.0`. Default `0.08`.
    pub minimum_score_margin: f32,
}

impl Default for CascadeConfig {
    fn default() -> Self {
        Self {
            top_k: 10,
            bm25_threshold: 0.3,
            hdc_threshold: 0.5,
            concept_graph_threshold: 0.4,
            merge_results: true,
            min_results: 3,
            enable_concept_expansion: true,
            fallback_policy: FallbackPolicy::Adaptive,
            local_confidence_threshold: 0.78,
            minimum_score_margin: 0.08,
        }
    }
}

/// Tier 4 embedding-fallback policy (issue #968).
///
/// The highest-cost, highest-tail-latency operation in the cascade is the
/// Tier 4 network embedding fallback. The policy calibrates that decision
/// against local retrieval confidence instead of result counts alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FallbackPolicy {
    /// Invoke Tier 4 only when the local result is insufficiently confident
    /// (top score below [`CascadeConfig::local_confidence_threshold`] or
    /// winner margin below [`CascadeConfig::minimum_score_margin`]).
    #[default]
    Adaptive,
    /// Always count a Tier 4 embedding call, even on local hits.
    ///
    /// For baseline comparisons and debugging; local results are still
    /// returned.
    AlwaysEmbed,
    /// Never count a Tier 4 call; return the best local results as-is.
    LocalOnly,
}

impl std::fmt::Display for FallbackPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FallbackPolicy::Adaptive => write!(f, "adaptive"),
            FallbackPolicy::AlwaysEmbed => write!(f, "always_embed"),
            FallbackPolicy::LocalOnly => write!(f, "local_only"),
        }
    }
}

/// Reason a query did or did not trigger a Tier 4 embedding call.
///
/// Values are intentionally bounded and free of query text or identifiers so
/// they are safe as metric labels (telemetry contract in
/// `plans/GOAP_FEATURE_WAVE_2026-09-04.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackReason {
    /// A CPU-local tier already satisfied the query.
    LocalTierSufficient,
    /// `Adaptive` policy: confident local result, no Tier 4 call.
    LocalConfident,
    /// `Adaptive` policy: local result exists but is not confident.
    InsufficientConfidence,
    /// `Adaptive` policy: no local results at all.
    NoLocalResults,
    /// `AlwaysEmbed` policy forced the Tier 4 call.
    AlwaysEmbedPolicy,
    /// `LocalOnly` policy suppressed the Tier 4 call.
    LocalOnlyPolicy,
}

impl std::fmt::Display for FallbackReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FallbackReason::LocalTierSufficient => write!(f, "local_tier_sufficient"),
            FallbackReason::LocalConfident => write!(f, "local_confident"),
            FallbackReason::InsufficientConfidence => write!(f, "insufficient_confidence"),
            FallbackReason::NoLocalResults => write!(f, "no_local_results"),
            FallbackReason::AlwaysEmbedPolicy => write!(f, "always_embed_policy"),
            FallbackReason::LocalOnlyPolicy => write!(f, "local_only_policy"),
        }
    }
}

/// Result from a single tier in the cascade.
#[derive(Debug, Clone)]
pub struct TierResult {
    /// Tier identifier (bm25, hdc, concept_graph, api).
    pub tier: String,
    /// Retrieved episode IDs with scores as tuples.
    pub results: Vec<(String, f32)>,
    /// Whether this tier produced sufficient results.
    pub sufficient: bool,
}

impl TierResult {
    /// Get episode IDs from results.
    #[must_use]
    pub fn ids(&self) -> Vec<String> {
        self.results.iter().map(|(id, _)| id.clone()).collect()
    }

    /// Get scores from results.
    #[must_use]
    pub fn scores(&self) -> Vec<f32> {
        self.results.iter().map(|(_, score)| *score).collect()
    }

    /// Check if results are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get number of results.
    #[must_use]
    pub fn len(&self) -> usize {
        self.results.len()
    }
}

/// Final result from the cascading retrieval pipeline.
#[derive(Debug, Clone)]
pub struct CascadeResult {
    /// Final merged/re-ranked episode IDs.
    pub episode_ids: Vec<String>,
    /// Final merged/re-ranked scores.
    pub scores: Vec<f32>,
    /// Which tier(s) contributed to the final result.
    pub contributing_tiers: Vec<String>,
    /// Number of API calls made (should be 0 or 1).
    pub api_calls: u32,
    /// Why Tier 4 was or was not invoked (issue #968).
    pub fallback_reason: FallbackReason,
    /// Top local score behind the fallback decision (`0.0` when empty).
    pub top_score: f32,
    /// Winner margin (top minus runner-up, absent score counts as `0.0`).
    pub score_margin: f32,
}

/// Error type for the cascading retrieval pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum CascadeError {
    /// The `csm` feature is disabled, so cascade retrieval is unavailable.
    CapabilityUnavailable,
}

impl std::fmt::Display for CascadeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CascadeError::CapabilityUnavailable => write!(
                f,
                "cascade retrieval is unavailable without the `csm` feature"
            ),
        }
    }
}

impl std::error::Error for CascadeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cascade_config_default_returns_expected_values() {
        let config = CascadeConfig::default();

        assert_eq!(config.top_k, 10);
        assert_eq!(config.bm25_threshold, 0.3);
        assert_eq!(config.hdc_threshold, 0.5);
        assert_eq!(config.concept_graph_threshold, 0.4);
        assert!(config.merge_results);
        assert_eq!(config.min_results, 3);
        assert!(config.enable_concept_expansion);
        assert_eq!(config.fallback_policy, FallbackPolicy::Adaptive);
        assert!((config.local_confidence_threshold - 0.78).abs() < f32::EPSILON);
        assert!((config.minimum_score_margin - 0.08).abs() < f32::EPSILON);
    }

    #[test]
    fn tier_result_ids_returns_episode_ids() {
        let result = TierResult {
            tier: "bm25".to_string(),
            results: vec![
                ("ep1".to_string(), 0.9),
                ("ep2".to_string(), 0.8),
                ("ep3".to_string(), 0.7),
            ],
            sufficient: true,
        };

        let ids = result.ids();

        assert_eq!(ids, vec!["ep1", "ep2", "ep3"]);
    }

    #[test]
    fn tier_result_scores_returns_score_values() {
        let result = TierResult {
            tier: "hdc".to_string(),
            results: vec![("ep1".to_string(), 0.95), ("ep2".to_string(), 0.85)],
            sufficient: true,
        };

        let scores = result.scores();

        assert_eq!(scores.len(), 2);
        assert!((scores[0] - 0.95).abs() < f32::EPSILON);
        assert!((scores[1] - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn tier_result_is_empty_true_when_no_results() {
        let result = TierResult {
            tier: "concept_graph".to_string(),
            results: vec![],
            sufficient: false,
        };

        assert!(result.is_empty());
    }

    #[test]
    fn tier_result_is_empty_false_when_has_results() {
        let result = TierResult {
            tier: "bm25".to_string(),
            results: vec![("ep1".to_string(), 0.9)],
            sufficient: true,
        };

        assert!(!result.is_empty());
    }

    #[test]
    fn tier_result_len_returns_correct_count() {
        let result = TierResult {
            tier: "api".to_string(),
            results: vec![
                ("ep1".to_string(), 0.9),
                ("ep2".to_string(), 0.8),
                ("ep3".to_string(), 0.7),
                ("ep4".to_string(), 0.6),
            ],
            sufficient: true,
        };

        assert_eq!(result.len(), 4);
    }

    #[test]
    fn tier_result_empty_results() {
        let result = TierResult {
            tier: "bm25".to_string(),
            results: vec![],
            sufficient: false,
        };

        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
        assert!(result.ids().is_empty());
        assert!(result.scores().is_empty());
    }

    #[test]
    fn cascade_result_construction_and_field_access() {
        let result = CascadeResult {
            episode_ids: vec!["ep1".to_string(), "ep2".to_string()],
            scores: vec![0.9, 0.8],
            contributing_tiers: vec!["bm25".to_string(), "hdc".to_string()],
            api_calls: 0,
            fallback_reason: FallbackReason::LocalTierSufficient,
            top_score: 0.9,
            score_margin: 0.1,
        };

        assert_eq!(result.episode_ids, vec!["ep1", "ep2"]);
        assert_eq!(result.scores.len(), 2);
        assert!((result.scores[0] - 0.9).abs() < f32::EPSILON);
        assert!((result.scores[1] - 0.8).abs() < f32::EPSILON);
        assert_eq!(result.contributing_tiers, vec!["bm25", "hdc"]);
        assert_eq!(result.api_calls, 0);
        assert_eq!(result.fallback_reason, FallbackReason::LocalTierSufficient);
        assert!((result.top_score - 0.9).abs() < f32::EPSILON);
        assert!((result.score_margin - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn fallback_policy_display_uses_bounded_label_values() {
        assert_eq!(FallbackPolicy::Adaptive.to_string(), "adaptive");
        assert_eq!(FallbackPolicy::AlwaysEmbed.to_string(), "always_embed");
        assert_eq!(FallbackPolicy::LocalOnly.to_string(), "local_only");
        assert_eq!(FallbackPolicy::default(), FallbackPolicy::Adaptive);
    }

    #[test]
    fn fallback_reason_display_uses_bounded_label_values() {
        assert_eq!(
            FallbackReason::LocalTierSufficient.to_string(),
            "local_tier_sufficient"
        );
        assert_eq!(
            FallbackReason::LocalConfident.to_string(),
            "local_confident"
        );
        assert_eq!(
            FallbackReason::InsufficientConfidence.to_string(),
            "insufficient_confidence"
        );
        assert_eq!(
            FallbackReason::NoLocalResults.to_string(),
            "no_local_results"
        );
        assert_eq!(
            FallbackReason::AlwaysEmbedPolicy.to_string(),
            "always_embed_policy"
        );
        assert_eq!(
            FallbackReason::LocalOnlyPolicy.to_string(),
            "local_only_policy"
        );
    }

    #[test]
    fn cascade_error_capability_unavailable_construct_and_partial_eq() {
        let error = CascadeError::CapabilityUnavailable;

        assert_eq!(error, CascadeError::CapabilityUnavailable);
    }

    #[test]
    fn cascade_error_display_and_error_impl() {
        let error = CascadeError::CapabilityUnavailable;

        assert!(
            error.to_string().contains("unavailable"),
            "display should describe the unavailable capability"
        );
        assert!(std::error::Error::source(&error).is_none());
    }
}
