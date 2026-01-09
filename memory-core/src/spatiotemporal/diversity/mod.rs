//! Diversity Maximization using Maximal Marginal Relevance (MMR)
//!
//! Implements MMR algorithm to select diverse, non-redundant results that balance
//! relevance to the query with diversity among selected items.
//!
//! ## Algorithm
//!
//! MMR iteratively selects items with the highest score:
//!
//! ```text
//! MMR Score(e) = λ * Relevance(e) - (1-λ) * max(Similarity(e, selected_i))
//! ```
//!
//! Where:
//! - `λ` (lambda) controls the relevance/diversity trade-off (0.0 to 1.0)
//!   - λ = 1.0: Pure relevance (standard ranking)
//!   - λ = 0.0: Pure diversity (maximum dissimilarity)
//!   - λ = 0.7 (default): 70% relevance, 30% diversity
//! - `Relevance(e)`: Pre-computed relevance score (e.g., from retrieval)
//! - `Similarity(e, selected_i)`: Cosine similarity between embeddings
//!
//! ## Research Foundation
//!
//! Based on "Hierarchical Spatiotemporal Memory Organization for Efficient Episodic Retrieval"
//! (arXiv Nov 2025) - Diversity maximization achieves ≥0.7 diversity score while maintaining
//! retrieval accuracy.

mod maximizer;
#[cfg(test)]
mod tests;
mod types;

pub use maximizer::DiversityMaximizer;
pub use types::ScoredEpisode;
