//! Tests for execution-signature retrieval (WG-121).

use super::*;

#[test]
fn test_signature_config_default() {
    let config = SignatureConfig::default();
    assert!((config.tool_weight - 0.4).abs() < 0.01);
    assert!((config.error_weight - 0.3).abs() < 0.01);
    assert!((config.structure_weight - 0.3).abs() < 0.01);
    assert!((config.min_overlap_threshold - 0.2).abs() < 0.01);
}

#[test]
fn test_signature_config_tool_focused() {
    let config = SignatureConfig::tool_focused();
    assert!((config.tool_weight - 0.6).abs() < 0.01);
}

#[test]
fn test_signature_config_error_focused() {
    let config = SignatureConfig::error_focused();
    assert!((config.error_weight - 0.6).abs() < 0.01);
}

#[test]
fn test_signature_config_balanced() {
    let config = SignatureConfig::balanced();
    assert!((config.tool_weight - 0.33).abs() < 0.01);
    assert!((config.error_weight - 0.33).abs() < 0.01);
    assert!((config.structure_weight - 0.34).abs() < 0.01);
}

#[test]
fn test_execution_signature_creation() {
    let episode_id = Uuid::new_v4();
    let sig = ExecutionSignature::new(episode_id);
    assert_eq!(sig.episode_id, episode_id);
    assert!(sig.tools.is_empty());
    assert!(sig.error_types.is_empty());
    assert_eq!(sig.total_steps, 0);
}

#[test]
fn test_execution_signature_add_tools() {
    let episode_id = Uuid::new_v4();
    let mut sig = ExecutionSignature::new(episode_id);
    sig.add_tool("Read");
    sig.add_tool("Bash");
    sig.add_tool("read"); // Duplicate (case-insensitive)

    assert_eq!(sig.tool_count(), 2);
    assert!(sig.has_tools());
    assert!(sig.tools.contains("read"));
    assert!(sig.tools.contains("bash"));
}

#[test]
fn test_execution_signature_add_errors() {
    let episode_id = Uuid::new_v4();
    let mut sig = ExecutionSignature::new(episode_id);
    sig.add_error("timeout");
    sig.add_error("Time Out"); // Normalized to same

    assert_eq!(sig.error_count(), 1);
    assert!(sig.has_errors());
    assert!(sig.error_types.contains("timeout"));
}

#[test]
fn test_execution_signature_record_steps() {
    let episode_id = Uuid::new_v4();
    let mut sig = ExecutionSignature::new(episode_id);
    sig.record_step(true);
    sig.record_step(true);
    sig.record_step(false);
    sig.record_step(true);

    assert_eq!(sig.total_steps, 4);
    assert_eq!(sig.successful_steps, 3);
    assert_eq!(sig.failed_steps, 1);
    assert!((sig.success_rate() - 0.75).abs() < 0.01);
}

#[test]
fn test_step_pattern_from_outcomes() {
    let pattern = StepPattern::from_outcomes(&[true, false, true, true]);
    assert_eq!(pattern.success_count, 3);
    assert_eq!(pattern.failure_count, 1);
    assert!(pattern.final_success);
    assert_eq!(pattern.pattern_code, "SFSS");
}

#[test]
fn test_step_pattern_similarity() {
    let p1 = StepPattern::from_outcomes(&[true, true, false, true]);
    let p2 = StepPattern::from_outcomes(&[true, true, false, true]);
    let p3 = StepPattern::from_outcomes(&[false, false, true, false]);

    // Same pattern should have high similarity
    let sim = p1.similarity(&p2);
    assert!(sim > 0.9);

    // Different pattern should have lower similarity
    let sim = p1.similarity(&p3);
    assert!(sim < 0.7);
}

#[test]
fn test_query_signature_creation() {
    let query = QuerySignature::new();
    assert!(query.expected_tools.is_empty());
    assert!(query.relevant_errors.is_empty());
    assert!(query.expected_pattern.is_none());
}

#[test]
fn test_query_signature_from_text() {
    let query = QuerySignature::from_query_text("Fix timeout error using cargo build");
    assert!(query.has_tool_expectations());
    assert!(query.expected_tools.contains("cargo"));
    assert!(query.has_error_expectations());
    assert!(query.relevant_errors.contains("timeout"));
}

#[test]
fn test_query_signature_add_tools() {
    let mut query = QuerySignature::new();
    query.add_tool("grep");
    query.add_tool("Bash");

    assert!(query.has_tool_expectations());
    assert!(query.expected_tools.contains("grep"));
    assert!(query.expected_tools.contains("bash"));
}

#[test]
fn test_signature_matcher_creation() {
    let matcher = SignatureMatcher::default_config();
    assert!((matcher.config().tool_weight - 0.4).abs() < 0.01);
}

#[test]
fn test_match_empty_signatures() {
    let matcher = SignatureMatcher::default_config();
    let query = QuerySignature::from_query_text("fix timeout");
    let matches = matcher.match_query(&query, &[]);

    assert!(matches.is_empty());
}

#[test]
fn test_match_single_signature() {
    let matcher = SignatureMatcher::default_config();
    let query = QuerySignature::from_query_text("cargo timeout");

    let episode_id = Uuid::new_v4();
    let mut sig = ExecutionSignature::new(episode_id);
    sig.add_tool("cargo");
    sig.add_error("timeout");
    sig.record_step(true);
    sig.record_step(true);

    let matches = matcher.match_query(&query, &[sig]);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].episode_id, episode_id);
    // Should have both tool and error overlap
    assert!(matches[0].tool_score > 0.0);
    assert!(matches[0].error_score > 0.0);
}

