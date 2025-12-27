//! # Quality Metrics MCP Tool
//!
//! This module provides the MCP tool integration for quality metrics tracking,
//! enabling monitoring of memory quality improvements and noise reduction through
//! the PREMem (Pre-Storage Reasoning for Episodic Memory) system.
//!
//! ## Features
//!
//! - Track average quality scores over time
//! - Calculate noise reduction rate (rejected episodes)
//! - Analyze quality trends (improving/stable/declining)
//! - Provide quality score distribution histogram
//! - Generate actionable recommendations

use crate::types::Tool;
use anyhow::{anyhow, Result};
use memory_core::pre_storage::{QualityAssessor, QualityConfig};
use memory_core::SelfLearningMemory;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// Input parameters for quality metrics query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsInput {
    /// Time range for metrics (e.g., "24h", "7d", "30d", "all")
    #[serde(default = "default_time_range")]
    pub time_range: String,
    /// Include quality trend analysis
    #[serde(default = "default_true")]
    pub include_trends: bool,
    /// Quality threshold to use for filtering (0.0-1.0)
    pub quality_threshold: Option<f32>,
}

fn default_time_range() -> String {
    "7d".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for QualityMetricsInput {
    fn default() -> Self {
        Self {
            time_range: default_time_range(),
            include_trends: true,
            quality_threshold: None,
        }
    }
}

/// Output from quality metrics query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsOutput {
    /// Average quality score across episodes
    pub average_quality_score: f32,
    /// Quality score distribution (buckets)
    pub quality_score_distribution: HashMap<String, usize>,
    /// Total episodes attempted in time range
    pub total_episodes_attempted: usize,
    /// Episodes that met quality threshold (accepted)
    pub episodes_accepted: usize,
    /// Episodes that failed quality threshold (rejected)
    pub episodes_rejected: usize,
    /// Noise reduction rate as percentage (0-100)
    pub noise_reduction_rate: f32,
    /// Quality trend analysis
    pub quality_trend: QualityTrend,
    /// Time period analyzed
    pub time_period: String,
    /// Recommendations for quality improvement
    pub recommendations: Vec<String>,
    /// Quality threshold used
    pub quality_threshold: f32,
}

/// Quality trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrend {
    /// Overall trend direction
    pub direction: TrendDirection,
    /// Recent quality scores for trend visualization
    pub recent_scores: Vec<f32>,
    /// Moving average of quality scores
    pub moving_average: f32,
    /// Confidence in trend assessment (0.0-1.0)
    pub confidence: f32,
    /// Change rate (positive = improving, negative = declining)
    pub change_rate: f32,
}

/// Trend direction classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Quality is improving over time
    Improving,
    /// Quality is stable (no significant change)
    Stable,
    /// Quality is declining over time
    Declining,
    /// Insufficient data to determine trend
    Unknown,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrendDirection::Improving => write!(f, "improving"),
            TrendDirection::Stable => write!(f, "stable"),
            TrendDirection::Declining => write!(f, "declining"),
            TrendDirection::Unknown => write!(f, "unknown"),
        }
    }
}

/// Quality metrics tool implementation
pub struct QualityMetricsTool {
    memory: Arc<SelfLearningMemory>,
}

impl QualityMetricsTool {
    /// Create a new quality metrics tool
    pub fn new(memory: Arc<SelfLearningMemory>) -> Self {
        Self { memory }
    }

