use super::*;

#[test]
fn test_compatibility_assessment() {
    let assessor = CompatibilityAssessor::default_config();

    let context = PatternContext {
        domain: "web-api".to_string(),
        data_quality: 0.85,
        occurrences: 10,
        temporal_stability: 0.9,
        available_memory_mb: 200,
        complexity: 0.5,
    };

    let assessment = assessor
        .assess_compatibility("test_pattern", "query_memory", &context)
        .unwrap();

    assert!(assessment.compatibility_score >= 0.0 && assessment.compatibility_score <= 1.0);
    assert!(assessment.confidence >= 0.0 && assessment.confidence <= 1.0);
}

#[test]
fn test_risk_level_determination() {
    let assessor = CompatibilityAssessor::default_config();

    let context = PatternContext {
        domain: "web-api".to_string(),
        data_quality: 0.9,
        occurrences: 20,
        temporal_stability: 0.95,
        available_memory_mb: 500,
        complexity: 0.3,
    };

    let assessment = assessor
        .assess_compatibility("test_pattern", "query_memory", &context)
        .unwrap();

    assert!(matches!(
        assessment.risk_level,
        RiskLevel::Low | RiskLevel::Medium
    ));
}

#[test]
fn test_risk_factor_identification() {
    let assessor = CompatibilityAssessor::default_config();

    let context = PatternContext {
        domain: "unsupported_domain".to_string(),
        data_quality: 0.3,
        occurrences: 1,
        temporal_stability: 0.5,
        available_memory_mb: 10,
        complexity: 0.9,
    };

    let assessment = assessor
        .assess_compatibility("test_pattern", "query_memory", &context)
        .unwrap();

    assert!(!assessment.risk_factors.is_empty());
}

#[test]
fn test_confidence_interval_computation() {
    let assessor = CompatibilityAssessor::default_config();

    let context = PatternContext {
        domain: "web-api".to_string(),
        data_quality: 0.8,
        occurrences: 100,
        temporal_stability: 0.9,
        available_memory_mb: 200,
        complexity: 0.5,
    };

    let assessment = assessor
        .assess_compatibility("test_pattern", "query_memory", &context)
        .unwrap();

    assert!(assessment.confidence_interval.0 >= 0.0);
    assert!(assessment.confidence_interval.1 <= 1.0);
    assert!(assessment.confidence_interval.0 <= assessment.compatibility_score);
    assert!(assessment.confidence_interval.1 >= assessment.compatibility_score);

    let interval_width = assessment.confidence_interval.1 - assessment.confidence_interval.0;
    assert!(interval_width < 0.3);
}

#[test]
fn test_batch_assessment() {
    let assessor = CompatibilityAssessor::default_config();

    let context = PatternContext {
        domain: "web-api".to_string(),
        data_quality: 0.8,
        occurrences: 10,
        temporal_stability: 0.9,
        available_memory_mb: 300,
        complexity: 0.5,
    };

    let tools = vec![
        "query_memory".to_string(),
        "analyze_patterns".to_string(),
        "advanced_pattern_analysis".to_string(),
    ];

    let assessments = assessor
        .batch_assess("test_pattern", &tools, &context)
        .unwrap();

    assert_eq!(assessments.len(), 3);
}

#[test]
fn test_best_tool_selection() {
    let assessor = CompatibilityAssessor::default_config();

    let context = PatternContext {
        domain: "data-processing".to_string(),
        data_quality: 0.85,
        occurrences: 15,
        temporal_stability: 0.95,
        available_memory_mb: 300,
        complexity: 0.6,
    };

    let tools = vec![
        "query_memory".to_string(),
        "analyze_patterns".to_string(),
        "advanced_pattern_analysis".to_string(),
    ];

    let best = assessor
        .get_best_tool("test_pattern", &tools, &context)
        .unwrap();

    assert!(best.is_some());
    let (tool_name, assessment) = best.unwrap();
    assert!(!tool_name.is_empty());
    assert!(matches!(
        assessment.risk_level,
        RiskLevel::Low | RiskLevel::Medium
    ));
}
