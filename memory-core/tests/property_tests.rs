//! Property-based tests for core memory APIs
//!
//! These tests use the `proptest` crate to verify invariants hold
//! across a wide range of generated inputs.

use chrono::{DateTime, Duration, Utc};
use memory_core::episode::{
    ApplicationOutcome, EpisodeRelationship, PatternApplication, RelationshipMetadata,
    RelationshipType,
};
use memory_core::{
    ComplexityLevel, Episode, ExecutionResult, ExecutionStep, OutcomeStats, Pattern,
    PatternEffectiveness, TaskContext, TaskOutcome, TaskType,
};
use proptest::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Test Data Generators (Strategies)
// ============================================================================

/// Generate a valid UUID
fn uuid_strategy() -> impl Strategy<Value = Uuid> {
    any::<[u8; 16]>().prop_map(Uuid::from_bytes)
}

/// Generate a valid timestamp (within last year)
fn timestamp_strategy() -> impl Strategy<Value = DateTime<Utc>> {
    (0..31_536_000i64).prop_map(|seconds_ago| Utc::now() - Duration::seconds(seconds_ago))
}

/// Generate a valid task type
fn task_type_strategy() -> impl Strategy<Value = TaskType> {
    prop_oneof![
        Just(TaskType::CodeGeneration),
        Just(TaskType::Debugging),
        Just(TaskType::Refactoring),
        Just(TaskType::Testing),
        Just(TaskType::Analysis),
        Just(TaskType::Documentation),
        Just(TaskType::Other),
    ]
}

/// Generate a valid complexity level
fn complexity_level_strategy() -> impl Strategy<Value = ComplexityLevel> {
    prop_oneof![
        Just(ComplexityLevel::Simple),
        Just(ComplexityLevel::Moderate),
        Just(ComplexityLevel::Complex),
    ]
}

/// Generate a valid tag string (alphanumeric, hyphens, underscores, 2-50 chars)
fn tag_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_]{2,50}".prop_map(|s| s.to_lowercase())
}

/// Generate a valid task context
fn task_context_strategy() -> impl Strategy<Value = TaskContext> {
    (
        proptest::option::of("[a-z]{2,10}"),
        proptest::option::of("[a-z]{2,15}"),
        complexity_level_strategy(),
        "[a-z-]{3,20}",
        proptest::collection::vec(tag_strategy(), 0..5),
    )
        .prop_map(
            |(language, framework, complexity, domain, tags)| TaskContext {
                language: language.map(String::from),
                framework: framework.map(String::from),
                complexity,
                domain: domain.to_lowercase(),
                tags: tags.into_iter().map(|s| s.to_lowercase()).collect(),
            },
        )
}

/// Generate a valid execution result
fn execution_result_strategy() -> impl Strategy<Value = ExecutionResult> {
    prop_oneof![
        "[a-zA-Z0-9_ ]{1,100}".prop_map(|output| ExecutionResult::Success {
            output: output.to_string(),
        }),
        "[a-zA-Z0-9_ ]{1,100}".prop_map(|message| ExecutionResult::Error {
            message: message.to_string(),
        }),
        Just(ExecutionResult::Timeout),
    ]
}

/// Generate a valid execution step
fn execution_step_strategy() -> impl Strategy<Value = ExecutionStep> {
    (
        1usize..100usize,
        "[a-z_]{3,20}",
        "[a-zA-Z0-9_ ]{5,100}",
        proptest::option::of(execution_result_strategy()),
        0u64..10_000u64,
        proptest::option::of(0usize..1_000_000usize),
    )
        .prop_map(
            |(step_number, tool, action, result, latency_ms, tokens_used)| ExecutionStep {
                step_number,
                timestamp: Utc::now(),
                tool: tool.to_string(),
                action: action.to_string(),
                parameters: serde_json::json!({}),
                result,
                latency_ms,
                tokens_used,
                metadata: HashMap::new(),
            },
        )
}

/// Generate a valid task outcome
fn task_outcome_strategy() -> impl Strategy<Value = TaskOutcome> {
    prop_oneof![
        (
            "[a-zA-Z0-9_ ]{10,200}",
            proptest::collection::vec("[a-z_./]{3,30}", 0..5)
        )
            .prop_map(|(verdict, artifacts)| TaskOutcome::Success {
                verdict: verdict.to_string(),
                artifacts: artifacts.into_iter().map(String::from).collect(),
            }),
        (
            "[a-zA-Z0-9_ ]{10,200}",
            proptest::collection::vec("[a-z_]{3,20}", 0..5),
            proptest::collection::vec("[a-z_]{3,20}", 0..5),
        )
            .prop_map(|(verdict, completed, failed)| TaskOutcome::PartialSuccess {
                verdict: verdict.to_string(),
                completed: completed.into_iter().map(String::from).collect(),
                failed: failed.into_iter().map(String::from).collect(),
            }),
        (
            "[a-zA-Z0-9_ ]{10,200}",
            proptest::option::of("[a-zA-Z0-9_ ]{10,200}")
        )
            .prop_map(|(reason, error_details)| TaskOutcome::Failure {
                reason: reason.to_string(),
                error_details: error_details.map(String::from),
            }),
    ]
}

