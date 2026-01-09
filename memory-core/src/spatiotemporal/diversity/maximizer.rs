//! DiversityMaximizer implementation using Maximal Marginal Relevance (MMR).

use crate::spatiotemporal::diversity::types::ScoredEpisode;
use std::cmp::Ordering;

/// Diversity maximizer using Maximal Marginal Relevance (MMR)
///
/// Balances relevance and diversity when selecting episodes from candidates.
/// Uses the lambda parameter to control the trade-off:
/// - Higher lambda (→1.0): Prioritize relevance
/// - Lower lambda (→0.0): Prioritize diversity
///
/// # Examples
///
/// ```
/// use memory_core::spatiotemporal::{DiversityMaximizer, ScoredEpisode};
///
/// // Create maximizer with default lambda (0.7)
/// let maximizer = DiversityMaximizer::default();
///
/// let candidates = vec![
///     ScoredEpisode::new("ep1".to_string(), 0.9, vec![1.0, 0.0]),
///     ScoredEpisode::new("ep2".to_string(), 0.85, vec![0.9, 0.1]),
///     ScoredEpisode::new("ep3".to_string(), 0.8, vec![0.1, 0.9]),
/// ];
///
/// // Select 2 diverse episodes
/// let diverse = maximizer.maximize_diversity(candidates, 2);
/// assert_eq!(diverse.len(), 2);
///
/// // Check diversity score
/// let diversity_score = maximizer.calculate_diversity_score(&diverse);
/// assert!(diversity_score >= 0.7);
/// ```
#[derive(Debug, Clone)]
pub struct DiversityMaximizer {
    /// Lambda parameter: balance between relevance (→1.0) and diversity (→0.0)
    lambda: f32,
}

impl DiversityMaximizer {
    /// Create a new diversity maximizer with specified lambda
    ///
    /// # Arguments
    ///
    /// * `lambda` - Trade-off parameter (0.0 to 1.0)
    ///   - 1.0 = pure relevance (no diversity)
    ///   - 0.7 = default (70% relevance, 30% diversity)
    ///   - 0.0 = pure diversity (no relevance)
    ///
    /// # Panics
    ///
    /// Panics if lambda is not in range [0.0, 1.0]
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::DiversityMaximizer;
    ///
    /// // Default: 70% relevance, 30% diversity
    /// let maximizer = DiversityMaximizer::new(0.7);
    ///
    /// // Pure relevance
    /// let relevance_only = DiversityMaximizer::new(1.0);
    ///
    /// // More diversity
    /// let more_diverse = DiversityMaximizer::new(0.5);
    /// ```
    #[must_use]
    pub fn new(lambda: f32) -> Self {
        assert!(
            (0.0..=1.0).contains(&lambda),
            "Lambda must be in range [0.0, 1.0], got {lambda}"
        );
        Self { lambda }
    }

    /// Get the current lambda value
    #[must_use]
    pub fn lambda(&self) -> f32 {
        self.lambda
    }

