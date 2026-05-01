//! Signature matcher implementation (WG-121).

use super::{ExecutionSignature, QuerySignature, SignatureConfig, SignatureMatch};

/// Matcher for execution signatures.
///
/// Matches query signatures against episode signatures to find
/// episodes with similar execution patterns.
pub struct SignatureMatcher {
    config: SignatureConfig,
}

impl SignatureMatcher {
    /// Create a new signature matcher.
    pub fn new(config: SignatureConfig) -> Self {
        Self { config }
    }

    /// Create a matcher with default configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(SignatureConfig::default())
    }

    /// Match a query against episode signatures.
    ///
    /// Returns ranked matches sorted by score descending.
    #[must_use]
    pub fn match_query(
        &self,
        query: &QuerySignature,
        signatures: &[ExecutionSignature],
    ) -> Vec<SignatureMatch> {
        if signatures.is_empty() {
            return Vec::new();
        }

        let mut matches: Vec<SignatureMatch> = Vec::new();

        for sig in signatures {
            let match_result = self.compute_match(query, sig);

            // Filter by minimum threshold
            if match_result.score >= self.config.min_overlap_threshold {
                matches.push(match_result);
            }
        }

        // Sort by score descending
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Normalize scores if configured
        if self.config.normalize_scores && !matches.is_empty() {
            let max_score = matches[0].score;
            if max_score > 0.0 {
                for m in &mut matches {
                    m.score /= max_score;
                }
            }
        }

        matches
    }

    /// Compute match score between query and signature.
    fn compute_match(&self, query: &QuerySignature, sig: &ExecutionSignature) -> SignatureMatch {
        let tool_score = self.compute_tool_overlap(query, sig);
        let error_score = self.compute_error_overlap(query, sig);
        let structure_score = self.compute_structure_similarity(query, sig);

        let contributing_components: Vec<String> = [
            (tool_score > 0.0, "tools"),
            (error_score > 0.0, "errors"),
            (structure_score > 0.0, "structure"),
        ]
        .iter()
        .filter(|(has, _)| *has)
        .map(|(_, name)| (*name).to_string())
        .collect();

        let score = tool_score * self.config.tool_weight
            + error_score * self.config.error_weight
            + structure_score * self.config.structure_weight;

        SignatureMatch {
            episode_id: sig.episode_id,
            score,
            tool_score,
            error_score,
            structure_score,
            contributing_components,
        }
    }

    /// Compute tool overlap score.
    fn compute_tool_overlap(&self, query: &QuerySignature, sig: &ExecutionSignature) -> f32 {
        if query.expected_tools.is_empty() || sig.tools.is_empty() {
            return 0.0;
        }

        let intersection = query.expected_tools.intersection(&sig.tools).count();

        // Jaccard-like similarity
        let union = query.expected_tools.union(&sig.tools).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f32 / union as f32
    }

    /// Compute error overlap score.
    fn compute_error_overlap(&self, query: &QuerySignature, sig: &ExecutionSignature) -> f32 {
        if query.relevant_errors.is_empty() {
            // If query doesn't specify errors, score based on episode having few errors
            if sig.has_errors() {
                return 0.5; // Episodes with errors might be relevant for debugging
            }
            return 0.8; // Episodes without errors are good examples
        }

        if sig.error_types.is_empty() {
            return 0.3; // Episode has no errors, might not help with error handling
        }

        let intersection = query.relevant_errors.intersection(&sig.error_types).count();

        let union = query.relevant_errors.union(&sig.error_types).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f32 / union as f32
    }

    /// Compute structure similarity score.
    fn compute_structure_similarity(
        &self,
        query: &QuerySignature,
        sig: &ExecutionSignature,
    ) -> f32 {
        match &query.expected_pattern {
            Some(pattern) => pattern.similarity(&sig.step_pattern),
            None => {
                // Without expected pattern, score based on success rate
                sig.success_rate()
            }
        }
    }

    /// Get the configuration for this matcher.
    #[must_use]
    pub fn config(&self) -> &SignatureConfig {
        &self.config
    }
}