    /// Get the tool definition for MCP
    pub fn tool_definition() -> Tool {
        Tool::new(
            "quality_metrics".to_string(),
            "Retrieve memory quality metrics and noise reduction statistics from the PREMem system"
                .to_string(),
            json!({
                "type": "object",
                "properties": {
                    "time_range": {
                        "type": "string",
                        "enum": ["24h", "7d", "30d", "90d", "all"],
                        "default": "7d",
                        "description": "Time range for metrics calculation"
                    },
                    "include_trends": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include quality trend analysis over time"
                    },
                    "quality_threshold": {
                        "type": "number",
                        "minimum": 0.0,
                        "maximum": 1.0,
                        "description": "Quality threshold to use (default: 0.7)"
                    }
                }
            }),
        )
    }

    /// Execute the quality metrics query
    #[instrument(skip(self, input), fields(time_range = %input.time_range))]
    pub async fn execute(&self, input: QualityMetricsInput) -> Result<QualityMetricsOutput> {
        info!(
            "Starting quality metrics calculation for time range: {}",
            input.time_range
        );

        // Parse time range and get episodes
        let time_cutoff = self.parse_time_range(&input.time_range)?;
        let episodes = self.get_episodes_in_range(time_cutoff).await?;

        if episodes.is_empty() {
            info!("No episodes found in time range");
            return Ok(self.empty_metrics(&input.time_range, input.quality_threshold));
        }

        info!("Analyzing {} episodes for quality metrics", episodes.len());

        // Initialize quality assessor
        let quality_threshold = input.quality_threshold.unwrap_or(0.7);
        let config = QualityConfig::new(quality_threshold);
        let assessor = QualityAssessor::new(config);

        // Calculate quality scores for all episodes
        let mut quality_scores = Vec::new();
        for episode in &episodes {
            let score = assessor.assess_episode(episode);
            quality_scores.push(score);
        }

        // Calculate metrics
        let average_quality_score = if quality_scores.is_empty() {
            0.0
        } else {
            quality_scores.iter().sum::<f32>() / quality_scores.len() as f32
        };

        let episodes_accepted = quality_scores
            .iter()
            .filter(|&&score| score >= quality_threshold)
            .count();
        let episodes_rejected = quality_scores.len() - episodes_accepted;
        let total_episodes_attempted = quality_scores.len();

        let noise_reduction_rate = if total_episodes_attempted > 0 {
            (episodes_rejected as f32 / total_episodes_attempted as f32) * 100.0
        } else {
            0.0
        };

        // Build quality score distribution
        let quality_score_distribution = self.build_distribution(&quality_scores);

        // Analyze trends if requested
        let quality_trend = if input.include_trends && quality_scores.len() >= 3 {
            self.analyze_trend(&quality_scores)
        } else {
            QualityTrend {
                direction: TrendDirection::Unknown,
                recent_scores: quality_scores.iter().take(10).copied().collect(),
                moving_average: average_quality_score,
                confidence: 0.0,
                change_rate: 0.0,
            }
        };

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            average_quality_score,
            noise_reduction_rate,
            &quality_trend,
            total_episodes_attempted,
        );

        let output = QualityMetricsOutput {
            average_quality_score,
            quality_score_distribution,
            total_episodes_attempted,
            episodes_accepted,
            episodes_rejected,
            noise_reduction_rate,
            quality_trend,
            time_period: input.time_range.clone(),
            recommendations,
            quality_threshold,
        };

        info!(
            "Quality metrics calculated: avg={:.3}, noise_reduction={:.1}%, trend={}",
            output.average_quality_score,
            output.noise_reduction_rate,
            output.quality_trend.direction
        );

        Ok(output)
    }

    /// Parse time range string into cutoff timestamp
    fn parse_time_range(&self, time_range: &str) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        let now = chrono::Utc::now();

        match time_range {
            "24h" => Ok(Some(now - chrono::Duration::hours(24))),
            "7d" => Ok(Some(now - chrono::Duration::days(7))),
            "30d" => Ok(Some(now - chrono::Duration::days(30))),
            "90d" => Ok(Some(now - chrono::Duration::days(90))),
            "all" => Ok(None),
            _ => Err(anyhow!(
                "Invalid time range: {}. Valid options: 24h, 7d, 30d, 90d, all",
                time_range
            )),
        }
    }

    /// Get episodes within the specified time range
    async fn get_episodes_in_range(
        &self,
        time_cutoff: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<memory_core::Episode>> {
        // Query memory for recent episodes
        let context = memory_core::TaskContext {
            domain: "all".to_string(),
            language: None,
            framework: None,
            complexity: memory_core::ComplexityLevel::Moderate,
            tags: vec![],
        };

        // Retrieve a large batch of recent episodes
        let mut episodes = self
            .memory
            .retrieve_relevant_context("all tasks".to_string(), context, 1000)
            .await;

        // Filter by time range if specified
        if let Some(cutoff) = time_cutoff {
            episodes.retain(|ep| ep.start_time >= cutoff);
        }

        debug!("Retrieved {} episodes for quality analysis", episodes.len());
        Ok(episodes)
    }

    /// Build quality score distribution histogram
    fn build_distribution(&self, scores: &[f32]) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        distribution.insert("0.0-0.3 (Low)".to_string(), 0);
        distribution.insert("0.3-0.5 (Below Average)".to_string(), 0);
        distribution.insert("0.5-0.7 (Average)".to_string(), 0);
        distribution.insert("0.7-0.9 (Good)".to_string(), 0);
        distribution.insert("0.9-1.0 (Excellent)".to_string(), 0);

        for &score in scores {
            let bucket = if score < 0.3 {
                "0.0-0.3 (Low)"
            } else if score < 0.5 {
                "0.3-0.5 (Below Average)"
            } else if score < 0.7 {
                "0.5-0.7 (Average)"
            } else if score < 0.9 {
                "0.7-0.9 (Good)"
            } else {
                "0.9-1.0 (Excellent)"
            };

            *distribution.get_mut(bucket).unwrap() += 1;
        }

        distribution
    }

    /// Analyze quality trend from scores
    fn analyze_trend(&self, scores: &[f32]) -> QualityTrend {
        if scores.len() < 3 {
            return QualityTrend {
                direction: TrendDirection::Unknown,
                recent_scores: scores.to_vec(),
                moving_average: scores.iter().sum::<f32>() / scores.len() as f32,
                confidence: 0.0,
                change_rate: 0.0,
            };
        }

        // Calculate moving average (last 30% of scores)
        let window_size = (scores.len() as f32 * 0.3).max(3.0) as usize;
        let recent_scores: Vec<f32> = scores
            .iter()
            .rev()
            .take(window_size)
            .rev()
            .copied()
            .collect();
        let moving_average = recent_scores.iter().sum::<f32>() / recent_scores.len() as f32;

        // Calculate trend using linear regression on recent scores
        let n = recent_scores.len() as f32;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = moving_average;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &score) in recent_scores.iter().enumerate() {
            let x_diff = i as f32 - x_mean;
            let y_diff = score - y_mean;
            numerator += x_diff * y_diff;
            denominator += x_diff * x_diff;
        }

        let slope = if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        };

        // Determine trend direction and confidence
        let (direction, confidence) = if slope.abs() < 0.01 {
            (TrendDirection::Stable, 0.7)
        } else if slope > 0.02 {
            (TrendDirection::Improving, (slope * 50.0).min(0.95))
        } else if slope < -0.02 {
            (TrendDirection::Declining, (slope.abs() * 50.0).min(0.95))
        } else {
            (TrendDirection::Stable, 0.6)
        };

        QualityTrend {
            direction,
            recent_scores: recent_scores.into_iter().take(20).collect(),
            moving_average,
            confidence,
            change_rate: slope * 100.0, // Convert to percentage
        }
    }

    /// Generate recommendations based on metrics
    fn generate_recommendations(
        &self,
        avg_quality: f32,
        noise_reduction: f32,
        trend: &QualityTrend,
        total_episodes: usize,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Data volume recommendations
        if total_episodes < 10 {
            recommendations.push(
                "Collect more episodes (< 10 total) for reliable quality metrics. Continue using the system.".to_string()
            );
        } else if total_episodes < 50 {
            recommendations.push(
                "Good data volume for initial analysis. Continue building episode history."
                    .to_string(),
            );
        }

        // Average quality recommendations
        if avg_quality < 0.5 {
            recommendations.push(
                "Low average quality score. Review task complexity and ensure tasks are meaningful.".to_string()
            );
            recommendations.push(
                "Consider adjusting quality threshold or improving task execution patterns."
                    .to_string(),
            );
        } else if (0.7..0.85).contains(&avg_quality) {
            recommendations.push(
                "Good average quality score. System is capturing valuable episodes.".to_string(),
            );
        } else if avg_quality >= 0.85 {
            recommendations.push(
                "Excellent average quality score! Memory system is highly effective.".to_string(),
            );
        }

        // Noise reduction recommendations
        if noise_reduction < 10.0 {
            recommendations.push(
                "Very low noise reduction (<10%). Most episodes are high quality. Consider slightly raising threshold.".to_string()
            );
        } else if noise_reduction > 50.0 {
            recommendations.push(
                format!("High noise reduction (>{:.0}%). Many episodes rejected. Consider lowering threshold or improving task quality.", noise_reduction)
            );
        } else {
            recommendations.push(format!(
                "Healthy noise reduction rate ({:.1}%). System is filtering effectively.",
                noise_reduction
            ));
        }

        // Trend-based recommendations
        match trend.direction {
            TrendDirection::Improving if trend.confidence > 0.7 => {
                recommendations.push(
                    "Quality trend is improving! Current practices are working well.".to_string(),
                );
            }
            TrendDirection::Declining if trend.confidence > 0.7 => {
                recommendations.push(
                    "Quality trend is declining. Review recent task execution patterns."
                        .to_string(),
                );
                recommendations.push(
                    "Investigate what changed in recent episodes causing quality drop.".to_string(),
                );
            }
            TrendDirection::Stable => {
                recommendations
                    .push("Quality is stable. Maintain current quality standards.".to_string());
            }
            _ => {}
        }

        // PREMem system recommendations
        if noise_reduction > 0.0 {
            recommendations.push(
                "PREMem system is actively filtering low-quality episodes. This improves memory efficiency.".to_string()
            );
        }

        recommendations
    }

    /// Create empty metrics for when no episodes are found
    fn empty_metrics(
        &self,
        time_range: &str,
        quality_threshold: Option<f32>,
    ) -> QualityMetricsOutput {
        let mut distribution = HashMap::new();
        distribution.insert("0.0-0.3 (Low)".to_string(), 0);
        distribution.insert("0.3-0.5 (Below Average)".to_string(), 0);
        distribution.insert("0.5-0.7 (Average)".to_string(), 0);
        distribution.insert("0.7-0.9 (Good)".to_string(), 0);
        distribution.insert("0.9-1.0 (Excellent)".to_string(), 0);

        QualityMetricsOutput {
            average_quality_score: 0.0,
            quality_score_distribution: distribution,
            total_episodes_attempted: 0,
            episodes_accepted: 0,
            episodes_rejected: 0,
            noise_reduction_rate: 0.0,
            quality_trend: QualityTrend {
                direction: TrendDirection::Unknown,
                recent_scores: vec![],
                moving_average: 0.0,
                confidence: 0.0,
                change_rate: 0.0,
            },
            time_period: time_range.to_string(),
            recommendations: vec![
                "No episodes found in time range. Start using the memory system to collect data."
                    .to_string(),
            ],
            quality_threshold: quality_threshold.unwrap_or(0.7),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memory() -> Arc<SelfLearningMemory> {
        Arc::new(SelfLearningMemory::new())
    }

    #[test]
    fn test_tool_definition() {
        let tool = QualityMetricsTool::tool_definition();
        assert_eq!(tool.name, "quality_metrics");
        assert!(!tool.description.is_empty());
        assert!(tool.input_schema.is_object());
    }

    #[test]
    fn test_time_range_parsing() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        assert!(tool.parse_time_range("24h").is_ok());
        assert!(tool.parse_time_range("7d").is_ok());
        assert!(tool.parse_time_range("30d").is_ok());
        assert!(tool.parse_time_range("90d").is_ok());
        assert!(tool.parse_time_range("all").is_ok());
        assert!(tool.parse_time_range("invalid").is_err());
    }

    #[test]
    fn test_distribution_building() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        let scores = vec![0.2, 0.4, 0.6, 0.8, 0.95, 0.15, 0.75, 0.85];
        let distribution = tool.build_distribution(&scores);

        assert_eq!(distribution.len(), 5);
        assert_eq!(distribution["0.0-0.3 (Low)"], 2); // 0.2, 0.15
        assert_eq!(distribution["0.3-0.5 (Below Average)"], 1); // 0.4
        assert_eq!(distribution["0.5-0.7 (Average)"], 1); // 0.6
        assert_eq!(distribution["0.7-0.9 (Good)"], 3); // 0.8, 0.75, 0.85
        assert_eq!(distribution["0.9-1.0 (Excellent)"], 1); // 0.95
    }

    #[test]
    fn test_trend_analysis_improving() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        // Improving trend: scores increasing over time
        let scores = vec![0.5, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8];
        let trend = tool.analyze_trend(&scores);

        assert_eq!(trend.direction, TrendDirection::Improving);
        assert!(trend.confidence > 0.5);
        assert!(trend.change_rate > 0.0);
    }

    #[test]
    fn test_trend_analysis_declining() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        // Declining trend: scores decreasing over time
        let scores = vec![0.8, 0.75, 0.7, 0.65, 0.6, 0.55, 0.5];
        let trend = tool.analyze_trend(&scores);

        assert_eq!(trend.direction, TrendDirection::Declining);
        assert!(trend.confidence > 0.5);
        assert!(trend.change_rate < 0.0);
    }

    #[test]
    fn test_trend_analysis_stable() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        // Stable trend: scores consistent
        let scores = vec![0.7, 0.71, 0.69, 0.7, 0.72, 0.68, 0.7];
        let trend = tool.analyze_trend(&scores);

        assert_eq!(trend.direction, TrendDirection::Stable);
    }

    #[test]
    fn test_trend_analysis_insufficient_data() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        let scores = vec![0.7, 0.8];
        let trend = tool.analyze_trend(&scores);

        assert_eq!(trend.direction, TrendDirection::Unknown);
        assert_eq!(trend.confidence, 0.0);
    }

    #[test]
    fn test_empty_metrics() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        let metrics = tool.empty_metrics("7d", Some(0.7));

        assert_eq!(metrics.total_episodes_attempted, 0);
        assert_eq!(metrics.episodes_accepted, 0);
        assert_eq!(metrics.episodes_rejected, 0);
        assert_eq!(metrics.noise_reduction_rate, 0.0);
        assert_eq!(metrics.quality_threshold, 0.7);
        assert!(!metrics.recommendations.is_empty());
    }

    #[test]
    fn test_recommendations_low_quality() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        let trend = QualityTrend {
            direction: TrendDirection::Stable,
            recent_scores: vec![],
            moving_average: 0.4,
            confidence: 0.7,
            change_rate: 0.0,
        };

        let recommendations = tool.generate_recommendations(0.4, 30.0, &trend, 100);

        assert!(recommendations
            .iter()
            .any(|r| r.contains("Low average quality")));
    }

    #[test]
    fn test_recommendations_high_noise() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        let trend = QualityTrend {
            direction: TrendDirection::Stable,
            recent_scores: vec![],
            moving_average: 0.6,
            confidence: 0.7,
            change_rate: 0.0,
        };

        let recommendations = tool.generate_recommendations(0.6, 60.0, &trend, 100);

        assert!(recommendations
            .iter()
            .any(|r| r.contains("High noise reduction")));
    }

    #[test]
    fn test_recommendations_improving_trend() {
        let memory = create_test_memory();
        let tool = QualityMetricsTool::new(memory);

        let trend = QualityTrend {
            direction: TrendDirection::Improving,
            recent_scores: vec![],
            moving_average: 0.8,
            confidence: 0.9,
            change_rate: 5.0,
        };

        let recommendations = tool.generate_recommendations(0.8, 25.0, &trend, 100);

        assert!(recommendations.iter().any(|r| r.contains("improving")));
    }
}
