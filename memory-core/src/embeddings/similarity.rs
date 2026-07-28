//! Vector similarity calculations and search utilities

// SAFETY: This module contains a safe-to-use AVX2 SIMD path for cosine similarity.
// The unsafe blocks are limited to the #[target_feature(enable = "avx2")] function
// that uses std::arch intrinsics, guarded by is_x86_feature_detected! at runtime.
#![allow(unsafe_code)]

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

/// Calculate cosine similarity between two vectors (scalar path).
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
fn cosine_similarity_scalar(a: &[f32], b: &[f32]) -> f32 {
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

/// Horizontal sum of an AVX 256-bit f32 register.
///
/// Reduces 8 lanes to a single f32 scalar.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn hsum256_ps(v: std::arch::x86_64::__m256) -> f32 {
    use std::arch::x86_64::{
        _mm_add_ps, _mm_add_ss, _mm_cvtss_f32, _mm_movehdup_ps, _mm_movehl_ps,
        _mm256_castps256_ps128, _mm256_extractf128_ps,
    };
    // AVX2 intrinsics are safe to call directly inside a #[target_feature(enable = "avx2")] fn.
    let lo = _mm256_castps256_ps128(v);
    let hi = _mm256_extractf128_ps(v, 1);
    let sum128 = _mm_add_ps(lo, hi);
    let shuf = _mm_movehdup_ps(sum128);
    let sums = _mm_add_ps(sum128, shuf);
    let shuf2 = _mm_movehl_ps(shuf, sums);
    _mm_cvtss_f32(_mm_add_ss(sums, shuf2))
}

/// AVX2 inner loop: accumulates dot product and squared norms over 8 f32 lanes.
///
/// Processes `a` and `b` in chunks of 8, accumulating `dot`, `na`, `nb` via
/// fused-multiply-add style wide SIMD operations. Remainder elements are handled
/// by scalar code.
///
/// # Safety
///
/// Caller must ensure:
/// - `a.len() == b.len()`
/// - CPU supports AVX2 (`is_x86_feature_detected!("avx2")` is true)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn cosine_similarity_avx2_impl(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::{_mm256_add_ps, _mm256_loadu_ps, _mm256_mul_ps, _mm256_setzero_ps};

    // AVX2 intrinsics are safe inside a #[target_feature(enable = "avx2")] fn.
    let mut vdot = _mm256_setzero_ps();
    let mut vna = _mm256_setzero_ps();
    let mut vnb = _mm256_setzero_ps();

    let chunks = a.len() / 8;
    for i in 0..chunks {
        let offset = i * 8;
        // SAFETY: offset + 8 <= a.len() because i < chunks = a.len() / 8.
        // Raw pointer arithmetic requires an explicit unsafe block even inside unsafe fn (Rust 2024).
        let (va, vb) = unsafe {
            (
                _mm256_loadu_ps(a.as_ptr().add(offset)),
                _mm256_loadu_ps(b.as_ptr().add(offset)),
            )
        };
        vdot = _mm256_add_ps(vdot, _mm256_mul_ps(va, vb));
        vna = _mm256_add_ps(vna, _mm256_mul_ps(va, va));
        vnb = _mm256_add_ps(vnb, _mm256_mul_ps(vb, vb));
    }

    // SAFETY: hsum256_ps is an unsafe fn; call requires explicit unsafe block (Rust 2024).
    let mut dot_product = unsafe { hsum256_ps(vdot) };
    let mut norm_a_sq = unsafe { hsum256_ps(vna) };
    let mut norm_b_sq = unsafe { hsum256_ps(vnb) };

    // Scalar remainder: safe indexed access (no unsafe needed here)
    let rem_start = chunks * 8;
    for i in rem_start..a.len() {
        let x = a[i];
        let y = b[i];
        dot_product += x * y;
        norm_a_sq += x * x;
        norm_b_sq += y * y;
    }

    if norm_a_sq <= 0.0 || norm_b_sq <= 0.0 {
        return 0.0;
    }

    let similarity = dot_product / (norm_a_sq.sqrt() * norm_b_sq.sqrt());
    (similarity + 1.0) / 2.0
}

/// Calculate cosine similarity using explicit SIMD (AVX2) when available.
///
/// On x86_64 CPUs with AVX2, this processes 8 f32 elements per iteration using
/// 256-bit SIMD registers for dot product and magnitude accumulation. Falls back
/// to the same 8-way unrolled scalar path as [`cosine_similarity`] on wasm32 and
/// non-AVX2 targets.
///
/// The result is normalized from `[-1, 1]` to `[0, 1]`.
///
/// # Accuracy
///
/// Results are within 1e-5 of the scalar path for typical embedding vectors.
///
/// # Example
///
/// ```
/// use do_memory_core::embeddings::cosine_similarity_simd;
///
/// let a = vec![1.0f32, 0.0, 0.0];
/// let b = vec![1.0f32, 0.0, 0.0];
/// let sim = cosine_similarity_simd(&a, &b);
/// assert!((sim - 1.0).abs() < 1e-5);
/// ```
#[must_use]
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: is_x86_feature_detected! confirmed AVX2 availability,
            // and a.len() == b.len() is checked above.
            return unsafe { cosine_similarity_avx2_impl(a, b) };
        }
    }

    cosine_similarity_scalar(a, b)
}

