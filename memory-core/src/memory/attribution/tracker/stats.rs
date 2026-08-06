//! Aggregate recommendation effectiveness statistics.

use tracing::instrument;

use super::RecommendationTracker;
use crate::memory::attribution::types::RecommendationStats;

impl RecommendationTracker {
    /// Calculate overall recommendation effectiveness statistics.
    #[instrument(skip(self))]
    pub async fn get_stats(&self) -> RecommendationStats {
        let sessions = self.sessions.read().await;
        let feedback = self.feedback.read().await;

        let total_sessions = sessions.len();
        let total_feedback = feedback.len();

        // Calculate pattern statistics
        let mut total_recommended: usize = 0;
        let mut total_applied: usize = 0;
        let mut successful_applications: usize = 0;
        let mut total_ratings: f32 = 0.0;
        let mut rating_count: usize = 0;

        for session in sessions.values() {
            total_recommended += session.recommended_pattern_ids.len();
        }

        for fb in feedback.values() {
            total_applied += fb.applied_pattern_ids.len();

            // Check if outcome was successful
            if matches!(
                fb.outcome,
                crate::types::TaskOutcome::Success { .. }
                    | crate::types::TaskOutcome::PartialSuccess { .. }
            ) {
                successful_applications += fb.applied_pattern_ids.len();
            }

            if let Some(rating) = fb.agent_rating {
                total_ratings += rating;
                rating_count += 1;
            }
        }

        let patterns_ignored = total_recommended.saturating_sub(total_applied);

        let adoption_rate = if total_recommended > 0 {
            total_applied as f32 / total_recommended as f32
        } else {
            0.0
        };

        let success_after_adoption_rate = if total_applied > 0 {
            successful_applications as f32 / total_applied as f32
        } else {
            0.0
        };

        let avg_agent_rating = if rating_count > 0 {
            Some(total_ratings / rating_count as f32)
        } else {
            None
        };

        RecommendationStats {
            total_sessions,
            total_feedback,
            patterns_applied: total_applied,
            patterns_ignored,
            successful_applications,
            adoption_rate,
            success_after_adoption_rate,
            avg_agent_rating,
        }
    }
}
