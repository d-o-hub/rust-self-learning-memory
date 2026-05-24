//! Quality metrics MCP tool module.

mod tool;
mod types;

pub use tool::QualityMetricsTool;
pub use types::{
    QualityMetricsInput, QualityMetricsOutput, QualitySummary, QualityTier, QualityTrend,
    TrendDirection,
};

#[cfg(test)]
mod security_logic_tests {
    use super::*;
    use do_memory_core::SelfLearningMemory;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_quality_threshold_clamping_internal() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tool = QualityMetricsTool::new(memory);

        // Test value > 1.0
        let input_high = QualityMetricsInput {
            time_range: "7d".to_string(),
            include_trends: true,
            quality_threshold: Some(5.0),
        };

        let result = tool.execute(input_high).await.unwrap();
        // The QualityMetricsOutput struct has a 'quality_threshold' field
        assert!((result.quality_threshold - 1.0).abs() < f32::EPSILON);

        // Test value < 0.0
        let input_low = QualityMetricsInput {
            time_range: "7d".to_string(),
            include_trends: true,
            quality_threshold: Some(-1.0),
        };
        let result_low = tool.execute(input_low).await.unwrap();
        assert!(result_low.quality_threshold.abs() < f32::EPSILON);
    }
}
