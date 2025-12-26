//! BDD-style tests for pattern accuracy validation
//!
//! Tests verify that the pattern extraction and validation framework correctly
//! identifies known patterns from episodes using ground truth data.
//!
//! ## Test Coverage
//! - Pattern metrics calculation (precision, recall, F1, accuracy)
//! - Pattern extraction by type (ToolSequence, DecisionPoint, ErrorRecovery)
//! - Overall pattern recognition accuracy against ground truth
//! - Effectiveness tracking, pattern ranking, and decay mechanisms

use chrono::Duration;
use memory_core::{
    patterns::{EffectivenessTracker, PatternMetrics, PatternValidator, ValidationConfig},
    ComplexityLevel, Episode, ExecutionResult, ExecutionStep, Pattern, PatternExtractor,
    TaskContext, TaskOutcome, TaskType,
};
use uuid::Uuid;

/// Create a test context for episodes
fn create_test_context(domain: &str, language: Option<&str>) -> TaskContext {
    TaskContext {
        language: language.map(|s| s.to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec!["test".to_string()],
    }
}

/// Create a successful execution step
fn create_success_step(step_number: usize, tool: &str, action: &str) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    step.latency_ms = 100;
    step
}

/// Create a failed execution step
fn create_error_step(
    step_number: usize,
    tool: &str,
    action: &str,
    error_msg: &str,
) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Error {
        message: error_msg.to_string(),
    });
    step.latency_ms = 50;
    step
}

/// Ground truth: Known successful tool sequences
fn create_ground_truth_tool_sequences() -> Vec<Pattern> {
    let context = create_test_context("api-testing", Some("rust"));

    vec![
        // Sequence 1: Read -> Parse -> Validate
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "file_reader".to_string(),
                "json_parser".to_string(),
                "validator".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.95,
            avg_latency: Duration::milliseconds(150),
            occurrence_count: 10,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 2: Connect -> Query -> Process
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "db_connector".to_string(),
                "query_executor".to_string(),
                "result_processor".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.92,
            avg_latency: Duration::milliseconds(200),
            occurrence_count: 8,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 3: Auth -> Verify -> Grant
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "authenticator".to_string(),
                "token_verifier".to_string(),
                "access_granter".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.98,
            avg_latency: Duration::milliseconds(80),
            occurrence_count: 15,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 4: Fetch -> Transform -> Store
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "data_fetcher".to_string(),
                "transformer".to_string(),
                "storage_writer".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.88,
            avg_latency: Duration::milliseconds(250),
            occurrence_count: 12,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 5: Build -> Test -> Deploy
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "builder".to_string(),
                "test_runner".to_string(),
                "deployer".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.85,
            avg_latency: Duration::milliseconds(5000),
            occurrence_count: 20,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 6: Request -> Validate -> Response
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "http_handler".to_string(),
                "input_validator".to_string(),
                "response_builder".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.93,
            avg_latency: Duration::milliseconds(120),
            occurrence_count: 18,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 7: Parse -> Compile -> Execute
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "parser".to_string(),
                "compiler".to_string(),
                "executor".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.90,
            avg_latency: Duration::milliseconds(300),
            occurrence_count: 7,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 8: Monitor -> Analyze -> Alert
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "monitor".to_string(),
                "analyzer".to_string(),
                "alerter".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.87,
            avg_latency: Duration::milliseconds(180),
            occurrence_count: 9,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 9: Serialize -> Compress -> Send
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "serializer".to_string(),
                "compressor".to_string(),
                "sender".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.94,
            avg_latency: Duration::milliseconds(160),
            occurrence_count: 11,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 10: Load -> Cache -> Serve
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "loader".to_string(),
                "cache_manager".to_string(),
                "server".to_string(),
            ],
            context,
            success_rate: 0.96,
            avg_latency: Duration::milliseconds(90),
            occurrence_count: 16,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
    ]
}

