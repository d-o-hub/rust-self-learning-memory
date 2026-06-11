//! Query-length-dependent tier weighting for cascade retrieval.

/// Weight computation for query-length-dependent tier weighting.
///
/// Short queries favor BM25 (keyword matching), long queries favor
/// HDC/semantic matching.
#[cfg(feature = "csm")]
pub fn compute_tier_weights(query: &str) -> (f32, f32, f32) {
    let len = query.len();
    if len < 20 {
        (0.7, 0.2, 0.1)
    } else if len < 100 {
        (0.4, 0.4, 0.2)
    } else {
        (0.2, 0.5, 0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_query_favors_bm25() {
        let (bm25, hdc, cg) = compute_tier_weights("rust error");
        assert!(bm25 > hdc);
        assert!(bm25 > cg);
    }

    #[test]
    fn test_long_query_favors_semantic() {
        let query = "a".repeat(120);
        let (bm25, hdc, cg) = compute_tier_weights(&query);
        assert!(hdc > bm25);
    }

    #[test]
    fn test_medium_query_balanced() {
        let query = "how to implement async rust pattern matching with tokio runtime";
        let (bm25, hdc, _cg) = compute_tier_weights(query);
        assert!((bm25 - hdc).abs() < 0.1);
    }
}
