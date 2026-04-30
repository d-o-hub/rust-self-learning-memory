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
        episode_desc_lower: &str,
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
        episode_desc_lower: &str,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::Episode;
    use crate::types::{ComplexityLevel, TaskContext, TaskType};
    use std::collections::HashSet;
    use uuid::Uuid;

    fn create_test_episode(domain: &str, lang: Option<&str>, tags: Vec<&str>) -> Arc<Episode> {
        Arc::new(Episode {
            episode_id: Uuid::new_v4(),
            task_type: TaskType::CodeGeneration,
            task_description: "Implement a rust web api with axum".to_string(),
            context: TaskContext {
                domain: domain.to_string(),
                language: lang.map(|s| s.to_string()),
                framework: Some("axum".to_string()),
                complexity: ComplexityLevel::Moderate,
                tags: tags.into_iter().map(|s| s.to_string()).collect(),
            },
            start_time: chrono::Utc::now(),
            end_time: Some(chrono::Utc::now()),
            steps: vec![],
            outcome: None,
            reward: None,
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            applied_patterns: vec![],
            salient_features: None,
            metadata: std::collections::HashMap::new(),
            tags: Vec::new(),
            checkpoints: Vec::new(),
        })
    }

    #[test]
    fn test_is_relevant_episode() {
        let memory = SelfLearningMemory::new();
        let episode = create_test_episode("web-api", Some("rust"), vec!["rest", "auth"]);
        let context = TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        };

        let query_tags_vec = vec!["rest".to_string()];
        let query_tags: HashSet<&String> = query_tags_vec.iter().collect();
        let query_words_gt3 = vec!["axum", "rust"];
        let desc_lower = episode.task_description.to_lowercase();

        // Domain match
        assert!(memory.is_relevant_episode(
            &episode,
            &context,
            &query_tags,
            &query_words_gt3,
            &desc_lower
        ));

        // Mismatch everything
        let context_mismatch = TaskContext {
            domain: "data".to_string(),
            language: Some("python".to_string()),
            ..Default::default()
        };
        let empty_tags = HashSet::new();
        let mismatch_words = vec!["data", "science"];
        assert!(!memory.is_relevant_episode(
            &episode,
            &context_mismatch,
            &empty_tags,
            &mismatch_words,
            &desc_lower
        ));

        // Tag match
        let context_tag_only = TaskContext {
            domain: "other".to_string(),
            ..Default::default()
        };
        assert!(memory.is_relevant_episode(
            &episode,
            &context_tag_only,
            &query_tags,
            &mismatch_words,
            &desc_lower
        ));

        // Word match
        assert!(memory.is_relevant_episode(
            &episode,
            &context_tag_only,
            &empty_tags,
            &query_words_gt3,
            &desc_lower
        ));
    }

    #[test]
    fn test_calculate_relevance_score() {
        let memory = SelfLearningMemory::new();
        let episode = create_test_episode("web-api", Some("rust"), vec!["rest"]);
        let context = TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        };

        let query_tags_vec = vec!["rest".to_string()];
        let query_tags: HashSet<&String> = query_tags_vec.iter().collect();
        let query_words = vec!["rust", "web", "api"];
        let query_words_gt3 = vec!["rust"];
        let desc_lower = episode.task_description.to_lowercase();

        let score = memory.calculate_relevance_score(
            &episode,
            &context,
            &query_tags,
            &query_words,
            &query_words_gt3,
            &desc_lower,
        );

        assert!(score > 0.0);
        // Domain match (0.4) + Lang match (0.3) + Tag match (0.1) = 0.8 context score
        // capped at 0.4.
        // Description similarity: 1 common word / 3 total words = 0.33 * 0.3 = 0.1
        // Total score ~ 0.5
        assert!(score >= 0.49 && score <= 0.51);
    }

    #[test]
    fn test_calculate_heuristic_relevance() {
        let memory = SelfLearningMemory::new();
        let heuristic = crate::pattern::Heuristic {
            heuristic_id: Uuid::new_v4(),
            condition: "In rust web-api using axum".to_string(),
            action: "Use middleware".to_string(),
            confidence: 0.8,
            evidence: crate::Evidence {
                episode_ids: vec![],
                success_rate: 0.9,
                sample_size: 1,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let domain_lower = "web-api";
        let lang_lower = Some("rust");
        let framework_lower = Some("axum");
        let tags_lower = vec!["auth".to_string()];

        let score = memory.calculate_heuristic_relevance(
            &heuristic,
            domain_lower,
            lang_lower,
            framework_lower,
            &tags_lower,
        );

        // Domain (1.0) + Lang (0.8) + Framework (0.5) = 2.3
        assert_eq!(score, 2.3);

        // Mismatch
        let score_mismatch =
            memory.calculate_heuristic_relevance(&heuristic, "data", Some("python"), None, &vec![]);
        assert_eq!(score_mismatch, 0.1); // Baseline
    }
}
