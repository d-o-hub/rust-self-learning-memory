//! # Pattern Validation and Effectiveness Tracking
//!
//! This module provides tools for validating pattern extraction quality
//! and tracking pattern effectiveness over time.
//!
//! ## Components
//!
//! - `extractors`: Hybrid pattern extraction system with specialized extractors
//! - `validation`: Pattern accuracy metrics (precision, recall, F1)
//! - `effectiveness`: Pattern usage and success tracking
//! - `clustering`: Pattern clustering and deduplication

pub mod clustering;
pub mod effectiveness;
pub mod extractors;
pub mod optimized_validator;
pub mod validation;

pub use clustering::{ClusterCentroid, ClusteringConfig, EpisodeCluster, PatternClusterer};
pub use effectiveness::{EffectivenessTracker, PatternUsage, UsageStats};
pub use extractors::{
    ContextPatternExtractor, DecisionPointExtractor, ErrorRecoveryExtractor,
    HybridPatternExtractor, PatternExtractor, ToolSequenceExtractor,
};
pub use optimized_validator::{
    EnhancedPatternApplicator, OptimizedPatternValidator, RiskAssessment,
};
pub use validation::{PatternMetrics, PatternValidator, ValidationConfig};
