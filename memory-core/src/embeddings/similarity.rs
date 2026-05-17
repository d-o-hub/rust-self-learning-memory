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
#[must_use]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let (dot_prod, sum_sq_a, sum_sq_b) = compute_cosine_components(a, b);
    let mag_a = sum_sq_a.sqrt();
    let mag_b = sum_sq_b.sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    // Normalize from [-1, 1] to [0, 1] range
    let similarity = dot_prod / (mag_a * mag_b);
    (similarity + 1.0) / 2.0
}

/// Compute dot product and sums of squares in a single pass.
#[allow(unsafe_code)]
fn compute_cosine_components(a: &[f32], b: &[f32]) -> (f32, f32, f32) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Runtime detection ensures feature availability.
            return unsafe { cosine_components_avx2_fma(a, b) };
        }
    }

    let mut dot = 0.0;
    let mut a2 = 0.0;
    let mut b2 = 0.0;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        a2 += x * x;
        b2 += y * y;
    }
    (dot, a2, b2)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
#[allow(unsafe_code)]
unsafe fn cosine_components_avx2_fma(a: &[f32], b: &[f32]) -> (f32, f32, f32) {
    use std::arch::x86_64::{_mm256_fmadd_ps, _mm256_loadu_ps, _mm256_setzero_ps};
    let n = a.len();
    let mut sum_dot = _mm256_setzero_ps();
    let mut sum_a2 = _mm256_setzero_ps();
    let mut sum_b2 = _mm256_setzero_ps();
    let mut i = 0;
    while i + 8 <= n {
        // SAFETY: We checked that i + 8 <= n, and we use unaligned loads.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };
        sum_dot = _mm256_fmadd_ps(va, vb, sum_dot);
        sum_a2 = _mm256_fmadd_ps(va, va, sum_a2);
        sum_b2 = _mm256_fmadd_ps(vb, vb, sum_b2);
        i += 8;
    }

    // SAFETY: Registers are initialized and avx2 is enabled.
    let mut dot = unsafe { avx2_reduce_add_ps(sum_dot) };
    let mut a2 = unsafe { avx2_reduce_add_ps(sum_a2) };
    let mut b2 = unsafe { avx2_reduce_add_ps(sum_b2) };

    while i < n {
        dot += a[i] * b[i];
        a2 += a[i] * a[i];
        b2 += b[i] * b[i];
        i += 1;
    }
    (dot, a2, b2)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[allow(unsafe_code)]
unsafe fn avx2_reduce_add_ps(x: std::arch::x86_64::__m256) -> f32 {
    use std::arch::x86_64::{
        _mm_add_ps, _mm_add_ss, _mm_cvtss_f32, _mm_movehl_ps, _mm_shuffle_ps,
        _mm256_castps256_ps128, _mm256_extractf128_ps,
    };
    // SAFETY: AVX2 is enabled and registers are valid.
    let x128 = _mm_add_ps(_mm256_extractf128_ps(x, 1), _mm256_castps256_ps128(x));
    let x64 = _mm_add_ps(x128, _mm_movehl_ps(x128, x128));
    let x32 = _mm_add_ss(x64, _mm_shuffle_ps(x64, x64, 0x55));
    _mm_cvtss_f32(x32)
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
    fn test_cosine_similarity_various_sizes() {
        for n in [1, 7, 8, 9, 15, 16, 17, 31, 32, 33, 128, 768] {
            let vec1 = vec![1.0; n];
            let vec2 = vec![1.0; n];
            let similarity = cosine_similarity(&vec1, &vec2);
            assert!(
                (similarity - 1.0).abs() < f32::EPSILON * 100.0,
                "Failed for size {}: expected 1.0, got {}",
                n,
                similarity
            );

            let mut vec3 = vec![0.0; n];
            vec3[0] = 1.0;
            let mut vec4 = vec![0.0; n];
            if n > 1 {
                vec4[1] = 1.0;
                let similarity = cosine_similarity(&vec3, &vec4);
                assert!(
                    (similarity - 0.5).abs() < f32::EPSILON * 100.0,
                    "Failed for size {}: expected 0.5, got {}",
                    n,
                    similarity
                );
            }
        }
    }
}
