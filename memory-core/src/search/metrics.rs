//! Retrieval evaluation metrics for benchmarking.
//!
//! Implements standard metrics from MTEB/BEIR methodology for retrieval-first evaluation
//! without LLM calls. These metrics allow objective comparison of retrieval quality.
//!
//! # Metrics Overview
//!
//! | Metric | Formula | Best For |
//! |--------|---------|----------|
//! | Recall@k | (# relevant in top k) / (total relevant) | When missing items is costly |
//! | Precision@k | (# relevant in top k) / k | When user only views top results |
//! | NDCG@k | DCG@k / IDCG@k | Graded relevance (0-3 scale) |
//! | MRR | 1/N Σ (1 / rank_first_relevant) | Question answering |
//! | MAP | 1/Q Σ (precision at each relevant item) | Ranked retrieval |
//!
//! # Example
//!
//! ```
//! use do_memory_core::search::metrics::{recall_at_k, ndcg_at_k, mrr};
//! use std::collections::{HashMap, HashSet};
//!
//! let retrieved = vec![1, 2, 3, 4, 5];
//! let relevant: HashSet<usize> = [2, 4, 6].into_iter().collect();
//!
//! let recall = recall_at_k(&retrieved, &relevant, 5);
//! assert!((recall - 0.666).abs() < 0.01); // 2 of 3 relevant items found
//!
//! let mut rel_scores = HashMap::new();
//! rel_scores.insert(2, 3.0); // highly relevant
//! rel_scores.insert(4, 1.0); // marginally relevant
//! let ndcg = ndcg_at_k(&retrieved, &rel_scores, 5);
//! assert!(ndcg > 0.0 && ndcg <= 1.0);
//! ```

use std::collections::{HashMap, HashSet};

/// Calculate Recall@k: fraction of relevant items retrieved in top k.
///
/// # Arguments
///
/// * `retrieved` - Ordered list of retrieved item IDs (ranked by relevance score)
/// * `relevant` - Set of relevant item IDs (ground truth)
/// * `k` - Number of top results to consider
///
/// # Returns
///
/// Recall@k in range [0.0, 1.0]. Returns 1.0 if there are no relevant items.
#[must_use]
pub fn recall_at_k(retrieved: &[usize], relevant: &HashSet<usize>, k: usize) -> f64 {
    if relevant.is_empty() {
        return 1.0;
    }

    let k = k.min(retrieved.len());
    let relevant_in_top_k = retrieved[..k]
        .iter()
        .filter(|id| relevant.contains(id))
        .count();

    relevant_in_top_k as f64 / relevant.len() as f64
}

/// Calculate Precision@k: fraction of top-k results that are relevant.
///
/// # Arguments
///
/// * `retrieved` - Ordered list of retrieved item IDs
/// * `relevant` - Set of relevant item IDs
/// * `k` - Number of top results to consider
///
/// # Returns
///
/// Precision@k in range [0.0, 1.0].
#[must_use]
pub fn precision_at_k(retrieved: &[usize], relevant: &HashSet<usize>, k: usize) -> f64 {
    if retrieved.is_empty() {
        return 0.0;
    }

    let k = k.min(retrieved.len());
    let relevant_in_top_k = retrieved[..k]
        .iter()
        .filter(|id| relevant.contains(id))
        .count();

    relevant_in_top_k as f64 / k as f64
}

/// Calculate NDCG@k (Normalized Discounted Cumulative Gain).
///
/// NDCG accounts for graded relevance (not just binary) and position in ranking.
///
/// # Arguments
///
/// * `retrieved` - Ordered list of retrieved item IDs
/// * `relevance_scores` - Map of item ID to relevance score (typically 0-3 scale)
/// * `k` - Number of top results to consider
///
/// # Returns
///
/// NDCG@k in range [0.0, 1.0]. Returns 1.0 if no relevant items exist.
#[must_use]
pub fn ndcg_at_k(retrieved: &[usize], relevance_scores: &HashMap<usize, f64>, k: usize) -> f64 {
    let k = k.min(retrieved.len());
    if k == 0 {
        return 0.0;
    }

    // Calculate DCG@k
    let dcg: f64 = retrieved[..k]
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let rel = relevance_scores.get(id).unwrap_or(&0.0);
            (2.0f64.powf(*rel) - 1.0) / (2.0 + i as f64).log2()
        })
        .sum();

    // Calculate IDCG@k (ideal DCG - items sorted by relevance)
    let mut ideal_rels: Vec<f64> = relevance_scores.values().copied().collect();
    ideal_rels.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ideal_rels.truncate(k);

    let idcg: f64 = ideal_rels
        .iter()
        .enumerate()
        .map(|(i, rel)| (2.0f64.powf(*rel) - 1.0) / (2.0 + i as f64).log2())
        .sum();

    if idcg == 0.0 {
        return 0.0;
    }

    dcg / idcg
}

