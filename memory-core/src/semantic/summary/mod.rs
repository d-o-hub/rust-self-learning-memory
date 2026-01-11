//! Episode semantic summarization.
//!
//! Provides semantic summarization of episodes into concise, searchable summaries
//! with key concepts, critical steps, and optional embeddings for retrieval.

mod extractors;
mod helpers;
mod summarizer;
mod types;

pub use extractors::{extract_key_concepts, extract_key_steps};
pub use helpers::{add_salient_features_summary, extract_step_number, is_stopword};
pub use summarizer::SemanticSummarizer;
pub use types::EpisodeSummary;
