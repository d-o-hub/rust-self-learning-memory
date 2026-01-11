//! Diversity module tests.

use super::DiversityMaximizer;
use crate::spatiotemporal::diversity::types::ScoredEpisode;

#[allow(dead_code)]
fn create_test_episode(id: &str, relevance: f32, embedding: Vec<f32>) -> ScoredEpisode {
    ScoredEpisode::new(id.to_string(), relevance, embedding)
}

#[test]
fn test_diversity_maximizer_creation() {
    let maximizer = DiversityMaximizer::new(0.7);
    assert_eq!(maximizer.lambda(), 0.7);
}

#[test]
fn test_default_lambda() {
    let maximizer = DiversityMaximizer::default();
    assert_eq!(maximizer.lambda(), 0.7);
}

#[test]
#[should_panic(expected = "Lambda must be in range")]
fn test_invalid_lambda_too_high() {
    let _ = DiversityMaximizer::new(1.5);
}

#[test]
#[should_panic(expected = "Lambda must be in range")]
fn test_invalid_lambda_negative() {
    let _ = DiversityMaximizer::new(-0.1);
}

#[test]
fn test_empty_candidates() {
    let maximizer = DiversityMaximizer::default();
    let result = maximizer.maximize_diversity(vec![], 5);
    assert!(result.is_empty());
}

#[test]
fn test_zero_limit() {
    let maximizer = DiversityMaximizer::default();
    let candidates = vec![create_test_episode("ep1", 0.9, vec![1.0, 0.0])];
    let result = maximizer.maximize_diversity(candidates, 0);
    assert!(result.is_empty());
}

#[test]
fn test_fewer_candidates_than_limit() {
    let maximizer = DiversityMaximizer::default();
    let candidates = vec![
        create_test_episode("ep1", 0.9, vec![1.0, 0.0]),
        create_test_episode("ep2", 0.8, vec![0.0, 1.0]),
    ];
    let result = maximizer.maximize_diversity(candidates.clone(), 5);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_cosine_similarity_identical() {
    let ep1 = create_test_episode("ep1", 0.9, vec![1.0, 0.0, 0.0]);
    let ep2 = create_test_episode("ep2", 0.8, vec![1.0, 0.0, 0.0]);

    let similarity = DiversityMaximizer::calculate_similarity(&ep1, &ep2);
    assert!((similarity - 1.0).abs() < 0.001);
}

#[test]
fn test_cosine_similarity_orthogonal() {
    let ep1 = create_test_episode("ep1", 0.9, vec![1.0, 0.0, 0.0]);
    let ep2 = create_test_episode("ep2", 0.8, vec![0.0, 1.0, 0.0]);

    let similarity = DiversityMaximizer::calculate_similarity(&ep1, &ep2);
    assert!(similarity.abs() < 0.001); // Should be ~0
}

#[test]
fn test_cosine_similarity_partial() {
    let ep1 = create_test_episode("ep1", 0.9, vec![1.0, 0.0]);
    let ep2 = create_test_episode("ep2", 0.8, vec![0.5, 0.866]); // 60 degrees

    let similarity = DiversityMaximizer::calculate_similarity(&ep1, &ep2);
    assert!((similarity - 0.5).abs() < 0.01); // cos(60°) ≈ 0.5
}

#[test]
fn test_cosine_similarity_dimension_mismatch() {
    let ep1 = create_test_episode("ep1", 0.9, vec![1.0, 0.0]);
    let ep2 = create_test_episode("ep2", 0.8, vec![1.0, 0.0, 0.0]);

    let similarity = DiversityMaximizer::calculate_similarity(&ep1, &ep2);
    assert_eq!(similarity, 0.0);
}

#[test]
fn test_cosine_similarity_empty_embeddings() {
    let ep1 = create_test_episode("ep1", 0.9, vec![]);
    let ep2 = create_test_episode("ep2", 0.8, vec![]);

    let similarity = DiversityMaximizer::calculate_similarity(&ep1, &ep2);
    assert_eq!(similarity, 0.0);
}

#[test]
fn test_mmr_pure_relevance() {
    // λ = 1.0: Pure relevance, should select highest scored episodes
    let maximizer = DiversityMaximizer::new(1.0);

    let candidates = vec![
        create_test_episode("ep1", 0.9, vec![1.0, 0.0]),
        create_test_episode("ep2", 0.85, vec![0.9, 0.1]),
        create_test_episode("ep3", 0.8, vec![0.8, 0.2]),
        create_test_episode("ep4", 0.75, vec![0.1, 0.9]),
    ];

    let result = maximizer.maximize_diversity(candidates, 2);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].episode_id(), "ep1"); // Highest relevance
    assert_eq!(result[1].episode_id(), "ep2"); // Second highest
}

