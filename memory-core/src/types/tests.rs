//! Tests for memory-core types.

#![allow(unsafe_code)]

use super::*;
use serial_test::serial;

#[test]
fn test_memory_config_default() {
    let config = MemoryConfig::default();

    // Verify Phase 2 defaults
    assert_eq!(config.max_episodes, None); // Unlimited by default
    assert!(matches!(
        config.eviction_policy,
        Some(crate::episode::EvictionPolicy::RelevanceWeighted)
    ));
    assert!(config.enable_summarization);
    assert_eq!(config.summary_min_length, 100);
    assert_eq!(config.summary_max_length, 200);
}

#[test]
#[serial]
fn test_memory_config_from_env_defaults() {
    // Clear any environment variables that might be set
    // SAFETY: test-only env var manipulation
    unsafe {
        std::env::remove_var("MEMORY_MAX_EPISODES");
        std::env::remove_var("MEMORY_EVICTION_POLICY");
        std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
    }

    let config = MemoryConfig::from_env();

    // Should match defaults
    assert_eq!(config.max_episodes, None);
    assert!(matches!(
        config.eviction_policy,
        Some(crate::episode::EvictionPolicy::RelevanceWeighted)
    ));
    assert!(config.enable_summarization);
}

#[test]
#[serial]
fn test_memory_config_from_env_with_values() {
    // Set environment variables
    // SAFETY: test-only env var manipulation
    unsafe {
        std::env::set_var("MEMORY_MAX_EPISODES", "10000");
        std::env::set_var("MEMORY_EVICTION_POLICY", "LRU");
        std::env::set_var("MEMORY_ENABLE_SUMMARIZATION", "false");
    }

    let config = MemoryConfig::from_env();

    assert_eq!(config.max_episodes, Some(10000));
    assert!(matches!(
        config.eviction_policy,
        Some(crate::episode::EvictionPolicy::LRU)
    ));
    assert!(!config.enable_summarization);

    // Cleanup
    // SAFETY: test-only env var manipulation
    unsafe {
        std::env::remove_var("MEMORY_MAX_EPISODES");
        std::env::remove_var("MEMORY_EVICTION_POLICY");
        std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
    }
}

#[test]
fn test_memory_config_fields() {
    // Test default values
    let config = MemoryConfig::default();
    assert_eq!(config.max_episodes, None);
    assert!(config.enable_summarization);
    assert_eq!(config.summary_min_length, 100);
    assert_eq!(config.summary_max_length, 200);
}

#[test]
fn test_execution_result_variants() {
    let success = ExecutionResult::Success {
        output: "test output".to_string(),
    };
    let error = ExecutionResult::Error {
        message: "test error".to_string(),
    };

    // Verify variants exist and have expected content
    if let ExecutionResult::Success { output } = success {
        assert_eq!(output, "test output");
    }

    if let ExecutionResult::Error { message } = error {
        assert_eq!(message, "test error");
    }
}

#[test]
fn test_task_outcome_variants() {
    let success = TaskOutcome::Success {
        verdict: "completed".to_string(),
        artifacts: vec!["file1.rs".to_string()],
    };

    if let TaskOutcome::Success { verdict, artifacts } = success {
        assert_eq!(verdict, "completed");
        assert_eq!(artifacts, vec!["file1.rs"]);
    }
}

#[test]
fn test_task_context_default() {
    let context = TaskContext::default();
    // TaskContext should have default values
    assert_eq!(context.domain, "general");
    assert_eq!(context.complexity, ComplexityLevel::Moderate);
    assert!(context.tags.is_empty());
    assert!(context.language.is_none());
    assert!(context.framework.is_none());
}

#[test]
fn test_task_type_display() {
    assert_eq!(format!("{}", TaskType::CodeGeneration), "code_generation");
    assert_eq!(format!("{}", TaskType::Debugging), "debugging");
    assert_eq!(format!("{}", TaskType::Refactoring), "refactoring");
}

#[test]
fn test_task_type_from_str() {
    assert_eq!(
        "code_generation".parse::<TaskType>().unwrap(),
        TaskType::CodeGeneration
    );
    assert_eq!(
        "debugging".parse::<TaskType>().unwrap(),
        TaskType::Debugging
    );
    assert!("invalid".parse::<TaskType>().is_err());
}

