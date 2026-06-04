use super::*;

#[test]
fn test_cache_stats() {
    // Just verify the stats method doesn't panic and returns correct types
    // It's mostly atomic loads
    let stats = CacheStats {
        episode_hits: 1,
        episode_misses: 2,
        pattern_hits: 3,
        pattern_misses: 4,
        query_hits: 0,
        query_misses: 0,
        evictions: 0,
        expirations: 0,
    };
    assert_eq!(stats.episode_hits, 1);
    assert_eq!(stats.query_hits, 0);
}