/// Generate a valid reward score
fn reward_score_strategy() -> impl Strategy<Value = memory_core::RewardScore> {
    (
        0.0f32..2.0f32,
        0.0f32..1.0f32,
        0.5f32..1.5f32,
        1.0f32..1.3f32,
        0.8f32..1.2f32,
        0.0f32..0.5f32,
    )
        .prop_map(
            |(total, base, efficiency, complexity_bonus, quality_multiplier, learning_bonus)| {
                memory_core::RewardScore {
                    total,
                    base,
                    efficiency,
                    complexity_bonus,
                    quality_multiplier,
                    learning_bonus,
                }
            },
        )
}

/// Generate a valid episode (incomplete - no end_time)
fn episode_incomplete_strategy() -> impl Strategy<Value = Episode> {
    (
        "[a-zA-Z0-9_ ]{10,200}",
        task_context_strategy(),
        task_type_strategy(),
        proptest::collection::vec(execution_step_strategy(), 0..10),
        proptest::collection::vec(tag_strategy(), 0..5),
    )
        .prop_map(
            |(task_description, context, task_type, steps, tags)| Episode {
                episode_id: Uuid::new_v4(),
                task_type,
                task_description: task_description.to_string(),
                context,
                start_time: Utc::now(),
                end_time: None,
                outcome: None,
                reward: None,
                reflection: None,
                steps,
                patterns: Vec::new(),
                heuristics: Vec::new(),
                applied_patterns: Vec::new(),
                salient_features: None,
                metadata: HashMap::new(),
                tags: tags.into_iter().map(|s| s.to_lowercase()).collect(),
            },
        )
}

/// Generate a valid relationship type
fn relationship_type_strategy() -> impl Strategy<Value = RelationshipType> {
    prop_oneof![
        Just(RelationshipType::ParentChild),
        Just(RelationshipType::DependsOn),
        Just(RelationshipType::Follows),
        Just(RelationshipType::RelatedTo),
        Just(RelationshipType::Blocks),
        Just(RelationshipType::Duplicates),
        Just(RelationshipType::References),
    ]
}

/// Generate valid relationship metadata
fn relationship_metadata_strategy() -> impl Strategy<Value = RelationshipMetadata> {
    (
        proptest::option::of("[a-zA-Z0-9_ ]{10,200}"),
        proptest::option::of("[a-z_]{3,20}"),
        proptest::option::of(1u8..10u8),
    )
        .prop_map(|(reason, created_by, priority)| RelationshipMetadata {
            reason: reason.map(String::from),
            created_by: created_by.map(String::from),
            priority,
            custom_fields: HashMap::new(),
        })
}

/// Generate a valid episode relationship between two different episodes
fn episode_relationship_strategy() -> impl Strategy<Value = EpisodeRelationship> {
    (
        uuid_strategy(),
        uuid_strategy(),
        relationship_type_strategy(),
        relationship_metadata_strategy(),
    )
        .prop_filter(
            "from and to episode IDs must be different",
            |(from_id, to_id, _, _)| from_id != to_id,
        )
        .prop_map(
            |(from_episode_id, to_episode_id, relationship_type, metadata)| {
                EpisodeRelationship::new(
                    from_episode_id,
                    to_episode_id,
                    relationship_type,
                    metadata,
                )
            },
        )
}

/// Generate a pattern effectiveness tracker
fn pattern_effectiveness_strategy() -> impl Strategy<Value = PatternEffectiveness> {
    (
        0usize..100usize,
        0usize..100usize,
        0usize..100usize,
        0usize..100usize,
        -1.0f32..1.0f32,
    )
        .prop_map(
            |(
                times_retrieved,
                times_applied,
                success_when_applied,
                failure_when_applied,
                avg_reward_delta,
            )| {
                PatternEffectiveness {
                    times_retrieved,
                    times_applied,
                    success_when_applied,
                    failure_when_applied,
                    avg_reward_delta,
                    last_used: Utc::now(),
                    created_at: Utc::now() - Duration::days(30),
                }
            },
        )
}

