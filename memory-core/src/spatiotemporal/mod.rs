//! Spatiotemporal memory organization for efficient episodic retrieval.
//!
//! This module provides a sophisticated hierarchical approach to organizing and retrieving
//! episodic memories. Unlike pure semantic retrieval which treats all memories as a flat
//! collection of vectors, spatiotemporal retrieval uses the natural structure of tasks
//! to progressively narrow the search space.
//!
//! # Why Spatiotemporal?
//!
//! Pure semantic retrieval often suffers from:
//! - **Context drift**: Finding semantically similar but contextually irrelevant tasks.
//! - **Recency neglect**: Older, less relevant tasks may outrank newer, more pertinent ones.
//! - **Scalability**: Scanning all embeddings becomes slow as memory grows.
//!
//! Spatiotemporal organization solves these by introducing a 4-level hierarchy:
//! 1. **Domain**: High-level logical grouping (e.g., "web-api", "frontend", "infrastructure").
//! 2. **Task Type**: Functional classification (e.g., `CodeGeneration`, `Debugging`, `Analysis`).
//! 3. **Temporal Clusters**: Time-based buckets (Weekly, Monthly, Quarterly) that favor recent patterns.
//! 4. **Semantic Similarity**: Fine-grained vector comparison within the filtered subset.
//!
//! # Key Components
//!
//! - [`SpatiotemporalIndex`]: A 3-level index structure (Domain -> TaskType -> Temporal).
//! - [`HierarchicalRetriever`]: Executes the coarse-to-fine retrieval strategy.
//! - [`DiversityMaximizer`]: Applies MMR (Maximal Marginal Relevance) to ensure result variety.
//! - [`ContextAwareEmbeddings`]: Task-specific embedding adaptation via contrastive learning.
//!
//! # Example: Basic Retrieval
//!
//! ```no_run
//! use do_memory_core::spatiotemporal::{HierarchicalRetriever, RetrievalQuery};
//! use do_memory_core::TaskType;
//! use std::collections::HashMap;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let retriever = HierarchicalRetriever::with_config(0.4, 5);
//!
//! let query = RetrievalQuery {
//!     query_text: "Fix authentication bug".to_string(),
//!     query_embedding: None, // Will fallback to text similarity if None
//!     domain: Some("identity-service".to_string()),
//!     task_type: Some(TaskType::Debugging),
//!     limit: 3,
//!     episode_embeddings: HashMap::new(),
//! };
//!
//! // let results = retriever.retrieve(&query, &all_episodes).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Scoring Composition
//!
//! The final relevance score is a weighted combination:
//! `Score = 0.3*Domain + 0.3*TaskType + W*Recency + (0.4-W)*Similarity`
//! where `W` is the `temporal_bias_weight` (default 0.3).

pub mod diversity;
pub mod embeddings;
pub mod index;
pub mod retriever;
pub mod types;

pub use diversity::{DiversityMaximizer, ScoredEpisode};
pub use embeddings::{ContextAwareEmbeddings, ContrastivePair, TaskAdapter};
pub use index::SpatiotemporalIndex;
pub use index::domain_index::DomainIndex;
pub use retriever::{HierarchicalRetriever, HierarchicalScore, RetrievalQuery};
pub use types::{TaskTypeIndex, TemporalCluster, TemporalGranularity};
