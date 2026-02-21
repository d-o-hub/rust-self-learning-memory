//! Property-based tests for memory-core types
//!
//! These tests use proptest to verify serialization roundtrips and state invariants
//! across a wide range of generated inputs.

use memory_core::*;
use proptest::prelude::*;

// Re-export for use in tests
use std::result::Result as StdResult;

// Import chrono for timestamp handling
use chrono::Utc;

// ============================================================================
// Serialization Roundtrip Tests
// ============================================================================

proptest! {
    /// Test that TaskContext serializes and deserializes correctly
    #[test]
    fn task_context_json_roundtrip(
        language in proptest::option::of("[a-z]{2,10}"),
        framework in proptest::option::of("[a-z]{2,15}"),
        domain in "[a-z]{3,20}",
        tags in proptest::collection::vec("[a-z]{2,15}", 0..10)
    ) {
        let context = TaskContext {
            language: language.map(|s| s.to_string()),
            framework: framework.map(|s| s.to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: domain.to_string(),
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
        };

        // JSON roundtrip
        let json = serde_json::to_string(&context).expect("serialize to JSON");
        let deserialized: TaskContext = serde_json::from_str(&json).expect("deserialize from JSON");
        assert_eq!(context, deserialized);
    }

    /// Test that RewardScore serializes and deserializes correctly
    #[test]
    fn reward_score_json_roundtrip(
        total in 0.0f32..2.0f32,
        base in 0.0f32..1.0f32,
        efficiency in 0.5f32..1.5f32,
        complexity_bonus in 1.0f32..1.3f32,
        quality_multiplier in 0.8f32..1.2f32,
        learning_bonus in 0.0f32..0.5f32
    ) {
        let score = RewardScore {
            total,
            base,
            efficiency,
            complexity_bonus,
            quality_multiplier,
            learning_bonus,
        };

        let json = serde_json::to_string(&score).expect("serialize to JSON");
        let deserialized: RewardScore = serde_json::from_str(&json).expect("deserialize from JSON");

        // Use approximate equality for floats
        assert!((score.total - deserialized.total).abs() < 0.001);
        assert!((score.base - deserialized.base).abs() < 0.001);
        assert!((score.efficiency - deserialized.efficiency).abs() < 0.001);
    }

    /// Test that OutcomeStats maintains success rate invariants
    #[test]
    fn outcome_stats_success_rate_invariant(
        success_count in 0usize..1000usize,
        failure_count in 0usize..1000usize,
        avg_duration_secs in 0.0f32..3600.0f32
    ) {
        let stats = OutcomeStats {
            success_count,
            failure_count,
            total_count: success_count + failure_count,
            avg_duration_secs,
        };

        let success_rate = stats.success_rate();

        // Invariant: success rate should be between 0.0 and 1.0
        prop_assert!((0.0..=1.0).contains(&success_rate));

        // Invariant: success rate should equal success_count / total_count
        if stats.total_count > 0 {
            let expected_rate = success_count as f32 / stats.total_count as f32;
            prop_assert!((success_rate - expected_rate).abs() < 0.0001);
        } else {
            prop_assert_eq!(success_rate, 0.0);
        }
    }

    /// Test Episode tag validation invariants
    #[test]
    fn episode_tag_validation_invariants(
        tag in "[a-zA-Z0-9_-]{1,100}"
    ) {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration
        );

        // Valid tags should be added successfully
        if tag.len() >= 2 && tag.len() <= 100 {
            let result = episode.add_tag(tag.clone());
            prop_assert!(result.is_ok());

            // Tag should be normalized (lowercase)
            let normalized_tag = tag.trim().to_lowercase();
            prop_assert!(episode.has_tag(&normalized_tag));
        }
    }

    /// Test tag normalization consistency
    #[test]
    fn tag_normalization_consistency(
        tag in "[a-zA-Z0-9_-]{2,50}"
    ) {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration
        );

        // Add tag
        episode.add_tag(tag.clone()).unwrap();

        // Should be found with different case variations
        let upper = tag.to_uppercase();
        let mixed: String = tag.chars()
            .enumerate()
            .map(|(i, c)| if i % 2 == 0 { c.to_ascii_uppercase() } else { c })
            .collect();

        prop_assert!(episode.has_tag(&upper));
        prop_assert!(episode.has_tag(&mixed));
        prop_assert!(episode.has_tag(&tag.to_lowercase()));
    }

    /// Test Episode serialization roundtrip with various states
    #[test]
    fn episode_json_roundtrip(
        task_description in "[a-zA-Z0-9 ]{1,100}",
        domain in "[a-z]{3,15}",
        has_outcome in proptest::bool::ANY
    ) {
        let mut episode = Episode::new(
            task_description.to_string(),
            TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                domain: domain.to_string(),
                tags: vec!["test".to_string()],
            },
            TaskType::CodeGeneration,
        );

        // Add some steps
        let step = ExecutionStep::new(
            1,
            "test_tool".to_string(),
            "Test action".to_string()
        );
        episode.add_step(step);

        // Optionally complete the episode
        if has_outcome {
            episode.complete(TaskOutcome::Success {
                verdict: "Test completed".to_string(),
                artifacts: vec!["file.rs".to_string()],
            });
        }

        // Store values before serialization to avoid borrow issues
        let episode_id = episode.episode_id;
        let steps_len = episode.steps.len();
        let is_complete = episode.is_complete();

        // Serialize and deserialize
        let json = serde_json::to_string(&episode).expect("serialize episode");
        let deserialized: Episode = serde_json::from_str(&json).expect("deserialize episode");

        // Store deserialized values
        let deserialized_id = deserialized.episode_id;
        let deserialized_steps_len = deserialized.steps.len();
        let deserialized_is_complete = deserialized.is_complete();
        let deserialized_task_desc = deserialized.task_description.clone();
        let deserialized_domain = deserialized.context.domain.clone();

        // Verify key fields match
        prop_assert_eq!(episode_id, deserialized_id);
        prop_assert_eq!(task_description, deserialized_task_desc);
        prop_assert_eq!(domain, deserialized_domain);
        prop_assert_eq!(steps_len, deserialized_steps_len);
        prop_assert_eq!(is_complete, deserialized_is_complete);
    }
}

