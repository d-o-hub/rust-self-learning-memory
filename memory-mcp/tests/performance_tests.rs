//! # Advanced Pattern Analysis Performance Tests
//!
//! Benchmarks the performance of the advanced pattern analysis tool.

use memory_core::SelfLearningMemory;
use memory_mcp::mcp::tools::advanced_pattern_analysis::{
    AdvancedPatternAnalysisInput, AdvancedPatternAnalysisTool, AnalysisConfig, AnalysisType,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Benchmark statistical analysis performance
#[tokio::test]
async fn benchmark_statistical_analysis() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Create test data of various sizes
    let sizes = vec![10, 50, 100, 500];

    for size in sizes {
        let mut data = HashMap::new();

        // Generate synthetic data
        for i in 0..5 {
            let series: Vec<f64> = (0..size).map(|x| (x as f64) + (i as f64) * 0.1).collect();
            data.insert(format!("var_{}", i), series);
        }

        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Statistical,
            time_series_data: data,
            config: Some(AnalysisConfig {
                parallel_processing: Some(false),
                max_data_points: Some(1000),
                ..Default::default()
            }),
        };

        let start = Instant::now();
        let result = tool.execute(input).await;
        let duration = start.elapsed();

        assert!(result.is_ok(), "Analysis should succeed for size {}", size);

        let output = result.unwrap();
        println!(
            "Statistical analysis size {}: {}ms",
            size, output.performance.total_time_ms
        );

        // Performance target: < 500ms for reasonable sizes
        if size <= 100 {
            assert!(
                duration.as_millis() < 500,
                "Analysis too slow for size {}: {}ms",
                size,
                duration.as_millis()
            );
        }
    }
}
