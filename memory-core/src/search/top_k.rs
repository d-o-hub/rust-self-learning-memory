//! Top-k selection utilities using O(n) partial sorting.
//!
//! Provides `select_nth_unstable_by` based functions to efficiently find
//! the top-k elements without sorting the entire collection.

/// Select the top-k elements from a slice using partial sorting.
///
/// This is O(n) for finding the k-th element plus O(k log k) for sorting
/// the top k, which is more efficient than O(n log n) full sort when k << n.
///
/// # Arguments
///
/// * `slice` - The slice to partially sort
/// * `k` - Number of top elements to select
/// * `compare` - Comparison function (should return Ordering::Less for "better" elements)
///
/// # Returns
///
/// A vector of the top-k elements in sorted order.
///
/// # Example
///
/// ```
/// use do_memory_core::search::top_k::select_top_k;
///
/// let mut scores = vec![0.1, 0.9, 0.3, 0.8, 0.5];
/// let top = select_top_k(&mut scores, 3, |a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
/// assert_eq!(top.len(), 3);
/// assert!(top[0] >= top[1]);
/// ```
pub fn select_top_k<T, F>(slice: &mut [T], k: usize, mut compare: F) -> Vec<T>
where
    F: FnMut(&T, &T) -> std::cmp::Ordering,
    T: Clone,
{
    let len = slice.len();
    if len == 0 || k == 0 {
        return Vec::new();
    }

    let k = k.min(len);

    // Use select_nth_unstable_by to partition around the k-th element
    // Elements 0..k will be the "top k" (according to compare)
    slice.select_nth_unstable_by(k - 1, &mut compare);

    // Extract and sort just the top k elements
    let mut top_k: Vec<T> = slice[..k].to_vec();
    top_k.sort_by(&mut compare);
    top_k
}

/// Select the top-k elements with their indices preserved.
///
/// Similar to `select_top_k` but works with indexed pairs, returning
/// both the index and the value.
pub fn select_top_k_with_index<T, F>(
    slice: &mut [(usize, T)],
    k: usize,
    mut compare: F,
) -> Vec<(usize, T)>
where
    F: FnMut(&T, &T) -> std::cmp::Ordering,
    T: Clone,
{
    let len = slice.len();
    if len == 0 || k == 0 {
        return Vec::new();
    }

    let k = k.min(len);

    slice.select_nth_unstable_by(k - 1, |a, b| compare(&a.1, &b.1));

    let mut top_k: Vec<(usize, T)> = slice[..k].to_vec();
    top_k.sort_by(|a, b| compare(&a.1, &b.1));
    top_k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_top_k_basic() {
        let mut scores = vec![0.1_f32, 0.9, 0.3, 0.8, 0.5];
        let top = select_top_k(&mut scores, 3, |a, b| {
            b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(top.len(), 3);
        assert_eq!(top, vec![0.9, 0.8, 0.5]);
    }

    #[test]
    fn test_select_top_k_empty() {
        let mut scores: Vec<f32> = vec![];
        let top = select_top_k(&mut scores, 3, |a, b| {
            b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
        });
        assert!(top.is_empty());
    }

    #[test]
    fn test_select_top_k_k_greater_than_n() {
        let mut scores = vec![0.1_f32, 0.9];
        let top = select_top_k(&mut scores, 5, |a, b| {
            b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn test_select_top_k_with_index() {
        let mut indexed: Vec<(usize, f32)> = vec![(0, 0.1), (1, 0.9), (2, 0.3), (3, 0.8)];
        let top = select_top_k_with_index(&mut indexed, 2, |a, b| {
            b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, 1); // Index of 0.9
        assert_eq!(top[1].0, 3); // Index of 0.8
    }
}
