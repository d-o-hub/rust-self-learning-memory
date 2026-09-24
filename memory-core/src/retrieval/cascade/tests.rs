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
    assert!(retriever.judge().is_none());
}

#[test]
fn test_cascade_retriever_with_judge() {
    use crate::retrieval::judgment::{
        CandidateJudgment, JudgmentCandidate, JudgmentError, RetrievalJudge,
    };
    use std::sync::Arc;

    struct DummyJudge;
    impl RetrievalJudge for DummyJudge {
        fn judge_candidates(
            &self,
            _query: &str,
            _candidates: &[JudgmentCandidate<'_>],
        ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
            Ok(vec![])
        }
    }

    let config = CascadeConfig::default();
    let retriever = CascadeRetriever::new(config).with_judge(Arc::new(DummyJudge));
    assert!(retriever.judge().is_some());
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

#[cfg(not(feature = "csm"))]
#[test]
fn test_retrieve_unavailable_without_csm() {
    let retriever = CascadeRetriever::default_config();
    // Without CSM feature, cascade retrieval is unavailable
    assert!(matches!(
        retriever.retrieve("test query"),
        Err(CascadeError::CapabilityUnavailable)
    ));
}

#[cfg(not(feature = "csm"))]
#[test]
fn test_non_csm_retriever_supports_episodes_and_reports_unavailable() {
    let mut retriever = CascadeRetriever::default_config();
    retriever.add_episode("ep-1", "Test episode");
    assert_eq!(retriever.len(), 1);

    let result = retriever.retrieve("test query");
    assert!(matches!(result, Err(CascadeError::CapabilityUnavailable)));
}

#[test]
fn test_estimate_api_call_probability() {
    let retriever = CascadeRetriever::new(CascadeConfig::default());
    let prob = retriever.estimate_api_call_probability("test");
    assert!((0.0..=1.0).contains(&prob));
}

#[test]
fn test_estimate_api_mid_length_query_probability() {
    let retriever = CascadeRetriever::new(CascadeConfig::default());
    // 20-50 char query exercises the second length-factor branch
    let prob = retriever.estimate_api_call_probability("how do database connection pools behave");
    assert!(
        (0.2..=0.6).contains(&prob),
        "mid-length query should land mid-range: {prob}"
    );
}

#[test]
fn test_estimate_api_short_query_low_probability() {
    let retriever = CascadeRetriever::new(CascadeConfig::default());
    // Short keyword query — should favor BM25, low API probability
    let prob = retriever.estimate_api_call_probability("rust error");
    assert!(
        prob < 0.3,
        "short keyword query should have low prob: {prob}"
    );
}

#[test]
fn test_estimate_api_long_query_high_probability() {
    let retriever = CascadeRetriever::new(CascadeConfig::default());
    // 100+ char query exercises the top length-factor branch
    let prob = retriever.estimate_api_call_probability(
        "explain in exhaustive detail the theoretical foundations of memory-augmented neural network architectures and their retrieval tradeoffs",
    );
    assert!(
        prob > 0.6,
        "very long abstract query should have high prob: {prob}"
    );
}

#[test]
fn test_estimate_api_long_query_higher_probability() {
    let retriever = CascadeRetriever::new(CascadeConfig::default());
    // Long abstract query — more likely needs semantic embedding
    let prob = retriever.estimate_api_call_probability(
        "explain the theoretical foundations of memory-augmented neural networks in detail",
    );
    assert!(
        prob > 0.3,
        "long abstract query should have higher prob: {prob}"
    );
}

#[test]
fn test_estimate_api_code_tokens_lower_probability() {
    let retriever = CascadeRetriever::new(CascadeConfig::default());
    // Code tokens (with :: and _) are BM25-friendly — should reduce probability vs pure prose
    let prose_prob = retriever.estimate_api_call_probability(
        "explain the theoretical foundations of memory augmented neural networks in detail",
    );
    let code_prob = retriever
        .estimate_api_call_probability("fix memory::storage::cache::wrapper::CacheStats::hit_rate");
    // Code tokens should reduce probability compared to same-length prose
    assert!(
        code_prob <= prose_prob,
        "code tokens should not increase prob: code={code_prob} prose={prose_prob}"
    );
}

#[test]
fn test_estimate_api_empty_query() {
    let retriever = CascadeRetriever::new(CascadeConfig::default());
    let prob = retriever.estimate_api_call_probability("");
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
        fallback_policy: FallbackPolicy::LocalOnly,
        local_confidence_threshold: 0.9,
        minimum_score_margin: 0.1,
    };
    let retriever = CascadeRetriever::new(config);
    assert_eq!(retriever.config().top_k, 5);
    assert!(!retriever.config().merge_results);
    assert!(!retriever.config().enable_concept_expansion);
    assert_eq!(
        retriever.config().fallback_policy,
        FallbackPolicy::LocalOnly
    );
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

/// Covers the ungated `with_semantic_rerank` constructor, so it must run
/// without the `csm` feature (the coverage job builds default features).
#[test]
fn test_with_semantic_rerank_rejects_invalid_config() {
    use crate::retrieval::rerank::{RerankConfigError, SemanticRerankConfig};

    let invalid = SemanticRerankConfig {
        enabled: true,
        shortlist_k: 4,
        output_k: 5,
        ..SemanticRerankConfig::default()
    };

    let outcome = CascadeRetriever::default_config().with_semantic_rerank(invalid);

    assert!(
        matches!(outcome, Err(RerankConfigError::InvalidOutputK { .. })),
        "output_k > shortlist_k must be rejected up front"
    );
}

#[test]
fn test_with_semantic_rerank_stores_normalized_weights() {
    use crate::retrieval::rerank::SemanticRerankConfig;

    let retriever = CascadeRetriever::default_config()
        .with_semantic_rerank(SemanticRerankConfig {
            enabled: true,
            local_weight: 2.0,
            semantic_weight: 6.0,
            ..SemanticRerankConfig::default()
        })
        .expect("weights are normalized rather than rejected");

    let config = retriever.semantic_rerank_config();
    assert!(config.enabled);
    assert!((config.local_weight - 0.25).abs() < f32::EPSILON);
    assert!((config.semantic_weight - 0.75).abs() < f32::EPSILON);
}

/// Tests for CSM-enabled cascade behavior.
#[cfg(feature = "csm")]
mod csm_tests {
    use super::*;

    #[test]
    fn test_retrieve_returns_ok_with_csm() {
        let mut retriever = CascadeRetriever::default_config();
        retriever.add_episode("ep-1", "authentication JWT token implementation");
        retriever.add_episode("ep-2", "authentication session management");
        retriever.add_episode("ep-3", "authentication refresh token handling");

        let result = retriever.retrieve("authentication JWT");

        assert!(result.is_ok());
        let result = result.expect("csm retrieve should succeed");
        assert!(!result.episode_ids.is_empty());
        assert_eq!(result.api_calls, 0);
    }

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
        let result = retriever
            .retrieve("authentication JWT token")
            .expect("csm retrieve should succeed");

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
            .expect("csm retrieve should succeed");

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
        let result = retriever
            .retrieve("any query")
            .expect("csm retrieve should succeed");

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
        let result = retriever
            .retrieve("unique_keyword_alpha")
            .expect("csm retrieve should succeed");
        assert!(result.contributing_tiers.contains(&"bm25".to_string()));

        // Query with no exact match but similar content should use HDC
        let result = retriever
            .retrieve("implement alpha feature")
            .expect("csm retrieve should succeed");
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
            fallback_policy: FallbackPolicy::Adaptive,
            local_confidence_threshold: 0.78,
            minimum_score_margin: 0.08,
        };
        let mut retriever = CascadeRetriever::new(config);

        // Add several episodes
        retriever.add_episode("ep-1", "Rust async programming patterns");
        retriever.add_episode("ep-2", "Tokio runtime configuration best practices");
        retriever.add_episode("ep-3", "async error handling strategies");
        retriever.add_episode("ep-4", "Rust concurrent programming guide");
        retriever.add_episode("ep-5", "async task spawning performance tips");

        // Query that matches multiple aspects
        let result = retriever
            .retrieve("Rust async programming")
            .expect("csm retrieve should succeed");

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
            fallback_policy: FallbackPolicy::Adaptive,
            local_confidence_threshold: 0.78,
            minimum_score_margin: 0.08,
        };
        let mut retriever = CascadeRetriever::new(config);

        retriever.add_episode("ep-1", "authentication implementation");
        retriever.add_episode("ep-2", "database connection setup");

        let result = retriever
            .retrieve("authentication")
            .expect("csm retrieve should succeed");

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

        let result = retriever
            .retrieve("authentication")
            .expect("csm retrieve should succeed");

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

        let result = retriever
            .retrieve("episode")
            .expect("csm retrieve should succeed");

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

    // ── ConceptGraph end-to-end cascade integration test ──
    // Unit tests for ConceptGraph are in concept_graph.rs

    #[test]
    fn test_concept_graph_e2e_cascade_tier3() {
        let config = CascadeConfig {
            top_k: 5,
            bm25_threshold: 0.5,
            hdc_threshold: 0.6,
            concept_graph_threshold: 0.1,
            merge_results: true,
            min_results: 1,
            enable_concept_expansion: true,
            fallback_policy: FallbackPolicy::Adaptive,
            local_confidence_threshold: 0.78,
            minimum_score_margin: 0.08,
        };
        let mut retriever = CascadeRetriever::new(config);

        // Add episodes that use domain-specific terminology
        retriever.add_episode("ep-1", "implement JWT authentication flow");
        retriever.add_episode("ep-2", "fix database connection pool timeout");
        retriever.add_episode("ep-3", "add rate limiting middleware");
        retriever.add_episode("ep-4", "optimize build artifact caching");
        retriever.add_episode("ep-5", "refactor error handling patterns");

        // Query with abbreviated terms that need expansion
        let result = retriever
            .retrieve("fix auth bug")
            .expect("csm retrieve should succeed");

        // Should find results via concept graph expansion ("auth" → authentication domain)
        assert!(!result.episode_ids.is_empty());
        // Should include the concept_graph tier or bm25 tier
        assert!(!result.contributing_tiers.is_empty());
        // Should have 0 API calls (CPU-local tier satisfied the query)
        assert_eq!(result.api_calls, 0);
    }

    #[test]
    fn test_adaptive_rescues_confident_but_few_local_results() {
        // One strong match with default min_results=3: the count-based tier
        // rules cannot suffice, so the pre-policy code always reported a
        // Tier 4 call here. Adaptive confidence must rescue it.
        let mut retriever = CascadeRetriever::default_config();
        retriever.add_episode("ep-1", "authentication JWT token implementation");

        let result = retriever
            .retrieve("authentication JWT token")
            .expect("csm retrieve should succeed");

        assert!(!result.episode_ids.is_empty());
        assert_eq!(result.api_calls, 0);
        assert_eq!(result.fallback_reason, FallbackReason::LocalConfident);
        assert!(result.top_score >= 0.78);
    }

    #[test]
    fn test_adaptive_empty_index_reports_no_local_results() {
        let retriever = CascadeRetriever::default_config();

        let result = retriever
            .retrieve("any query")
            .expect("csm retrieve should succeed");

        assert_eq!(result.api_calls, 1);
        assert_eq!(result.fallback_reason, FallbackReason::NoLocalResults);
        assert_eq!(result.contributing_tiers, vec!["none".to_string()]);
    }

    #[test]
    fn test_always_embed_counts_call_on_local_hit() {
        let config = CascadeConfig {
            fallback_policy: FallbackPolicy::AlwaysEmbed,
            ..CascadeConfig::default()
        };
        let mut retriever = CascadeRetriever::new(config);
        retriever.add_episode("ep-1", "authentication JWT token implementation");
        retriever.add_episode("ep-2", "authentication session management");
        retriever.add_episode("ep-3", "authentication refresh token handling");

        let result = retriever
            .retrieve("authentication JWT")
            .expect("csm retrieve should succeed");

        // Local hit is still returned, but the baseline policy counts Tier 4.
        assert!(!result.episode_ids.is_empty());
        assert_eq!(result.api_calls, 1);
        assert_eq!(result.fallback_reason, FallbackReason::AlwaysEmbedPolicy);
    }

    #[test]
    fn test_local_only_suppresses_call_on_empty_index() {
        let config = CascadeConfig {
            fallback_policy: FallbackPolicy::LocalOnly,
            ..CascadeConfig::default()
        };
        let retriever = CascadeRetriever::new(config);

        let result = retriever
            .retrieve("any query")
            .expect("csm retrieve should succeed");

        assert_eq!(result.api_calls, 0);
        assert_eq!(result.fallback_reason, FallbackReason::LocalOnlyPolicy);
        assert_eq!(result.contributing_tiers, vec!["none".to_string()]);
    }

    // ── Semantic rerank integration (issue #1031) ──

    use crate::monitoring::metrics::global_retrieval_metrics;
    use crate::retrieval::judgment::{
        AtomicScore, CandidateJudgment, JudgmentCandidate, JudgmentError, RetrievalJudge,
    };
    use crate::retrieval::rerank::SemanticRerankConfig;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Deterministic fake judge: counts provider calls, promotes one candidate
    /// id to maximum relevance, and can fail like an unavailable provider.
    struct RecordingJudge {
        calls: Arc<AtomicUsize>,
        promote: String,
        fail: bool,
    }

    impl RetrievalJudge for RecordingJudge {
        fn judge_candidates(
            &self,
            _query: &str,
            candidates: &[JudgmentCandidate<'_>],
        ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                return Err(JudgmentError::Unavailable);
            }
            Ok(candidates
                .iter()
                .map(|candidate| CandidateJudgment {
                    id: candidate.id.to_string(),
                    relevance: if candidate.id == self.promote {
                        AtomicScore::new(1.0, 1.0)
                    } else {
                        AtomicScore::new(0.0, 1.0)
                    },
                    useful_evidence: AtomicScore::new(0.5, 1.0),
                    contradiction: AtomicScore::new(0.0, 1.0),
                    instruction_like: AtomicScore::new(0.0, 1.0),
                })
                .collect())
        }
    }

    /// Rerank enabled with a shortlist/output budget that never truncates the
    /// small test corpora (`output_k <= shortlist_k`).
    fn enabled_rerank_config() -> SemanticRerankConfig {
        SemanticRerankConfig {
            enabled: true,
            shortlist_k: 20,
            output_k: 20,
            ..SemanticRerankConfig::default()
        }
    }

    /// BM25-only cascade shape: Tier 1 suffices, so the merged/HDC/ConceptGraph
    /// branches are never reached.
    fn bm25_path_config() -> CascadeConfig {
        CascadeConfig {
            merge_results: false,
            enable_concept_expansion: false,
            min_results: 1,
            ..CascadeConfig::default()
        }
    }

    /// Three episodes that all match the `AUTH_QUERY` tokens, so Tier 1
    /// returns a shortlist longer than one candidate.
    const AUTH_CORPUS: [(&str, &str); 3] = [
        ("ep-1", "authentication JWT token implementation details"),
        ("ep-2", "authentication JWT validation"),
        ("ep-3", "authentication JWT parsing"),
    ];
    const AUTH_QUERY: &str = "authentication JWT token";

    /// One BM25 match plus two unrelated episodes: the merged path needs both tiers.
    const MERGED_CORPUS: [(&str, &str); 3] = [
        ("ep-1", "authentication JWT token"),
        ("ep-2", "database connection pool timeout"),
        ("ep-3", "rate limiting middleware design"),
    ];

    /// No episode shares a token with the query, so only HDC can answer.
    const SEMANTICLESS_CORPUS: [(&str, &str); 3] = [
        ("ep-1", "database connection pool timeout"),
        ("ep-2", "rate limiting middleware design"),
        ("ep-3", "build artifact caching strategy"),
    ];

    /// Episodes matched through ontology expansion ("auth" -> authentication
    /// domain terms), reachable only in Tier 3.
    const ONTOLOGY_CORPUS: [(&str, &str); 2] = [
        ("ep-1", "implement JWT authentication flow"),
        ("ep-2", "login session handling"),
    ];
    const ONTOLOGY_QUERY: &str = "fix auth bug";

    fn add_corpus(retriever: &mut CascadeRetriever, corpus: &[(&str, &str)]) {
        for (id, text) in corpus {
            retriever.add_episode(id, text);
        }
    }

    /// Local reference run: same cascade config, no judge, rerank disabled.
    fn local_baseline(
        query: &str,
        config: CascadeConfig,
        corpus: &[(&str, &str)],
    ) -> CascadeResult {
        let mut retriever = CascadeRetriever::new(config);
        add_corpus(&mut retriever, corpus);
        retriever
            .retrieve(query)
            .expect("csm retrieve should succeed")
    }

    /// Judge invocation count observed by the fake provider.
    fn judge_calls(calls: &Arc<AtomicUsize>) -> usize {
        calls.load(Ordering::SeqCst)
    }

    /// The candidate that must lead the reranked list: the local trailer is
    /// never the local leader, so a leader change proves reranking applied.
    fn promoted_candidate(local: &CascadeResult) -> String {
        let promoted = local
            .episode_ids
            .last()
            .expect("precondition: the local path returned candidates")
            .clone();
        assert_ne!(
            local.episode_ids.first(),
            Some(&promoted),
            "precondition: the promoted candidate is not the local leader"
        );
        promoted
    }

    /// Rerank-enabled retriever whose judge promotes `promote`.
    fn reranking_retriever(
        config: CascadeConfig,
        calls: &Arc<AtomicUsize>,
        promote: &str,
    ) -> CascadeRetriever {
        CascadeRetriever::new(config)
            .with_judge(Arc::new(RecordingJudge {
                calls: Arc::clone(calls),
                promote: promote.to_string(),
                fail: false,
            }))
            .with_semantic_rerank(enabled_rerank_config())
            .expect("enabled rerank config is valid")
    }

    /// Rerank-enabled retriever whose judge fails like an unavailable provider.
    fn failing_rerank_retriever(
        config: CascadeConfig,
        calls: &Arc<AtomicUsize>,
    ) -> CascadeRetriever {
        CascadeRetriever::new(config)
            .with_judge(Arc::new(RecordingJudge {
                calls: Arc::clone(calls),
                promote: String::new(),
                fail: true,
            }))
            .with_semantic_rerank(enabled_rerank_config())
            .expect("enabled rerank config is valid")
    }

    #[test]
    fn test_semantic_rerank_applies_on_bm25_path() {
        let config = bm25_path_config();
        let local = local_baseline(AUTH_QUERY, config.clone(), &AUTH_CORPUS);
        assert_eq!(
            local.contributing_tiers,
            vec!["bm25".to_string()],
            "precondition: BM25 alone satisfies the query"
        );
        let promoted = promoted_candidate(&local);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut retriever = reranking_retriever(config, &calls, &promoted);
        add_corpus(&mut retriever, &AUTH_CORPUS);

        let reranked = retriever
            .retrieve(AUTH_QUERY)
            .expect("csm retrieve should succeed");

        assert_eq!(reranked.contributing_tiers, vec!["bm25".to_string()]);
        assert_eq!(
            reranked.episode_ids.first(),
            Some(&promoted),
            "the judge's strongest candidate must lead the reranked list"
        );
        assert_ne!(
            reranked.episode_ids.first(),
            local.episode_ids.first(),
            "the BM25 ordering must have been reranked"
        );
        assert_eq!(judge_calls(&calls), 1, "one provider call per retrieval");
    }

    #[test]
    fn test_semantic_rerank_applies_on_merged_bm25_hdc_path() {
        // Default cascade: the single BM25 match cannot reach `min_results`,
        // so BM25 and HDC are merged before finalization.
        let config = CascadeConfig::default();
        let query = "authentication JWT token";
        let local = local_baseline(query, config.clone(), &MERGED_CORPUS);
        assert_eq!(
            local.contributing_tiers,
            vec!["bm25".to_string(), "hdc".to_string()],
            "precondition: the merged BM25+HDC path is used"
        );
        let promoted = promoted_candidate(&local);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut retriever = reranking_retriever(config, &calls, &promoted);
        add_corpus(&mut retriever, &MERGED_CORPUS);

        let reranked = retriever
            .retrieve(query)
            .expect("csm retrieve should succeed");

        assert_eq!(
            reranked.contributing_tiers,
            vec!["bm25".to_string(), "hdc".to_string()]
        );
        assert_eq!(reranked.episode_ids.first(), Some(&promoted));
        assert_ne!(reranked.episode_ids.first(), local.episode_ids.first());
        assert_eq!(judge_calls(&calls), 1, "one provider call per retrieval");
    }

    #[test]
    fn test_semantic_rerank_applies_on_hdc_path() {
        let config = CascadeConfig {
            min_results: 1,
            // Cosine similarity is always >= -1.0, so Tier 2 suffices for any
            // encoding of the indexed episodes.
            hdc_threshold: -1.0,
            ..CascadeConfig::default()
        };
        // No query token occurs in the corpus, so Tier 1 yields nothing.
        let query = "zzzznomatch qqqq";
        let local = local_baseline(query, config.clone(), &SEMANTICLESS_CORPUS);
        assert_eq!(
            local.contributing_tiers,
            vec!["hdc".to_string()],
            "precondition: HDC alone satisfies the query"
        );
        let promoted = promoted_candidate(&local);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut retriever = reranking_retriever(config, &calls, &promoted);
        add_corpus(&mut retriever, &SEMANTICLESS_CORPUS);

        let reranked = retriever
            .retrieve(query)
            .expect("csm retrieve should succeed");

        assert_eq!(reranked.contributing_tiers, vec!["hdc".to_string()]);
        assert_eq!(reranked.episode_ids.first(), Some(&promoted));
        assert_ne!(reranked.episode_ids.first(), local.episode_ids.first());
        assert_eq!(judge_calls(&calls), 1, "one provider call per retrieval");
    }

    #[test]
    fn test_semantic_rerank_applies_on_concept_graph_path() {
        let config = CascadeConfig {
            min_results: 1,
            // HDC can never clear this for two distinct episodes, so Tier 3 decides.
            hdc_threshold: 1.0,
            concept_graph_threshold: 0.02,
            ..CascadeConfig::default()
        };
        let local = local_baseline(ONTOLOGY_QUERY, config.clone(), &ONTOLOGY_CORPUS);
        assert_eq!(
            local.contributing_tiers,
            vec!["concept_graph".to_string()],
            "precondition: ConceptGraph alone satisfies the query"
        );
        let promoted = promoted_candidate(&local);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut retriever = reranking_retriever(config, &calls, &promoted);
        add_corpus(&mut retriever, &ONTOLOGY_CORPUS);

        let reranked = retriever
            .retrieve(ONTOLOGY_QUERY)
            .expect("csm retrieve should succeed");

        assert_eq!(
            reranked.contributing_tiers,
            vec!["concept_graph".to_string()]
        );
        assert_eq!(reranked.episode_ids.first(), Some(&promoted));
        assert_ne!(reranked.episode_ids.first(), local.episode_ids.first());
        assert_eq!(judge_calls(&calls), 1, "one provider call per retrieval");
    }

    #[test]
    fn test_semantic_rerank_applies_once_on_tier4_path() {
        let config = CascadeConfig {
            merge_results: false,
            enable_concept_expansion: false,
            // Unreachable with three episodes, so no tier can suffice.
            min_results: 4,
            ..CascadeConfig::default()
        };
        let query = "authentication JWT";
        let local = local_baseline(query, config.clone(), &AUTH_CORPUS);
        assert!(
            local.episode_ids.len() >= 2,
            "precondition: Tier 4 still holds local candidates to rerank"
        );
        let promoted = promoted_candidate(&local);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut retriever = reranking_retriever(config, &calls, &promoted);
        add_corpus(&mut retriever, &AUTH_CORPUS);

        let reranked = retriever
            .retrieve(query)
            .expect("csm retrieve should succeed");

        assert_eq!(
            judge_calls(&calls),
            1,
            "Tier 4 must rerank the local shortlist exactly once"
        );
        assert_eq!(
            reranked.episode_ids.first(),
            Some(&promoted),
            "tier accounting must see the reranked ordering"
        );
        assert_eq!(
            reranked.top_score, reranked.scores[0],
            "the fallback decision must report the reranked top score"
        );
        assert_eq!(reranked.episode_ids.len(), reranked.scores.len());
    }

    #[test]
    fn test_rerank_disabled_matches_baseline_and_makes_no_provider_call() {
        let config = bm25_path_config();
        let local = local_baseline(AUTH_QUERY, config.clone(), &AUTH_CORPUS);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut retriever = CascadeRetriever::new(config)
            .with_judge(Arc::new(RecordingJudge {
                calls: Arc::clone(&calls),
                promote: "ep-1".to_string(),
                fail: false,
            }))
            .with_semantic_rerank(SemanticRerankConfig::default())
            .expect("the default rerank config is valid");
        add_corpus(&mut retriever, &AUTH_CORPUS);

        assert!(
            !retriever.semantic_rerank_config().enabled,
            "reranking must default to disabled"
        );
        let disabled = retriever
            .retrieve(AUTH_QUERY)
            .expect("csm retrieve should succeed");

        assert_eq!(disabled.episode_ids, local.episode_ids);
        assert_eq!(disabled.scores, local.scores);
        assert_eq!(
            judge_calls(&calls),
            0,
            "disabled reranking performs no provider work"
        );
    }

    #[test]
    fn test_rerank_enabled_without_judge_matches_baseline() {
        let config = bm25_path_config();
        let local = local_baseline(AUTH_QUERY, config.clone(), &AUTH_CORPUS);

        let mut retriever = CascadeRetriever::new(config)
            .with_semantic_rerank(enabled_rerank_config())
            .expect("enabled rerank config is valid");
        assert!(retriever.judge().is_none());
        add_corpus(&mut retriever, &AUTH_CORPUS);

        let unchanged = retriever
            .retrieve(AUTH_QUERY)
            .expect("csm retrieve should succeed");

        assert!(
            retriever.semantic_rerank_config().enabled,
            "precondition: reranking is enabled but no judge is attached"
        );
        assert_eq!(unchanged.episode_ids, local.episode_ids);
        assert_eq!(unchanged.scores, local.scores);
    }

    #[test]
    fn test_rerank_provider_error_preserves_local_results() {
        let config = bm25_path_config();
        let local = local_baseline(AUTH_QUERY, config.clone(), &AUTH_CORPUS);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut retriever = failing_rerank_retriever(config, &calls);
        add_corpus(&mut retriever, &AUTH_CORPUS);

        let degraded = retriever
            .retrieve(AUTH_QUERY)
            .expect("a provider failure must not fail retrieval");

        assert_eq!(degraded.episode_ids, local.episode_ids, "ids preserved");
        assert_eq!(degraded.scores, local.scores, "scores preserved");
        assert_eq!(judge_calls(&calls), 1, "the provider was attempted once");
    }

    #[test]
    fn test_rerank_preserves_always_embed_accounting() {
        let config = CascadeConfig {
            fallback_policy: FallbackPolicy::AlwaysEmbed,
            merge_results: false,
            enable_concept_expansion: false,
            min_results: 1,
            ..CascadeConfig::default()
        };
        let local = local_baseline(AUTH_QUERY, config.clone(), &AUTH_CORPUS);
        let promoted = promoted_candidate(&local);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut retriever = reranking_retriever(config, &calls, &promoted);
        add_corpus(&mut retriever, &AUTH_CORPUS);

        let result = retriever
            .retrieve(AUTH_QUERY)
            .expect("csm retrieve should succeed");

        assert_eq!(
            result.api_calls, 1,
            "AlwaysEmbed still counts the Tier 4 call"
        );
        assert_eq!(result.fallback_reason, FallbackReason::AlwaysEmbedPolicy);
        assert!(result.contributing_tiers.contains(&"api".to_string()));
        assert_eq!(
            result.episode_ids.first(),
            Some(&promoted),
            "reranking still runs on the AlwaysEmbed path"
        );
    }

    #[test]
    fn test_rerank_telemetry_stays_redacted() {
        let query_marker = "secretquerymarker";
        let id_marker = "ep-secretmarker";
        let query = format!("{query_marker} authentication JWT token");

        let calls = Arc::new(AtomicUsize::new(0));
        let mut retriever =
            reranking_retriever(bm25_path_config(), &calls, &format!("{id_marker}-3"));
        for (id, text) in AUTH_CORPUS {
            retriever.add_episode(&format!("{id_marker}-{id}"), text);
        }

        let result = retriever
            .retrieve(&query)
            .expect("csm retrieve should succeed");
        assert_eq!(judge_calls(&calls), 1, "precondition: reranking ran");
        assert!(result.contributing_tiers.contains(&"bm25".to_string()));

        let metrics = global_retrieval_metrics();
        let snapshot = metrics.snapshot().to_string();
        let exposition = metrics.export_prometheus();
        for marker in [query_marker, id_marker] {
            assert!(
                !snapshot.contains(marker),
                "JSON snapshot must not carry {marker}"
            );
            assert!(
                !exposition.contains(marker),
                "Prometheus exposition must not carry {marker}"
            );
        }
        assert!(
            exposition.contains("operation=\"cascade\""),
            "cascade telemetry is recorded while staying bounded"
        );
    }
}
