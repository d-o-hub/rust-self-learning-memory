//! Matching logic for EpisodeFilter
//!
//! Implements the core matching algorithms for filtering episodes.

use super::types::{EpisodeFilter, OutcomeType};
use crate::episode::Episode;
use crate::types::TaskOutcome;

impl EpisodeFilter {
    /// Check if this filter matches the given episode
    #[must_use]
    pub fn matches(&self, episode: &Episode) -> bool {
        // Check tags - ANY match
        if let Some(ref tags) = self.with_any_tags {
            if !tags.is_empty() {
                let has_any_tag = tags.iter().any(|tag| episode.context.tags.contains(tag));
                if !has_any_tag {
                    return false;
                }
            }
        }

        // Check tags - ALL match
        if let Some(ref tags) = self.with_all_tags {
            if !tags.is_empty() {
                let has_all_tags = tags.iter().all(|tag| episode.context.tags.contains(tag));
                if !has_all_tags {
                    return false;
                }
            }
        }

        // Check task types
        if let Some(ref types) = self.task_types {
            if !types.contains(&episode.task_type) {
                return false;
            }
        }

        // Check domains
        if let Some(ref domains) = self.domains {
            if !domains.contains(&episode.context.domain) {
                return false;
            }
        }

        // Check date range - start date
        if let Some(from) = self.date_from {
            if episode.start_time < from {
                return false;
            }
        }

        // Check date range - end date
        if let Some(to) = self.date_to {
            if episode.start_time > to {
                return false;
            }
        }

        // Check completed status
        if let Some(completed) = self.completed_only {
            if completed && !episode.is_complete() {
                return false;
            }
        }

        // Check archived status
        let is_archived = episode.metadata.contains_key("archived_at");

        if let Some(true) = self.archived_only {
            if !is_archived {
                return false;
            }
        }

        if let Some(true) = self.exclude_archived {
            if is_archived {
                return false;
            }
        }

        // Check success status
        if let Some(true) = self.success_only {
            if let Some(ref outcome) = episode.outcome {
                if !matches!(outcome, TaskOutcome::Success { .. }) {
                    return false;
                }
            } else {
                return false; // Not completed, so not successful
            }
        }

        // Check outcome type
        if let Some(outcome_type) = self.outcome_type {
            if let Some(ref outcome) = episode.outcome {
                let matches_outcome = matches!(
                    (outcome_type, outcome),
                    (OutcomeType::Success, TaskOutcome::Success { .. })
                        | (
                            OutcomeType::PartialSuccess,
                            TaskOutcome::PartialSuccess { .. }
                        )
                        | (OutcomeType::Failure, TaskOutcome::Failure { .. })
                );
                if !matches_outcome {
                    return false;
                }
            } else {
                return false; // No outcome yet
            }
        }

        // Check reward range
        if let Some(min_reward) = self.min_reward {
            if let Some(ref reward) = episode.reward {
                if reward.total < min_reward {
                    return false;
                }
            } else {
                return false; // No reward yet
            }
        }

        if let Some(max_reward) = self.max_reward {
            if let Some(ref reward) = episode.reward {
                if reward.total > max_reward {
                    return false;
                }
            } else {
                return false; // No reward yet
            }
        }

        // Check search text
        if let Some(ref search) = self.search_text {
            let search_lower = search.to_lowercase();
            let description_lower = episode.task_description.to_lowercase();
            if !description_lower.contains(&search_lower) {
                return false;
            }
        }

        true
    }

    /// Apply this filter to a collection of episodes
    #[must_use]
    pub fn apply(&self, episodes: Vec<Episode>) -> Vec<Episode> {
        episodes.into_iter().filter(|e| self.matches(e)).collect()
    }

    /// Count how many episodes match this filter
    #[must_use]
    pub fn count_matches(&self, episodes: &[Episode]) -> usize {
        episodes.iter().filter(|e| self.matches(e)).count()
    }
}
