//! # Predictive Analysis Module
//!
//! Provides forecasting models, anomaly detection, and causal inference capabilities
//! using advanced algorithms from augurs and deep_causality.

#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
use serde::{Deserialize, Serialize};

/// DBSCAN Anomaly Detection Implementation
/// Density-adaptive DBSCAN with KD-tree spatial indexing for efficient
/// real-time streaming anomaly detection in memory patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    /// Point ID for tracking
    pub id: usize,
    /// Time series values
    pub values: Vec<f64>,
    /// Semantic embedding (optional, for enhanced clustering)
    pub embedding: Option<Vec<f32>>,
    /// Temporal position
    pub timestamp: f64,
    /// Pre-calculated features for clustering
    pub features: Vec<f64>,
}

impl Point {
    /// Create a new point from time series data
    pub fn new(id: usize, values: &[f64], embedding: Option<Vec<f32>>, timestamp: f64) -> Self {
        let features = Self::extract_features(values);
        Self {
            id,
            values: values.to_vec(),
            embedding,
            timestamp,
            features,
        }
    }

    /// Extract features from time series values for clustering
    fn extract_features(values: &[f64]) -> Vec<f64> {
        if values.len() < 3 {
            return vec![0.0; 8]; // Default feature vector
        }

        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 =
            values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let mut features = Vec::with_capacity(8);

        // Statistical features
        features.push(mean);
        features.push(std_dev);

        // Trend features
        #[allow(clippy::cast_precision_loss)]
        let trend = if let (Some(&first_val), Some(&last_val)) = (values.first(), values.last()) {
            (last_val - first_val) / (values.len() - 1) as f64
        } else {
            0.0
        };
        features.push(trend);

        // Volatility (rolling standard deviation)
        let volatility = if values.len() > 3 {
            let window = values.len().min(5);
            let mut rolling_std = Vec::new();
            for i in (window - 1)..values.len() {
                let start = i.saturating_sub(window - 1);
                let window_data = &values[start..=i];
                let window_mean: f64 = window_data.iter().sum::<f64>() / window_data.len() as f64;
                let window_var: f64 = window_data
                    .iter()
                    .map(|&x| (x - window_mean).powi(2))
                    .sum::<f64>()
                    / window_data.len() as f64;
                rolling_std.push(window_var.sqrt());
            }
            rolling_std.iter().sum::<f64>() / rolling_std.len() as f64
        } else {
            std_dev
        };
        features.push(volatility);

        // Skewness
        let skewness = if std_dev > 0.0 {
            values
                .iter()
                .map(|&x| ((x - mean) / std_dev).powi(3))
                .sum::<f64>()
                / values.len() as f64
        } else {
            0.0
        };
        features.push(skewness);

        // Kurtosis
        let kurtosis = if std_dev > 0.0 {
            values
                .iter()
                .map(|&x| ((x - mean) / std_dev).powi(4))
                .sum::<f64>()
                / values.len() as f64
                - 3.0
        } else {
            0.0
        };
        features.push(kurtosis);

        // Autocorrelation (lag 1)
        let autocorr = if values.len() > 1 {
            let mut numerator = 0.0;
            let mut denom_x = 0.0;
            let mut denom_y = 0.0;
            for i in 1..values.len() {
                numerator += (values[i - 1] - mean) * (values[i] - mean);
                denom_x += (values[i - 1] - mean).powi(2);
                denom_y += (values[i] - mean).powi(2);
            }
            if denom_x > 0.0 && denom_y > 0.0 {
                numerator / (denom_x.sqrt() * denom_y.sqrt())
            } else {
                0.0
            }
        } else {
            0.0
        };
        features.push(autocorr);

        // Recent change (last 3 points trend)
        let recent_change = if values.len() >= 3 {
            let recent_mean: f64 = values.iter().rev().take(3).sum::<f64>() / 3.0;
            (recent_mean - mean) / std_dev.max(1e-6)
        } else {
            0.0
        };
        features.push(recent_change);

        features
    }
}

/// KD-tree node for spatial indexing
#[derive(Debug)]
struct KDNode {
    point: Point,
    axis: usize,
    left: Option<Box<KDNode>>,
    right: Option<Box<KDNode>>,
}

impl KDNode {
    fn new(point: Point, axis: usize) -> Self {
        Self {
            point,
            axis,
            left: None,
            right: None,
        }
    }

    /// Insert a new point into the KD-tree
    fn insert(&mut self, point: Point) {
        let axis = self.axis;
        let comparison = if point.features.len() > axis {
            point.features[axis] < self.point.features[axis]
        } else {
            point.id < self.point.id
        };

        if comparison {
            if let Some(left) = &mut self.left {
                left.insert(point);
            } else {
                self.left = Some(Box::new(KDNode::new(
                    point,
                    (axis + 1) % self.point.features.len(),
                )));
            }
        } else if let Some(right) = &mut self.right {
            right.insert(point);
        } else {
            self.right = Some(Box::new(KDNode::new(
                point,
                (axis + 1) % self.point.features.len(),
            )));
        }
    }

    /// Find neighbors within distance epsilon
    fn range_query(&self, epsilon: f64, center: &[f64], results: &mut Vec<Point>) {
        // Calculate distance to this node
        let distance = calculate_distance(&self.point.features, center);
        if distance <= epsilon {
            results.push(self.point.clone());
        }

        // Recurse to children
        let axis = self.axis;
        if self.point.features.len() > axis {
            let plane_distance = if center.len() > axis {
                (center[axis] - self.point.features[axis]).abs()
            } else {
                0.0
            };

            if plane_distance <= epsilon {
                if let Some(left) = &self.left {
                    left.range_query(epsilon, center, results);
                }
                if let Some(right) = &self.right {
                    right.range_query(epsilon, center, results);
                }
            } else {
                // Only search the side that could contain points
                if center.len() > axis && center[axis] < self.point.features[axis] {
                    if let Some(left) = &self.left {
                        left.range_query(epsilon, center, results);
                    }
                } else if let Some(right) = &self.right {
                    right.range_query(epsilon, center, results);
                }
            }
        }
    }
}

/// KD-tree spatial index for efficient neighbor queries
#[derive(Debug)]
pub struct KDTree {
    root: Option<Box<KDNode>>,
    #[allow(dead_code)]
    max_depth: usize,
}

impl KDTree {
    pub(crate) fn new() -> Self {
        Self {
            root: None,
            max_depth: 10,
        }
    }

    /// Build KD-tree from points
    pub fn build(points: &[Point]) -> Self {
        if points.is_empty() {
            return Self::new();
        }

        let mut tree = Self::new();
        let axis = 0; // Start with first dimension

        if let Some(root_point) = points.first() {
            tree.root = Some(Box::new(KDNode::new(root_point.clone(), axis)));

            for point in points.iter().skip(1) {
                if let Some(root) = tree.root.as_mut() {
                    root.insert(point.clone());
                }
            }
        }

        tree
    }

    /// Find neighbors within distance epsilon
    pub fn find_neighbors(&self, center: &[f64], epsilon: f64) -> Vec<Point> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            root.range_query(epsilon, center, &mut results);
        }
        results
    }
}

/// Calculate Euclidean distance between two feature vectors
pub fn calculate_distance(a: &[f64], b: &[f64]) -> f64 {
    let mut sum = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        let diff = x - y;
        sum += diff * diff;
    }
    sum.sqrt()
}
