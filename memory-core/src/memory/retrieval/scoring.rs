//! Scoring functions for relevance calculations

use crate::episode::Episode;
use crate::types::TaskContext;
use std::sync::Arc;

use super::super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Check if episode is relevant to the query
    pub(super) fn is_relevant_episode(
        &self,
        episode: &Arc<Episode>,
        context: &TaskContext,
        task_description: &str,
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

        // Match on tags
        let common_tags: Vec<_> = episode
            .context
            .tags
            .iter()
            .filter(|t| context.tags.contains(t))
            .collect();

        if !common_tags.is_empty() {
            return true;
        }

        // Simple text matching on description (very basic)
        let desc_lower = task_description.to_lowercase();
        let episode_desc_lower = episode.task_description.to_lowercase();

        let common_words: Vec<_> = desc_lower
            .split_whitespace()
            .filter(|w| w.len() > 3) // Ignore short words
            .filter(|w| episode_desc_lower.contains(w))
            .collect();

        !common_words.is_empty()
    }

    /// Calculate relevance score for an episode
    pub(super) fn calculate_relevance_score(
        &self,
        episode: &Arc<Episode>,
        context: &TaskContext,
        task_description: &str,
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

        let common_tags: Vec<_> = episode_ref
            .context
            .tags
            .iter()
            .filter(|t| context.tags.contains(t))
            .collect();

        if !common_tags.is_empty() {
            context_score += 0.1 * common_tags.len() as f32;
        }

        score += context_score.min(0.4);

        // Description similarity (30% weight)
        let desc_lower = task_description.to_lowercase();
        let episode_desc_lower = episode_ref.task_description.to_lowercase();

        let desc_words: Vec<_> = desc_lower.split_whitespace().collect();
        let common_words: Vec<_> = desc_words
            .iter()
            .filter(|w| w.len() > 3)
            .filter(|w| episode_desc_lower.contains(**w))
            .collect();

        if !desc_words.is_empty() {
            let similarity = common_words.len() as f32 / desc_words.len() as f32;
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
        context: &TaskContext,
    ) -> f32 {
        let mut score = 0.0;

        // Extract context from the heuristic condition
        // Heuristics store context information in their condition string
        let condition_lower = heuristic.condition.to_lowercase();

        // Check domain match (look for domain in condition string)
        if condition_lower.contains(&context.domain.to_lowercase()) {
            score += 1.0;
        }

        // Check language match
        if let Some(lang) = &context.language {
            if condition_lower.contains(&lang.to_lowercase()) {
                score += 0.8;
            }
        }

        // Check framework match
        if let Some(framework) = &context.framework {
            if condition_lower.contains(&framework.to_lowercase()) {
                score += 0.5;
            }
        }

        // Check tag overlap
        for tag in &context.tags {
            if condition_lower.contains(&tag.to_lowercase()) {
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