/// Generate a valid tool sequence pattern
fn tool_sequence_pattern_strategy() -> impl Strategy<Value = Pattern> {
    (
        uuid_strategy(),
        proptest::collection::vec("[a-z_]{3,15}", 1..5),
        task_context_strategy(),
        0.0f32..1.0f32,
        0i64..10_000i64,
        1usize..100usize,
        pattern_effectiveness_strategy(),
    )
        .prop_map(
            |(
                id,
                tools,
                context,
                success_rate,
                avg_latency_ms,
                occurrence_count,
                effectiveness,
            )| {
                Pattern::ToolSequence {
                    id,
                    tools: tools.into_iter().map(String::from).collect(),
                    context,
                    success_rate,
                    avg_latency: Duration::milliseconds(avg_latency_ms),
                    occurrence_count,
                    effectiveness,
                }
            },
        )
}

/// Generate a valid decision point pattern
fn decision_point_pattern_strategy() -> impl Strategy<Value = Pattern> {
    (
        uuid_strategy(),
        "[a-zA-Z0-9_ ]{10,100}",
        "[a-zA-Z0-9_ ]{10,100}",
        task_context_strategy(),
        0usize..50usize,
        0usize..20usize,
        pattern_effectiveness_strategy(),
    )
        .prop_map(
            |(id, condition, action, context, success_count, failure_count, effectiveness)| {
                Pattern::DecisionPoint {
                    id,
                    condition: condition.to_string(),
                    action: action.to_string(),
                    outcome_stats: OutcomeStats {
                        success_count,
                        failure_count,
                        total_count: success_count + failure_count,
                        avg_duration_secs: 30.0,
                    },
                    context,
                    effectiveness,
                }
            },
        )
}

/// Generate a valid error recovery pattern
fn error_recovery_pattern_strategy() -> impl Strategy<Value = Pattern> {
    (
        uuid_strategy(),
        "[a-zA-Z0-9_]{5,30}",
        proptest::collection::vec("[a-z_]{3,20}", 1..5),
        task_context_strategy(),
        0.0f32..1.0f32,
        pattern_effectiveness_strategy(),
    )
        .prop_map(
            |(id, error_type, recovery_steps, context, success_rate, effectiveness)| {
                Pattern::ErrorRecovery {
                    id,
                    error_type: error_type.to_string(),
                    recovery_steps: recovery_steps.into_iter().map(String::from).collect(),
                    success_rate,
                    context,
                    effectiveness,
                }
            },
        )
}

/// Generate a valid context pattern
fn context_pattern_strategy() -> impl Strategy<Value = Pattern> {
    (
        uuid_strategy(),
        proptest::collection::vec("[a-zA-Z0-9_:]{5,30}", 1..5),
        "[a-zA-Z0-9_ ]{20,200}",
        proptest::collection::vec(uuid_strategy(), 1..10),
        0.0f32..1.0f32,
        pattern_effectiveness_strategy(),
    )
        .prop_map(
            |(
                id,
                context_features,
                recommended_approach,
                evidence,
                success_rate,
                effectiveness,
            )| {
                Pattern::ContextPattern {
                    id,
                    context_features: context_features.into_iter().map(String::from).collect(),
                    recommended_approach: recommended_approach.to_string(),
                    evidence,
                    success_rate,
                    effectiveness,
                }
            },
        )
}

/// Generate any valid pattern
fn pattern_strategy() -> impl Strategy<Value = Pattern> {
    prop_oneof![
        tool_sequence_pattern_strategy(),
        decision_point_pattern_strategy(),
        error_recovery_pattern_strategy(),
        context_pattern_strategy(),
    ]
}

/// Generate an application outcome
fn application_outcome_strategy() -> impl Strategy<Value = ApplicationOutcome> {
    prop_oneof![
        Just(ApplicationOutcome::Helped),
        Just(ApplicationOutcome::NoEffect),
        Just(ApplicationOutcome::Hindered),
        Just(ApplicationOutcome::Pending),
    ]
}

/// Generate a pattern application record
fn pattern_application_strategy() -> impl Strategy<Value = PatternApplication> {
    (
        uuid_strategy(),
        1usize..50usize,
        application_outcome_strategy(),
        proptest::option::of("[a-zA-Z0-9_ ]{5,100}"),
    )
        .prop_map(
            |(pattern_id, applied_at_step, outcome, notes)| PatternApplication {
                pattern_id,
                applied_at_step,
                outcome,
                notes: notes.map(String::from),
            },
        )
}

