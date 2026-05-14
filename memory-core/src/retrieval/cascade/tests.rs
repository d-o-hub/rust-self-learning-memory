//! Tests for the cascading retrieval pipeline (WG-131).

use super::*;

#[test]
fn test_cascade_config_default() {
    let config = CascadeConfig::default();
    assert_eq!(config.top_k, 10);
    assert!(config.bm25_threshold > 0.0);
    assert!(config.hdc_threshold > 0.0);
    assert!(config.concept_graph_threshold > 0.0);
    assert!(config.merge_results);
    assert_eq!(config.min_results, 3);
    assert!(config.enable_concept_expansion);
}

#[test]
fn test_cascade_retriever_creation() {
    let config = CascadeConfig::default();
    let retriever = CascadeRetriever::new(config);
    assert_eq!(retriever.config().top_k, 10);
    assert!(retriever.is_empty());
}

#[test]
fn test_default_config_creation() {
    let retriever = CascadeRetriever::default_config();
    assert_eq!(retriever.config().top_k, 10);
}

#[test]
fn test_add_episode() {
    let mut retriever = CascadeRetriever::default_config();
    retriever.add_episode("ep-1", "Implement authentication in Rust using JWT tokens");
    assert_eq!(retriever.len(), 1);
    assert!(!retriever.is_empty());

    retriever.add_episode("ep-2", "Fix bug in database connection pool");
    assert_eq!(retriever.len(), 2);
}

#[test]
fn test_clear_episodes() {
    let mut retriever = CascadeRetriever::default_config();
    retriever.add_episode("ep-1", "Test episode 1");
    retriever.add_episode("ep-2", "Test episode 2");
    assert_eq!(retriever.len(), 2);

    retriever.clear();
    assert_eq!(retriever.len(), 0);
    assert!(retriever.is_empty());
}

#[test]
fn test_placeholder_retrieve_without_csm() {
    #[cfg(not(feature = "csm"))]
    {
        let mut retriever = CascadeRetriever::default_config();
        retriever.add_episode("ep-1", "Test episode");
        let result = retriever.retrieve("test query").unwrap();
        // Without CSM feature, returns empty results
        assert!(result.episode_ids.is_empty());
        assert!(result.scores.is_empty());
        assert_eq!(result.api_calls, 0);
    }
}

#[test]
fn test_estimate_api_call_probability() {
    let retriever = CascadeRetriever::new(CascadeConfig::default());
    let prob = retriever.estimate_api_call_probability("test");
    assert!((0.0..=1.0).contains(&prob));
}

#[test]
fn test_config_custom_values() {
    let config = CascadeConfig {
        top_k: 5,
        bm25_threshold: 0.4,
        hdc_threshold: 0.6,
        concept_graph_threshold: 0.5,
        merge_results: false,
        min_results: 2,
        enable_concept_expansion: false,
    };
    let retriever = CascadeRetriever::new(config);
    assert_eq!(retriever.config().top_k, 5);
    assert!(!retriever.config().merge_results);
    assert!(!retriever.config().enable_concept_expansion);
}

#[test]
fn test_tier_result_helpers() {
    let tier_result = TierResult {
        tier: "test".to_string(),
        results: vec![("id1".to_string(), 0.9), ("id2".to_string(), 0.7)],
        sufficient: true,
    };

    assert_eq!(tier_result.ids(), vec!["id1", "id2"]);
    assert_eq!(tier_result.scores(), vec![0.9, 0.7]);
    assert_eq!(tier_result.len(), 2);
    assert!(!tier_result.is_empty());
}

/// Tests for CSM-enabled cascade behavior.
#[cfg(feature = "csm")]
mod csm_tests {
    use super::*;

    #[test]
    fn test_bm25_exact_match_zero_api_calls() {
        let mut retriever = CascadeRetriever::default_config();

        // Add episodes with distinct keywords
        retriever.add_episode("ep-1", "authentication JWT token Rust implementation");
        retriever.add_episode("ep-2", "database connection pool timeout fix");
        retriever.add_episode("ep-3", "API rate limiting middleware design");
        retriever.add_episode("ep-4", "memory cache optimization performance");
        retriever.add_episode("ep-5", "error handling patterns in async code");

        // Query with exact keyword match should return 0 API calls
        let result = retriever.retrieve("authentication JWT token").unwrap();

        // Should find the matching episode
        assert!(!result.episode_ids.is_empty());
        // Should be from BM25 tier (exact keyword match)
        assert!(result.contributing_tiers.contains(&"bm25".to_string()));
        // Should have 0 API calls (CPU-local retrieval)
        assert_eq!(result.api_calls, 0);
    }

    #[test]
    fn test_hdc_similarity_zero_api_calls() {
        let mut retriever = CascadeRetriever::default_config();

        // Add episodes
        retriever.add_episode(
            "ep-1",
            "implement user login system with secure password storage",
        );
        retriever.add_episode("ep-2", "create authentication flow for web application");
        retriever.add_episode("ep-3", "setup database migration scripts");
        retriever.add_episode("ep-4", "optimize query performance with indexes");
        retriever.add_episode("ep-5", "fix memory leak in cache implementation");

        // Semantic-like query (similar words but not exact match)
        let result = retriever
            .retrieve("user authentication and login security")
            .unwrap();

        // Should find related episodes
        assert!(!result.episode_ids.is_empty());
        // Should be from BM25 or HDC tier (not API)
        assert!(
            result.contributing_tiers.contains(&"bm25".to_string())
                || result.contributing_tiers.contains(&"hdc".to_string())
        );
        // Should have 0 API calls (CPU-local retrieval)
        assert_eq!(result.api_calls, 0);
    }