    /// Maximize diversity using MMR algorithm
    ///
    /// Iteratively selects episodes with highest MMR score:
    /// ```text
    /// MMR(e) = λ * Relevance(e) - (1-λ) * max(Similarity(e, selected))
    /// ```
    ///
    /// # Algorithm
    ///
    /// 1. Start with empty selection
    /// 2. While selection size < limit and candidates remain:
    ///    a. For each remaining candidate, calculate MMR score
    ///    b. Select candidate with highest MMR score
    ///    c. Move selected candidate from remaining to selected
    /// 3. Return selected episodes
    ///
    /// # Arguments
    ///
    /// * `candidates` - Pre-ranked episodes with embeddings
    /// * `limit` - Maximum number of episodes to select
    ///
    /// # Returns
    ///
    /// Vector of diverse episodes (up to `limit` items), ordered by selection
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::{DiversityMaximizer, ScoredEpisode};
    ///
    /// let maximizer = DiversityMaximizer::new(0.7);
    ///
    /// let candidates = vec![
    ///     ScoredEpisode::new("ep1".to_string(), 0.9, vec![1.0, 0.0]),
    ///     ScoredEpisode::new("ep2".to_string(), 0.85, vec![0.9, 0.1]),
    ///     ScoredEpisode::new("ep3".to_string(), 0.8, vec![0.1, 0.9]),
    ///     ScoredEpisode::new("ep4".to_string(), 0.75, vec![0.0, 1.0]),
    /// ];
    ///
    /// let diverse = maximizer.maximize_diversity(candidates, 2);
    /// assert_eq!(diverse.len(), 2);
    /// // ep1 likely selected first (highest relevance)
    /// // ep3 or ep4 likely selected second (most different from ep1)
    /// ```
    #[must_use]
    pub fn maximize_diversity(
        &self,
        candidates: Vec<ScoredEpisode>,
        limit: usize,
    ) -> Vec<ScoredEpisode> {
        // Handle edge cases
        if candidates.is_empty() || limit == 0 {
            return Vec::new();
        }

        if candidates.len() <= limit {
            return candidates;
        }

        let mut selected: Vec<ScoredEpisode> = Vec::with_capacity(limit);
        let mut remaining = candidates;

        // Iterative MMR selection
        while selected.len() < limit && !remaining.is_empty() {
            // Find episode with maximum MMR score
            let best_idx = self.find_max_mmr_index(&remaining, &selected);

            // Move from remaining to selected
            let best_episode = remaining.remove(best_idx);
            selected.push(best_episode);
        }

        selected
    }

