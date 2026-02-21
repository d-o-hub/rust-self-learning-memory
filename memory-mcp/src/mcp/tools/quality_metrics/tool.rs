//! Tool implementation for quality metrics.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use memory_core::SelfLearningMemory;
use memory_core::pre_storage::{QualityAssessor, QualityConfig};
use serde_json::json;
use tracing::{debug, info, instrument};

use crate::types::Tool;

use super::types::{QualityMetricsInput, QualityMetricsOutput, QualityTrend, TrendDirection};

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

        // Retrieve a large batch of recent episodes (returns Vec<Arc<Episode>>)
        let arc_episodes = self
            .memory
            .retrieve_relevant_context("all tasks".to_string(), context, 1000)
            .await;

        // Convert Vec<Arc<Episode>> to Vec<Episode> by cloning
        let mut episodes: Vec<memory_core::Episode> = arc_episodes
            .into_iter()
            .map(|arc_ep| arc_ep.as_ref().clone())
            .collect();

        // Filter by time range if specified
        if let Some(cutoff) = time_cutoff {
            episodes.retain(|ep| ep.start_time >= cutoff);
        }

        debug!("Retrieved {} episodes for quality analysis", episodes.len());
        Ok(episodes)
    }

    /// Build quality score distribution histogram
    #[allow(clippy::expect_used)]
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

            *distribution
                .get_mut(bucket)
                .expect("bucket exists: all buckets initialized in distribution HashMap") += 1;
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
            change_rate: slope * 100.0,
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