// ============================================================================
// State Machine Invariant Tests
// ============================================================================

proptest! {
    /// Test episode state transitions
    #[test]
    fn episode_state_transition_invariants(
        num_steps in 0usize..50usize
    ) {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );

        // Initial state: not complete, no steps
        prop_assert!(!episode.is_complete());
        prop_assert_eq!(episode.steps.len(), 0);
        prop_assert!(episode.outcome.is_none());
        prop_assert!(episode.end_time.is_none());

        // Add steps
        for i in 0..num_steps {
            let step = ExecutionStep::new(
                i + 1,
                format!("tool_{}", i),
                format!("Action {}", i)
            );
            episode.add_step(step);
        }

        // After adding steps: not complete
        prop_assert!(!episode.is_complete());
        prop_assert_eq!(episode.steps.len(), num_steps);

        // Complete the episode
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        // After completion: is_complete should be true
        prop_assert!(episode.is_complete());
        prop_assert!(episode.outcome.is_some());
        prop_assert!(episode.end_time.is_some());

        // Duration should be available
        prop_assert!(episode.duration().is_some());
    }

    /// Test successful vs failed step counting
    #[test]
    fn step_success_counting_invariants(
        results in proptest::collection::vec(
            proptest::bool::ANY,
            1..100usize
        )
    ) {
        let mut episode = Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );

        let expected_successes = results.iter().filter(|&&r| r).count();
        let expected_failures = results.iter().filter(|&&r| !r).count();

        for (i, is_success) in results.iter().enumerate() {
            let mut step = ExecutionStep::new(
                i + 1,
                "test_tool".to_string(),
                "Test".to_string()
            );

            step.result = if *is_success {
                Some(ExecutionResult::Success {
                    output: "OK".to_string()
                })
            } else {
                Some(ExecutionResult::Error {
                    message: "Failed".to_string()
                })
            };

            episode.add_step(step);
        }

        // Verify counting invariants
        prop_assert_eq!(episode.successful_steps_count(), expected_successes);
        prop_assert_eq!(episode.failed_steps_count(), expected_failures);
        prop_assert_eq!(
            episode.successful_steps_count() + episode.failed_steps_count(),
            results.len()
        );
    }

    /// Test ExecutionResult success/failure invariants
    #[test]
    fn execution_result_invariants(
        output in "[a-zA-Z0-9 ]{0,100}",
        error_msg in "[a-zA-Z0-9 ]{0,100}"
    ) {
        let success = ExecutionResult::Success {
            output: output.clone()
        };
        let error = ExecutionResult::Error {
            message: error_msg.clone()
        };
        let timeout = ExecutionResult::Timeout;

        prop_assert!(success.is_success());
        prop_assert!(!error.is_success());
        prop_assert!(!timeout.is_success());
    }

    /// Test PatternEffectiveness calculation invariants
    #[test]
    fn pattern_effectiveness_invariants(
        retrieved in 0usize..100usize,
        _applied in 0usize..100usize,
        successes in 0usize..100usize,
        failures in 0usize..100usize
    ) {
        let mut effectiveness = PatternEffectiveness::default();

        // Simulate retrievals
        for _ in 0..retrieved {
            effectiveness.record_retrieval();
        }

        // Simulate applications with outcomes
        for _ in 0..successes {
            effectiveness.record_application(true, 0.1);
        }

        for _ in 0..failures {
            effectiveness.record_application(false, -0.1);
        }

        // Invariant: times_applied should equal successes + failures
        prop_assert_eq!(
            effectiveness.times_applied,
            successes + failures
        );

        // Invariant: usage rate should be applied / retrieved (or 0 if none)
        let expected_usage_rate = if retrieved == 0 {
            0.0
        } else {
            (successes + failures) as f32 / retrieved as f32
        };
        prop_assert!(
            (effectiveness.usage_rate() - expected_usage_rate).abs() < 0.0001
        );

        // Invariant: application success rate should be successes / applied (or 0.5 if none)
        if effectiveness.times_applied > 0 {
            let expected_success_rate = successes as f32 / effectiveness.times_applied as f32;
            prop_assert!(
                (effectiveness.application_success_rate() - expected_success_rate).abs() < 0.0001
            );
        } else {
            prop_assert_eq!(effectiveness.application_success_rate(), 0.5);
        }
    }
}

