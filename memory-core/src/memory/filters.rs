//! Advanced filtering for episodes
//!
//! This module provides comprehensive filtering capabilities for episodes,
//! allowing users to query episodes based on multiple criteria including
//! tags, dates, task types, outcomes, and more.

use crate::episode::Episode;
use crate::types::{TaskOutcome, TaskType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Comprehensive filter for querying episodes
///
/// Provides rich filtering capabilities across multiple dimensions.
/// All filter criteria are combined with AND logic (must match all specified criteria).
///
/// # Examples
///
/// ```
/// use memory_core::EpisodeFilter;
/// use memory_core::TaskType;
/// use chrono::Utc;
///
/// // Filter for successful code generation episodes in the last week
/// let filter = EpisodeFilter::builder()
///     .task_types(vec![TaskType::CodeGeneration])
///     .success_only(true)
///     .date_from(Utc::now() - chrono::Duration::days(7))
///     .build();
///
/// // Filter by tags
/// let tagged_filter = EpisodeFilter::builder()
///     .with_any_tags(vec!["async".to_string(), "networking".to_string()])
///     .build();
///
/// // Filter archived episodes
/// let archived_filter = EpisodeFilter::builder()
///     .archived_only(true)
///     .build();
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EpisodeFilter {
    /// Filter by tags - matches episodes with ANY of these tags
    pub with_any_tags: Option<Vec<String>>,

    /// Filter by tags - matches episodes with ALL of these tags
    pub with_all_tags: Option<Vec<String>>,

    /// Filter by task types
    pub task_types: Option<Vec<TaskType>>,

    /// Filter by domains
    pub domains: Option<Vec<String>>,

    /// Episodes started after this date
    pub date_from: Option<DateTime<Utc>>,

    /// Episodes started before this date
    pub date_to: Option<DateTime<Utc>>,

    /// Only include completed episodes
    pub completed_only: Option<bool>,

    /// Only include archived episodes
    pub archived_only: Option<bool>,

    /// Exclude archived episodes
    pub exclude_archived: Option<bool>,

    /// Only include successful episodes
    pub success_only: Option<bool>,

    /// Minimum reward score
    pub min_reward: Option<f32>,

    /// Maximum reward score
    pub max_reward: Option<f32>,

    /// Search text in task description (case-insensitive)
    pub search_text: Option<String>,

    /// Filter by specific outcome type
    pub outcome_type: Option<OutcomeType>,
}

/// Outcome type for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeType {
    /// Only successful outcomes
    Success,
    /// Only partial success outcomes
    PartialSuccess,
    /// Only failure outcomes
    Failure,
}

impl EpisodeFilter {
    /// Create a new empty filter (matches all episodes)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for fluent filter construction
    pub fn builder() -> EpisodeFilterBuilder {
        EpisodeFilterBuilder::default()
    }

    /// Check if this filter matches the given episode
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
                let matches_outcome = match (outcome_type, outcome) {
                    (OutcomeType::Success, TaskOutcome::Success { .. }) => true,
                    (OutcomeType::PartialSuccess, TaskOutcome::PartialSuccess { .. }) => true,
                    (OutcomeType::Failure, TaskOutcome::Failure { .. }) => true,
                    _ => false,
                };
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
    pub fn apply(&self, episodes: Vec<Episode>) -> Vec<Episode> {
        episodes.into_iter().filter(|e| self.matches(e)).collect()
    }

    /// Count how many episodes match this filter
    pub fn count_matches(&self, episodes: &[Episode]) -> usize {
        episodes.iter().filter(|e| self.matches(e)).count()
    }
}

/// Builder for constructing episode filters with a fluent API
#[derive(Debug, Clone, Default)]
pub struct EpisodeFilterBuilder {
    filter: EpisodeFilter,
}

impl EpisodeFilterBuilder {
    /// Match episodes with ANY of these tags
    pub fn with_any_tags(mut self, tags: Vec<String>) -> Self {
        self.filter.with_any_tags = Some(tags);
        self
    }

    /// Match episodes with ALL of these tags
    pub fn with_all_tags(mut self, tags: Vec<String>) -> Self {
        self.filter.with_all_tags = Some(tags);
        self
    }

    /// Filter by task types
    pub fn task_types(mut self, types: Vec<TaskType>) -> Self {
        self.filter.task_types = Some(types);
        self
    }

    /// Filter by domains
    pub fn domains(mut self, domains: Vec<String>) -> Self {
        self.filter.domains = Some(domains);
        self
    }

    /// Episodes started after this date
    pub fn date_from(mut self, date: DateTime<Utc>) -> Self {
        self.filter.date_from = Some(date);
        self
    }

    /// Episodes started before this date
    pub fn date_to(mut self, date: DateTime<Utc>) -> Self {
        self.filter.date_to = Some(date);
        self
    }

    /// Only completed episodes
    pub fn completed_only(mut self, completed: bool) -> Self {
        self.filter.completed_only = Some(completed);
        self
    }

    /// Only archived episodes
    pub fn archived_only(mut self, archived: bool) -> Self {
        self.filter.archived_only = Some(archived);
        self
    }

    /// Exclude archived episodes
    pub fn exclude_archived(mut self, exclude: bool) -> Self {
        self.filter.exclude_archived = Some(exclude);
        self
    }

    /// Only successful episodes
    pub fn success_only(mut self, success: bool) -> Self {
        self.filter.success_only = Some(success);
        self
    }

