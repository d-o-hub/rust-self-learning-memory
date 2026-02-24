//! # Pattern Extraction Module
//!
//! This module extracts patterns from DBSCAN clusters and summarizes their characteristics.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::dbscan::{Cluster, ClusterLabel};

/// Extracted pattern from a cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPattern {
    /// Pattern ID
    pub id: String,
    /// Cluster ID this pattern was extracted from
    pub cluster_id: usize,
    /// Pattern description
    pub description: String,
    /// Cluster characteristics
    pub characteristics: ClusterCharacteristics,
    /// Pattern quality score (0-1)
    pub quality_score: f64,
    /// Pattern type
    pub pattern_type: PatternType,
    /// Variable names involved in this pattern
    pub variables: Vec<String>,
    /// Temporal range (start, end indices)
    pub temporal_range: (usize, usize),
}

/// Cluster characteristics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterCharacteristics {
    /// Number of points in cluster
    pub size: usize,
    /// Cluster centroid
    pub centroid: Vec<f64>,
    /// Cluster density
    pub density: f64,
    /// Cluster variance
    pub variance: f64,
    /// Cluster compactness (inverse of spread)
    pub compactness: f64,
    /// Time span of cluster
    pub time_span: f64,
}

/// Pattern type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Temporal pattern (trend, seasonality)
    Temporal { pattern: String },
    /// Anomaly pattern (outliers)
    Anomaly { severity: String },
    /// Stable pattern (consistent behavior)
    Stable { consistency: f64 },
    /// Transition pattern (change between states)
    Transition { from: String, to: String },
    /// Unknown pattern type
    Unknown,
}

/// Pattern extraction configuration
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Minimum cluster quality threshold
    pub min_quality: f64,
    /// Minimum cluster size
    pub min_cluster_size: usize,
    /// Whether to generate detailed descriptions
    pub verbose: bool,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            min_quality: 0.6,
            min_cluster_size: 3,
            verbose: true,
        }
    }
}

/// Pattern extraction engine
pub struct PatternExtractor {
    config: ExtractionConfig,
}

impl PatternExtractor {
    /// Create a new pattern extractor
    pub fn new(config: ExtractionConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(ExtractionConfig::default())
    }

    /// Extract patterns from DBSCAN clusters
    pub fn extract_patterns(
        &self,
        clusters: &[Cluster],
        labels: &[ClusterLabel],
        variable_names: &[String],
    ) -> Result<Vec<ExtractedPattern>> {
        let mut patterns = Vec::new();

        for (cluster_idx, cluster) in clusters.iter().enumerate() {
            // Check if cluster meets quality criteria
            if cluster.points.len() < self.config.min_cluster_size {
                continue;
            }

            let characteristics = self.compute_cluster_characteristics(cluster)?;

            let quality_score = self.compute_quality_score(cluster, &characteristics);

            if quality_score < self.config.min_quality {
                continue;
            }

            let pattern_type = self.classify_pattern_type(cluster, &characteristics);

            let description = if self.config.verbose {
                self.generate_detailed_description(cluster, &characteristics, &pattern_type)
            } else {
                self.generate_simple_description(cluster, &pattern_type)
            };

            let temporal_range = self.compute_temporal_range(cluster);

            let pattern = ExtractedPattern {
                id: format!("pattern_{}", cluster_idx),
                cluster_id: cluster_idx,
                description,
                characteristics,
                quality_score,
                pattern_type,
                variables: variable_names.to_vec(),
                temporal_range,
            };

            patterns.push(pattern);
        }

        // Also extract noise patterns (anomalies)
        let noise_patterns = self.extract_noise_patterns(labels, variable_names)?;
        patterns.extend(noise_patterns);

        Ok(patterns)
    }