// ============================================================================
// Episode Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 50,
        ..ProptestConfig::default()
    })]

    /// Property: Episode serialization roundtrip preserves all data
    ///
    /// For any valid episode, serializing to JSON and deserializing
    /// should produce an equivalent episode.
    #[test]
    fn prop_episode_roundtrip(episode in episode_incomplete_strategy()) {
        let json = serde_json::to_string(&episode).expect("serialization should succeed");
        let deserialized: Episode = serde_json::from_str(&json)
            .expect("deserialization should succeed");

        // Check all fields match
        prop_assert_eq!(episode.episode_id, deserialized.episode_id);
        prop_assert_eq!(episode.task_type, deserialized.task_type);
        prop_assert_eq!(episode.task_description, deserialized.task_description);
        prop_assert_eq!(episode.context.domain, deserialized.context.domain);
        prop_assert_eq!(episode.start_time.timestamp(), deserialized.start_time.timestamp());
        prop_assert_eq!(episode.steps.len(), deserialized.steps.len());
        prop_assert_eq!(episode.tags.len(), deserialized.tags.len());

        // Check completion status matches
        prop_assert_eq!(episode.is_complete(), deserialized.is_complete());
        if episode.is_complete() {
            prop_assert!(deserialized.end_time.is_some());
            prop_assert!(deserialized.outcome.is_some());
        }
    }

    /// Property: Episodes with different data produce different IDs
    ///
    /// Creating two episodes with different task descriptions or contexts
    /// should always produce different episode IDs.
    #[test]
    fn prop_episode_id_unique(
        desc1 in "[a-zA-Z0-9_ ]{10,100}",
        desc2 in "[a-zA-Z0-9_ ]{10,100}",
        ctx1 in task_context_strategy(),
        ctx2 in task_context_strategy(),
    ) {
        prop_assume!(desc1 != desc2 || ctx1.domain != ctx2.domain);

        let episode1 = Episode::new(desc1.to_string(), ctx1, TaskType::CodeGeneration);
        let episode2 = Episode::new(desc2.to_string(), ctx2, TaskType::CodeGeneration);

        prop_assert_ne!(episode1.episode_id, episode2.episode_id);
    }

    /// Property: Completed episode has end_time > start_time
    ///
    /// When an episode is completed, the end_time should always be
    /// after or equal to the start_time.
    #[test]
    fn prop_episode_completion(
        mut episode in episode_incomplete_strategy(),
        outcome in task_outcome_strategy(),
    ) {
        let start_time = episode.start_time;

        episode.complete(outcome);

        prop_assert!(episode.is_complete());
        prop_assert!(episode.end_time.is_some());
        prop_assert!(episode.outcome.is_some());

        let end_time = episode.end_time.unwrap();
        prop_assert!(
            end_time >= start_time,
            "end_time ({:?}) should be >= start_time ({:?})",
            end_time, start_time
        );
    }

    /// Property: Episode duration is correctly calculated
    ///
    /// For a completed episode, duration() should return the time
    /// difference between end_time and start_time.
    #[test]
    fn prop_episode_duration(
        mut episode in episode_incomplete_strategy(),
    ) {
        // Before completion, duration should be None
        prop_assert!(episode.duration().is_none());

        // Complete the episode
        episode.complete(TaskOutcome::Success {
            verdict: "Test".to_string(),
            artifacts: vec![],
        });

        // After completion, duration should be Some
        let duration = episode.duration();
        prop_assert!(duration.is_some());

        // Duration should be non-negative
        let dur = duration.unwrap();
        prop_assert!(dur.num_milliseconds() >= 0);
    }
}

// ============================================================================
// Relationship Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 50,
        ..ProptestConfig::default()
    })]

    /// Property: Relationship inverse swaps from/to correctly
    ///
    /// For a directional relationship, calling inverse() should
    /// swap the from_episode_id and to_episode_id.
    #[test]
    fn prop_relationship_symmetry(rel in episode_relationship_strategy()) {
        // Only test directional relationships
        prop_assume!(rel.is_directional());

        if let Some(inverse) = rel.inverse() {
            prop_assert_eq!(inverse.from_episode_id, rel.to_episode_id);
            prop_assert_eq!(inverse.to_episode_id, rel.from_episode_id);
            prop_assert_eq!(inverse.relationship_type, rel.relationship_type);
        }
    }

    /// Property: Self-relationships should be identifiable
    ///
    /// A relationship where from_id == to_id is a self-relationship
    /// and should be handled appropriately.
    #[test]
    fn prop_no_self_relationship(
        episode_id in uuid_strategy(),
        rel_type in relationship_type_strategy(),
        metadata in relationship_metadata_strategy(),
    ) {
        // Create a self-relationship
        let self_rel = EpisodeRelationship::new(
            episode_id,
            episode_id,
            rel_type,
            metadata,
        );

        // Self-relationship should have same from and to
        prop_assert_eq!(self_rel.from_episode_id, self_rel.to_episode_id);
        prop_assert_eq!(self_rel.from_episode_id, episode_id);
    }

    /// Property: Relationship serialization roundtrip
    #[test]
    fn prop_relationship_roundtrip(rel in episode_relationship_strategy()) {
        let json = serde_json::to_string(&rel).expect("serialization should succeed");
        let deserialized: EpisodeRelationship = serde_json::from_str(&json)
            .expect("deserialization should succeed");

        prop_assert_eq!(rel.id, deserialized.id);
        prop_assert_eq!(rel.from_episode_id, deserialized.from_episode_id);
        prop_assert_eq!(rel.to_episode_id, deserialized.to_episode_id);
        prop_assert_eq!(rel.relationship_type, deserialized.relationship_type);
    }

    /// Property: Relationship type string conversion is reversible
    #[test]
    fn prop_relationship_type_roundtrip(rel_type in relationship_type_strategy()) {
        let s = rel_type.as_str();
        let parsed = RelationshipType::parse(s).expect("parsing should succeed");
        prop_assert_eq!(rel_type, parsed);
    }
}