#[test]
fn test_mmr_pure_diversity() {
    // λ = 0.0: Pure diversity, should select most different episodes
    let maximizer = DiversityMaximizer::new(0.0);

    let candidates = vec![
        create_test_episode("ep1", 0.9, vec![1.0, 0.0]),
        create_test_episode("ep2", 0.85, vec![0.9, 0.1]), // Similar to ep1
        create_test_episode("ep3", 0.8, vec![0.0, 1.0]),  // Orthogonal to ep1
        create_test_episode("ep4", 0.75, vec![0.1, 0.9]), // Similar to ep3
    ];

    let result = maximizer.maximize_diversity(candidates, 2);
    assert_eq!(result.len(), 2);

    // Should select most different episodes
    let ids: Vec<&str> = result.iter().map(|e| e.episode_id()).collect();
    // ep1 and ep3 are orthogonal, should be selected
    assert!(ids.contains(&"ep1") || ids.contains(&"ep2"));
    assert!(ids.contains(&"ep3") || ids.contains(&"ep4"));
}

#[test]
fn test_mmr_balanced() {
    // λ = 0.7: Balanced relevance and diversity
    let maximizer = DiversityMaximizer::new(0.7);

    let candidates = vec![
        create_test_episode("ep1", 0.9, vec![1.0, 0.0, 0.0]),
        create_test_episode("ep2", 0.85, vec![0.9, 0.1, 0.0]), // Similar to ep1
        create_test_episode("ep3", 0.8, vec![0.0, 1.0, 0.0]),  // Orthogonal to ep1
        create_test_episode("ep4", 0.75, vec![0.0, 0.9, 0.1]), // Similar to ep3
    ];

    let result = maximizer.maximize_diversity(candidates, 2);
    assert_eq!(result.len(), 2);

    // First should be ep1 (highest relevance)
    assert_eq!(result[0].episode_id(), "ep1");

    // Second should be ep3 (good balance of relevance and diversity from ep1)
    assert_eq!(result[1].episode_id(), "ep3");
}

#[test]
fn test_diversity_score_identical_episodes() {
    let maximizer = DiversityMaximizer::default();

    let episodes = vec![
        create_test_episode("ep1", 0.9, vec![1.0, 0.0]),
        create_test_episode("ep2", 0.8, vec![1.0, 0.0]),
        create_test_episode("ep3", 0.7, vec![1.0, 0.0]),
    ];

    let diversity = maximizer.calculate_diversity_score(&episodes);
    assert!(diversity < 0.01); // Should be ~0 (no diversity)
}

#[test]
fn test_diversity_score_orthogonal_episodes() {
    let maximizer = DiversityMaximizer::default();

    let episodes = vec![
        create_test_episode("ep1", 0.9, vec![1.0, 0.0, 0.0]),
        create_test_episode("ep2", 0.8, vec![0.0, 1.0, 0.0]),
        create_test_episode("ep3", 0.7, vec![0.0, 0.0, 1.0]),
    ];

    let diversity = maximizer.calculate_diversity_score(&episodes);
    assert!(diversity > 0.99); // Should be ~1.0 (maximum diversity)
}

#[test]
fn test_diversity_score_single_episode() {
    let maximizer = DiversityMaximizer::default();

    let episodes = vec![create_test_episode("ep1", 0.9, vec![1.0, 0.0])];

    let diversity = maximizer.calculate_diversity_score(&episodes);
    assert_eq!(diversity, 1.0); // Single item is "perfectly diverse"
}

#[test]
fn test_diversity_score_empty() {
    let maximizer = DiversityMaximizer::default();
    let diversity = maximizer.calculate_diversity_score(&[]);
    assert_eq!(diversity, 0.0);
}

#[test]
fn test_diversity_score_target() {
    // Test that MMR achieves ≥0.7 diversity
    let maximizer = DiversityMaximizer::new(0.7);

    let candidates = vec![
        create_test_episode("ep1", 0.9, vec![1.0, 0.0, 0.0]),
        create_test_episode("ep2", 0.88, vec![0.95, 0.05, 0.0]),
        create_test_episode("ep3", 0.86, vec![0.9, 0.1, 0.0]),
        create_test_episode("ep4", 0.8, vec![0.0, 1.0, 0.0]),
        create_test_episode("ep5", 0.78, vec![0.0, 0.95, 0.05]),
        create_test_episode("ep6", 0.75, vec![0.0, 0.0, 1.0]),
    ];

    let result = maximizer.maximize_diversity(candidates, 3);
    let diversity = maximizer.calculate_diversity_score(&result);

    assert!(diversity >= 0.7, "Expected diversity ≥0.7, got {diversity}");
}

#[test]
fn test_mmr_with_various_lambda_values() {
    let test_cases = vec![0.0, 0.3, 0.5, 0.7, 1.0];

    for lambda in test_cases {
        let maximizer = DiversityMaximizer::new(lambda);

        let candidates = vec![
            create_test_episode("ep1", 0.9, vec![1.0, 0.0]),
            create_test_episode("ep2", 0.85, vec![0.9, 0.1]),
            create_test_episode("ep3", 0.8, vec![0.1, 0.9]),
            create_test_episode("ep4", 0.75, vec![0.0, 1.0]),
        ];

        let result = maximizer.maximize_diversity(candidates, 2);
        assert_eq!(result.len(), 2);
    }
}

#[test]
fn test_scored_episode_accessors() {
    let episode = ScoredEpisode::new("test-id".to_string(), 0.85, vec![1.0, 0.5]);

    assert_eq!(episode.episode_id(), "test-id");
    assert_eq!(episode.relevance_score(), 0.85);
    assert_eq!(episode.embedding(), &[1.0, 0.5]);
}
