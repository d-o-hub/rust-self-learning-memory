//! Tests for gist extraction and hierarchical reranking.

use std::sync::Arc;

use super::*;
use crate::TaskContext;
use crate::episode::Episode;
use crate::types::TaskType;

#[test]
fn test_rerank_config_default() {
    let config = RerankConfig::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.max_key_points, 3);
    assert!(config.min_density_threshold > 0.0);
}

#[test]
fn test_rerank_config_dense() {
    let config = RerankConfig::dense();
    assert!(config.validate().is_ok());
    assert!(config.density_weight > config.relevance_weight);
}

#[test]
fn test_rerank_config_validation() {
    let invalid = RerankConfig {
        relevance_weight: 0.5,
        density_weight: 0.6,
        gist_query_similarity_weight: 0.0,
        recency_weight: 0.5, // Sum > 1.0
        diversity_lambda: 0.7,
        max_key_points: 3,
        min_density_threshold: 0.3,
        recency_half_life_days: 30.0,
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn test_gist_extractor_default() {
    let extractor = GistExtractor::default();
    assert_eq!(extractor.max_key_points(), 3);
}

#[test]
fn test_gist_extractor_empty() {
    let extractor = GistExtractor::default();
    let gist = extractor.extract("");
    assert!(gist.key_points.is_empty());
    assert_eq!(gist.density, 0.0);
}

#[test]
fn test_gist_extractor_single_sentence() {
    let extractor = GistExtractor::default();
    let gist = extractor.extract("Fixed authentication bug by adding JWT validation.");
    assert!(!gist.key_points.is_empty());
    assert!(gist.density > 0.0);
}

#[test]
fn test_gist_extractor_multiple_sentences() {
    let extractor = GistExtractor::default();
    let gist = extractor
        .extract("Fixed authentication bug. Added JWT validation. Improved error handling.");
    assert!(gist.key_points.len() <= 3);
    assert!(gist.density > 0.0);
}

#[test]
fn test_gist_extractor_high_value_keywords() {
    let extractor = GistExtractor::default();
    let gist = extractor.extract("Fixed critical bug in authentication module.");
    // Should extract this sentence due to "fixed" and "bug" keywords
    assert!(!gist.key_points.is_empty());
}

#[test]
fn test_gist_extractor_positional_bias() {
    // Use max_key_points=2 on 3 sentences to force positional selection
    let extractor = GistExtractor::new(2);
    let text =
        "This is the first sentence. Middle sentence that is neutral. Final conclusion sentence.";
    let gist = extractor.extract(text);

    // With only 2 key points from 3 sentences, first and last should be favored
    assert_eq!(
        gist.key_points.len(),
        2,
        "Should select exactly 2 key points"
    );
    assert!(
        gist.key_points.iter().any(|s| s.contains("first")),
        "Should favor first sentence (positional bias)"
    );
    assert!(
        gist.key_points.iter().any(|s| s.contains("Final")),
        "Should favor last sentence (positional bias)"
    );
}

#[test]
fn test_gist_extractor_cognitive_markers() {
    let extractor = GistExtractor::default();
    // Use sentences of similar length to isolate cognitive marker effect
    let s1 = "The code needs a simple update today."; // No cognitive markers
    let s2 = "I learned and fixed the important bug today."; // Has cognitive + action markers

    let sentences = vec![s1.to_string(), s2.to_string()];
    let scored = extractor.score_sentences(&sentences);

    assert!(
        scored[1].1 > scored[0].1,
        "Cognitive markers should boost score independent of length"
    );
}

#[test]
fn test_episode_gist_compression_ratio() {
    let gist = EpisodeGist {
        episode_id: "test".to_string(),
        key_points: vec!["Fixed bug.".to_string()],
        density: 0.8,
        original_length: 100,
        gist_length: 10,
    };
    assert!((gist.compression_ratio() - 0.1).abs() < 0.01);
}

#[test]
fn test_episode_gist_summary() {
    let gist = EpisodeGist {
        episode_id: "test".to_string(),
        key_points: vec!["Fixed bug.".to_string(), "Added feature.".to_string()],
        density: 0.8,
        original_length: 100,
        gist_length: 20,
    };
    let summary = gist.summary();
    assert!(summary.contains("Fixed bug"));
    assert!(summary.contains("Added feature"));
}

#[test]
fn test_hierarchical_reranker_empty() {
    let reranker = HierarchicalReranker::dense();
    let result = reranker.rerank(Vec::new(), 5);
    assert!(result.is_empty());
}

#[test]
fn test_hierarchical_reranker_single() {
    let reranker = HierarchicalReranker::default();
    let episode = Arc::new(Episode::new(
        "Fix bug".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let items = vec![(episode, 0.9)];
    let result = reranker.rerank(items, 5);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_hierarchical_reranker_multiple() {
    let reranker = HierarchicalReranker::default();

    let ep1 = Arc::new(Episode::new(
        "Fixed authentication bug in login module".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let ep2 = Arc::new(Episode::new(
        "Added new feature for user profile".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    ));
    let ep3 = Arc::new(Episode::new(
        "Refactored database connection pooling".to_string(),
        TaskContext::default(),
        TaskType::Refactoring,
    ));

    let items = vec![(ep1, 0.9), (ep2, 0.85), (ep3, 0.8)];
    let result = reranker.rerank(items, 2);

    // Should return at most 2 items
    assert!(result.len() <= 2);
    // Should be sorted by combined score
    if result.len() > 1 {
        assert!(result[0].combined_score() >= result[1].combined_score());
    }
}

#[test]
fn test_rerank_config_cognirank() {
    let config = RerankConfig::cognirank();
    assert!(config.validate().is_ok());
    assert!(config.gist_query_similarity_weight > config.relevance_weight);
    assert!(config.gist_query_similarity_weight > config.density_weight);
}

#[test]
fn test_hierarchical_reranker_cognirank() {
    let reranker = HierarchicalReranker::new(RerankConfig::cognirank());

    let ep1 = Arc::new(Episode::new(
        "Fixed authentication bug in login module".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let ep2 = Arc::new(Episode::new(
        "Updated the documentation for the project".to_string(),
        TaskContext::default(),
        TaskType::Documentation,
    ));

    let items = vec![(ep1, 0.9), (ep2, 0.95)]; // ep2 has higher original relevance

    // Query specifically about authentication
    let result = reranker.rerank_with_query(items, "authentication login", 1);

    assert_eq!(result.len(), 1);
    // ep1 should be selected despite lower original relevance because its gist matches the query better
    assert!(
        result[0]
            .episode()
            .task_description
            .contains("authentication")
    );
}

#[test]
fn test_hierarchical_reranker_density_threshold() {
    let config = RerankConfig {
        min_density_threshold: 0.9, // Very high threshold
        ..RerankConfig::default()
    };
    let reranker = HierarchicalReranker::new(config);

    let episode = Arc::new(Episode::new(
        "Some task".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let items = vec![(episode, 0.9)];

    // Should filter out items below density threshold
    let result = reranker.rerank(items, 5);
    // May be empty due to high threshold
    assert!(result.len() <= 1);
}

#[test]
fn test_compute_text_similarity_identical() {
    let reranker = HierarchicalReranker::default();

    let gist1 = EpisodeGist::new("ep1".to_string(), vec!["fixed bug".to_string()], 0.8);
    let gist2 = EpisodeGist::new("ep2".to_string(), vec!["fixed bug".to_string()], 0.8);

    let ep = Arc::new(Episode::new(
        "test".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let item1 = GistScoredItem::new(ep.clone(), gist1, 0.9);
    let item2 = GistScoredItem::new(ep, gist2, 0.9);

    let sim = reranker.compute_text_similarity(&item1, &item2);
    // Identical gists should have high similarity
    assert!(sim > 0.9);
}

#[test]
fn test_compute_text_similarity_different() {
    let reranker = HierarchicalReranker::default();

    let gist1 = EpisodeGist::new(
        "ep1".to_string(),
        vec!["fixed authentication bug".to_string()],
        0.8,
    );
    let gist2 = EpisodeGist::new(
        "ep2".to_string(),
        vec!["added new feature".to_string()],
        0.8,
    );

    let ep = Arc::new(Episode::new(
        "test".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let item1 = GistScoredItem::new(ep.clone(), gist1, 0.9);
    let item2 = GistScoredItem::new(ep, gist2, 0.9);

    let sim = reranker.compute_text_similarity(&item1, &item2);
    // Different gists should have low similarity
    assert!(sim < 0.5);
}

// ========== Codecov coverage: config validation ==========

#[test]
fn test_rerank_config_individual_weight_out_of_range() {
    let config = RerankConfig {
        relevance_weight: -0.1,
        ..RerankConfig::default()
    };
    let err = config.validate().unwrap_err();
    assert!(err.contains("relevance_weight"));
    assert!(err.contains("[0.0, 1.0]"));
}

#[test]
fn test_rerank_config_density_weight_out_of_range() {
    let config = RerankConfig {
        density_weight: 1.5,
        ..RerankConfig::default()
    };
    let err = config.validate().unwrap_err();
    assert!(err.contains("density_weight"));
}

#[test]
fn test_rerank_config_gist_query_weight_out_of_range() {
    let config = RerankConfig {
        gist_query_similarity_weight: 2.0,
        ..RerankConfig::default()
    };
    let err = config.validate().unwrap_err();
    assert!(err.contains("gist_query_similarity_weight"));
}

#[test]
fn test_rerank_config_recency_weight_out_of_range() {
    let config = RerankConfig {
        recency_weight: -0.5,
        ..RerankConfig::default()
    };
    let err = config.validate().unwrap_err();
    assert!(err.contains("recency_weight"));
}

#[test]
fn test_rerank_config_diversity_lambda_out_of_range() {
    let config = RerankConfig {
        diversity_lambda: 1.5,
        ..RerankConfig::default()
    };
    let err = config.validate().unwrap_err();
    assert!(err.contains("diversity_lambda"));
}

#[test]
fn test_rerank_config_max_key_points_zero() {
    let config = RerankConfig {
        max_key_points: 0,
        ..RerankConfig::default()
    };
    let err = config.validate().unwrap_err();
    assert!(err.contains("max_key_points"));
}

#[test]
fn test_rerank_config_all_valid_presets() {
    assert!(RerankConfig::default().validate().is_ok());
    assert!(RerankConfig::dense().validate().is_ok());
    assert!(RerankConfig::comprehensive().validate().is_ok());
    assert!(RerankConfig::cognirank().validate().is_ok());
}

// ========== Codecov coverage: extractor edge cases ==========

#[test]
fn test_gist_extractor_very_short_sentence() {
    // Sentence < 10 chars gets filtered by split_sentences (len >= 5 required)
    let extractor = GistExtractor::default();
    let gist = extractor.extract("Hi.");
    // "Hi" after split is 2 chars, filtered out → empty gist
    assert!(gist.key_points.is_empty());
    assert_eq!(gist.density, 0.0);
}

#[test]
fn test_gist_extractor_below_optimal_length() {
    // Sentence 10-19 chars gets length_score = 0.5
    let extractor = GistExtractor::default();
    let gist = extractor.extract("Short fix.");
    // "Short fix" is 9 chars, passes filter (>=5), gets length_score 0.3 (<10)
    // Actually "Short fix" is 9 chars → len < 10 → length_score 0.3
    assert!(!gist.key_points.is_empty());
    assert!(gist.density > 0.0);
}

#[test]
fn test_gist_extractor_very_long_sentence() {
    // Sentence > 100 chars gets length_score = 0.4
    let long = "This is an extremely long sentence that goes on and on and on and on and on and on and on and on and on and on and on and on beyond one hundred characters.";
    let extractor = GistExtractor::default();
    let gist = extractor.extract(long);
    // Should still extract something, though with penalty
    assert!(gist.key_points.len() <= extractor.max_key_points());
}

#[test]
fn test_gist_extractor_short_sentence() {
    // Sentence 10-19 chars gets length_score = 0.5
    let extractor = GistExtractor::default();
    let gist = extractor.extract("Short text here.");
    assert!(!gist.key_points.is_empty());
}

#[test]
fn test_gist_extractor_all_keyword_types() {
    // Exercises all three keyword categories: action + outcome + cognitive
    let extractor = GistExtractor::default();
    let sentences = vec!["I fixed and learned about the bug issue today.".to_string()];
    let scored = extractor.score_sentences(&sentences);
    // Should have max cognitive score (0.4 + 0.2 + 0.2 + 0.2 = 1.0, capped at 1.0)
    assert!(scored[0].1 >= 0.8);
}

#[test]
fn test_gist_extractor_single_sentence_position() {
    // Single sentence gets position_score = 1.0 (exercises total == 1 branch)
    let extractor = GistExtractor::default();
    let gist =
        extractor.extract("Fixed critical authentication vulnerability in the login module.");
    assert_eq!(gist.key_points.len(), 1);
}
#[test]
fn test_gist_extractor_compression_poor() {
    // Exercise poor compression: when very few sentences pass the filter relative
    // to max_key_points, coverage drops and density decreases.
    let extractor = GistExtractor::new(10);
    let text = "Abcde. Bcdef.";
    let gist = extractor.extract(text);
    // Only 2 sentences pass filter, max_key_points=10 → coverage=0.2 → low density
    assert_eq!(gist.key_points.len(), 2);
    assert!(gist.density >= 0.0 && gist.density <= 1.0);
    // With coverage=2/10=0.2 and moderate compression → density should be ≤ 0.65
    assert!(
        gist.density <= 0.65,
        "Low coverage should yield low density"
    );
}

// ========== Codecov coverage: reranker edge cases ==========

#[test]
fn test_reranker_top_k_zero() {
    let reranker = HierarchicalReranker::default();
    let episode = Arc::new(Episode::new(
        "Fix bug".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let items = vec![(episode, 0.9)];
    let result = reranker.rerank(items, 0);
    assert!(result.is_empty());
}

#[test]
fn test_reranker_all_filtered_by_density() {
    let config = RerankConfig {
        min_density_threshold: 0.99,
        ..RerankConfig::default()
    };
    let reranker = HierarchicalReranker::new(config);

    let episode = Arc::new(Episode::new(
        "short".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let items = vec![(episode, 0.9)];
    let result = reranker.rerank(items, 5);
    assert!(result.is_empty());
}

#[test]
fn test_compute_gist_query_similarity_empty_query() {
    let reranker = HierarchicalReranker::default();
    let gist = EpisodeGist::new(
        "ep1".to_string(),
        vec!["fixed authentication bug".to_string()],
        0.8,
    );
    let ep = Arc::new(Episode::new(
        "test".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let item = GistScoredItem::new(ep, gist, 0.9);
    // Empty query should return 0.0
    let sim = reranker.compute_gist_query_similarity(&item, "");
    assert_eq!(sim, 0.0);
}

#[test]
fn test_compute_gist_query_similarity_short_words() {
    let reranker = HierarchicalReranker::default();
    let gist = EpisodeGist::new("ep1".to_string(), vec!["a b c".to_string()], 0.8);
    let ep = Arc::new(Episode::new(
        "test".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let item = GistScoredItem::new(ep, gist, 0.9);
    // All words < 3 chars are filtered out, so word sets are empty → sim = 0.0
    let sim = reranker.compute_gist_query_similarity(&item, "x y");
    assert_eq!(sim, 0.0);
}

#[test]
fn test_compute_text_similarity_empty_words() {
    let reranker = HierarchicalReranker::default();
    let gist1 = EpisodeGist::new("ep1".to_string(), vec!["a b".to_string()], 0.8);
    let gist2 = EpisodeGist::new("ep2".to_string(), vec!["c d".to_string()], 0.8);
    let ep = Arc::new(Episode::new(
        "test".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let item1 = GistScoredItem::new(ep.clone(), gist1, 0.9);
    let item2 = GistScoredItem::new(ep, gist2, 0.9);
    // All words are < 3 chars, filtered out → sim = 0.0
    let sim = reranker.compute_text_similarity(&item1, &item2);
    assert_eq!(sim, 0.0);
}

#[test]
fn test_compute_gist_query_similarity_case_insensitive() {
    let reranker = HierarchicalReranker::default();
    let gist = EpisodeGist::new(
        "ep1".to_string(),
        vec!["Fixed Authentication Bug".to_string()],
        0.8,
    );
    let ep = Arc::new(Episode::new(
        "test".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let item = GistScoredItem::new(ep, gist, 0.9);
    // Query with different case should still match
    let sim = reranker.compute_gist_query_similarity(&item, "authentication bug");
    assert!(sim > 0.0, "Case-insensitive matching should find overlap");
}

#[test]
fn test_compute_gist_query_similarity_punctuation_stripping() {
    let reranker = HierarchicalReranker::default();
    let gist = EpisodeGist::new(
        "ep1".to_string(),
        vec!["Fixed the authentication bug.".to_string()],
        0.8,
    );
    let ep = Arc::new(Episode::new(
        "test".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    ));
    let item = GistScoredItem::new(ep, gist, 0.9);
    // "bug." should be normalized to "bug" via punctuation stripping
    let sim = reranker.compute_gist_query_similarity(&item, "authentication bug");
    assert!(sim > 0.0, "Punctuation stripping should normalize tokens");
}
