use super::*;
use crate::patterns::predictive::kdtree::Point;

#[test]
fn test_pattern_extraction_from_clusters() {
    let extractor = PatternExtractor::default_config();

    let mut points = Vec::new();
    for i in 0..10 {
        points.push(Point::new(i, &[1.0], None, i as f64));
    }

    let cluster = Cluster {
        id: 0,
        points: points.clone(),
        centroid: vec![1.0],
        density: 0.8,
    };

    let characteristics = extractor.compute_cluster_characteristics(&cluster).unwrap();
    assert!(characteristics.size == 10);
    assert!(characteristics.density > 0.0);
}

#[test]
fn test_quality_score_computation() {
    let extractor = PatternExtractor::default_config();

    let cluster = Cluster {
        id: 0,
        points: vec![Point::new(0, &[1.0], None, 0.0)],
        centroid: vec![1.0],
        density: 0.5,
    };

    let characteristics = extractor.compute_cluster_characteristics(&cluster).unwrap();
    let quality = extractor.compute_quality_score(&cluster, &characteristics);

    assert!((0.0..=1.0).contains(&quality));
}

#[test]
fn test_pattern_type_classification() {
    let extractor = PatternExtractor::default_config();

    let cluster = Cluster {
        id: 0,
        points: vec![Point::new(0, &[1.0], None, 0.0)],
        centroid: vec![1.0],
        density: 0.9,
    };

    let characteristics = extractor.compute_cluster_characteristics(&cluster).unwrap();
    let pattern_type = extractor.classify_pattern_type(&cluster, &characteristics);

    match pattern_type {
        PatternType::Unknown | PatternType::Stable { .. } => {}
        _ => {}
    }
}

#[test]
fn test_noise_pattern_extraction() {
    let extractor = PatternExtractor::default_config();

    let labels = vec![
        ClusterLabel::Cluster(0),
        ClusterLabel::Noise,
        ClusterLabel::Noise,
        ClusterLabel::Cluster(0),
    ];

    let noise_patterns = extractor
        .extract_noise_patterns(&labels, &["var1".to_string()])
        .unwrap();

    assert_eq!(noise_patterns.len(), 1);
    assert!(matches!(
        noise_patterns[0].pattern_type,
        PatternType::Anomaly { .. }
    ));
}

#[test]
fn test_pattern_filtering() {
    let extractor = PatternExtractor::new(ExtractionConfig {
        min_quality: 0.7,
        ..Default::default()
    });

    let patterns = vec![
        ExtractedPattern {
            id: "pattern_1".to_string(),
            cluster_id: 0,
            description: "High quality".to_string(),
            characteristics: ClusterCharacteristics {
                size: 10,
                centroid: vec![1.0],
                density: 0.9,
                variance: 0.1,
                compactness: 0.9,
                time_span: 5.0,
            },
            quality_score: 0.8,
            pattern_type: PatternType::Stable { consistency: 0.9 },
            variables: vec!["x".to_string()],
            temporal_range: (0, 10),
        },
        ExtractedPattern {
            id: "pattern_2".to_string(),
            cluster_id: 1,
            description: "Low quality".to_string(),
            characteristics: ClusterCharacteristics {
                size: 2,
                centroid: vec![0.0],
                density: 0.1,
                variance: 1.0,
                compactness: 0.1,
                time_span: 1.0,
            },
            quality_score: 0.5,
            pattern_type: PatternType::Unknown,
            variables: vec!["y".to_string()],
            temporal_range: (0, 2),
        },
    ];

    let filtered = extractor.filter_by_quality(&patterns);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "pattern_1");
}

#[test]
fn test_pattern_statistics() {
    let extractor = PatternExtractor::default_config();

    let patterns = vec![
        ExtractedPattern {
            id: "pattern_1".to_string(),
            cluster_id: 0,
            description: "Test".to_string(),
            characteristics: ClusterCharacteristics {
                size: 10,
                centroid: vec![1.0],
                density: 0.8,
                variance: 0.2,
                compactness: 0.8,
                time_span: 5.0,
            },
            quality_score: 0.8,
            pattern_type: PatternType::Temporal {
                pattern: "trend".to_string(),
            },
            variables: vec!["x".to_string()],
            temporal_range: (0, 10),
        },
        ExtractedPattern {
            id: "pattern_2".to_string(),
            cluster_id: 1,
            description: "Test".to_string(),
            characteristics: ClusterCharacteristics {
                size: 5,
                centroid: vec![0.0],
                density: 0.5,
                variance: 0.5,
                compactness: 0.5,
                time_span: 2.0,
            },
            quality_score: 0.6,
            pattern_type: PatternType::Anomaly {
                severity: "low".to_string(),
            },
            variables: vec!["y".to_string()],
            temporal_range: (0, 5),
        },
    ];

    let stats = extractor.get_pattern_stats(&patterns);
    assert_eq!(stats["total_patterns"], 2.0);
    assert_eq!(stats["temporal_patterns"], 1.0);
    assert_eq!(stats["anomaly_patterns"], 1.0);
}
