//! Semantic summarization and embedding capabilities.
//!
//! This module provides semantic summarization of episodes into concise,
//! searchable summaries with key concepts, critical steps, and optional embeddings.
//!
//! # Components
//!
//! - [`SemanticSummarizer`]: Compress episodes into semantic summaries
//! - [`EpisodeSummary`]: Condensed episode representation with embeddings
//!
//! # Examples
//!
//! ```no_run
//! use memory_core::semantic::{SemanticSummarizer, EpisodeSummary};
//! use memory_core::{Episode, TaskContext, TaskType};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let summarizer = SemanticSummarizer::new();
//!
//! let episode = Episode::new(
//!     "Implement authentication".to_string(),
//!     TaskContext::default(),
//!     TaskType::CodeGeneration,
//! );
//!
//! let summary = summarizer.summarize_episode(&episode).await?;
//! println!("Summary: {}", summary.summary_text);
//! println!("Key concepts: {:?}", summary.key_concepts);
//! println!("Key steps: {:?}", summary.key_steps);
//! # Ok(())
//! # }
//! ```

pub mod summary;

pub use summary::{EpisodeSummary, SemanticSummarizer};
