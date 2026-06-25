//! Shard router implementation (WG-122).

use super::{EpisodeMetadata, RoutingResult, ScopeFilter, ShardConfig};

/// Shard router for scope-before-search filtering.
///
/// Pre-filters episodes by metadata before expensive vector search.
pub struct ShardRouter {
    config: ShardConfig,
}

impl ShardRouter {
    /// Create a new shard router.
    pub fn new(config: ShardConfig) -> Self {
        Self { config }
    }

    /// Create a router with default configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(ShardConfig::default())
    }

    /// Route query through scope filter.
    ///
    /// Returns filtered candidates for vector search.
    #[must_use]
    pub fn route(&self, filter: &ScopeFilter, episodes: &[EpisodeMetadata]) -> RoutingResult {
        if episodes.is_empty() {
            return RoutingResult::empty(filter.clone());
        }

        let original_count = episodes.len();
        let mut scored_candidates: Vec<(uuid::Uuid, f32)> = Vec::new();

        for ep in episodes {
            let score = self.compute_route_score(filter, ep);

            // Apply hard filters
            if !self.passes_hard_filters(filter, ep) {
                continue;
            }

            if score > 0.0 {
                scored_candidates.push((ep.episode_id, score));
            }
        }

        // Sort by score descending
        scored_candidates
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Cap to max_candidates
        let capped = scored_candidates.len() > self.config.max_candidates;
        scored_candidates.truncate(self.config.max_candidates);

        let candidates: Vec<uuid::Uuid> = scored_candidates.iter().map(|(id, _)| *id).collect();
        let scores: Vec<f32> = scored_candidates.iter().map(|(_, s)| *s).collect();
        let filtered_count = candidates.len();

        RoutingResult {
            candidates,
            original_count,
            filtered_count,
            capped,
            filter: filter.clone(),
            scores,
        }
    }

    /// Compute routing score for an episode.
    fn compute_route_score(&self, filter: &ScopeFilter, ep: &EpisodeMetadata) -> f32 {
        let tag_score = self.compute_tag_score(filter, ep);
        let task_type_score = self.compute_task_type_score(filter, ep);
        let timeframe_score = self.compute_timeframe_score(filter, ep);

        let base_score = tag_score * self.config.tag_weight
            + task_type_score * self.config.task_type_weight
            + timeframe_score * self.config.timeframe_weight;

        // Apply temporal decay if configured
        let final_score = if self.config.use_temporal_decay {
            let decay = self.compute_temporal_decay(ep);
            base_score * decay
        } else {
            base_score
        };

        // Apply success rate bonus if configured
        let success_bonus = if let Some(min_rate) = filter.min_success_rate {
            if ep.success_rate >= min_rate {
                0.1 // Bonus for meeting success rate threshold
            } else {
                0.0
            }
        } else {
            0.0
        };

        final_score + success_bonus
    }

    /// Compute tag overlap score.
    fn compute_tag_score(&self, filter: &ScopeFilter, ep: &EpisodeMetadata) -> f32 {
        if filter.required_tags.is_empty() {
            return 0.5; // No tag constraint, neutral score
        }

        let matching_tags = filter.required_tags.intersection(&ep.tags).count();

        if matching_tags == 0 {
            return 0.0;
        }

        matching_tags as f32 / filter.required_tags.len().max(1) as f32
    }

    /// Compute task type match score.
    fn compute_task_type_score(&self, filter: &ScopeFilter, ep: &EpisodeMetadata) -> f32 {
        if filter.required_task_types.is_empty() {
            return 0.5; // No task type constraint
        }

        if filter
            .required_task_types
            .iter()
            .any(|tt| ep.matches_task_type(tt))
        {
            return 1.0;
        }

        0.0
    }

    /// Compute timeframe match score.
    fn compute_timeframe_score(&self, filter: &ScopeFilter, ep: &EpisodeMetadata) -> f32 {
        match &filter.time_range {
            Some(range) => {
                if range.contains(ep.created_at) {
                    // Higher score for more recent episodes within range
                    let age_days = ep.age_days();
                    let range_days = range.duration_days();
                    if range_days > 0 {
                        1.0 - (age_days as f32 / range_days as f32).min(1.0)
                    } else {
                        1.0
                    }
                } else {
                    0.0
                }
            }
            None => 0.5, // No time constraint
        }
    }

    /// Compute temporal decay factor.
    fn compute_temporal_decay(&self, ep: &EpisodeMetadata) -> f32 {
        let age_days = ep.age_days();
        if age_days <= 7 {
            1.0 // Full score for very recent
        } else if age_days <= self.config.stale_days {
            // Linear decay from 1.0 to 0.5
            1.0 - (0.5 * (age_days - 7) as f32 / (self.config.stale_days - 7) as f32)
        } else {
            0.3 // Minimum for stale episodes
        }
    }

    /// Check if episode passes hard filters (must-match constraints).
    fn passes_hard_filters(&self, filter: &ScopeFilter, ep: &EpisodeMetadata) -> bool {
        // Check required tags - must have at least one matching
        if !filter.required_tags.is_empty() {
            let has_match = filter.required_tags.intersection(&ep.tags).count() > 0;
            if !has_match {
                return false;
            }
        }

        // Check excluded tags - must not have any
        if filter.excluded_tags.iter().any(|t| ep.has_tag(t)) {
            return false;
        }

        // Check required task types - must match at least one
        if !filter.required_task_types.is_empty()
            && !filter
                .required_task_types
                .iter()
                .any(|tt| ep.matches_task_type(tt))
        {
            return false;
        }

        // Check time range - must be within range if specified
        if let Some(range) = &filter.time_range {
            if !range.contains(ep.created_at) {
                return false;
            }
        }

        // Check min success rate if specified
        if let Some(min_rate) = filter.min_success_rate {
            if ep.success_rate < min_rate {
                return false;
            }
        }

        true
    }

    /// Estimate candidate reduction from routing.
    ///
    /// Returns estimated percentage of episodes that will be filtered.
    #[must_use]
    pub fn estimate_reduction(&self, filter: &ScopeFilter, _total_episodes: usize) -> f32 {
        if !filter.has_constraints() {
            return 0.0;
        }

        // Estimate based on constraint count
        let constraint_factor = filter.constraint_count() as f32 * 0.15;
        let time_factor = if filter.time_range.is_some() {
            0.2
        } else {
            0.0
        };

        // Cap estimate at 80%
        ((constraint_factor + time_factor) * 100.0).min(80.0)
    }

    /// Get configuration.
    #[must_use]
    pub fn config(&self) -> &ShardConfig {
        &self.config
    }
}