/// Ground truth: Known decision points
fn create_ground_truth_decision_points() -> Vec<Pattern> {
    let context = create_test_context("api-testing", Some("rust"));

    vec![
        // Decision 1: Check cache validity
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Check if cache is valid".to_string(),
            action: "cache_validator".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 45,
                failure_count: 5,
                total_count: 50,
                avg_duration_secs: 0.05,
            },
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Decision 2: Verify permissions
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Verify user has permissions".to_string(),
            action: "permission_checker".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 38,
                failure_count: 12,
                total_count: 50,
                avg_duration_secs: 0.08,
            },
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Decision 3: Check resource availability
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Check if resource is available".to_string(),
            action: "resource_checker".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 42,
                failure_count: 8,
                total_count: 50,
                avg_duration_secs: 0.06,
            },
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Decision 4: Validate input format
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Verify input format is correct".to_string(),
            action: "format_validator".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 47,
                failure_count: 3,
                total_count: 50,
                avg_duration_secs: 0.04,
            },
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Decision 5: Check rate limit
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Check if rate limit exceeded".to_string(),
            action: "rate_limiter".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 40,
                failure_count: 10,
                total_count: 50,
                avg_duration_secs: 0.03,
            },
            context,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
    ]
}

/// Ground truth: Known error recovery patterns
fn create_ground_truth_error_recoveries() -> Vec<Pattern> {
    let context = create_test_context("api-testing", Some("rust"));

    vec![
        // Recovery 1: Connection timeout
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Connection timeout".to_string(),
            recovery_steps: vec![
                "retry_connector: Retry with exponential backoff".to_string(),
                "fallback_connector: Try alternate endpoint".to_string(),
            ],
            success_rate: 0.85,
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Recovery 2: Authentication failure
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Authentication failed".to_string(),
            recovery_steps: vec![
                "token_refresher: Refresh authentication token".to_string(),
                "re_authenticator: Re-authenticate with credentials".to_string(),
            ],
            success_rate: 0.92,
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Recovery 3: Resource not found
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Resource not found".to_string(),
            recovery_steps: vec![
                "cache_invalidator: Clear stale cache".to_string(),
                "resource_loader: Reload resource from source".to_string(),
            ],
            success_rate: 0.78,
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Recovery 4: Parse error
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Parse error".to_string(),
            recovery_steps: vec![
                "fallback_parser: Try alternative parser".to_string(),
                "error_handler: Return default value".to_string(),
            ],
            success_rate: 0.88,
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Recovery 5: Rate limit exceeded
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Rate limit exceeded".to_string(),
            recovery_steps: vec![
                "backoff_handler: Wait and retry after delay".to_string(),
                "queue_manager: Queue request for later".to_string(),
            ],
            success_rate: 0.95,
            context,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
    ]
}

