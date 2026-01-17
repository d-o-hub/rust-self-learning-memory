//! Pattern search scoring functions

use crate::embeddings::SemanticService;
use crate::pattern::{Pattern, PatternEffectiveness};
use crate::types::TaskContext;
use crate::Result;
use chrono::Utc;
use std::collections::HashSet;
use std::sync::Arc;

use super::types::{ScoreBreakdown, SearchConfig};

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    // Normalize from [-1, 1] to [0, 1] range
    let similarity = dot_product / (magnitude_a * magnitude_b);
    (similarity + 1.0) / 2.0
}

/// Calculate comprehensive score for a pattern
pub async fn calculate_pattern_score(
    query_embedding: &[f32],
    pattern: &Pattern,
    context: &TaskContext,
    semantic_service: Option<&Arc<SemanticService>>,
    config: &SearchConfig,
) -> Result<ScoreBreakdown> {
    // 1. Semantic similarity
    let semantic_similarity = if query_embedding.is_empty() {
        // Fallback: keyword-based similarity
        calculate_keyword_similarity(pattern, context)
    } else if let Some(service) = semantic_service {
        // Generate embedding for pattern
        let pattern_embedding = service.embed_pattern(pattern).await?;
        cosine_similarity(query_embedding, &pattern_embedding)
    } else {
        0.5 // Neutral if no service
    };

    // 2. Context match
    let context_match = calculate_context_match(pattern, context);

    // 3. Effectiveness score
    let effectiveness = pattern.effectiveness().effectiveness_score();

    // 4. Recency score
    let recency = calculate_recency_score(pattern.effectiveness());

    // 5. Success rate
    let success_rate = pattern.success_rate();

    Ok(ScoreBreakdown {
        semantic_similarity,
        context_match,
        effectiveness,
        recency,
        success_rate,
    })
}

/// Combine individual scores into overall relevance score
pub fn combine_scores(breakdown: &ScoreBreakdown, config: &SearchConfig) -> f32 {
    breakdown.semantic_similarity * config.semantic_weight
        + breakdown.context_match * config.context_weight
        + breakdown.effectiveness * config.effectiveness_weight
        + breakdown.recency * config.recency_weight
        + breakdown.success_rate * config.success_weight
}

/// Calculate context match score (domain, task type, tags)
pub fn calculate_context_match(pattern: &Pattern, query_context: &TaskContext) -> f32 {
    let Some(pattern_context) = pattern.context() else {
        return 0.3; // Neutral for patterns without context
    };

    let mut score = 0.0;
    let mut components = 0;

    // Domain match
    if pattern_context.domain == query_context.domain {
        score += 1.0;
    }
    components += 1;

    // Tag overlap (Jaccard similarity)
    let pattern_tags: HashSet<_> = pattern_context.tags.iter().collect();
    let query_tags: HashSet<_> = query_context.tags.iter().collect();

    if !pattern_tags.is_empty() || !query_tags.is_empty() {
        let intersection = pattern_tags.intersection(&query_tags).count();
        let union = pattern_tags.union(&query_tags).count();
        if union > 0 {
            score += intersection as f32 / union as f32;
            components += 1;
        }
    }

    if components > 0 {
        score / components as f32
    } else {
        0.5 // Neutral
    }
}

/// Calculate recency score based on last usage
pub fn calculate_recency_score(effectiveness: &PatternEffectiveness) -> f32 {
    let now = Utc::now();
    let last_used = effectiveness.last_used;
    let age_days = (now - last_used).num_days() as f32;

    // Exponential decay: score = e^(-age/30)
    // Patterns used recently (< 7 days) score highly
    // Patterns older than 90 days score near 0
    (-age_days / 30.0).exp()
}

/// Fallback keyword-based similarity when embeddings unavailable
pub fn calculate_keyword_similarity(_pattern: &Pattern, _context: &TaskContext) -> f32 {
    // Simple fallback - could be enhanced with TF-IDF or BM25
    0.5 // Neutral score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::PatternEffectiveness;
    use crate::types::{ComplexityLevel, TaskContext};
    use uuid::Uuid;

    fn create_test_pattern(domain: &str, success_rate: f32) -> Pattern {
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: TaskContext {
                domain: domain.to_string(),
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                tags: vec!["rust".to_string()],
            },
            success_rate,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        }
    }

    #[test]
    fn test_calculate_context_match() {
        let pattern = create_test_pattern("web-api", 0.9);
        let context = TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: vec!["rust".to_string()],
        };

        let score = calculate_context_match(&pattern, &context);
        assert!(score > 0.5); // Domain + tag match
    }

    #[test]
    fn test_calculate_recency_score() {
        let mut effectiveness = PatternEffectiveness::new();
        effectiveness.last_used = Utc::now();

        let score = calculate_recency_score(&effectiveness);
        assert!(score > 0.9); // Recently used

        // Test old pattern
        effectiveness.last_used = Utc::now() - chrono::Duration::days(90);
        let old_score = calculate_recency_score(&effectiveness);
        assert!(old_score < 0.1); // Very old
    }

    #[test]
    fn test_combine_scores() {
        let breakdown = ScoreBreakdown {
            semantic_similarity: 0.8,
            context_match: 0.9,
            effectiveness: 0.7,
            recency: 0.6,
            success_rate: 0.85,
        };

        let config = SearchConfig::default();
        let score = combine_scores(&breakdown, &config);

        // Should be weighted average
        assert!(score > 0.6 && score < 0.9);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);

        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert_eq!(cosine_similarity(&a, &b), 0.5); // (0 + 0) / (1 * 1) = 0, then normalized to 0.5
    }

    #[test]
    fn test_calculate_keyword_similarity() {
        let pattern = create_test_pattern("web-api", 0.9);
        let context = TaskContext {
            domain: "cli".to_string(),
            language: None,
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: vec![],
        };

        let score = calculate_keyword_similarity(&pattern, &context);
        assert_eq!(score, 0.5); // Neutral fallback
    }
}