// ============================================================================
// Pattern Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 50,
        ..ProptestConfig::default()
    })]

    /// Property: Pattern serialization roundtrip preserves data
    #[test]
    fn prop_pattern_roundtrip(pattern in pattern_strategy()) {
        let json = serde_json::to_string(&pattern).expect("serialization should succeed");
        let deserialized: Pattern = serde_json::from_str(&json)
            .expect("deserialization should succeed");

        prop_assert_eq!(pattern.id(), deserialized.id());
        prop_assert!((pattern.success_rate() - deserialized.success_rate()).abs() < 0.0001);
        prop_assert_eq!(pattern.sample_size(), deserialized.sample_size());
    }

    /// Property: Pattern similarity score is symmetric
    ///
    /// similarity_score(a, b) should equal similarity_score(b, a)
    #[test]
    fn prop_pattern_similarity_symmetric(pattern1 in pattern_strategy()) {
        // Create a similar pattern by cloning and modifying slightly
        let mut pattern2 = pattern1.clone();

        // For ToolSequence, we can create a similar one
        if let Pattern::ToolSequence { ref tools, ref context, .. } = pattern1 {
            if tools.len() > 1 {
                // Create pattern with same tools but different success rate
                pattern2 = Pattern::ToolSequence {
                    id: pattern1.id(),
                    tools: tools.clone(),
                    context: context.clone(),
                    success_rate: 0.8,
                    avg_latency: chrono::Duration::milliseconds(100),
                    occurrence_count: 5,
                    effectiveness: PatternEffectiveness::new(),
                };

                let sim1 = pattern1.similarity_score(&pattern2);
                let sim2 = pattern2.similarity_score(&pattern1);

                prop_assert!((sim1 - sim2).abs() < 0.0001,
                    "similarity should be symmetric: {} vs {}", sim1, sim2);
            }
        }
    }

    /// Property: Pattern confidence calculation is monotonic with sample size
    ///
    /// For the same success rate, higher sample size should give
    /// higher or equal confidence.
    #[test]
    fn prop_pattern_confidence_monotonic(
        success_rate in 0.1f32..1.0f32,
        count1 in 1usize..100usize,
        count2 in 1usize..100usize,
    ) {
        let pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: TaskContext::default(),
            success_rate,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: count1,
            effectiveness: PatternEffectiveness::new(),
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: TaskContext::default(),
            success_rate,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: count2,
            effectiveness: PatternEffectiveness::new(),
        };

        let conf1 = pattern1.confidence();
        let conf2 = pattern2.confidence();

        // Confidence should be proportional to sqrt(sample_size)
        if count1 > count2 {
            prop_assert!(conf1 > conf2 || (conf1 - conf2).abs() < 0.0001,
                "higher count should give higher or equal confidence");
        } else if count2 > count1 {
            prop_assert!(conf2 > conf1 || (conf1 - conf2).abs() < 0.0001,
                "higher count should give higher or equal confidence");
        } else {
            prop_assert!((conf1 - conf2).abs() < 0.0001,
                "same count should give same confidence");
        }
    }

    /// Property: Pattern effectiveness score is in valid range
    #[test]
    fn prop_pattern_effectiveness_bounds(eff in pattern_effectiveness_strategy()) {
        let score = eff.effectiveness_score();

        // Score should be non-negative
        prop_assert!(score >= 0.0, "effectiveness score should be non-negative");

        // Success rate should be in [0, 1]
        let success_rate = eff.application_success_rate();
        prop_assert!(success_rate >= 0.0 && success_rate <= 1.0,
            "success rate should be in [0, 1]");

        // Usage rate should be in [0, 1]
        let usage_rate = eff.usage_rate();
        prop_assert!(usage_rate >= 0.0 && usage_rate <= 1.0,
            "usage rate should be in [0, 1]");
    }

    /// Property: Recording retrieval increases retrieval count
    #[test]
    fn prop_pattern_record_retrieval(mut pattern in pattern_strategy()) {
        let initial_retrieved = pattern.effectiveness().times_retrieved;

        pattern.record_retrieval();

        prop_assert_eq!(pattern.effectiveness().times_retrieved, initial_retrieved + 1);
    }

    /// Property: Recording application updates counts correctly
    #[test]
    fn prop_pattern_record_application(
        mut pattern in pattern_strategy(),
        success in proptest::bool::ANY,
        reward_delta in -1.0f32..1.0f32,
    ) {
        let initial_applied = pattern.effectiveness().times_applied;
        let initial_success = pattern.effectiveness().success_when_applied;
        let initial_failure = pattern.effectiveness().failure_when_applied;

        pattern.record_application(success, reward_delta);

        prop_assert_eq!(pattern.effectiveness().times_applied, initial_applied + 1);
        if success {
            prop_assert_eq!(pattern.effectiveness().success_when_applied, initial_success + 1);
            prop_assert_eq!(pattern.effectiveness().failure_when_applied, initial_failure);
        } else {
            prop_assert_eq!(pattern.effectiveness().success_when_applied, initial_success);
            prop_assert_eq!(pattern.effectiveness().failure_when_applied, initial_failure + 1);
        }
    }
}