/// Calculate Mean Reciprocal Rank (MRR).
///
/// MRR measures where the first relevant item appears in the ranking.
///
/// # Arguments
///
/// * `retrieved_lists` - List of ranked result lists (one per query)
/// * `relevant_sets` - List of relevant item sets (one per query)
///
/// # Returns
///
/// MRR in range [0.0, 1.0]. Higher is better.
#[must_use]
pub fn mrr(retrieved_lists: &[Vec<usize>], relevant_sets: &[HashSet<usize>]) -> f64 {
    if retrieved_lists.is_empty() || retrieved_lists.len() != relevant_sets.len() {
        return 0.0;
    }

    let reciprocal_ranks: f64 = retrieved_lists
        .iter()
        .zip(relevant_sets.iter())
        .map(|(retrieved, relevant)| {
            if relevant.is_empty() {
                return 0.0;
            }
            retrieved
                .iter()
                .position(|id| relevant.contains(id))
                .map(|pos| 1.0 / (pos + 1) as f64)
                .unwrap_or(0.0)
        })
        .sum();

    reciprocal_ranks / retrieved_lists.len() as f64
}

/// Calculate Mean Average Precision (MAP).
///
/// MAP considers precision at each relevant item position.
///
/// # Arguments
///
/// * `retrieved_lists` - List of ranked result lists (one per query)
/// * `relevant_sets` - List of relevant item sets (one per query)
///
/// # Returns
///
/// MAP in range [0.0, 1.0]. Higher is better.
#[must_use]
pub fn map(retrieved_lists: &[Vec<usize>], relevant_sets: &[HashSet<usize>]) -> f64 {
    if retrieved_lists.is_empty() || retrieved_lists.len() != relevant_sets.len() {
        return 0.0;
    }

    let average_precisions: f64 = retrieved_lists
        .iter()
        .zip(relevant_sets.iter())
        .map(|(retrieved, relevant)| {
            if relevant.is_empty() {
                return 0.0;
            }

            let mut sum_precision = 0.0;
            let mut relevant_count = 0;

            for (i, id) in retrieved.iter().enumerate() {
                if relevant.contains(id) {
                    relevant_count += 1;
                    #[allow(clippy::cast_precision_loss)]
                    let precision_at_i = f64::from(relevant_count) / (i + 1) as f64;
                    sum_precision += precision_at_i;
                }
            }

            sum_precision / relevant.len() as f64
        })
        .sum();

    average_precisions / retrieved_lists.len() as f64
}

/// Calculate Hit Rate@k: binary metric checking if any relevant item is in top k.
///
/// # Arguments
///
/// * `retrieved` - Ordered list of retrieved item IDs
/// * `relevant` - Set of relevant item IDs
/// * `k` - Number of top results to consider
///
/// # Returns
///
/// 1.0 if any relevant item is in top k, 0.0 otherwise.
#[must_use]
pub fn hit_rate_at_k(retrieved: &[usize], relevant: &HashSet<usize>, k: usize) -> f64 {
    let k = k.min(retrieved.len());
    if k == 0 || relevant.is_empty() {
        return 0.0;
    }

    let has_hit = retrieved[..k].iter().any(|id| relevant.contains(id));
    if has_hit { 1.0 } else { 0.0 }
}

