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
    use super::tool::QualityMetricsTool;
    use super::types::QualityMetricsInput;
    use super::*;
    use do_memory_core::SelfLearningMemory;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_quality_threshold_clamping_internal() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tool = QualityMetricsTool::new(memory);

        let input = QualityMetricsInput {
            time_range: "7d".to_string(),
            include_trends: true,
            quality_threshold: Some(5.0),
        };

        let result = tool.execute(input).await.unwrap();
        assert!(result.quality_threshold <= 1.0);
    }
}

#[cfg(test)]
mod security_logic_tests {
    use super::*;
    use crate::mcp::tools::quality_metrics::tool::QualityMetricsTool;
    use crate::mcp::tools::quality_metrics::types::QualityMetricsInput;
    use do_memory_core::SelfLearningMemory;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_quality_threshold_clamping_internal() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tool = QualityMetricsTool::new(memory);

        let input = QualityMetricsInput {
            time_range: "7d".to_string(),
            include_trends: true,
            quality_threshold: Some(5.0),
        };

        let result = tool.execute(input).await.unwrap();
        assert!(result.quality_threshold <= 1.0);
    }
}

#[cfg(test)]
mod security_logic_tests {
    use super::*;
    use crate::mcp::tools::quality_metrics::tool::QualityMetricsTool;
    use crate::mcp::tools::quality_metrics::types::QualityMetricsInput;
    use do_memory_core::SelfLearningMemory;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_quality_threshold_clamping_internal() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tool = QualityMetricsTool::new(memory);

        let input = QualityMetricsInput {
            time_range: "7d".to_string(),
            include_trends: true,
            quality_threshold: Some(5.0),
        };

        let result = tool.execute(input).await.unwrap();
        // Check that it was clamped (using epsilon-based comparison to satisfy Clippy)
        assert!((result.quality_threshold - 1.0).abs() < f32::EPSILON);
    }
}

#[cfg(test)]
mod security_logic_tests {
    use super::*;
    use crate::mcp::tools::quality_metrics::tool::QualityMetricsTool;
    use crate::mcp::tools::quality_metrics::types::QualityMetricsInput;
    use do_memory_core::SelfLearningMemory;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_quality_threshold_clamping_internal() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tool = QualityMetricsTool::new(memory);

        let input = QualityMetricsInput {
            time_range: "7d".to_string(),
            include_trends: true,
            quality_threshold: Some(5.0),
        };

        let result = tool.execute(input).await.unwrap();
        assert!((result.quality_threshold - 1.0).abs() < f32::EPSILON);
    }
}