    /// Find index of episode with maximum MMR score
    ///
    /// # Arguments
    ///
    /// * `candidates` - Remaining candidates to score
    /// * `selected` - Already selected episodes
    ///
    /// # Returns
    ///
    /// Index of candidate with highest MMR score
    fn find_max_mmr_index(
        &self,
        candidates: &[ScoredEpisode],
        selected: &[ScoredEpisode],
    ) -> usize {
        candidates
            .iter()
            .enumerate()
            .map(|(idx, episode)| {
                let mmr_score = self.calculate_mmr_score(episode, selected);
                (idx, mmr_score)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
            .map_or(0, |(idx, _)| idx)
    }

    /// Calculate MMR score for an episode
    ///
    /// ```text
    /// MMR(e) = λ * Relevance(e) - (1-λ) * max(Similarity(e, selected_i))
    /// ```
    ///
    /// # Arguments
    ///
    /// * `episode` - Candidate episode
    /// * `selected` - Already selected episodes
    ///
    /// # Returns
    ///
    /// MMR score (higher = better candidate)
    fn calculate_mmr_score(&self, episode: &ScoredEpisode, selected: &[ScoredEpisode]) -> f32 {
        let relevance = episode.relevance_score();

        // If no episodes selected yet, MMR = relevance
        if selected.is_empty() {
            return self.lambda * relevance;
        }

        // Find maximum similarity to any selected episode
        let max_similarity = selected
            .iter()
            .map(|s| Self::calculate_similarity(episode, s))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_or(0.0);

        // MMR = λ * Relevance - (1-λ) * max(Similarity)
        self.lambda * relevance - (1.0 - self.lambda) * max_similarity
    }

    /// Calculate cosine similarity between two episodes
    ///
    /// Formula: `similarity = dot(a, b) / (||a|| * ||b||)`
    ///
    /// # Arguments
    ///
    /// * `episode1` - First episode
    /// * `episode2` - Second episode
    ///
    /// # Returns
    ///
    /// Cosine similarity (0.0 to 1.0, higher = more similar)
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::{DiversityMaximizer, ScoredEpisode};
    ///
    /// let maximizer = DiversityMaximizer::default();
    ///
    /// let ep1 = ScoredEpisode::new("ep1".to_string(), 0.9, vec![1.0, 0.0, 0.0]);
    /// let ep2 = ScoredEpisode::new("ep2".to_string(), 0.8, vec![1.0, 0.0, 0.0]);
    /// let ep3 = ScoredEpisode::new("ep3".to_string(), 0.7, vec![0.0, 1.0, 0.0]);
    ///
    /// let sim_high = DiversityMaximizer::calculate_similarity(&ep1, &ep2);
    /// let sim_low = DiversityMaximizer::calculate_similarity(&ep1, &ep3);
    ///
    /// assert!(sim_high > 0.99); // Nearly identical
    /// assert!(sim_low < 0.01);  // Orthogonal
    /// ```
    #[must_use]
    pub fn calculate_similarity(episode1: &ScoredEpisode, episode2: &ScoredEpisode) -> f32 {
        let emb1 = episode1.embedding();
        let emb2 = episode2.embedding();

        // Handle dimension mismatch
        if emb1.len() != emb2.len() {
            return 0.0;
        }

        if emb1.is_empty() {
            return 0.0;
        }

        // Calculate dot product
        let dot_product: f32 = emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum();

        // Calculate magnitudes
        let magnitude1: f32 = emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = emb2.iter().map(|x| x * x).sum::<f32>().sqrt();

        // Avoid division by zero
        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0;
        }

        // Cosine similarity
        let similarity = dot_product / (magnitude1 * magnitude2);

        // Clamp to [0, 1] (cosine can be negative for opposite vectors)
        similarity.clamp(0.0, 1.0)
    }

    /// Calculate diversity score for a set of episodes
    ///
    /// Measures average pairwise dissimilarity:
    ///
    /// ```text
    /// Diversity = (1/n²) * Σ(i,j) Dissimilarity(e_i, e_j)
    ///           = (1/n²) * Σ(i,j) (1 - Similarity(e_i, e_j))
    /// ```
    ///
    /// Target: ≥0.7 for diverse result sets
    ///
    /// # Arguments
    ///
    /// * `selected` - Episodes to measure diversity for
    ///
    /// # Returns
    ///
    /// Diversity score (0.0 to 1.0, higher = more diverse)
    /// - 0.0 = all episodes identical
    /// - 1.0 = all episodes completely different
    /// - ≥0.7 = target diversity
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::{DiversityMaximizer, ScoredEpisode};
    ///
    /// let maximizer = DiversityMaximizer::default();
    ///
    /// // Diverse episodes (orthogonal vectors)
    /// let diverse = vec![
    ///     ScoredEpisode::new("ep1".to_string(), 0.9, vec![1.0, 0.0, 0.0]),
    ///     ScoredEpisode::new("ep2".to_string(), 0.8, vec![0.0, 1.0, 0.0]),
    ///     ScoredEpisode::new("ep3".to_string(), 0.7, vec![0.0, 0.0, 1.0]),
    /// ];
    ///
    /// let diversity = maximizer.calculate_diversity_score(&diverse);
    /// assert!(diversity >= 0.9); // Very diverse
    ///
    /// // Similar episodes
    /// let similar = vec![
    ///     ScoredEpisode::new("ep1".to_string(), 0.9, vec![1.0, 0.0]),
    ///     ScoredEpisode::new("ep2".to_string(), 0.8, vec![0.9, 0.1]),
    /// ];
    ///
    /// let low_diversity = maximizer.calculate_diversity_score(&similar);
    /// assert!(low_diversity < 0.3); // Low diversity
    /// ```
    #[must_use]
    pub fn calculate_diversity_score(&self, selected: &[ScoredEpisode]) -> f32 {
        // Edge cases
        if selected.is_empty() {
            return 0.0;
        }

        if selected.len() == 1 {
            return 1.0; // Single item is perfectly "diverse"
        }

        let n = selected.len();
        let mut total_dissimilarity = 0.0_f32;
        let mut pair_count = 0;

        // Calculate pairwise dissimilarity
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    let similarity = Self::calculate_similarity(&selected[i], &selected[j]);
                    let dissimilarity = 1.0 - similarity;
                    total_dissimilarity += dissimilarity;
                    pair_count += 1;
                }
            }
        }

        // Average dissimilarity
        if pair_count > 0 {
            total_dissimilarity / pair_count as f32
        } else {
            0.0
        }
    }
}

impl Default for DiversityMaximizer {
    /// Create maximizer with default lambda (0.7)
    ///
    /// Default balances 70% relevance and 30% diversity, as recommended
    /// by the research paper for optimal retrieval accuracy and diversity.
    fn default() -> Self {
        Self::new(0.7)
    }
}
