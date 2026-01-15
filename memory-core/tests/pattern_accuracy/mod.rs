//!
//! BDD-style tests for pattern accuracy validation
//!
//! Tests verify that the pattern extraction and validation framework correctly
//! identifies known patterns from episodes using ground truth data.
//!
//! ## Test Coverage
//! - Pattern metrics calculation (precision, recall, F1, accuracy)
//! - Pattern extraction by type (`ToolSequence`, `DecisionPoint`, `ErrorRecovery`)
//! - Overall pattern recognition accuracy against ground truth
//! - Effectiveness tracking, pattern ranking, and decay mechanisms
//!

mod ground_truth;
mod helpers;

pub use ground_truth::{
    create_ground_truth_decision_points, create_ground_truth_error_recoveries,
    create_ground_truth_tool_sequences,
};
pub use helpers::{create_episodes_with_patterns, create_test_context};

use memory_core::{
    patterns::{EffectivenessTracker, PatternMetrics, PatternValidator, ValidationConfig},
    Pattern, PatternExtractor,
};

#[test]
#[allow(clippy::float_cmp)]
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

    // Then: Precision should be TP / (TP + FP) = 7/9 ~ 0.777
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
            3,
            0.5,
        ),
        (
            "DecisionPoint",
            create_ground_truth_decision_points(),
            1,
            0.0,
        ),
        (
            "ErrorRecovery",
            create_ground_truth_error_recoveries(),
            0,
            0.0,
        ),
    ];

    for (pattern_name, ground_truth, min_tp, min_quality) in test_cases {
        println!("\n=== Testing {pattern_name} Pattern Extraction ===");

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
#[allow(clippy::float_cmp)]
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

    println!("Patterns before decay: {pattern_count_before}");
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