// ============================================================================
// Tag Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 50,
        ..ProptestConfig::default()
    })]

    /// Property: Adding same tag twice is idempotent
    ///
    /// Adding a tag that already exists should return false and
    /// not change the tag list.
    #[test]
    fn prop_tag_add_idempotent(
        mut episode in episode_incomplete_strategy(),
        tag in tag_strategy(),
    ) {
        // First add should succeed
        let result1 = episode.add_tag(tag.clone());
        prop_assert!(result1.is_ok());
        prop_assert!(result1.unwrap(), "first add should return true");

        let tag_count_after_first = episode.tags.len();

        // Second add should return false (already exists)
        let result2 = episode.add_tag(tag.clone());
        prop_assert!(result2.is_ok());
        prop_assert!(!result2.unwrap(), "second add should return false");

        // Tag count should not change
        prop_assert_eq!(episode.tags.len(), tag_count_after_first);

        // Tag should be present
        prop_assert!(episode.has_tag(&tag));
    }

    /// Property: Adding tag with different case normalizes correctly
    #[test]
    fn prop_tag_case_normalization(
        mut episode in episode_incomplete_strategy(),
        tag in "[a-z]{3,20}",
    ) {
        let lower = tag.to_lowercase();
        let upper = tag.to_uppercase();
        let mixed = format!("{}{}", &tag[..1].to_uppercase(), &tag[1..]);

        // Add lowercase version
        episode.add_tag(lower.clone()).expect("add should succeed");

        // Try adding uppercase version - should be detected as duplicate
        let result = episode.add_tag(upper);
        prop_assert!(result.is_ok());
        prop_assert!(!result.unwrap(), "should detect as duplicate");

        // Try adding mixed case version - should be detected as duplicate
        let result = episode.add_tag(mixed);
        prop_assert!(result.is_ok());
        prop_assert!(!result.unwrap(), "should detect as duplicate");

        // Should only have one tag
        prop_assert_eq!(episode.tags.len(), 1);
        prop_assert_eq!(episode.tags[0], lower);
    }

    /// Property: Removing non-existent tag is no-op
    ///
    /// Removing a tag that doesn't exist should return false and
    /// not change the tag list.
    #[test]
    fn prop_tag_remove_idempotent(
        mut episode in episode_incomplete_strategy(),
        existing_tags in proptest::collection::vec(tag_strategy(), 0..5),
        non_existent_tag in tag_strategy(),
    ) {
        // Add existing tags
        for tag in &existing_tags {
            let _ = episode.add_tag(tag.clone());
        }

        // Ensure non_existent_tag is not in the list
        prop_assume!(!existing_tags.contains(&non_existent_tag));

        let tag_count_before = episode.tags.len();

        // Try to remove non-existent tag
        let removed = episode.remove_tag(&non_existent_tag);
        prop_assert!(!removed, "removing non-existent tag should return false");

        // Tag count should not change
        prop_assert_eq!(episode.tags.len(), tag_count_before);
    }

    /// Property: Tag removal is case-insensitive
    #[test]
    fn prop_tag_remove_case_insensitive(
        mut episode in episode_incomplete_strategy(),
        tag in "[a-z]{3,20}",
    ) {
        let lower = tag.to_lowercase();
        let upper = tag.to_uppercase();

        // Add lowercase version
        episode.add_tag(lower.clone()).expect("add should succeed");
        prop_assert!(episode.has_tag(&lower));

        // Remove with uppercase - should work
        let removed = episode.remove_tag(&upper);
        prop_assert!(removed, "removing with different case should work");
        prop_assert!(!episode.has_tag(&lower));
    }

    /// Property: has_tag returns false for invalid tags
    #[test]
    fn prop_has_tag_invalid_input(episode in episode_incomplete_strategy()) {
        // Empty tag
        prop_assert!(!episode.has_tag(""));

        // Whitespace-only
        prop_assert!(!episode.has_tag("   "));

        // Tag with spaces
        prop_assert!(!episode.has_tag("invalid tag"));

        // Tag with special characters
        prop_assert!(!episode.has_tag("tag@invalid"));
        prop_assert!(!episode.has_tag("tag/invalid"));
    }

    /// Property: Tag validation rejects invalid tags
    #[test]
    fn prop_tag_validation_rejects_invalid(mut episode in episode_incomplete_strategy()) {
        // Empty tag
        let result = episode.add_tag("".to_string());
        prop_assert!(result.is_err());

        // Single character
        let result = episode.add_tag("a".to_string());
        prop_assert!(result.is_err());

        // Tag with spaces
        let result = episode.add_tag("invalid tag".to_string());
        prop_assert!(result.is_err());

        // Tag with special characters
        let result = episode.add_tag("tag@invalid".to_string());
        prop_assert!(result.is_err());
    }

    /// Property: Clear tags removes all tags
    #[test]
    fn prop_clear_tags_removes_all(
        mut episode in episode_incomplete_strategy(),
        tags in proptest::collection::vec(tag_strategy(), 1..10),
    ) {
        // Add tags
        for tag in &tags {
            let _ = episode.add_tag(tag.clone());
        }
        prop_assert_eq!(episode.tags.len(), tags.len());

        // Clear all tags
        episode.clear_tags();

        // Verify all tags removed
        prop_assert!(episode.tags.is_empty());
        for tag in &tags {
            prop_assert!(!episode.has_tag(tag));
        }
    }
}

