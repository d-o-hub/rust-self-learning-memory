//! Matching logic for `EpisodeFilter`
//!
//! Implements the core matching algorithms for filtering episodes.

use super::types::{EpisodeFilter, OutcomeType};
use crate::episode::Episode;
use crate::search::{fuzzy_match, fuzzy_search_in_text, regex_search, SearchField, SearchMode};
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

        // Check search text with configurable search mode
        if let Some(ref search) = self.search_text {
            if !self.matches_search_text(episode, search) {
                return false;
            }
        }

        true
    }

    /// Helper method to check if episode matches search text
    fn matches_search_text(&self, episode: &Episode, search: &str) -> bool {
        let search_mode = self.search_mode.as_ref().unwrap_or(&SearchMode::Exact);
        let search_fields = self
            .search_fields
            .as_deref()
            .unwrap_or(&[SearchField::Description]);

        // Collect texts to search based on selected fields
        let texts_to_search = self.collect_searchable_texts(episode, search_fields);

        // Perform search based on mode
        self.search_in_texts(&texts_to_search, search, search_mode)
    }

    /// Collect texts from episode based on selected fields
    fn collect_searchable_texts<'a>(
        &self,
        episode: &'a Episode,
        fields: &[SearchField],
    ) -> Vec<&'a str> {
        let mut texts = Vec::new();

        for field in fields {
            match field {
                SearchField::Description => {
                    texts.push(episode.task_description.as_str());
                }
                SearchField::Steps => {
                    self.collect_step_texts(episode, &mut texts);
                }
                SearchField::Outcome => {
                    self.collect_outcome_text(episode, &mut texts);
                }
                SearchField::Tags => {
                    texts.extend(episode.context.tags.iter().map(String::as_str));
                }
                SearchField::Domain => {
                    texts.push(episode.context.domain.as_str());
                }
                SearchField::All => {
                    texts.push(episode.task_description.as_str());
                    texts.push(episode.context.domain.as_str());
                    texts.extend(episode.context.tags.iter().map(String::as_str));
                    self.collect_step_texts(episode, &mut texts);
                    self.collect_outcome_text(episode, &mut texts);
                }
            }
        }

        texts
    }

    /// Collect searchable text from episode steps
    fn collect_step_texts<'a>(&self, episode: &'a Episode, texts: &mut Vec<&'a str>) {
        for step in &episode.steps {
            texts.push(step.action.as_str());
            if let Some(ref result) = step.result {
                match result {
                    crate::types::ExecutionResult::Success { output } => {
                        texts.push(output.as_str());
                    }
                    crate::types::ExecutionResult::Error { message } => {
                        texts.push(message.as_str());
                    }
                    crate::types::ExecutionResult::Timeout => {
                        // No text to search
                    }
                }
            }
        }
    }

    /// Collect searchable text from episode outcome
    fn collect_outcome_text<'a>(&self, episode: &'a Episode, texts: &mut Vec<&'a str>) {
        if let Some(ref outcome) = episode.outcome {
            let outcome_text = match outcome {
                TaskOutcome::Success { verdict, .. } => verdict.as_str(),
                TaskOutcome::PartialSuccess { verdict, .. } => verdict.as_str(),
                TaskOutcome::Failure { reason, .. } => reason.as_str(),
            };
            texts.push(outcome_text);
        }
    }

    /// Search for query in collected texts based on search mode
    fn search_in_texts(&self, texts: &[&str], query: &str, mode: &SearchMode) -> bool {
        match mode {
            SearchMode::Exact => {
                let search_lower = query.to_lowercase();
                texts
                    .iter()
                    .any(|text| text.to_lowercase().contains(&search_lower))
            }
            SearchMode::Fuzzy { threshold } => texts.iter().any(|text| {
                // Try both direct match and search in text
                fuzzy_match(text, query, *threshold).is_some()
                    || !fuzzy_search_in_text(text, query, *threshold).is_empty()
            }),
            SearchMode::Regex => {
                // Try regex search on each text
                texts.iter().any(|text| {
                    regex_search(text, query)
                        .map(|matches| !matches.is_empty())
                        .unwrap_or(false)
                })
            }
        }
    }

    /// Apply this filter to a collection of episodes
    #[must_use]
    pub fn apply(&self, episodes: Vec<Episode>) -> Vec<Episode> {
        episodes.into_iter().filter(|e| self.matches(e)).collect()
    }

    /// Apply this filter to a collection of Arc-wrapped episodes
    /// This avoids cloning when filtering `Arc<Episode>` collections
    #[must_use]
    pub fn apply_arc(
        &self,
        episodes: Vec<std::sync::Arc<Episode>>,
    ) -> Vec<std::sync::Arc<Episode>> {
        episodes.into_iter().filter(|e| self.matches(e)).collect()
    }

    /// Count how many episodes match this filter
    #[must_use]
    pub fn count_matches(&self, episodes: &[Episode]) -> usize {
        episodes.iter().filter(|e| self.matches(e)).count()
    }
}