#[test]
fn test_memory_config_concurrency_defaults() {
    let config = MemoryConfig::default();
    assert_eq!(config.concurrency.max_concurrent_cache_ops, 10);
}

#[test]
fn test_storage_config_validation() {
    let storage = StorageConfig::default();

    // Verify default values
    assert_eq!(storage.max_episodes_cache, 1000);
    assert_eq!(storage.sync_interval_secs, 300);
    assert!(storage.enable_compression);
}

#[test]
fn test_storage_config_custom_values() {
    let storage = StorageConfig {
        max_episodes_cache: 5000,
        sync_interval_secs: 60,
        enable_compression: true,
    };

    assert_eq!(storage.max_episodes_cache, 5000);
    assert_eq!(storage.sync_interval_secs, 60);
    assert!(storage.enable_compression);
}

#[test]
fn test_memory_config_eviction_policy_variants() {
    let test_cases = vec![
        ("lru", crate::episode::EvictionPolicy::LRU),
        ("LRU", crate::episode::EvictionPolicy::LRU),
        (
            "relevanceweighted",
            crate::episode::EvictionPolicy::RelevanceWeighted,
        ),
        (
            "relevance_weighted",
            crate::episode::EvictionPolicy::RelevanceWeighted,
        ),
        (
            "relevance-weighted",
            crate::episode::EvictionPolicy::RelevanceWeighted,
        ),
        (
            "RelevanceWeighted",
            crate::episode::EvictionPolicy::RelevanceWeighted,
        ),
    ];

    for (input, expected) in test_cases {
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("MEMORY_EVICTION_POLICY", input);
        }
        let config = MemoryConfig::from_env();
        assert!(
            matches!(config.eviction_policy, Some(policy) if policy == expected),
            "Failed for input: {input}"
        );
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::remove_var("MEMORY_EVICTION_POLICY");
        }
    }
}

#[test]
fn test_memory_config_invalid_eviction_policy() {
    // Clear any existing value first to avoid pollution from other tests
    // SAFETY: test-only env var manipulation
    unsafe {
        std::env::remove_var("MEMORY_EVICTION_POLICY");
        std::env::set_var("MEMORY_EVICTION_POLICY", "invalid_policy");
    }
    let config = MemoryConfig::from_env();

    // Should fall back to default (RelevanceWeighted)
    assert!(
        matches!(
            config.eviction_policy,
            Some(crate::episode::EvictionPolicy::RelevanceWeighted)
        ),
        "Expected RelevanceWeighted fallback, got {:?}",
        config.eviction_policy
    );

    // SAFETY: test-only env var manipulation
    unsafe {
        std::env::remove_var("MEMORY_EVICTION_POLICY");
    }
}

#[test]
fn test_memory_config_summarization_boolean_variants() {
    let true_cases = vec!["true", "TRUE", "1", "yes", "YES", "on", "ON"];
    let false_cases = vec!["false", "FALSE", "0", "no", "NO", "off", "OFF"];

    for input in true_cases {
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("MEMORY_ENABLE_SUMMARIZATION", input);
        }
        let config = MemoryConfig::from_env();
        assert!(config.enable_summarization, "Failed for input: {input}");
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
        }
    }

    for input in false_cases {
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("MEMORY_ENABLE_SUMMARIZATION", input);
        }
        let config = MemoryConfig::from_env();
        assert!(!config.enable_summarization, "Failed for input: {input}");
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
        }
    }
}

#[test]
fn test_memory_config_max_episodes_parsing() {
    // Valid number
    // SAFETY: test-only env var manipulation
    unsafe {
        std::env::set_var("MEMORY_MAX_EPISODES", "5000");
    }
    let config = MemoryConfig::from_env();
    assert_eq!(config.max_episodes, Some(5000));
    // SAFETY: test-only env var manipulation
    unsafe {
        std::env::remove_var("MEMORY_MAX_EPISODES");
    }

    // Invalid number - should fall back to None
    // SAFETY: test-only env var manipulation
    unsafe {
        std::env::set_var("MEMORY_MAX_EPISODES", "not_a_number");
    }
    let config = MemoryConfig::from_env();
    assert_eq!(config.max_episodes, None);
    // SAFETY: test-only env var manipulation
    unsafe {
        std::env::remove_var("MEMORY_MAX_EPISODES");
    }
}

