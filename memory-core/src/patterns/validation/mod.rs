//! # Pattern Validation and Accuracy Metrics
//!
//! Provides tools to validate pattern extraction quality using standard
//! classification metrics: precision, recall, F1 score, and accuracy.
//!
//! ## Example
//!
//! ```
//! use memory_core::patterns::validation::{PatternValidator, ValidationConfig};
//! use memory_core::Pattern;
//!
//! let validator = PatternValidator::new(ValidationConfig::default());
//!
//! let ground_truth = vec![/* known patterns */];
//! let extracted = vec![/* patterns from extractor */];
//!
//! let metrics = validator.calculate_metrics(&ground_truth, &extracted);
//! println!("Precision: {:.2}", metrics.precision);
//! println!("Recall: {:.2}", metrics.recall);
//! println!("F1 Score: {:.2}", metrics.f1_score);
//! ```

pub mod types;
pub mod validator;

pub use types::{PatternMetrics, PatternValidator, ValidationConfig};
