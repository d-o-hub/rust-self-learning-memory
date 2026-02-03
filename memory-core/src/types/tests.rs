//! Tests for memory-core types.

use super::*;
use serial_test::serial;

#[test]
fn test_memory_config_default() {
    let config = MemoryConfig::default();

    // Verify Phase 2 defaults
    assert_eq!(config.max_episodes, None); // Unlimited by default
    assert!(matches!(
        config.eviction_policy,
        Some(crate::episodic::EvictionPolicy::RelevanceWeighted)
    ));
    assert!(config.enable_summarization);
    assert_eq!(config.summary_min_length, 100);
    assert_eq!(config.summary_max_length, 200);
}

#[test]
#[serial]
fn test_memory_config_from_env_defaults() {
    // Clear any environment variables that might be set
    std::env::remove_var("MEMORY_MAX_EPISODES");
    std::env::remove_var("MEMORY_EVICTION_POLICY");
    std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");

    let config = MemoryConfig::from_env();

    // Should match defaults
    assert_eq!(config.max_episodes, None);
    assert!(matches!(
        config.eviction_policy,
        Some(crate::episodic::EvictionPolicy::RelevanceWeighted)
    ));
    assert!(config.enable_summarization);
}

#[test]
#[serial]
fn test_memory_config_from_env_with_values() {
    // Set environment variables
    std::env::set_var("MEMORY_MAX_EPISODES", "10000");
    std::env::set_var("MEMORY_EVICTION_POLICY", "LRU");
    std::env::set_var("MEMORY_ENABLE_SUMMARIZATION", "false");

    let config = MemoryConfig::from_env();

    assert_eq!(config.max_episodes, Some(10000));
    assert!(matches!(
        config.eviction_policy,
        Some(crate::episodic::EvictionPolicy::LRU)
    ));
    assert!(!config.enable_summarization);

    // Cleanup
    std::env::remove_var("MEMORY_MAX_EPISODES");
    std::env::remove_var("MEMORY_EVICTION_POLICY");
    std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
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
    assert!(!storage.enable_compression);
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
        ("lru", crate::episodic::EvictionPolicy::LRU),
        ("LRU", crate::episodic::EvictionPolicy::LRU),
        (
            "relevanceweighted",
            crate::episodic::EvictionPolicy::RelevanceWeighted,
        ),
        (
            "relevance_weighted",
            crate::episodic::EvictionPolicy::RelevanceWeighted,
        ),
        (
            "relevance-weighted",
            crate::episodic::EvictionPolicy::RelevanceWeighted,
        ),
        (
            "RelevanceWeighted",
            crate::episodic::EvictionPolicy::RelevanceWeighted,
        ),
    ];

    for (input, expected) in test_cases {
        std::env::set_var("MEMORY_EVICTION_POLICY", input);
        let config = MemoryConfig::from_env();
        assert!(
            matches!(config.eviction_policy, Some(policy) if policy == expected),
            "Failed for input: {input}"
        );
        std::env::remove_var("MEMORY_EVICTION_POLICY");
    }
}

#[test]
fn test_memory_config_invalid_eviction_policy() {
    // Clear any existing value first to avoid pollution from other tests
    std::env::remove_var("MEMORY_EVICTION_POLICY");
    std::env::set_var("MEMORY_EVICTION_POLICY", "invalid_policy");
    let config = MemoryConfig::from_env();

    // Should fall back to default (RelevanceWeighted)
    assert!(
        matches!(
            config.eviction_policy,
            Some(crate::episodic::EvictionPolicy::RelevanceWeighted)
        ),
        "Expected RelevanceWeighted fallback, got {:?}",
        config.eviction_policy
    );

    std::env::remove_var("MEMORY_EVICTION_POLICY");
}

#[test]
fn test_memory_config_summarization_boolean_variants() {
    let true_cases = vec!["true", "TRUE", "1", "yes", "YES", "on", "ON"];
    let false_cases = vec!["false", "FALSE", "0", "no", "NO", "off", "OFF"];

    for input in true_cases {
        std::env::set_var("MEMORY_ENABLE_SUMMARIZATION", input);
        let config = MemoryConfig::from_env();
        assert!(config.enable_summarization, "Failed for input: {input}");
        std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
    }

    for input in false_cases {
        std::env::set_var("MEMORY_ENABLE_SUMMARIZATION", input);
        let config = MemoryConfig::from_env();
        assert!(!config.enable_summarization, "Failed for input: {input}");
        std::env::remove_var("MEMORY_ENABLE_SUMMARIZATION");
    }
}

#[test]
fn test_memory_config_max_episodes_parsing() {
    // Valid number
    std::env::set_var("MEMORY_MAX_EPISODES", "5000");
    let config = MemoryConfig::from_env();
    assert_eq!(config.max_episodes, Some(5000));
    std::env::remove_var("MEMORY_MAX_EPISODES");

    // Invalid number - should fall back to None
    std::env::set_var("MEMORY_MAX_EPISODES", "not_a_number");
    let config = MemoryConfig::from_env();
    assert_eq!(config.max_episodes, None);
    std::env::remove_var("MEMORY_MAX_EPISODES");
}