#[test]
fn test_dual_reward_score_spawn_new_cluster() {
    let overlap_threshold = DualRewardScore::DEFAULT_OVERLAP_THRESHOLD;
    let spawn_threshold = DualRewardScore::DEFAULT_SPAWN_THRESHOLD;

    // Case 1: Novelty exactly at threshold - should be false (> is used)
    let score = DualRewardScore {
        stability_score: 0.1,
        novelty_score: spawn_threshold,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!(!score.should_spawn_new_cluster(overlap_threshold));

    // Case 2: Novelty just above threshold and low stability - should be true
    let score = DualRewardScore {
        stability_score: 0.1,
        novelty_score: spawn_threshold + 0.01,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!(score.should_spawn_new_cluster(overlap_threshold));

    // Case 3: High novelty but stability exactly at overlap_threshold - should be false (< is used)
    let score = DualRewardScore {
        stability_score: overlap_threshold,
        novelty_score: 0.8,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!(!score.should_spawn_new_cluster(overlap_threshold));

    // Case 4: High novelty and stability just below overlap_threshold - should be true
    let score = DualRewardScore {
        stability_score: overlap_threshold - 0.01,
        novelty_score: 0.8,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!(score.should_spawn_new_cluster(overlap_threshold));
}

#[test]
fn test_dual_reward_score_merge() {
    let merge_threshold = DualRewardScore::DEFAULT_MERGE_THRESHOLD;

    // Case 1: Stability exactly at threshold - should be false (> is used)
    let score = DualRewardScore {
        stability_score: merge_threshold,
        novelty_score: 0.1,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!(!score.should_merge());

    // Case 2: Stability just above threshold - should be true
    let score = DualRewardScore {
        stability_score: merge_threshold + 0.01,
        novelty_score: 0.1,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!(score.should_merge());
}

#[test]
fn test_dual_reward_score_uncertain() {
    // Neither merge nor spawn
    let score = DualRewardScore {
        stability_score: 0.5,
        novelty_score: 0.5,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!(score.is_uncertain());

    // Is merge -> not uncertain
    let score = DualRewardScore {
        stability_score: 0.9,
        novelty_score: 0.1,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!(!score.is_uncertain());

    // Is spawn -> not uncertain
    let score = DualRewardScore {
        stability_score: 0.1,
        novelty_score: 0.8,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!(!score.is_uncertain());
}

#[test]
fn test_dual_reward_score_balance_ratio() {
    let score = DualRewardScore {
        stability_score: 0.7,
        novelty_score: 0.3,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!((score.balance_ratio() - 0.4).abs() < f32::EPSILON);

    let score = DualRewardScore {
        stability_score: 0.2,
        novelty_score: 0.8,
        effectiveness_score: 1.0,
        raw_reward: 1.0,
        normalized_reward: 1.0,
        decayed_reward: 1.0,
        effective_reward: 1.0,
    };
    assert!((score.balance_ratio() - (-0.6)).abs() < f32::EPSILON);
}

#[test]
fn test_dual_reward_score_from_similarity() {
    // Normal case
    let score = DualRewardScore::from_similarity(0.7, 1.5);
    assert!((score.stability_score - 0.7).abs() < f32::EPSILON);
    assert!((score.novelty_score - 0.3).abs() < f32::EPSILON);
    assert_eq!(score.effectiveness_score, 1.5);

    // Clamping low
    let score = DualRewardScore::from_similarity(-0.1, 1.0);
    assert_eq!(score.stability_score, 0.0);
    assert_eq!(score.novelty_score, 1.0);

    // Clamping high
    let score = DualRewardScore::from_similarity(1.1, 1.0);
    assert_eq!(score.stability_score, 1.0);
    assert_eq!(score.novelty_score, 0.0);
}

#[test]
fn test_reward_score_default() {
    let score = RewardScore::default();
    assert_eq!(score.total, 0.0);
    assert_eq!(score.base, 0.0);
    assert_eq!(score.efficiency, 1.0);
    assert_eq!(score.complexity_bonus, 1.0);
    assert_eq!(score.quality_multiplier, 1.0);
    assert_eq!(score.learning_bonus, 0.0);
    assert_eq!(score.abstention_score, 0.0);
    assert_eq!(score.raw_reward, 0.0);
    assert_eq!(score.normalized_reward, 0.0);
    assert_eq!(score.decayed_reward, 0.0);
    assert_eq!(score.effective_reward, 0.0);
}
