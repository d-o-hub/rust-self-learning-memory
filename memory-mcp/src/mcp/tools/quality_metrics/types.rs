//! Types for quality metrics tool.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Quality metrics summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySummary {
    /// Overall quality score (0-100)
    pub overall_score: u8,
    /// Current quality tier
    pub quality_tier: QualityTier,
    /// Number of high-quality episodes in time range
    pub high_quality_episodes: usize,
    /// Number of low-quality episodes that were rejected
    pub low_quality_rejected: usize,
    /// Whether quality is improving
    pub is_improving: bool,
    /// Primary recommendation for quality improvement
    pub primary_recommendation: String,
}

/// Quality tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityTier {
    /// Score 0-25: Needs significant improvement
    Poor,
    /// Score 26-50: Below average
    Fair,
    /// Score 51-75: Good quality
    Good,
    /// Score 76-90: High quality
    Excellent,
    /// Score 91-100: Exceptional quality
    Outstanding,
}

impl From<f32> for TrendDirection {
    fn from(change_rate: f32) -> Self {
        if change_rate > 0.05 {
            Self::Improving
        } else if change_rate < -0.05 {
            Self::Declining
        } else {
            Self::Stable
        }
    }
}

impl From<u8> for QualityTier {
    fn from(score: u8) -> Self {
        match score {
            0..=25 => Self::Poor,
            26..=50 => Self::Fair,
            51..=75 => Self::Good,
            76..=90 => Self::Excellent,
            _ => Self::Outstanding,
        }
    }
}