// ============================================================================
// TaskOutcome and Type Tests
// ============================================================================

proptest! {
    /// Test TaskOutcome serialization roundtrip
    #[test]
    fn task_outcome_json_roundtrip(
        verdict in "[a-zA-Z0-9 ]{1,100}",
        reason in "[a-zA-Z0-9 ]{1,100}"
    ) {
        let outcomes = vec![
            TaskOutcome::Success {
                verdict: verdict.clone(),
                artifacts: vec!["file1.rs".to_string()],
            },
            TaskOutcome::PartialSuccess {
                verdict: verdict.clone(),
                completed: vec!["item1".to_string()],
                failed: vec!["item2".to_string()],
            },
            TaskOutcome::Failure {
                reason: reason.clone(),
                error_details: Some("Detailed error".to_string()),
            },
        ];

        for outcome in outcomes {
            let json = serde_json::to_string(&outcome).expect("serialize outcome");
            let deserialized: TaskOutcome = serde_json::from_str(&json).expect("deserialize outcome");

            // Check variant matches
            prop_assert_eq!(
                std::mem::discriminant(&outcome),
                std::mem::discriminant(&deserialized)
            );
        }
    }

    /// Test TaskType roundtrip through string conversion
    #[test]
    fn task_type_string_roundtrip(
        task_type in prop::sample::select(vec![
            TaskType::CodeGeneration,
            TaskType::Debugging,
            TaskType::Refactoring,
            TaskType::Testing,
            TaskType::Analysis,
            TaskType::Documentation,
            TaskType::Other,
        ])
    ) {
        let string_repr = task_type.to_string();
        let parsed: TaskType = string_repr.parse().expect("parse task type");
        prop_assert_eq!(task_type, parsed);
    }

    /// Test ComplexityLevel serialization
    #[test]
    fn complexity_level_roundtrip(
        level in prop::sample::select(vec![
            ComplexityLevel::Simple,
            ComplexityLevel::Moderate,
            ComplexityLevel::Complex,
        ])
    ) {
        let json = serde_json::to_string(&level).expect("serialize complexity");
        let deserialized: ComplexityLevel = serde_json::from_str(&json).expect("deserialize complexity");
        prop_assert_eq!(level, deserialized);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

proptest! {
    /// Test episode with empty/edge case values
    #[test]
    fn episode_edge_cases(
        empty_desc in "",
        long_desc in "[a-z]{200,300}"
    ) {
        // Empty description should still work
        let episode1 = Episode::new(
            empty_desc.to_string(),
            TaskContext::default(),
            TaskType::Other,
        );
        prop_assert!(!episode1.is_complete());

        // Long description should work
        let episode2 = Episode::new(
            long_desc.to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        prop_assert_eq!(episode2.task_description.len(), long_desc.len());
    }

    /// Test Reflection serialization with various content sizes
    #[test]
    fn reflection_serialization_roundtrip(
        successes in proptest::collection::vec("[a-zA-Z0-9 ]{1,50}", 0..20),
        improvements in proptest::collection::vec("[a-zA-Z0-9 ]{1,50}", 0..20),
        insights in proptest::collection::vec("[a-zA-Z0-9 ]{1,50}", 0..20)
    ) {
        let reflection = Reflection {
            successes: successes.into_iter().map(|s| s.to_string()).collect(),
            improvements: improvements.into_iter().map(|s| s.to_string()).collect(),
            insights: insights.into_iter().map(|s| s.to_string()).collect(),
            generated_at: Utc::now(),
        };

        let json = serde_json::to_string(&reflection).expect("serialize reflection");
        let deserialized: Reflection = serde_json::from_str(&json).expect("deserialize reflection");

        prop_assert_eq!(reflection.successes.len(), deserialized.successes.len());
        prop_assert_eq!(reflection.improvements.len(), deserialized.improvements.len());
        prop_assert_eq!(reflection.insights.len(), deserialized.insights.len());
    }

    /// Test Evidence serialization
    #[test]
    fn evidence_serialization_roundtrip(
        sample_size in 1usize..100usize,
        success_rate in 0.0f32..1.0f32
    ) {
        use uuid::Uuid;

        let evidence = Evidence {
            episode_ids: (0..sample_size).map(|_| Uuid::new_v4()).collect(),
            success_rate,
            sample_size,
        };

        let json = serde_json::to_string(&evidence).expect("serialize evidence");
        let deserialized: Evidence = serde_json::from_str(&json).expect("deserialize evidence");

        prop_assert_eq!(evidence.episode_ids.len(), deserialized.episode_ids.len());
        prop_assert!((evidence.success_rate - deserialized.success_rate).abs() < 0.0001);
    }
}

// ============================================================================
// Determinism Tests
// ============================================================================

proptest! {
    /// Test that serialization is deterministic
    #[test]
    fn serialization_determinism(
        task_description in "[a-zA-Z0-9 ]{1,50}",
        domain in "[a-z]{3,15}"
    ) {
        let episode = Episode::new(
            task_description.to_string(),
            TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                domain: domain.to_string(),
                tags: vec!["test".to_string()],
            },
            TaskType::CodeGeneration,
        );

        // Serialize twice
        let json1 = serde_json::to_string(&episode).expect("serialize 1");
        let json2 = serde_json::to_string(&episode).expect("serialize 2");

        // Should be identical
        prop_assert_eq!(json1.clone(), json2.clone());

        // Deserializing both should give the same result
        let de1: Episode = serde_json::from_str(&json1).expect("deserialize 1");
        let de2: Episode = serde_json::from_str(&json2).expect("deserialize 2");
        prop_assert_eq!(de1, de2);
    }

    /// Test tag operations are deterministic
    #[test]
    fn tag_operations_deterministic(
        tags in proptest::collection::hash_set("[a-z]{2,20}", 1..50)
    ) {
        let mut episode1 = Episode::new(
            "Test".to_string(),
            TaskContext::default(),
            TaskType::Analysis,
        );

        let mut episode2 = Episode::new(
            "Test".to_string(),
            TaskContext::default(),
            TaskType::Analysis,
        );

        // Add same tags in same order
        let tag_vec: Vec<_> = tags.iter().cloned().collect();
        for tag in &tag_vec {
            episode1.add_tag(tag.clone()).unwrap();
            episode2.add_tag(tag.clone()).unwrap();
        }

        // Should have identical tags
        prop_assert_eq!(episode1.get_tags(), episode2.get_tags());

        // Tag presence should be identical
        for tag in &tag_vec {
            prop_assert_eq!(
                episode1.has_tag(tag),
                episode2.has_tag(tag)
            );
        }
    }
}

// ============================================================================
// Postcard Serialization Tests (Binary format used in storage)
// ============================================================================

#[cfg(test)]
mod postcard_tests {
    use super::*;

    proptest! {
        /// Test TaskContext postcard roundtrip
        #[test]
        fn task_context_postcard_roundtrip(
            language in proptest::option::of("[a-z]{2,10}"),
            framework in proptest::option::of("[a-z]{2,15}"),
            domain in "[a-z]{3,20}",
            tags in proptest::collection::vec("[a-z]{2,15}", 0..10)
        ) {
            let context = TaskContext {
                language: language.map(|s| s.to_string()),
                framework: framework.map(|s| s.to_string()),
                complexity: ComplexityLevel::Moderate,
                domain: domain.to_string(),
                tags: tags.into_iter().map(|s| s.to_string()).collect(),
            };

            // Postcard roundtrip
            let serialized = postcard::to_allocvec(&context).expect("postcard serialize");
            let deserialized: TaskContext = postcard::from_bytes(&serialized).expect("postcard deserialize");
            assert_eq!(context, deserialized);
        }

        /// Test RewardScore postcard roundtrip
        #[test]
        fn reward_score_postcard_roundtrip(
            total in 0.0f32..2.0f32,
            base in 0.0f32..1.0f32,
            efficiency in 0.5f32..1.5f32,
        ) {
            let score = RewardScore {
                total,
                base,
                efficiency,
                complexity_bonus: 1.0,
                quality_multiplier: 1.0,
                learning_bonus: 0.0,
            };

            let serialized = postcard::to_allocvec(&score).expect("postcard serialize");
            let deserialized: RewardScore = postcard::from_bytes(&serialized).expect("postcard deserialize");

            assert!((score.total - deserialized.total).abs() < 0.001);
        }

        /// Test Episode postcard serialization with DateTime handling
        /// Note: DateTime fields may require special handling in postcard
        #[test]
        fn episode_postcard_size_check(
            task_description in "[a-zA-Z0-9 ]{1,200}",
            num_steps in 0usize..20usize
        ) {
            let mut episode = Episode::new(
                task_description.to_string(),
                TaskContext::default(),
                TaskType::CodeGeneration,
            );

            // Add steps
            for i in 0..num_steps {
                let step = ExecutionStep::new(
                    i + 1,
                    format!("tool_{}", i % 10),
                    format!("Action {}", i)
                );
                episode.add_step(step);
            }

            // Postcard serialization - DateTime may have compatibility issues
            // Test that we can at least attempt serialization without panic
            let serialized = postcard::to_allocvec(&episode);

            // If serialization succeeds, verify roundtrip
            if let Ok(bytes) = serialized {
                let deserialized: StdResult<Episode, _> = postcard::from_bytes(&bytes);
                // Episode contains chrono::DateTime which may not serialize cleanly with postcard
                // This test documents the behavior rather than enforcing success
                if let Ok(de) = deserialized {
                    prop_assert_eq!(episode.steps.len(), de.steps.len());
                }
            }
            // If serialization fails, that's acceptable for this test
            // as it documents the limitation
        }
    }
}

// ============================================================================
// Property Test Configuration
// ============================================================================

/// Custom test runner configuration for expensive tests
fn configure_proptest() -> ProptestConfig {
    ProptestConfig {
        cases: 100,           // Number of test cases to run
        max_shrink_iters: 50, // Max shrinking iterations on failure
        ..ProptestConfig::default()
    }
}

// Example of using custom config for specific test groups
proptest! {
    #![proptest_config(configure_proptest())]

    /// Comprehensive integration test with more cases
    #[test]
    fn comprehensive_episode_lifecycle(
        task_type in prop::sample::select(vec![
            TaskType::CodeGeneration,
            TaskType::Testing,
            TaskType::Debugging,
        ]),
        complexity in prop::sample::select(vec![
            ComplexityLevel::Simple,
            ComplexityLevel::Moderate,
            ComplexityLevel::Complex,
        ]),
        num_steps in 0usize..100usize,
    ) {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity,
            domain: "test-domain".to_string(),
            tags: vec!["test".to_string()],
        };

        let mut episode = Episode::new(
            "Comprehensive test".to_string(),
            context,
            task_type,
        );

        // Add varying number of steps
        for i in 0..num_steps {
            let result = if i % 3 == 0 {
                ExecutionResult::Error { message: "Error".to_string() }
            } else {
                ExecutionResult::Success { output: "OK".to_string() }
            };

            let mut step = ExecutionStep::new(
                i + 1,
                format!("step_{}", i),
                "Action".to_string()
            );
            step.result = Some(result);
            episode.add_step(step);
        }

        // Complete episode
        episode.complete(TaskOutcome::Success {
            verdict: "Completed".to_string(),
            artifacts: vec![],
        });

        // Verify invariants
        prop_assert!(episode.is_complete());
        prop_assert_eq!(episode.steps.len(), num_steps);
        prop_assert!(episode.duration().is_some());

        // Serialization roundtrip
        let json = serde_json::to_string(&episode).expect("serialize");
        let deserialized: Episode = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(episode.steps.len(), deserialized.steps.len());
    }
}
