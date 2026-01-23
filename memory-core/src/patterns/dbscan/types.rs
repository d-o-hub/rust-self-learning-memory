//! # DBSCAN Types
//!
//! Type definitions for the DBSCAN anomaly detection system.

use crate::episode::Episode;

/// Configuration for DBSCAN algorithm
#[derive(Debug, Clone)]
pub struct DBSCANConfig {
    /// Epsilon: maximum distance between two points to be considered neighbors
    pub eps: f64,
    /// Minimum number of points required to form a dense region (cluster)
    pub min_samples: usize,
    /// Whether to use adaptive epsilon based on data distribution
    pub adaptive_eps: bool,
    /// Feature weights for distance calculation
    pub feature_weights: FeatureWeights,
    /// Minimum cluster size to be considered valid
    pub min_cluster_size: usize,
}

impl Default for DBSCANConfig {
    fn default() -> Self {
        Self {
            eps: 0.5,
            min_samples: 3,
            adaptive_eps: true,
            feature_weights: FeatureWeights::default(),
            min_cluster_size: 2,
        }
    }
}

/// Weights for different features in distance calculation
#[derive(Debug, Clone)]
pub struct FeatureWeights {
    /// Weight for context features (domain, language, etc.)
    pub context: f64,
    /// Weight for step count similarity
    pub step_count: f64,
    /// Weight for duration similarity
    pub duration: f64,
    /// Weight for outcome type similarity
    pub outcome: f64,
    /// Weight for task type similarity
    pub task_type: f64,
}

impl Default for FeatureWeights {
    fn default() -> Self {
        Self {
            context: 0.3,
            step_count: 0.2,
            duration: 0.2,
            outcome: 0.15,
            task_type: 0.15,
        }
    }
}

/// Result of DBSCAN clustering
#[derive(Debug, Clone)]
pub struct DBSCANClusterResult {
    /// Clusters found by DBSCAN
    pub clusters: Vec<EpisodeCluster>,
    /// Anomalies (points not belonging to any cluster)
    pub anomalies: Vec<Anomaly>,
    /// Number of iterations to converge
    pub iterations: usize,
    /// Statistics about the clustering
    pub stats: DBSCANStats,
}

/// Statistics about the DBSCAN clustering result
#[derive(Debug, Clone, Default)]
pub struct DBSCANStats {
    /// Total number of points processed
    pub total_points: usize,
    /// Number of points assigned to clusters
    pub clustered_points: usize,
    /// Number of anomalies detected
    pub anomaly_count: usize,
    /// Average distance to nearest cluster for anomalies
    pub avg_anomaly_distance: f64,
    /// Maximum distance to nearest cluster for anomalies
    pub max_anomaly_distance: f64,
}

/// A cluster of similar episodes
#[derive(Debug, Clone)]
pub struct EpisodeCluster {
    /// Unique identifier for this cluster
    pub id: usize,
    /// Episodes belonging to this cluster
    pub episodes: Vec<Episode>,
    /// Centroid (mean feature vector) of this cluster
    pub centroid: ClusterCentroid,
    /// Density score (points / radius)
    pub density: f64,
}

/// Centroid representation for a cluster
#[derive(Debug, Clone)]
pub struct ClusterCentroid {
    /// Average context encoding
    pub context_encoding: Vec<f64>,
    /// Average number of steps
    pub avg_steps: f64,
    /// Average duration in milliseconds
    pub avg_duration_ms: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Task type distribution
    pub task_type_encoding: Vec<f64>,
}

/// An anomalous episode that doesn't fit any cluster
#[derive(Debug, Clone)]
pub struct Anomaly {
    /// The anomalous episode
    pub episode: Episode,
    /// Distance to the nearest cluster centroid
    pub distance_to_cluster: f64,
    /// ID of the nearest cluster (if any)
    pub nearest_cluster_id: Option<usize>,
    /// Reason why this episode is anomalous
    pub reason: AnomalyReason,
}

/// Reason for flagging an episode as anomalous
#[derive(Debug, Clone)]
pub enum AnomalyReason {
    /// Point is isolated (fewer neighbors than `min_samples`)
    Isolated { neighbor_count: usize },
    /// Point is far from any cluster centroid
    Outlier { distance: f64, threshold: f64 },
    /// Point is in a sparse region
    SparseRegion { density: f64, threshold: f64 },
    /// Point differs significantly in multiple dimensions
    MultidimensionalOutlier { deviations: Vec<(String, f64)> },
}
