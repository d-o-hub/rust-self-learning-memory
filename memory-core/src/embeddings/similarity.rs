//! Vector similarity calculations and search utilities

#![allow(dead_code)]

// Remove unused imports for now
use serde::{Deserialize, Serialize};

/// Result from similarity search containing the item and similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilaritySearchResult<T> {
    /// The found item (episode or pattern)
    pub item: T,
    /// Similarity score (0.0 to 1.0, higher = more similar)
    pub similarity: f32,
    /// Additional metadata about the match
    pub metadata: SimilarityMetadata,
}

/// Metadata about a similarity match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityMetadata {
    /// Which embedding was used for the match
    pub embedding_model: String,
    /// Timestamp of when the embedding was generated
    pub embedding_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional context about the match
    pub context: serde_json::Value,
}

/// Embedding vector with associated metadata
#[derive(Debug, Clone)]
pub struct EmbeddingWithMetadata {
    /// The embedding vector
    pub embedding: Vec<f32>,
    /// Associated metadata
    pub metadata: EmbeddingMetadata,
}

/// Metadata associated with an embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    /// ID of the item this embedding represents
    pub item_id: String,
    /// Type of item (episode, pattern, etc.)
    pub item_type: String,
    /// Model used to generate the embedding
    pub model: String,
    /// When the embedding was generated
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Hash of the source text (for cache invalidation)
    pub text_hash: String,
    /// Embedding dimension
    pub dimension: usize,
}

/// Calculate cosine similarity between two vectors
///
/// Cosine similarity measures the cosine of the angle between two vectors,
/// giving a similarity score between -1 and 1 (normalized to 0-1 for convenience).
/// Higher scores indicate greater similarity.
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
/// Similarity score between 0.0 and 1.0
///
/// # Example
/// ```ignore
/// // Note: similarity module is private, this is for internal documentation only
/// use memory_core::embeddings::similarity::cosine_similarity;
///
/// let vec1 = vec![1.0, 2.0, 3.0];
/// let vec2 = vec![1.0, 2.0, 3.0];
/// let similarity = cosine_similarity(&vec1, &vec2);
/// assert_eq!(similarity, 1.0); // Identical vectors
/// ```
#[must_use]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0; // Different dimensions = no similarity
    }

    if a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    // Normalize from [-1, 1] to [0, 1] range
    let similarity = dot_product / (magnitude_a * magnitude_b);
    (similarity + 1.0) / 2.0
}

/// Calculate Euclidean distance between two vectors
///
/// Smaller distances indicate greater similarity.
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
/// Euclidean distance (0 = identical, higher = more different)
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::INFINITY;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Calculate Manhattan (L1) distance between two vectors
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
/// Manhattan distance
pub fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::INFINITY;
    }

    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

/// Convert Euclidean distance to similarity score (0-1)
pub fn distance_to_similarity(distance: f32, max_distance: f32) -> f32 {
    if max_distance <= 0.0 {
        return 0.0;
    }
    1.0 - (distance / max_distance).min(1.0)
}

/// Efficient approximate similarity calculation for large-scale search
///
/// Uses dimensionality reduction or locality-sensitive hashing for faster
/// similarity calculations at the cost of some accuracy.
pub fn approximate_similarity(a: &[f32], b: &[f32], precision: SimilarityPrecision) -> f32 {
    match precision {
        SimilarityPrecision::High => cosine_similarity(a, b),
        SimilarityPrecision::Medium => {
            // Use only every other dimension for faster calculation
            let a_sampled: Vec<f32> = a.iter().step_by(2).copied().collect();
            let b_sampled: Vec<f32> = b.iter().step_by(2).copied().collect();
            cosine_similarity(&a_sampled, &b_sampled)
        }
        SimilarityPrecision::Low => {
            // Use only every 4th dimension
            let a_sampled: Vec<f32> = a.iter().step_by(4).copied().collect();
            let b_sampled: Vec<f32> = b.iter().step_by(4).copied().collect();
            cosine_similarity(&a_sampled, &b_sampled)
        }
    }
}

/// Precision level for similarity calculations
#[derive(Debug, Clone, Copy)]
pub enum SimilarityPrecision {
    /// Full precision using all dimensions
    High,
    /// Medium precision using half the dimensions
    Medium,
    /// Low precision using quarter of the dimensions
    Low,
}

/// Batch similarity calculation for multiple comparisons
///
/// Efficiently calculate similarities between a query vector and multiple candidates.
///
/// # Arguments
/// * `query` - Query vector
/// * `candidates` - Vector of candidate embeddings
/// * `top_k` - Number of top results to return
///
/// # Returns
/// Top k most similar candidates with similarity scores
pub fn batch_similarity_search(
    query: &[f32],
    candidates: &[EmbeddingWithMetadata],
    top_k: usize,
) -> Vec<(usize, f32)> {
    let mut similarities: Vec<(usize, f32)> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| (i, cosine_similarity(query, &candidate.embedding)))
        .collect();

    // Sort by similarity (highest first)
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return top k results
    similarities.into_iter().take(top_k).collect()
}

