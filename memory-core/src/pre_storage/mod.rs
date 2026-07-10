//! Pre-storage ingest layer for episodic memory quality enhancement.
//!
//! This module handles the **ingest** stage of the memory pipeline:
//! `Episode → [pre_storage: ingest] → [storage: persistence] → Backend`
//!
//! Responsibilities: sanitize, enrich, normalize, and validate episodes
//! before they reach the persistence layer.
//!
//! Implements the `PREMem` (Pre-Storage Reasoning for Episodic Memory)
//! approach from EMNLP 2025.
//!
//! # Components
//!
//! - [`QualityAssessor`]: Assess episode quality before storage
//! - [`SalientExtractor`]: Extract key information from episodes
//!
//! # Examples
//!
//! ```no_run
//! use do_memory_core::pre_storage::{QualityAssessor, QualityConfig, SalientExtractor};
//! use do_memory_core::{Episode, TaskContext, TaskType};
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