/// Calculate cosine similarity between two vectors.
///
/// Cosine similarity measures the cosine of the angle between two vectors,
/// giving a similarity score between -1 and 1 (normalized to 0-1 for convenience).
/// Higher scores indicate greater similarity.
///
/// On x86_64 CPUs with AVX2, automatically dispatches to the SIMD-accelerated path.
/// On wasm32 and non-AVX2 targets, uses the 8-way unrolled scalar accumulator.
///
/// # Optimization:
/// 1. Dispatches to AVX2 SIMD path at runtime when available (x86_64 only).
/// 2. Scalar fallback processes vector chunks of size 8 using chunks_exact,
///    with 8 separate accumulators to break data dependency chains.
/// 3. Maintains dynamic range stability by using individual square roots.
#[must_use]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    cosine_similarity_simd(a, b)
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

    /// Verify SIMD path matches scalar path within the 1e-5 tolerance.
    #[test]
    fn test_cosine_similarity_simd_matches_scalar() {
        // Deterministic pseudo-random vectors to avoid flakiness
        let a: Vec<f32> = (0..384).map(|i| (i as f32 * 0.017_453_3).sin()).collect();
        let b: Vec<f32> = (0..384).map(|i| (i as f32 * 0.034_906_6).cos()).collect();

        let scalar = cosine_similarity_scalar(&a, &b);
        let simd = cosine_similarity_simd(&a, &b);
        assert!(
            (scalar - simd).abs() < 1e-5,
            "SIMD and scalar differ by {}: scalar={scalar}, simd={simd}",
            (scalar - simd).abs()
        );

        // Also test with dim=768
        let a2: Vec<f32> = (0..768).map(|i| (i as f32 * 0.012_217_3).sin()).collect();
        let b2: Vec<f32> = (0..768).map(|i| (i as f32 * 0.024_434_6).cos()).collect();

        let scalar2 = cosine_similarity_scalar(&a2, &b2);
        let simd2 = cosine_similarity_simd(&a2, &b2);
        assert!(
            (scalar2 - simd2).abs() < 1e-5,
            "SIMD and scalar (768) differ by {}: scalar={scalar2}, simd={simd2}",
            (scalar2 - simd2).abs()
        );
    }

    /// Verify the SIMD path gives correct results for the standard test cases.
    #[test]
    fn test_cosine_similarity_simd_standard_cases() {
        // Identical vectors → 1.0
        let v1: Vec<f32> = (1..=16).map(|i| i as f32).collect();
        let v2 = v1.clone();
        let result = cosine_similarity_simd(&v1, &v2);
        assert!((result - 1.0).abs() < 1e-5, "identical: {result}");

        // Orthogonal vectors → 0.5
        let v3 = vec![1.0f32, 0.0];
        let v4 = vec![0.0f32, 1.0];
        let result = cosine_similarity_simd(&v3, &v4);
        assert!((result - 0.5).abs() < 1e-5, "orthogonal: {result}");

        // Opposite vectors → 0.0
        let v5: Vec<f32> = (1..=16).map(|i| i as f32).collect();
        let v6: Vec<f32> = (1..=16).map(|i| -(i as f32)).collect();
        let result = cosine_similarity_simd(&v5, &v6);
        assert!((result - 0.0).abs() < 1e-5, "opposite: {result}");

        // Length mismatch → 0.0
        let v7 = vec![1.0f32, 2.0];
        let v8 = vec![1.0f32, 2.0, 3.0];
        assert_eq!(cosine_similarity_simd(&v7, &v8), 0.0, "length mismatch");

        // Empty → 0.0
        let empty: Vec<f32> = vec![];
        assert_eq!(cosine_similarity_simd(&empty, &empty), 0.0, "empty");

        // Zero vector → 0.0
        let v9 = vec![0.0f32; 16];
        let v10: Vec<f32> = (1..=16).map(|i| i as f32).collect();
        assert_eq!(cosine_similarity_simd(&v9, &v10), 0.0, "zero magnitude");
    }

    /// Verify that cosine_similarity delegates to cosine_similarity_simd.
    #[test]
    fn test_cosine_similarity_delegates_to_simd() {
        let a: Vec<f32> = (0..512).map(|i| (i as f32).sin()).collect();
        let b: Vec<f32> = (0..512).map(|i| (i as f32).cos()).collect();
        let direct = cosine_similarity_simd(&a, &b);
        let delegated = cosine_similarity(&a, &b);
        assert_eq!(direct, delegated);
    }
}