// ============================================================================
// Execution Step Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 50,
        ..ProptestConfig::default()
    })]

    /// Property: Execution step serialization roundtrip
    #[test]
    fn prop_execution_step_roundtrip(step in execution_step_strategy()) {
        let json = serde_json::to_string(&step).expect("serialization should succeed");
        let deserialized: ExecutionStep = serde_json::from_str(&json)
            .expect("deserialization should succeed");

        prop_assert_eq!(step.step_number, deserialized.step_number);
        prop_assert_eq!(step.tool, deserialized.tool);
        prop_assert_eq!(step.action, deserialized.action);
        prop_assert_eq!(step.latency_ms, deserialized.latency_ms);
        prop_assert_eq!(step.tokens_used, deserialized.tokens_used);
    }

    /// Property: is_success returns true only for Success result
    #[test]
    fn prop_step_is_success(step in execution_step_strategy()) {
        match &step.result {
            Some(ExecutionResult::Success { .. }) => {
                prop_assert!(step.is_success());
            }
            Some(ExecutionResult::Error { .. }) | Some(ExecutionResult::Timeout) | None => {
                prop_assert!(!step.is_success());
            }
        }
    }

    /// Property: Adding step increases step count
    #[test]
    fn prop_add_step_increases_count(
        mut episode in episode_incomplete_strategy(),
        steps in proptest::collection::vec(execution_step_strategy(), 1..10),
    ) {
        let initial_count = episode.steps.len();

        for step in steps {
            episode.add_step(step);
        }

        prop_assert_eq!(episode.steps.len(), initial_count + steps.len());
    }
}

// ============================================================================
// Task Context Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 50,
        ..ProptestConfig::default()
    })]

    /// Property: Task context serialization roundtrip
    #[test]
    fn prop_task_context_roundtrip(ctx in task_context_strategy()) {
        let json = serde_json::to_string(&ctx).expect("serialization should succeed");
        let deserialized: TaskContext = serde_json::from_str(&json)
            .expect("deserialization should succeed");

        prop_assert_eq!(ctx.language, deserialized.language);
        prop_assert_eq!(ctx.framework, deserialized.framework);
        prop_assert_eq!(ctx.complexity, deserialized.complexity);
        prop_assert_eq!(ctx.domain, deserialized.domain);
        prop_assert_eq!(ctx.tags.len(), deserialized.tags.len());
    }

    /// Property: Default context has expected values
    #[test]
    fn prop_default_context() {
        let ctx = TaskContext::default();

        prop_assert!(ctx.language.is_none());
        prop_assert!(ctx.framework.is_none());
        prop_assert_eq!(ctx.complexity, ComplexityLevel::Moderate);
        prop_assert_eq!(ctx.domain, "general");
        prop_assert!(ctx.tags.is_empty());
    }
}

// ============================================================================
// Reward Score Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 50,
        ..ProptestConfig::default()
    })]

    /// Property: Reward score serialization roundtrip
    #[test]
    fn prop_reward_score_roundtrip(score in reward_score_strategy()) {
        let json = serde_json::to_string(&score).expect("serialization should succeed");
        let deserialized: memory_core::RewardScore = serde_json::from_str(&json)
            .expect("deserialization should succeed");

        prop_assert!((score.total - deserialized.total).abs() < 0.0001);
        prop_assert!((score.base - deserialized.base).abs() < 0.0001);
        prop_assert!((score.efficiency - deserialized.efficiency).abs() < 0.0001);
    }
}