/// Reciprocal Rank Fusion (RRF) for combining multiple ranked lists.
///
/// RRF is robust to score scale differences and doesn't require normalization.
///
/// # Arguments
///
/// * `result_lists` - Multiple ranked lists with (item_id, score) tuples
/// * `k` - RRF constant (typically 60)
///
/// # Returns
///
/// Fused ranked list with RRF scores.
#[must_use]
pub fn reciprocal_rank_fusion<T: Clone + Eq + std::hash::Hash + std::cmp::Ord>(
    result_lists: &[Vec<(T, f32)>],
    k: u32,
) -> Vec<(T, f32)> {
    use std::collections::BTreeMap;

    let mut rrf_scores: BTreeMap<T, f32> = BTreeMap::new();

    for list in result_lists {
        for (rank, (item, _)) in list.iter().enumerate() {
            let rrf_contribution = 1.0 / (k as f32 + rank as f32 + 1.0);
            *rrf_scores.entry(item.clone()).or_insert(0.0) += rrf_contribution;
        }
    }

    let mut fused: Vec<(T, f32)> = rrf_scores.into_iter().collect();
    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recall_at_k() {
        let retrieved = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let relevant: HashSet<usize> = [2, 5, 7, 11].into_iter().collect();

        // 4 relevant total, 3 found in top 10
        assert!((recall_at_k(&retrieved, &relevant, 10) - 0.75).abs() < 0.001);

        // 4 relevant total, 1 found in top 3
        assert!((recall_at_k(&retrieved, &relevant, 3) - 0.25).abs() < 0.001);

        // Empty relevant set
        let empty: HashSet<usize> = HashSet::new();
        assert_eq!(recall_at_k(&retrieved, &empty, 5), 1.0);
    }

    #[test]
    fn test_precision_at_k() {
        let retrieved = vec![1, 2, 3, 4, 5];
        let relevant: HashSet<usize> = [2, 4].into_iter().collect();

        // 2 relevant in top 5
        assert!((precision_at_k(&retrieved, &relevant, 5) - 0.4).abs() < 0.001);

        // 1 relevant in top 3
        assert!((precision_at_k(&retrieved, &relevant, 3) - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_ndcg_at_k() {
        let retrieved = vec![1, 2, 3, 4, 5];
        let mut rel_scores = HashMap::new();
        rel_scores.insert(1, 3.0); // highly relevant
        rel_scores.insert(2, 2.0); // relevant
        rel_scores.insert(3, 0.0); // not relevant

        // Perfect ranking would have item 1 first, item 2 second
        let ndcg = ndcg_at_k(&retrieved, &rel_scores, 3);
        assert!(ndcg > 0.0 && ndcg <= 1.0);
    }

    #[test]
    fn test_mrr() {
        let retrieved_lists = vec![
            vec![1, 2, 3, 4],    // First relevant at position 2 (rank 2)
            vec![5, 6, 7, 8],    // First relevant at position 1 (rank 1)
            vec![9, 10, 11, 12], // No relevant items
        ];
        let relevant_sets = vec![
            [2, 4].into_iter().collect(),
            [7].into_iter().collect(),
            [13].into_iter().collect(),
        ];

        // MRR = (1/2 + 1/3 + 0) / 3 = 0.277...
        let mrr_score = mrr(&retrieved_lists, &relevant_sets);
        assert!((mrr_score - 0.277).abs() < 0.01);
    }

    #[test]
    fn test_map() {
        let retrieved_lists = vec![vec![1, 2, 3, 4, 5]];
        let relevant_sets: Vec<HashSet<usize>> = vec![[2, 4].into_iter().collect()];

        // Precision at rank 2 = 1/2, precision at rank 4 = 2/4
        // AP = (0.5 + 0.5) / 2 = 0.5
        let map_score = map(&retrieved_lists, &relevant_sets);
        assert!((map_score - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_reciprocal_rank_fusion() {
        let list1 = vec![("a", 0.9), ("b", 0.8), ("c", 0.7)];
        let list2 = vec![("c", 0.95), ("a", 0.85), ("d", 0.75)];

        let fused = reciprocal_rank_fusion(&[list1, list2], 60);

        // Both lists have "a" and "c" high, they should rank well
        assert!(!fused.is_empty());
        assert!(fused.iter().any(|(item, _)| *item == "a"));
        assert!(fused.iter().any(|(item, _)| *item == "c"));
    }

    #[test]
    fn test_hit_rate_at_k() {
        let retrieved = vec![1, 2, 3, 4, 5];
        let relevant: HashSet<usize> = [6, 7].into_iter().collect();

        // No relevant items in top 5
        assert_eq!(hit_rate_at_k(&retrieved, &relevant, 5), 0.0);

        let relevant2: HashSet<usize> = [3, 7].into_iter().collect();
        // Has relevant item in top 5
        assert_eq!(hit_rate_at_k(&retrieved, &relevant2, 5), 1.0);
    }
}