    /// Compute cluster characteristics
    fn compute_cluster_characteristics(&self, cluster: &Cluster) -> Result<ClusterCharacteristics> {
        if cluster.points.is_empty() {
            anyhow::bail!("Cannot compute characteristics for empty cluster");
        }

        // Size
        let size = cluster.points.len();

        // Centroid (already computed)
        let centroid = cluster.centroid.clone();

        // Density
        let density = cluster.density;

        // Variance (average squared distance from centroid)
        let variance = if size > 1 {
            cluster
                .points
                .iter()
                .map(|p| {
                    p.features
                        .iter()
                        .zip(&centroid)
                        .map(|(&x, &c)| (x - c).powi(2))
                        .sum::<f64>()
                })
                .sum::<f64>()
                / size as f64
        } else {
            0.0
        };

        // Compactness (inverse of variance, normalized)
        let compactness = if variance > 0.0 {
            1.0 / (1.0 + variance)
        } else {
            1.0
        };

        // Time span
        let time_span = if size > 1 {
            let min_time = cluster
                .points
                .iter()
                .map(|p| p.timestamp)
                .fold(f64::INFINITY, f64::min);
            let max_time = cluster
                .points
                .iter()
                .map(|p| p.timestamp)
                .fold(f64::NEG_INFINITY, f64::max);
            max_time - min_time
        } else {
            0.0
        };

        Ok(ClusterCharacteristics {
            size,
            centroid,
            density,
            variance,
            compactness,
            time_span,
        })
    }

    /// Compute quality score for a cluster
    fn compute_quality_score(
        &self,
        cluster: &Cluster,
        characteristics: &ClusterCharacteristics,
    ) -> f64 {
        // Quality factors:
        // 1. Size (more points = better, up to a threshold)
        let size_score = (cluster.points.len() as f64).ln() / 10.0;
        let size_score = size_score.clamp(0.0, 1.0);

        // 2. Density (higher density = better)
        let density_score = characteristics.density.clamp(0.0, 1.0);

        // 3. Compactness (higher compactness = better)
        let compactness_score = characteristics.compactness;

        // 4. Stability (lower variance = better)
        let stability_score = 1.0 / (1.0 + characteristics.variance);

        // Weighted combination
        0.3 * size_score + 0.3 * density_score + 0.2 * compactness_score + 0.2 * stability_score
    }

    /// Classify pattern type
    fn classify_pattern_type(
        &self,
        cluster: &Cluster,
        characteristics: &ClusterCharacteristics,
    ) -> PatternType {
        // Check for temporal pattern
        if characteristics.time_span > cluster.points.len() as f64 * 0.5 {
            // Cluster spans significant time

            // Check for trend
            let first_point = &cluster.points[0];
            let last_point = &cluster.points[cluster.points.len() - 1];

            if !first_point.features.is_empty() && !last_point.features.is_empty() {
                let trend = last_point.features[0] - first_point.features[0];

                if trend.abs() > 0.1 {
                    return PatternType::Temporal {
                        pattern: if trend > 0.0 {
                            "increasing_trend".to_string()
                        } else {
                            "decreasing_trend".to_string()
                        },
                    };
                }
            }

            return PatternType::Temporal {
                pattern: "temporal_pattern".to_string(),
            };
        }

        // Check for stable pattern
        if characteristics.compactness > 0.8 && characteristics.variance < 0.5 {
            return PatternType::Stable {
                consistency: characteristics.compactness,
            };
        }

        // Default to unknown
        PatternType::Unknown
    }

    /// Generate detailed description
    fn generate_detailed_description(
        &self,
        cluster: &Cluster,
        characteristics: &ClusterCharacteristics,
        pattern_type: &PatternType,
    ) -> String {
        let mut desc = String::new();

        desc.push_str(&format!(
            "Cluster {} contains {} points with density {:.2}. ",
            cluster.id, characteristics.size, characteristics.density
        ));

        desc.push_str(&format!(
            "Centroid: {:?}, Variance: {:.2}, Compactness: {:.2}. ",
            characteristics.centroid, characteristics.variance, characteristics.compactness
        ));

        match pattern_type {
            PatternType::Temporal { pattern } => {
                desc.push_str(&format!("Pattern type: Temporal ({})", pattern));
            }
            PatternType::Anomaly { severity } => {
                desc.push_str(&format!("Pattern type: Anomaly (severity: {})", severity));
            }
            PatternType::Stable { consistency } => {
                desc.push_str(&format!(
                    "Pattern type: Stable (consistency: {:.2})",
                    consistency
                ));
            }
            PatternType::Transition { from, to } => {
                desc.push_str(&format!("Pattern type: Transition ({} -> {})", from, to));
            }
            PatternType::Unknown => {
                desc.push_str("Pattern type: Unknown");
            }
        }

        desc
    }

