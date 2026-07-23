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
/// # Optimization:
/// 1. Processes vector chunks of size 8 using chunks_exact to allow LLVM to generate
///    highly efficient SIMD instruction sets (AVX/SSE/NEON).
/// 2. Employs 8 separate accumulators for the dot product and magnitude components
///    to break data dependency chains, improving instruction-level parallelism.
/// 3. Maintains dynamic range stability by using individual square roots.
#[must_use]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    if len != b.len() || len == 0 {
        return 0.0;
    }

    // Unroll 8-way to break dependency chains & trigger autovectorization
    let mut dp0 = 0.0f32;
    let mut dp1 = 0.0f32;
    let mut dp2 = 0.0f32;
    let mut dp3 = 0.0f32;
    let mut dp4 = 0.0f32;
    let mut dp5 = 0.0f32;
    let mut dp6 = 0.0f32;
    let mut dp7 = 0.0f32;

    let mut na0 = 0.0f32;
    let mut na1 = 0.0f32;
    let mut na2 = 0.0f32;
    let mut na3 = 0.0f32;
    let mut na4 = 0.0f32;
    let mut na5 = 0.0f32;
    let mut na6 = 0.0f32;
    let mut na7 = 0.0f32;

    let mut nb0 = 0.0f32;
    let mut nb1 = 0.0f32;
    let mut nb2 = 0.0f32;
    let mut nb3 = 0.0f32;
    let mut nb4 = 0.0f32;
    let mut nb5 = 0.0f32;
    let mut nb6 = 0.0f32;
    let mut nb7 = 0.0f32;

    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();

    for (ca, cb) in chunks_a.zip(chunks_b) {
        dp0 += ca[0] * cb[0];
        dp1 += ca[1] * cb[1];
        dp2 += ca[2] * cb[2];
        dp3 += ca[3] * cb[3];
        dp4 += ca[4] * cb[4];
        dp5 += ca[5] * cb[5];
        dp6 += ca[6] * cb[6];
        dp7 += ca[7] * cb[7];

        na0 += ca[0] * ca[0];
        na1 += ca[1] * ca[1];
        na2 += ca[2] * ca[2];
        na3 += ca[3] * ca[3];
        na4 += ca[4] * ca[4];
        na5 += ca[5] * ca[5];
        na6 += ca[6] * ca[6];
        na7 += ca[7] * ca[7];

        nb0 += cb[0] * cb[0];
        nb1 += cb[1] * cb[1];
        nb2 += cb[2] * cb[2];
        nb3 += cb[3] * cb[3];
        nb4 += cb[4] * cb[4];
        nb5 += cb[5] * cb[5];
        nb6 += cb[6] * cb[6];
        nb7 += cb[7] * cb[7];
    }

    let mut dot_product = dp0 + dp1 + dp2 + dp3 + dp4 + dp5 + dp6 + dp7;
    let mut norm_a_sq = na0 + na1 + na2 + na3 + na4 + na5 + na6 + na7;
    let mut norm_b_sq = nb0 + nb1 + nb2 + nb3 + nb4 + nb5 + nb6 + nb7;

    for (&x, &y) in rem_a.iter().zip(rem_b.iter()) {
        dot_product += x * y;
        norm_a_sq += x * x;
        norm_b_sq += y * y;
    }

    if norm_a_sq <= 0.0 || norm_b_sq <= 0.0 {
        return 0.0;
    }

    let similarity = dot_product / (norm_a_sq.sqrt() * norm_b_sq.sqrt());

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