    /// Minimum reward score
    pub fn min_reward(mut self, min: f32) -> Self {
        self.filter.min_reward = Some(min);
        self
    }

    /// Maximum reward score
    pub fn max_reward(mut self, max: f32) -> Self {
        self.filter.max_reward = Some(max);
        self
    }

    /// Search text in description
    pub fn search_text(mut self, text: String) -> Self {
        self.filter.search_text = Some(text);
        self
    }

    /// Filter by outcome type
    pub fn outcome_type(mut self, outcome: OutcomeType) -> Self {
        self.filter.outcome_type = Some(outcome);
        self
    }

    /// Build the final filter
    pub fn build(self) -> EpisodeFilter {
        self.filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityLevel, RewardScore, TaskContext};

    fn create_test_episode(
        task_type: TaskType,
        domain: &str,
        tags: Vec<String>,
        completed: bool,
        success: bool,
        reward: Option<f32>,
    ) -> Episode {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext {
                language: None,
                framework: None,
                complexity: ComplexityLevel::Moderate,
                domain: domain.to_string(),
                tags,
            },
            task_type,
        );

        if completed {
            let outcome = if success {
                TaskOutcome::Success {
                    verdict: "Success".to_string(),
                    artifacts: vec![],
                }
            } else {
                TaskOutcome::Failure {
                    reason: "Failed".to_string(),
                    error_details: None,
                }
            };
            episode.complete(outcome);
        }

        if let Some(r) = reward {
            episode.reward = Some(RewardScore {
                total: r,
                base: 1.0,
                efficiency: 1.0,
                complexity_bonus: 1.0,
                quality_multiplier: 1.0,
                learning_bonus: 0.0,
            });
        }

        episode
    }

    #[test]
    fn test_filter_by_tags_any() {
        let ep1 = create_test_episode(
            TaskType::CodeGeneration,
            "web",
            vec!["async".to_string(), "http".to_string()],
            true,
            true,
            None,
        );

        let ep2 = create_test_episode(
            TaskType::Debugging,
            "web",
            vec!["sync".to_string()],
            true,
            true,
            None,
        );

        let filter = EpisodeFilter::builder()
            .with_any_tags(vec!["async".to_string()])
            .build();

        assert!(filter.matches(&ep1));
        assert!(!filter.matches(&ep2));
    }

    #[test]
    fn test_filter_by_tags_all() {
        let ep1 = create_test_episode(
            TaskType::CodeGeneration,
            "web",
            vec!["async".to_string(), "http".to_string()],
            true,
            true,
            None,
        );

        let ep2 = create_test_episode(
            TaskType::Debugging,
            "web",
            vec!["async".to_string()],
            true,
            true,
            None,
        );

        let filter = EpisodeFilter::builder()
            .with_all_tags(vec!["async".to_string(), "http".to_string()])
            .build();

        assert!(filter.matches(&ep1));
        assert!(!filter.matches(&ep2));
    }

    #[test]
    fn test_filter_by_task_type() {
        let ep1 = create_test_episode(TaskType::CodeGeneration, "web", vec![], true, true, None);

        let ep2 = create_test_episode(TaskType::Debugging, "web", vec![], true, true, None);

        let filter = EpisodeFilter::builder()
            .task_types(vec![TaskType::CodeGeneration])
            .build();

        assert!(filter.matches(&ep1));
        assert!(!filter.matches(&ep2));
    }

    #[test]
    fn test_filter_by_completion() {
        let ep_complete =
            create_test_episode(TaskType::CodeGeneration, "web", vec![], true, true, None);

        let ep_incomplete =
            create_test_episode(TaskType::CodeGeneration, "web", vec![], false, false, None);

        let filter = EpisodeFilter::builder().completed_only(true).build();

        assert!(filter.matches(&ep_complete));
        assert!(!filter.matches(&ep_incomplete));
    }

    #[test]
    fn test_filter_by_success() {
        let ep_success =
            create_test_episode(TaskType::CodeGeneration, "web", vec![], true, true, None);

        let ep_failure =
            create_test_episode(TaskType::CodeGeneration, "web", vec![], true, false, None);

        let filter = EpisodeFilter::builder().success_only(true).build();

        assert!(filter.matches(&ep_success));
        assert!(!filter.matches(&ep_failure));
    }

    #[test]
    fn test_filter_by_reward() {
        let ep_high = create_test_episode(
            TaskType::CodeGeneration,
            "web",
            vec![],
            true,
            true,
            Some(1.5),
        );

        let ep_low = create_test_episode(
            TaskType::CodeGeneration,
            "web",
            vec![],
            true,
            true,
            Some(0.5),
        );

        let filter = EpisodeFilter::builder().min_reward(1.0).build();

        assert!(filter.matches(&ep_high));
        assert!(!filter.matches(&ep_low));
    }

    #[test]
    fn test_filter_combined() {
        let ep = create_test_episode(
            TaskType::CodeGeneration,
            "web-api",
            vec!["async".to_string(), "rest".to_string()],
            true,
            true,
            Some(1.5),
        );

        let filter = EpisodeFilter::builder()
            .task_types(vec![TaskType::CodeGeneration])
            .domains(vec!["web-api".to_string()])
            .with_any_tags(vec!["async".to_string()])
            .completed_only(true)
            .success_only(true)
            .min_reward(1.0)
            .build();

        assert!(filter.matches(&ep));
    }
}
