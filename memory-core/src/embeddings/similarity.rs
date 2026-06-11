//! Vector similarity calculations and search utilities

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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SimilarityMetadata {
    /// Which embedding was used for the match
    #[serde(default)]
    pub embedding_model: String,
    /// Timestamp of when the embedding was generated
    pub embedding_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional context about the match
    #[serde(default)]
    pub context: serde_json::Value,
}

/// Calculate cosine similarity between two vectors
///
/// Cosine similarity measures the cosine of the angle between two vectors,
/// giving a similarity score between -1 and 1 (normalized to 0-1 for convenience).
/// Higher scores indicate greater similarity.
///
/// # Optimization (v2):
/// 1. Uses f32 accumulators to enable LLVM SIMD autovectorization.
/// 2. Reduces sqrt calls from two to one by calculating sqrt(norm_a_sq * norm_b_sq).
/// 3. Uses a single zip pass for all dot-product and magnitude calculations.
#[must_use]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0f32;
    let mut norm_a_sq = 0.0f32;
    let mut norm_b_sq = 0.0f32;

    // Single pass for dot product and norms calculation.
    // Using f32 enables compiler SIMD autovectorization (AVX/SSE/NEON).
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot_product += x * y;
        norm_a_sq += x * x;
        norm_b_sq += y * y;
    }

    if norm_a_sq <= 0.0 || norm_b_sq <= 0.0 {
        return 0.0;
    }

    // Optimization: One sqrt instead of two.
    // similarity = dot / (sqrt(a) * sqrt(b)) = dot / sqrt(a * b)
    let similarity = dot_product / (norm_a_sq * norm_b_sq).sqrt();

    // Normalize from [-1, 1] to [0, 1] range for semantic scores
    (similarity + 1.0) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);

        let vec3 = vec![1.0, 0.0];
        let vec4 = vec![0.0, 1.0];
        let similarity = cosine_similarity(&vec3, &vec4);
        assert!((similarity - 0.5).abs() < 0.001);

        let vec5 = vec![1.0, 2.0, 3.0];
        let vec6 = vec![-1.0, -2.0, -3.0];
        let similarity = cosine_similarity(&vec5, &vec6);
        assert!((similarity - 0.0).abs() < 0.001);

        let vec7 = vec![1.0, 2.0];
        let vec8 = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&vec7, &vec8);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let vec1: Vec<f32> = vec![];
        let vec2: Vec<f32> = vec![];
        assert_eq!(cosine_similarity(&vec1, &vec2), 0.0);

        let vec3 = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&vec1, &vec3), 0.0);
        assert_eq!(cosine_similarity(&vec3, &vec1), 0.0);
    }

    #[test]
    fn test_cosine_similarity_zero_magnitude() {
        let vec1 = vec![0.0, 0.0, 0.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&vec1, &vec2), 0.0);

        let vec3 = vec![1.0, 2.0, 3.0];
        let vec4 = vec![0.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&vec3, &vec4), 0.0);

        let vec5 = vec![0.0, 0.0, 0.0];
        let vec6 = vec![0.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&vec5, &vec6), 0.0);
    }
}