    /// Generate simple description
    fn generate_simple_description(&self, cluster: &Cluster, pattern_type: &PatternType) -> String {
        match pattern_type {
            PatternType::Temporal { pattern } => {
                format!(
                    "Temporal pattern: {} ({} points)",
                    pattern,
                    cluster.points.len()
                )
            }
            PatternType::Anomaly { severity } => {
                format!(
                    "Anomaly detected: {} ({} points)",
                    severity,
                    cluster.points.len()
                )
            }
            PatternType::Stable { consistency } => {
                format!(
                    "Stable pattern: {:.2}% consistency ({} points)",
                    consistency * 100.0,
                    cluster.points.len()
                )
            }
            PatternType::Transition { from, to } => {
                format!(
                    "Transition: {} -> {} ({} points)",
                    from,
                    to,
                    cluster.points.len()
                )
            }
            PatternType::Unknown => {
                format!(
                    "Cluster {} with {} points",
                    cluster.id,
                    cluster.points.len()
                )
            }
        }
    }

    /// Compute temporal range of a cluster
    fn compute_temporal_range(&self, cluster: &Cluster) -> (usize, usize) {
        if cluster.points.is_empty() {
            return (0, 0);
        }

        let min_idx = cluster.points.iter().map(|p| p.id).min().unwrap_or(0);
        let max_idx = cluster.points.iter().map(|p| p.id).max().unwrap_or(0);

        (min_idx, max_idx)
    }

    /// Extract noise patterns (anomalies)
    fn extract_noise_patterns(
        &self,
        labels: &[ClusterLabel],
        variable_names: &[String],
    ) -> Result<Vec<ExtractedPattern>> {
        let mut noise_indices = Vec::new();

        for (i, label) in labels.iter().enumerate() {
            if matches!(label, ClusterLabel::Noise) {
                noise_indices.push(i);
            }
        }

        if noise_indices.is_empty() {
            return Ok(Vec::new());
        }

        // Create anomaly pattern
        let pattern = ExtractedPattern {
            id: "anomaly_pattern".to_string(),
            cluster_id: usize::MAX,
            description: format!(
                "Detected {} anomaly points across {} variables",
                noise_indices.len(),
                variable_names.len()
            ),
            characteristics: ClusterCharacteristics {
                size: noise_indices.len(),
                centroid: vec![0.0], // No meaningful centroid for noise
                density: 0.0,
                variance: f64::INFINITY,
                compactness: 0.0,
                time_span: 0.0,
            },
            quality_score: 0.5,
            pattern_type: PatternType::Anomaly {
                severity: if noise_indices.len() > 5 {
                    "high".to_string()
                } else if noise_indices.len() > 2 {
                    "medium".to_string()
                } else {
                    "low".to_string()
                },
            },
            variables: variable_names.to_vec(),
            temporal_range: (
                *noise_indices.first().unwrap_or(&0),
                *noise_indices.last().unwrap_or(&0),
            ),
        };

        Ok(vec![pattern])
    }

    /// Filter patterns by quality threshold
    pub fn filter_by_quality(&self, patterns: &[ExtractedPattern]) -> Vec<ExtractedPattern> {
        patterns
            .iter()
            .filter(|p| p.quality_score >= self.config.min_quality)
            .cloned()
            .collect()
    }

    /// Get pattern statistics
    pub fn get_pattern_stats(&self, patterns: &[ExtractedPattern]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        stats.insert("total_patterns".to_string(), patterns.len() as f64);

        let temporal_count = patterns
            .iter()
            .filter(|p| matches!(p.pattern_type, PatternType::Temporal { .. }))
            .count();
        stats.insert("temporal_patterns".to_string(), temporal_count as f64);

        let anomaly_count = patterns
            .iter()
            .filter(|p| matches!(p.pattern_type, PatternType::Anomaly { .. }))
            .count();
        stats.insert("anomaly_patterns".to_string(), anomaly_count as f64);

        let stable_count = patterns
            .iter()
            .filter(|p| matches!(p.pattern_type, PatternType::Stable { .. }))
            .count();
        stats.insert("stable_patterns".to_string(), stable_count as f64);

        let avg_quality = if patterns.is_empty() {
            0.0
        } else {
            patterns.iter().map(|p| p.quality_score).sum::<f64>() / patterns.len() as f64
        };
        stats.insert("average_quality".to_string(), avg_quality);

        stats
    }
}

#[cfg(test)]
mod tests;
