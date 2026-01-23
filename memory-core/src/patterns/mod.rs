//! # Pattern Validation and Effectiveness Tracking
//!
//! This module provides tools for validating pattern extraction quality
//! and tracking pattern effectiveness over time.
//!
//! ## Components
//!
//! - `changepoint`: Changepoint detection for pattern metric monitoring
//! - `clustering`: Pattern clustering and deduplication
//! - `dbscan`: DBSCAN-based anomaly detection for episodes
//! - `effectiveness`: Pattern usage and success tracking
//! - `extractors`: Hybrid pattern extraction system with specialized extractors
//! - `optimized_validator`: Enhanced pattern validation
//! - `validation`: Pattern accuracy metrics (precision, recall, F1)

pub mod changepoint;
pub mod clustering;
pub mod dbscan;
pub mod effectiveness;
pub mod extractors;
pub mod optimized_validator;
pub mod validation;

pub use changepoint::{
    ChangeDirection, ChangeType, Changepoint, ChangepointConfig, ChangepointDetector,
    SegmentComparison, SegmentComparisonConfig, SegmentStats,
};
pub use clustering::{ClusterCentroid, ClusteringConfig, EpisodeCluster, PatternClusterer};
pub use dbscan::{
    Anomaly, AnomalyReason, DBSCANAnomalyDetector, DBSCANClusterResult, DBSCANConfig, DBSCANStats,
    FeatureWeights,
};
pub use effectiveness::{EffectivenessTracker, OverallStats, PatternUsage, UsageStats};
pub use extractors::{
    ContextPatternExtractor, DecisionPointExtractor, ErrorRecoveryExtractor,
    HybridPatternExtractor, PatternExtractor, ToolSequenceExtractor,
};
pub use optimized_validator::{
    EnhancedPatternApplicator, OptimizedPatternValidator, RiskAssessment,
};
pub use validation::{PatternMetrics, PatternValidator, ValidationConfig};