/// Advanced similarity calculation that considers context weighting
///
/// Adjusts similarity based on contextual factors like recency, success rate, etc.
///
/// # Arguments
/// * `embedding_similarity` - Base embedding similarity (0-1)
/// * `context_weight` - Context-based weight factor (0-1)
/// * `recency_weight` - Recency-based weight factor (0-1)
/// * `success_weight` - Success rate-based weight factor (0-1)
///
/// # Returns
/// Weighted similarity score
pub fn weighted_similarity(
    embedding_similarity: f32,
    context_weight: f32,
    recency_weight: f32,
    success_weight: f32,
) -> f32 {
    // Weighted average of different similarity factors
    let weights = [0.6, 0.2, 0.1, 0.1]; // Embedding gets highest weight
    let scores = [
        embedding_similarity,
        context_weight,
        recency_weight,
        success_weight,
    ];

    weights
        .iter()
        .zip(scores.iter())
        .map(|(w, s)| w * s)
        .sum::<f32>()
}

/// Compute similarity matrix for a set of embeddings
///
/// Useful for clustering or finding duplicate embeddings.
///
/// # Arguments
/// * `embeddings` - Vector of embeddings to compare
///
/// # Returns
/// Symmetric similarity matrix where matrix[i][j] = similarity(embedding[i], embedding[j])
pub fn similarity_matrix(embeddings: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let n = embeddings.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in i..n {
            let similarity = if i == j {
                1.0
            } else {
                cosine_similarity(&embeddings[i], &embeddings[j])
            };
            matrix[i][j] = similarity;
            matrix[j][i] = similarity; // Symmetric
        }
    }

    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        // Identical vectors should have similarity 1.0
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);

        // Orthogonal vectors should have similarity 0.5 (normalized from 0)
        let vec3 = vec![1.0, 0.0];
        let vec4 = vec![0.0, 1.0];
        let similarity = cosine_similarity(&vec3, &vec4);
        assert!((similarity - 0.5).abs() < 0.001);

        // Opposite vectors should have similarity 0.0 (normalized from -1)
        let vec5 = vec![1.0, 2.0, 3.0];
        let vec6 = vec![-1.0, -2.0, -3.0];
        let similarity = cosine_similarity(&vec5, &vec6);
        assert!((similarity - 0.0).abs() < 0.001);

        // Different dimensions should return 0
        let vec7 = vec![1.0, 2.0];
        let vec8 = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&vec7, &vec8);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_euclidean_distance() {
        // Identical vectors should have distance 0
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let distance = euclidean_distance(&vec1, &vec2);
        assert!((distance - 0.0).abs() < 0.001);

        // Known distance
        let vec3 = vec![0.0, 0.0];
        let vec4 = vec![3.0, 4.0];
        let distance = euclidean_distance(&vec3, &vec4);
        assert!((distance - 5.0).abs() < 0.001); // 3-4-5 triangle

        // Different dimensions should return infinity
        let vec5 = vec![1.0, 2.0];
        let vec6 = vec![1.0, 2.0, 3.0];
        let distance = euclidean_distance(&vec5, &vec6);
        assert_eq!(distance, f32::INFINITY);
    }

    #[test]
    fn test_manhattan_distance() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 6.0, 8.0];
        let distance = manhattan_distance(&vec1, &vec2);
        // |1-4| + |2-6| + |3-8| = 3 + 4 + 5 = 12
        assert!((distance - 12.0).abs() < 0.001);
    }

    #[test]
    fn test_distance_to_similarity() {
        // Distance 0 should give similarity 1.0
        assert_eq!(distance_to_similarity(0.0, 10.0), 1.0);

        // Distance equal to max should give similarity 0.0
        assert_eq!(distance_to_similarity(10.0, 10.0), 0.0);

        // Half the max distance should give similarity 0.5
        assert_eq!(distance_to_similarity(5.0, 10.0), 0.5);

        // Distance greater than max should give similarity 0.0
        assert_eq!(distance_to_similarity(15.0, 10.0), 0.0);
    }

    #[test]
    fn test_approximate_similarity() {
        let vec1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let vec2 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

        // High precision should be identical to full cosine similarity
        let high = approximate_similarity(&vec1, &vec2, SimilarityPrecision::High);
        let exact = cosine_similarity(&vec1, &vec2);
        assert!((high - exact).abs() < 0.001);

        // Medium and low precision should be reasonably close for identical vectors
        let medium = approximate_similarity(&vec1, &vec2, SimilarityPrecision::Medium);
        let low = approximate_similarity(&vec1, &vec2, SimilarityPrecision::Low);

        assert!(medium > 0.9); // Should still be very high for identical vectors
        assert!(low > 0.9); // Should still be very high for identical vectors
    }

    #[test]
    fn test_weighted_similarity() {
        let similarity = weighted_similarity(0.8, 0.7, 0.6, 0.9);

        // Should be weighted average: 0.6*0.8 + 0.2*0.7 + 0.1*0.6 + 0.1*0.9
        let expected = 0.6 * 0.8 + 0.2 * 0.7 + 0.1 * 0.6 + 0.1 * 0.9;
        assert!((similarity - expected).abs() < 0.001);
    }

    #[test]
    fn test_similarity_matrix() {
        let embeddings = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]];

        let matrix = similarity_matrix(&embeddings);

        // Matrix should be 3x3
        assert_eq!(matrix.len(), 3);
        assert_eq!(matrix[0].len(), 3);

        // Diagonal should be 1.0 (self-similarity)
        for (i, row) in matrix.iter().enumerate().take(3) {
            assert!((row[i] - 1.0).abs() < 0.001);
        }

        // Matrix should be symmetric
        for (i, row) in matrix.iter().enumerate().take(3) {
            for (j, _val) in row.iter().enumerate().take(3) {
                assert!((matrix[i][j] - matrix[j][i]).abs() < 0.001);
            }
        }

        // First two embeddings are orthogonal, should have similarity ~0.5
        assert!((matrix[0][1] - 0.5).abs() < 0.001);
    }
}
