//! Pre-storage reasoning for episodic memory quality enhancement.
//!
//! This module implements the `PREMem` (Pre-Storage Reasoning for Episodic Memory)
//! approach from EMNLP 2025. It provides quality assessment and filtering before
//! storing episodes to improve memory quality and reduce noise.
//!
//! # Components
//!
//! - [`QualityAssessor`]: Assess episode quality before storage
//! - [`SalientExtractor`]: Extract key information from episodes
//!
//! # Examples
//!
//! ```no_run
//! use memory_core::pre_storage::{QualityAssessor, QualityConfig, SalientExtractor};
//! use memory_core::{Episode, TaskContext, TaskType};
//!
//! let config = QualityConfig::default();
//! let assessor = QualityAssessor::new(config);
//! let extractor = SalientExtractor::new();
//!
//! let episode = Episode::new(
//!     "Implement authentication".to_string(),
//!     TaskContext::default(),
//!     TaskType::CodeGeneration,
//! );
//!
//! let quality_score = assessor.assess_episode(&episode);
//! if quality_score >= 0.7 {
//!     let features = extractor.extract(&episode);
//!     println!("High-quality episode with {} salient features, storing...", features.count());
//! } else {
//!     println!("Low-quality episode, rejecting...");
//! }
//! ```

pub mod extractor;
pub mod quality;

pub use extractor::{SalientExtractor, SalientFeatures};
pub use quality::{QualityAssessor, QualityConfig, QualityFeature};
