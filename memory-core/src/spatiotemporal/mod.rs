//! Spatiotemporal memory organization for efficient episodic retrieval
//!
//! This module implements Phase 3 (Spatiotemporal Memory Organization) features
//! from the research integration plan, including:
//!
//! - Context-aware embeddings with task-specific adaptation
//! - Hierarchical spatiotemporal indexing (future)
//! - Coarse-to-fine retrieval strategy (future)
//! - MMR-based diversity maximization (future)
//!
//! # Phase 3 Goals
//!
//! - **Hierarchical Indexing**: Domain → Task Type → Temporal clusters
//! - **Coarse-to-Fine Retrieval**: Multi-level search for speed and accuracy
//! - **Diversity Maximization**: MMR algorithm to avoid redundant results
//! - **Context-Aware Embeddings**: Task-specific embedding adaptation
//!
//! # Current Status
//!
//! - ✅ Context-aware embeddings (Task 4.1, 4.2, 4.3)
//! - ✅ Diversity maximization (Task 3.1, 3.2) - **COMPLETE**
//! - ⏳ Hierarchical indexing (Task 1.1, 1.2, 1.3)
//! - ⏳ Coarse-to-fine retrieval (Task 2.1, 2.2, 2.3)

pub mod diversity;
pub mod embeddings;
pub mod index;
pub mod retriever;
pub mod types;

pub use diversity::{DiversityMaximizer, ScoredEpisode};
pub use embeddings::{ContextAwareEmbeddings, ContrastivePair, TaskAdapter};
pub use index::domain_index::DomainIndex;
pub use index::SpatiotemporalIndex;
pub use retriever::{HierarchicalRetriever, HierarchicalScore, RetrievalQuery};
pub use types::{TaskTypeIndex, TemporalCluster, TemporalGranularity};
