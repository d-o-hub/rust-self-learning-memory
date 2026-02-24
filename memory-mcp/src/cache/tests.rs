use super::*;

#[test]
fn cache_starts_enabled_by_default() {
    let cache = QueryCache::new();
    let stats = cache.stats();
    assert!(stats.enabled);
}