#[test]
fn test_match_multiple_signatures() {
    let matcher = SignatureMatcher::default_config();
    let query = QuerySignature::from_query_text("cargo test");

    // Signature with matching tool
    let ep1 = Uuid::new_v4();
    let mut sig1 = ExecutionSignature::new(ep1);
    sig1.add_tool("cargo");
    sig1.add_tool("test");
    sig1.record_step(true);
    sig1.record_step(true);

    // Signature with partial match
    let ep2 = Uuid::new_v4();
    let mut sig2 = ExecutionSignature::new(ep2);
    sig2.add_tool("cargo");
    sig2.record_step(false);

    // Signature with no match
    let ep3 = Uuid::new_v4();
    let mut sig3 = ExecutionSignature::new(ep3);
    sig3.add_tool("npm");
    sig3.record_step(true);

    let matches = matcher.match_query(&query, &[sig1, sig2, sig3]);

    assert!(matches.len() >= 2);
    // Should be sorted by score
    assert!(matches[0].score >= matches[1].score);
    // Best match should be ep1 (has both cargo and test)
    assert_eq!(matches[0].episode_id, ep1);
}

#[test]
fn test_match_below_threshold() {
    let config = SignatureConfig {
        min_overlap_threshold: 0.5,
        ..SignatureConfig::default()
    };
    let matcher = SignatureMatcher::new(config);
    let query = QuerySignature::from_query_text("npm");

    // Signature with unrelated tool
    let ep1 = Uuid::new_v4();
    let mut sig1 = ExecutionSignature::new(ep1);
    sig1.add_tool("cargo");

    let matches = matcher.match_query(&query, &[sig1]);

    // Should be filtered by threshold
    assert!(matches.is_empty());
}

#[test]
fn test_signature_match_is_strong() {
    let match_result = SignatureMatch {
        episode_id: Uuid::new_v4(),
        score: 0.6,
        tool_score: 0.5,
        error_score: 0.3,
        structure_score: 0.2,
        contributing_components: vec!["tools".to_string()],
    };

    assert!(match_result.is_strong_match());
    assert!(match_result.is_weak_match());
}

#[test]
fn test_signature_match_is_weak() {
    let match_result = SignatureMatch {
        episode_id: Uuid::new_v4(),
        score: 0.25,
        tool_score: 0.2,
        error_score: 0.1,
        structure_score: 0.1,
        contributing_components: vec!["tools".to_string()],
    };

    assert!(!match_result.is_strong_match());
    assert!(match_result.is_weak_match());
}

#[test]
fn test_normalize_error_type() {
    assert_eq!(normalize_error_type("timeout"), "timeout");
    assert_eq!(normalize_error_type("Time Out"), "timeout");
    assert_eq!(normalize_error_type("TIMED OUT"), "timeout");

    assert_eq!(normalize_error_type("panic"), "panic");
    assert_eq!(normalize_error_type("Panicked"), "panic");

    assert_eq!(normalize_error_type("deadlock"), "deadlock");
    assert_eq!(normalize_error_type("dead lock"), "deadlock");
}

#[test]
fn test_success_rate_empty() {
    let episode_id = Uuid::new_v4();
    let sig = ExecutionSignature::new(episode_id);
    assert!((sig.success_rate() - 0.0).abs() < 0.01);
}

#[test]
fn test_success_rate_full() {
    let episode_id = Uuid::new_v4();
    let mut sig = ExecutionSignature::new(episode_id);
    for _ in 0..10 {
        sig.record_step(true);
    }
    assert!((sig.success_rate() - 1.0).abs() < 0.01);
}

#[test]
fn test_error_overlap_no_query_errors() {
    let matcher = SignatureMatcher::default_config();
    let query = QuerySignature::new();

    let episode_id = Uuid::new_v4();
    let mut sig = ExecutionSignature::new(episode_id);
    sig.record_step(true);

    let match_result = matcher.match_query(&query, &[sig]);
    // Query with no error expectations should still match
    assert!(!match_result.is_empty());
}

#[test]
fn test_structure_similarity_without_expected_pattern() {
    let matcher = SignatureMatcher::default_config();
    let query = QuerySignature::new();

    // High success rate signature
    let ep1 = Uuid::new_v4();
    let mut sig1 = ExecutionSignature::new(ep1);
    for _ in 0..10 {
        sig1.record_step(true);
    }

    // Low success rate signature
    let ep2 = Uuid::new_v4();
    let mut sig2 = ExecutionSignature::new(ep2);
    for _ in 0..5 {
        sig2.record_step(false);
    }

    let matches = matcher.match_query(&query, &[sig1, sig2]);

    // High success rate should score higher
    assert!(!matches.is_empty());
    if matches.len() >= 2 {
        assert!(matches[0].episode_id == ep1);
    }
}

#[test]
fn test_contributing_components() {
    let matcher = SignatureMatcher::default_config();
    let mut query = QuerySignature::new();
    query.add_tool("cargo");
    query.add_error("timeout");

    let episode_id = Uuid::new_v4();
    let mut sig = ExecutionSignature::new(episode_id);
    sig.add_tool("cargo");
    sig.add_error("timeout");
    sig.record_step(true);

    let matches = matcher.match_query(&query, &[sig]);

    assert!(!matches.is_empty());
    // Should have both tools and errors as contributing
    assert!(
        matches[0]
            .contributing_components
            .contains(&"tools".to_string())
    );
    assert!(
        matches[0]
            .contributing_components
            .contains(&"errors".to_string())
    );
}