    #[test]
    fn test_empty_index_returns_api_needed() {
        let retriever = CascadeRetriever::default_config();

        // Empty index should indicate API call needed
        let result = retriever.retrieve("any query").unwrap();

        assert!(result.episode_ids.is_empty());
        // Should indicate API fallback needed
        assert!(result.api_calls > 0);
    }

    #[test]
    fn test_cascade_tier_escalation() {
        let mut retriever = CascadeRetriever::default_config();

        // Add episodes with specific keywords
        retriever.add_episode("ep-1", "unique_keyword_alpha implementation");
        retriever.add_episode("ep-2", "unique_keyword_beta configuration");
        retriever.add_episode("ep-3", "unique_keyword_gamma optimization");

        // Query that matches exactly should hit BM25
        let result = retriever.retrieve("unique_keyword_alpha").unwrap();
        assert!(result.contributing_tiers.contains(&"bm25".to_string()));

        // Query with no exact match but similar content should use HDC
        let result = retriever.retrieve("implement alpha feature").unwrap();
        // Either BM25 (if partial match) or HDC (if semantic similarity)
        assert!(!result.episode_ids.is_empty() || result.api_calls > 0);
    }

    #[test]
    fn test_merged_results_bm25_hdc() {
        let config = CascadeConfig {
            top_k: 10,
            bm25_threshold: 0.2,
            hdc_threshold: 0.3,
            concept_graph_threshold: 0.4,
            merge_results: true,
            min_results: 3,
            enable_concept_expansion: true,
        };
        let mut retriever = CascadeRetriever::new(config);

        // Add several episodes
        retriever.add_episode("ep-1", "Rust async programming patterns");
        retriever.add_episode("ep-2", "Tokio runtime configuration best practices");
        retriever.add_episode("ep-3", "async error handling strategies");
        retriever.add_episode("ep-4", "Rust concurrent programming guide");
        retriever.add_episode("ep-5", "async task spawning performance tips");

        // Query that matches multiple aspects
        let result = retriever.retrieve("Rust async programming").unwrap();

        // Should have results from merging BM25 and HDC
        assert!(!result.episode_ids.is_empty());
        // May have multiple contributing tiers if merge happened
        assert!(!result.contributing_tiers.is_empty());
        // Should be 0 API calls
        assert_eq!(result.api_calls, 0);
    }

    #[test]
    fn test_disable_merge_results() {
        let config = CascadeConfig {
            top_k: 5,
            bm25_threshold: 0.3,
            hdc_threshold: 0.5,
            concept_graph_threshold: 0.4,
            merge_results: false,
            min_results: 1,
            enable_concept_expansion: false,
        };
        let mut retriever = CascadeRetriever::new(config);

        retriever.add_episode("ep-1", "authentication implementation");
        retriever.add_episode("ep-2", "database connection setup");

        let result = retriever.retrieve("authentication").unwrap();

        // Without merge, should only use single tier
        assert!(result.contributing_tiers.len() <= 1);
    }

    #[test]
    fn test_compute_tier_weights_short_query() {
        let weights = compute_tier_weights("short");
        // Short query: favor BM25
        assert!(weights.0 > weights.1); // BM25 weight > HDC weight
    }

    #[test]
    fn test_compute_tier_weights_medium_query() {
        let weights = compute_tier_weights("this is a medium length query with more words");
        // Medium query: balanced
        assert!(weights.0 > 0.3 && weights.1 > 0.3);
    }

    #[test]
    fn test_compute_tier_weights_long_query() {
        let long_query = "this is a very long query that contains many words and should favor semantic matching over keyword matching in the cascade retrieval pipeline";
        let weights = compute_tier_weights(long_query);
        // Long query: favor HDC
        assert!(weights.1 > weights.0); // HDC weight > BM25 weight
    }

    #[test]
    fn test_scores_normalized() {
        let mut retriever = CascadeRetriever::default_config();

        retriever.add_episode("ep-1", "authentication token JWT");
        retriever.add_episode("ep-2", "database pool connection");
        retriever.add_episode("ep-3", "rate limiting API");

        let result = retriever.retrieve("authentication").unwrap();

        // All scores should be in 0.0-1.0 range
        for score in &result.scores {
            assert!((0.0..=1.0).contains(score));
        }
    }

    #[test]
    fn test_top_k_limit() {
        let config = CascadeConfig {
            top_k: 3,
            ..CascadeConfig::default()
        };
        let mut retriever = CascadeRetriever::new(config);

        // Add more episodes than top_k
        for i in 1..=10 {
            retriever.add_episode(&format!("ep-{i}"), &format!("episode {i} content"));
        }

        let result = retriever.retrieve("episode").unwrap();

        // Should not exceed top_k
        assert!(result.episode_ids.len() <= 3);
    }

    #[test]
    fn test_tokenize_works() {
        let tokens = CascadeRetriever::tokenize("Hello World Test");
        // Should tokenize into words
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_hdc_encoder_similarity() {
        // Use the encoder from the retrieval module's re-export
        use crate::retrieval::HdcEncoder;
        let encoder = HdcEncoder::new();
        let vec1 = encoder.encode("authentication login");
        let vec2 = encoder.encode("authentication login");
        let vec3 = encoder.encode("database connection");

        // Same text should have high similarity
        let sim_same = vec1.cosine_similarity(&vec2);
        assert!(sim_same > 0.9);

        // Different text should have lower similarity
        let sim_diff = vec1.cosine_similarity(&vec3);
        assert!(sim_diff < sim_same);
    }
}