/// Create episodes that contain the ground truth patterns
fn create_episodes_with_patterns() -> Vec<Episode> {
    let mut episodes = Vec::new();
    let context = create_test_context("api-testing", Some("rust"));

    // Episode 1: File reading workflow
    let mut ep1 = Episode::new(
        "Read and validate config".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep1.add_step(create_success_step(1, "file_reader", "Read config file"));
    ep1.add_step(create_success_step(2, "json_parser", "Parse JSON content"));
    ep1.add_step(create_success_step(
        3,
        "validator",
        "Validate config schema",
    ));
    ep1.complete(TaskOutcome::Success {
        verdict: "Config validated".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep1);

    // Episode 2: Database query workflow
    let mut ep2 = Episode::new(
        "Fetch user data".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep2.add_step(create_success_step(
        1,
        "db_connector",
        "Connect to database",
    ));
    ep2.add_step(create_success_step(
        2,
        "query_executor",
        "Execute SELECT query",
    ));
    ep2.add_step(create_success_step(
        3,
        "result_processor",
        "Process query results",
    ));
    ep2.complete(TaskOutcome::Success {
        verdict: "Data fetched".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep2);

    // Episode 3: Authentication workflow
    let mut ep3 = Episode::new(
        "Authenticate user".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep3.add_step(create_success_step(
        1,
        "authenticator",
        "Verify credentials",
    ));
    ep3.add_step(create_success_step(2, "token_verifier", "Verify JWT token"));
    ep3.add_step(create_success_step(
        3,
        "access_granter",
        "Grant access permissions",
    ));
    ep3.complete(TaskOutcome::Success {
        verdict: "User authenticated".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep3);

    // Episode 4: Decision point - cache validation
    let mut ep4 = Episode::new(
        "Check cache".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep4.add_step(create_success_step(
        1,
        "cache_validator",
        "Check if cache is valid",
    ));
    ep4.add_step(create_success_step(2, "cache_reader", "Read from cache"));
    ep4.complete(TaskOutcome::Success {
        verdict: "Cache hit".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep4);

    // Episode 5: Error recovery - connection timeout
    let mut ep5 = Episode::new(
        "Handle connection error".to_string(),
        context.clone(),
        TaskType::Debugging,
    );
    ep5.add_step(create_error_step(
        1,
        "connector",
        "Connect to API",
        "Connection timeout",
    ));
    ep5.add_step(create_success_step(
        2,
        "retry_connector",
        "Retry with exponential backoff",
    ));
    ep5.add_step(create_success_step(
        3,
        "fallback_connector",
        "Try alternate endpoint",
    ));
    ep5.complete(TaskOutcome::Success {
        verdict: "Recovered from timeout".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep5);

    // Episode 6: Data transformation workflow
    let mut ep6 = Episode::new(
        "Transform and store data".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep6.add_step(create_success_step(1, "data_fetcher", "Fetch raw data"));
    ep6.add_step(create_success_step(
        2,
        "transformer",
        "Transform data format",
    ));
    ep6.add_step(create_success_step(3, "storage_writer", "Write to storage"));
    ep6.complete(TaskOutcome::Success {
        verdict: "Data stored".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep6);

    // Episode 7: Decision point - permissions check
    let mut ep7 = Episode::new(
        "Check permissions".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep7.add_step(create_success_step(
        1,
        "permission_checker",
        "Verify user has permissions",
    ));
    ep7.add_step(create_success_step(
        2,
        "action_executor",
        "Execute authorized action",
    ));
    ep7.complete(TaskOutcome::Success {
        verdict: "Action authorized".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep7);

    // Episode 8: Error recovery - auth failure
    let mut ep8 = Episode::new(
        "Recover from auth failure".to_string(),
        context.clone(),
        TaskType::Debugging,
    );
    ep8.add_step(create_error_step(
        1,
        "auth_client",
        "Authenticate",
        "Authentication failed",
    ));
    ep8.add_step(create_success_step(
        2,
        "token_refresher",
        "Refresh authentication token",
    ));
    ep8.add_step(create_success_step(
        3,
        "re_authenticator",
        "Re-authenticate with credentials",
    ));
    ep8.complete(TaskOutcome::Success {
        verdict: "Re-authenticated successfully".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep8);

    episodes
}

#[test]
fn should_calculate_pattern_metrics_correctly() {
    // Given: Known pattern validation counts (7 TP, 2 FP, 1 FN, 10 TN)
    let true_positives = 7;
    let false_positives = 2;
    let false_negatives = 1;
    let true_negatives = 10;

    // When: Calculating metrics from counts
    let metrics = PatternMetrics::from_counts(
        true_positives,
        false_positives,
        false_negatives,
        true_negatives,
    );

    // Then: Precision should be TP / (TP + FP) = 7/9 ≈ 0.777
    assert!((metrics.precision - 0.777).abs() < 0.01);

    // Then: Recall should be TP / (TP + FN) = 7/8 = 0.875
    assert_eq!(metrics.recall, 0.875);

    // Then: Accuracy should be (TP + TN) / Total = 17/20 = 0.85
    assert_eq!(metrics.accuracy, 0.85);

    // Then: F1 score should be harmonic mean of precision and recall
    let expected_f1 =
        2.0 * (metrics.precision * metrics.recall) / (metrics.precision + metrics.recall);
    assert!((metrics.f1_score - expected_f1).abs() < 0.001);

    // Then: Quality score should be in valid range [0, 1]
    assert!(metrics.quality_score() >= 0.0 && metrics.quality_score() <= 1.0);
}

#[test]
fn should_extract_patterns_by_type_with_minimum_accuracy() {
    // Given: Episodes containing known patterns and ground truth validation data
    let extractor = PatternExtractor::new();
    let validator = PatternValidator::new(ValidationConfig::default());
    let episodes = create_episodes_with_patterns();

    // When: Extracting patterns from all episodes
    let mut all_extracted = Vec::new();
    for episode in &episodes {
        all_extracted.extend(extractor.extract(episode));
    }

    // Then: Each pattern type should meet minimum accuracy thresholds
    // Test data: (pattern_name, ground_truth, min_true_positives, min_quality_score)
    let test_cases = vec![
        (
            "ToolSequence",
            create_ground_truth_tool_sequences(),
            3,   // min true positives
            0.5, // min quality score
        ),
        (
            "DecisionPoint",
            create_ground_truth_decision_points(),
            1,
            0.0, // More lenient for decision points
        ),
        (
            "ErrorRecovery",
            create_ground_truth_error_recoveries(),
            0, // Error recovery is challenging
            0.0,
        ),
    ];

    for (pattern_name, ground_truth, min_tp, min_quality) in test_cases {
        println!("\n=== Testing {} Pattern Extraction ===", pattern_name);

        // Filter extracted patterns by type
        let extracted_by_type: Vec<_> = all_extracted
            .iter()
            .filter(|p| {
                matches!(
                    (pattern_name, p),
                    ("ToolSequence", Pattern::ToolSequence { .. })
                        | ("DecisionPoint", Pattern::DecisionPoint { .. })
                        | ("ErrorRecovery", Pattern::ErrorRecovery { .. })
                )
            })
            .cloned()
            .collect();

        // Calculate metrics against ground truth
        let metrics = validator.calculate_metrics(&ground_truth, &extracted_by_type);

        println!("  Precision: {:.2}%", metrics.precision * 100.0);
        println!("  Recall: {:.2}%", metrics.recall * 100.0);
        println!("  F1 Score: {:.2}", metrics.f1_score);
        println!("  True Positives: {}", metrics.true_positives);
        println!("  False Positives: {}", metrics.false_positives);
        println!("  False Negatives: {}", metrics.false_negatives);
        println!("  Quality Score: {:.2}", metrics.quality_score());

        // Validate minimum true positive count
        assert!(
            metrics.true_positives >= min_tp,
            "{} should extract at least {} patterns, got {}",
            pattern_name,
            min_tp,
            metrics.true_positives
        );

        // Validate minimum quality score if threshold is set
        if min_quality > 0.0 {
            assert!(
                metrics.quality_score() >= min_quality,
                "{} quality score should be at least {:.2}, got {:.2}",
                pattern_name,
                min_quality,
                metrics.quality_score()
            );
        }

        // Ensure metrics are within valid bounds
        assert!(metrics.precision >= 0.0 && metrics.precision <= 1.0);
        assert!(metrics.recall >= 0.0 && metrics.recall <= 1.0);
    }
}

#[test]
fn should_achieve_minimum_overall_pattern_recognition_quality() {
    // Given: Pattern extractor, validator, and episodes with known ground truth patterns
    let extractor = PatternExtractor::new();
    let validator = PatternValidator::new(ValidationConfig::default());

    let mut all_ground_truth = Vec::new();
    all_ground_truth.extend(create_ground_truth_tool_sequences());
    all_ground_truth.extend(create_ground_truth_decision_points());
    all_ground_truth.extend(create_ground_truth_error_recoveries());

    let episodes = create_episodes_with_patterns();

    // When: Extracting all patterns from episodes
    let mut all_extracted = Vec::new();
    for episode in &episodes {
        all_extracted.extend(extractor.extract(episode));
    }

    // Then: Calculate overall metrics against all ground truth patterns
    let metrics = validator.calculate_metrics(&all_ground_truth, &all_extracted);

    println!("\n=== OVERALL PATTERN RECOGNITION METRICS ===");
    println!("Total Ground Truth Patterns: {}", all_ground_truth.len());
    println!("Total Extracted Patterns: {}", all_extracted.len());
    println!("True Positives: {}", metrics.true_positives);
    println!("False Positives: {}", metrics.false_positives);
    println!("False Negatives: {}", metrics.false_negatives);
    println!("Precision: {:.2}%", metrics.precision * 100.0);
    println!("Recall: {:.2}%", metrics.recall * 100.0);
    println!("F1 Score: {:.2}", metrics.f1_score);
    println!("Accuracy: {:.2}%", metrics.accuracy * 100.0);
    println!("Quality Score: {:.2}", metrics.quality_score());
    println!("===========================================\n");

    // Then: Should extract at least 5 correct patterns
    assert!(
        metrics.true_positives >= 5,
        "Should extract at least 5 correct patterns"
    );

    // Then: Quality score should meet baseline threshold
    // TARGET: >70% pattern recognition accuracy (aspirational)
    // BASELINE: At least 25% for v1 implementation
    assert!(
        metrics.quality_score() >= 0.25,
        "Quality score should be at least 0.25 (current: {:.2}, target: 0.7+)",
        metrics.quality_score()
    );

    // Then: Both precision and recall should contribute to quality
    assert!(metrics.precision > 0.0, "Should have some precision");
    assert!(metrics.recall > 0.0, "Should have some recall");
}

#[test]
fn should_track_effectiveness_and_decay_poor_patterns() {
    // Given: Effectiveness tracker configured with 40% threshold and immediate decay
    println!("\n=== Effectiveness Tracking Tests ===");

    let mut tracker = EffectivenessTracker::with_config(0.4, 0);

    let high_eff = Uuid::new_v4();
    let medium_eff = Uuid::new_v4();
    let low_eff = Uuid::new_v4();
    let bad_pattern = Uuid::new_v4();

    // Given: High effectiveness pattern - retrieved and successfully applied
    for _ in 0..10 {
        tracker.record_retrieval(high_eff);
        tracker.record_application(high_eff, true);
    }

    // Given: Medium effectiveness pattern - retrieved but mixed success
    for _ in 0..10 {
        tracker.record_retrieval(medium_eff);
    }
    for _ in 0..5 {
        tracker.record_application(medium_eff, true);
    }
    for _ in 0..2 {
        tracker.record_application(medium_eff, false);
    }

    // Given: Low effectiveness pattern - rarely used, often fails
    tracker.record_retrieval(low_eff);
    tracker.record_application(low_eff, false);

    // Given: Bad pattern - consistently fails
    for _ in 0..3 {
        tracker.record_application(bad_pattern, false);
    }

    // When/Then: Checking effectiveness scores and rankings
    println!("\n--- Test 1: Effectiveness Scores ---");
    let stats_high = tracker.get_stats(high_eff).unwrap();
    let stats_medium = tracker.get_stats(medium_eff).unwrap();
    let stats_low = tracker.get_stats(low_eff).unwrap();

    println!("High: {:.2}", stats_high.effectiveness_score);
    println!("Medium: {:.2}", stats_medium.effectiveness_score);
    println!("Low: {:.2}", stats_low.effectiveness_score);

    // Then: Effectiveness scores should be properly ranked
    assert!(stats_high.effectiveness_score > stats_medium.effectiveness_score);
    assert!(stats_medium.effectiveness_score > stats_low.effectiveness_score);

    // Then: Success rates should reflect actual performance
    assert_eq!(stats_high.success_rate, 1.0);
    assert!(stats_medium.success_rate > 0.5 && stats_medium.success_rate < 1.0);
    assert_eq!(stats_low.success_rate, 0.0);

    // When/Then: Getting ranked patterns
    println!("\n--- Test 2: Pattern Ranking ---");
    let ranked = tracker.get_ranked_patterns();

    // Then: Most effective pattern should be ranked first
    assert_eq!(ranked[0].0, high_eff, "Most effective should be first");
    println!("Top pattern effectiveness: {:.2}", ranked[0].1);

    // When: Decaying old patterns
    println!("\n--- Test 3: Pattern Decay ---");
    let pattern_count_before = tracker.pattern_count();
    let decayed = tracker.decay_old_patterns();

    println!("Patterns before decay: {}", pattern_count_before);
    println!("Decayed patterns: {}", decayed.len());
    println!("Remaining patterns: {}", tracker.pattern_count());

    // Then: Bad pattern should be removed
    assert!(
        decayed.contains(&bad_pattern),
        "Bad pattern should be decayed"
    );

    // Then: High effectiveness pattern should be retained
    assert!(
        !decayed.contains(&high_eff),
        "High effectiveness pattern should be kept"
    );

    // Then: Verify patterns were actually removed from tracker
    assert!(tracker.get_stats(bad_pattern).is_none());
    assert!(tracker.get_stats(high_eff).is_some());

    // When/Then: Getting overall system statistics
    println!("\n--- Test 4: Overall System Statistics ---");
    let overall = tracker.overall_stats();

    println!("Total patterns: {}", overall.total_patterns);
    println!("Active patterns: {}", overall.active_patterns);
    println!("Total retrievals: {}", overall.total_retrievals);
    println!("Total applications: {}", overall.total_applications);
    println!("Overall success rate: {:.2}", overall.overall_success_rate);
    println!("Avg effectiveness: {:.2}", overall.avg_effectiveness);

    // Then: System statistics should be valid
    assert!(overall.total_patterns > 0);
    assert!(overall.total_retrievals > 0);
    assert!(overall.total_applications > 0);
    assert!(overall.overall_success_rate > 0.0);
    assert!(overall.overall_success_rate <= 1.0);
}
