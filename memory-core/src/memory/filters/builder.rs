//! Builder pattern for `EpisodeFilter`
//!
//! Provides a fluent API for constructing `EpisodeFilter` instances.

use super::types::{EpisodeFilter, OutcomeType};
use chrono::{DateTime, Utc};

/// Builder for constructing episode filters with a fluent API
#[derive(Debug, Clone, Default)]
pub struct EpisodeFilterBuilder {
    filter: EpisodeFilter,
}

impl EpisodeFilterBuilder {
    /// Match episodes with ANY of these tags
    #[must_use]
    pub fn with_any_tags(mut self, tags: Vec<String>) -> Self {
        self.filter.with_any_tags = Some(tags);
        self
    }

    /// Match episodes with ALL of these tags
    #[must_use]
    pub fn with_all_tags(mut self, tags: Vec<String>) -> Self {
        self.filter.with_all_tags = Some(tags);
        self
    }

    /// Filter by task types
    #[must_use]
    pub fn task_types(mut self, types: Vec<crate::types::TaskType>) -> Self {
        self.filter.task_types = Some(types);
        self
    }

    /// Filter by domains
    #[must_use]
    pub fn domains(mut self, domains: Vec<String>) -> Self {
        self.filter.domains = Some(domains);
        self
    }

    /// Episodes started after this date
    #[must_use]
    pub fn date_from(mut self, date: DateTime<Utc>) -> Self {
        self.filter.date_from = Some(date);
        self
    }

    /// Episodes started before this date
    #[must_use]
    pub fn date_to(mut self, date: DateTime<Utc>) -> Self {
        self.filter.date_to = Some(date);
        self
    }

    /// Only completed episodes
    #[must_use]
    pub fn completed_only(mut self, completed: bool) -> Self {
        self.filter.completed_only = Some(completed);
        self
    }

    /// Only archived episodes
    #[must_use]
    pub fn archived_only(mut self, archived: bool) -> Self {
        self.filter.archived_only = Some(archived);
        self
    }

    /// Exclude archived episodes
    #[must_use]
    pub fn exclude_archived(mut self, exclude: bool) -> Self {
        self.filter.exclude_archived = Some(exclude);
        self
    }

    /// Only successful episodes
    #[must_use]
    pub fn success_only(mut self, success: bool) -> Self {
        self.filter.success_only = Some(success);
        self
    }

    /// Minimum reward score
    #[must_use]
    pub fn min_reward(mut self, min: f32) -> Self {
        self.filter.min_reward = Some(min);
        self
    }

    /// Maximum reward score
    #[must_use]
    pub fn max_reward(mut self, max: f32) -> Self {
        self.filter.max_reward = Some(max);
        self
    }

    /// Search text in description
    #[must_use]
    pub fn search_text(mut self, text: String) -> Self {
        self.filter.search_text = Some(text);
        self
    }

    /// Filter by outcome type
    #[must_use]
    pub fn outcome_type(mut self, outcome: OutcomeType) -> Self {
        self.filter.outcome_type = Some(outcome);
        self
    }

    /// Build the final filter
    #[must_use]
    pub fn build(self) -> EpisodeFilter {
        self.filter
    }
}
