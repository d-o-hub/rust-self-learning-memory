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
