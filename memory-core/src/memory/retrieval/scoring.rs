//! Scoring functions for relevance calculations

use crate::episode::Episode;
use crate::types::TaskContext;
use std::collections::HashSet;
use std::sync::Arc;

use super::super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Check if episode is relevant to the query
    pub(super) fn is_relevant_episode(
        &self,
        episode: &Arc<Episode>,
        context: &TaskContext,
        query_tags: &HashSet<&String>,
        query_words_gt3: &[&str],
    ) -> bool {
        // Match on domain
        if episode.context.domain == context.domain {
            return true;
        }

        // Match on language
        if episode.context.language == context.language && episode.context.language.is_some() {
            return true;
        }

        // Match on framework
        if episode.context.framework == context.framework && episode.context.framework.is_some() {
            return true;
        }

        // Match on tags using pre-calculated HashSet for O(1) lookup
        if episode.context.tags.iter().any(|t| query_tags.contains(t)) {
            return true;
        }

        // Simple text matching on description (very basic)
        // Optimization: Use pre-calculated words and avoid intermediate Vec allocation
        let episode_desc_lower = episode.task_description.to_lowercase();
        query_words_gt3
            .iter()
            .any(|&w| episode_desc_lower.contains(w))
    }

    /// Calculate relevance score for an episode
    pub(super) fn calculate_relevance_score(
        &self,
        episode: &Arc<Episode>,
        context: &TaskContext,
        query_tags: &HashSet<&String>,
        query_words: &[&str],
        query_words_gt3: &[&str],
    ) -> f32 {
        let episode_ref: &Episode = episode.as_ref();
        let mut score = 0.0;

        // Reward quality (30% weight)
        if let Some(reward) = &episode_ref.reward {
            score += reward.total * 0.3;
        }

        // Context match (40% weight)
        let mut context_score = 0.0;

        if episode_ref.context.domain == context.domain {
            context_score += 0.4;
        }

        if episode_ref.context.language == context.language
            && episode_ref.context.language.is_some()
        {
            context_score += 0.3;
        }

        if episode_ref.context.framework == context.framework
            && episode_ref.context.framework.is_some()
        {
            context_score += 0.2;
        }

        // Optimization: Use pre-calculated HashSet and avoid intermediate Vec allocation
        let common_tags_count = episode_ref
            .context
            .tags
            .iter()
            .filter(|t| query_tags.contains(t))
            .count();

        if common_tags_count > 0 {
            context_score += 0.1 * common_tags_count as f32;
        }

        score += context_score.min(0.4);

        // Description similarity (30% weight)
        // Optimization: Use pre-calculated words and avoid intermediate Vec allocation
        if !query_words.is_empty() {
            let episode_desc_lower = episode_ref.task_description.to_lowercase();
            let common_words_count = query_words_gt3
                .iter()
                .filter(|&&w| episode_desc_lower.contains(w))
                .count();

            let similarity = common_words_count as f32 / query_words.len() as f32;
            score += similarity * 0.3;
        }

        score
    }

    /// Calculate relevance score for a heuristic based on context
    ///
    /// Scoring:
    /// - Domain exact match: +1.0
    /// - Language exact match: +0.8
    /// - Framework match: +0.5
    /// - Tag overlap: +0.3 per matching tag
    pub(super) fn calculate_heuristic_relevance(
        &self,
        heuristic: &crate::pattern::Heuristic,
        domain_lower: &str,
        language_lower: Option<&str>,
        framework_lower: Option<&str>,
        tags_lower: &[String],
    ) -> f32 {
        let mut score = 0.0;

        // Extract context from the heuristic condition
        // Heuristics store context information in their condition string
        let condition_lower = heuristic.condition.to_lowercase();

        // Check domain match (look for domain in condition string)
        if condition_lower.contains(domain_lower) {
            score += 1.0;
        }

        // Check language match
        if let Some(lang) = language_lower {
            if condition_lower.contains(lang) {
                score += 0.8;
            }
        }

        // Check framework match
        if let Some(framework) = framework_lower {
            if condition_lower.contains(framework) {
                score += 0.5;
            }
        }

        // Check tag overlap
        for tag in tags_lower {
            if condition_lower.contains(tag) {
                score += 0.3;
            }
        }

        // If no specific matches, give a small baseline score for general heuristics
        if score == 0.0 {
            score = 0.1;
        }

        score
    }
}
