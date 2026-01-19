//! Capacity management types.

use serde::{Deserialize, Serialize};

/// Eviction policy for capacity-constrained storage.
///
/// Determines which episodes to evict when capacity limits are reached.
///
/// # Examples
///
/// ```
/// use memory_core::episodic::EvictionPolicy;
///
/// let policy = EvictionPolicy::RelevanceWeighted;
/// let lru_policy = EvictionPolicy::LRU;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EvictionPolicy {
    /// Least Recently Used - evict oldest episodes first
    LRU,
    /// Relevance-weighted - evict episodes with lowest quality + recency scores
    #[default]
    RelevanceWeighted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eviction_policy_enum() {
        assert_eq!(EvictionPolicy::default(), EvictionPolicy::RelevanceWeighted);
        assert_ne!(EvictionPolicy::LRU, EvictionPolicy::RelevanceWeighted);
    }
}