// ============================================================================
// Outcome Stats Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 50,
        ..ProptestConfig::default()
    })]

    /// Property: Success rate calculation is correct
    #[test]
    fn prop_outcome_stats_success_rate(
        success_count in 0usize..100usize,
        failure_count in 0usize..100usize,
    ) {
        let total_count = success_count + failure_count;
        let stats = OutcomeStats {
            success_count,
            failure_count,
            total_count,
            avg_duration_secs: 30.0,
        };

        let expected_rate = if total_count == 0 {
            0.0
        } else {
            success_count as f32 / total_count as f32
        };

        prop_assert!((stats.success_rate() - expected_rate).abs() < 0.0001);
    }

    /// Property: Success rate is in valid range
    #[test]
    fn prop_success_rate_bounds(
        success_count in 0usize..100usize,
        failure_count in 0usize..100usize,
    ) {
        let total_count = success_count + failure_count;
        let stats = OutcomeStats {
            success_count,
            failure_count,
            total_count,
            avg_duration_secs: 30.0,
        };

        let rate = stats.success_rate();
        prop_assert!(rate >= 0.0 && rate <= 1.0,
            "success rate should be in [0, 1]");
    }
}

// ============================================================================
// Pattern Application Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        max_shrink_iters: 50,
        ..ProptestConfig::default()
    })]

    /// Property: Application outcome is_success matches variant
    #[test]
    fn prop_application_outcome_is_success(outcome in application_outcome_strategy()) {
        match outcome {
            ApplicationOutcome::Helped => prop_assert!(outcome.is_success()),
            ApplicationOutcome::NoEffect |
            ApplicationOutcome::Hindered |
            ApplicationOutcome::Pending => prop_assert!(!outcome.is_success()),
        }
    }

    /// Property: Pattern application serialization roundtrip
    #[test]
    fn prop_pattern_application_roundtrip(app in pattern_application_strategy()) {
        let json = serde_json::to_string(&app).expect("serialization should succeed");
        let deserialized: PatternApplication = serde_json::from_str(&json)
            .expect("deserialization should succeed");

        prop_assert_eq!(app.pattern_id, deserialized.pattern_id);
        prop_assert_eq!(app.applied_at_step, deserialized.applied_at_step);
        prop_assert_eq!(app.outcome, deserialized.outcome);
        prop_assert_eq!(app.notes, deserialized.notes);
    }
}

// ============================================================================
// Integration Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 50,
        max_shrink_iters: 30,
        ..ProptestConfig::default()
    })]

    /// Property: Complete episode workflow maintains invariants
    ///
    /// Tests that a complete workflow of creating an episode,
    /// adding steps, adding tags, and completing maintains all
    /// expected invariants.
    #[test]
    fn prop_complete_workflow(
        task_desc in "[a-zA-Z0-9_ ]{10,200}",
        ctx in task_context_strategy(),
        task_type in task_type_strategy(),
        steps in proptest::collection::vec(execution_step_strategy(), 1..10),
        tags in proptest::collection::vec(tag_strategy(), 0..5),
        outcome in task_outcome_strategy(),
    ) {
        // Create episode
        let mut episode = Episode::new(task_desc.to_string(), ctx, task_type);

        // Verify initial state
        prop_assert!(!episode.is_complete());
        prop_assert!(episode.end_time.is_none());
        prop_assert!(episode.outcome.is_none());
        prop_assert!(episode.steps.is_empty());
        prop_assert!(episode.tags.is_empty());

        // Add steps
        for step in steps {
            episode.add_step(step);
        }
        prop_assert!(!episode.steps.is_empty());

        // Add tags
        for tag in &tags {
            let _ = episode.add_tag(tag.clone());
        }

        // Complete episode
        episode.complete(outcome);

        // Verify final state
        prop_assert!(episode.is_complete());
        prop_assert!(episode.end_time.is_some());
        prop_assert!(episode.outcome.is_some());
        prop_assert!(episode.duration().is_some());
        prop_assert!(episode.duration().unwrap().num_milliseconds() >= 0);
    }

    /// Property: Pattern relevance is consistent
    ///
    /// A pattern should be relevant to contexts that match its domain,
    /// language, or tags.
    #[test]
    fn prop_pattern_relevance_consistency(
        pattern in pattern_strategy(),
    ) {
        if let Some(pattern_ctx) = pattern.context() {
            // Pattern should be relevant to its own context
            prop_assert!(pattern.is_relevant_to(pattern_ctx));

            // Pattern should be relevant to same domain
            let same_domain_ctx = TaskContext {
                domain: pattern_ctx.domain.clone(),
                ..TaskContext::default()
            };
            prop_assert!(pattern.is_relevant_to(&same_domain_ctx));
        }
    }
}
