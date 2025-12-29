# GitHub Issue: Implement Domain-Based Cache Invalidation

**Title**: Implement domain-based cache invalidation for query cache

**Labels**: enhancement, performance, v0.1.13+

**Priority**: P2 (Future Enhancement)

---

## Summary

The current v0.1.12 query cache implementation invalidates **all cached queries** when a new episode completes. This is conservative and correct, but suboptimal for multi-domain workloads or high episode completion rates.

**Goal**: Implement domain-based invalidation to only clear cache entries matching the completed episode's domain, improving cache hit rates.

## Background

**Current Behavior** (v0.1.12):
```rust
// On episode completion in domain "web-api"
self.query_cache.invalidate_all();  // Clears ALL domains 😢
```

**Desired Behavior** (v0.1.13+):
```rust
// On episode completion in domain "web-api"
self.query_cache.invalidate_domain("web-api");  // Only clears "web-api" queries ✨
```

## Impact Analysis

**Scenarios where domain-based invalidation helps:**

| Scenario | Episode Rate | Domains | Current Hit Rate | Expected Improvement |
|----------|--------------|---------|------------------|---------------------|
| Multi-domain agent | 1/min | 5 domains | 20% | +15-20% (→35-40%) |
| Single-domain agent | 1/min | 1 domain | 40% | Minimal (+2-5%) |
| High-throughput | 10/min | 3 domains | 10% | +20-30% (→30-40%) |

**Trigger Conditions** (when to implement):
1. Production monitoring shows cache hit rate <30% for >1 week
2. Multi-domain workloads become common (>50% of users)
3. Episode completion rate >1/minute in production

## Acceptance Criteria

**Must Have:**
1. `QueryCache::invalidate_domain(domain: &str)` method
2. Cache tracks domain per key (extend `CacheKey` to store domain)
3. Integration test showing domain isolation (invalidate A doesn't clear B)
4. Benchmark showing <10µs overhead per invalidation
5. Documentation update in cache.rs module docs

**Nice to Have:**
6. Metrics tracking invalidations per domain
7. Cache statistics by domain (`get_domain_metrics(domain)`)
8. Configurable invalidation strategy (full vs domain-based)

## Technical Design

### 1. Extend Cache to Track Domains

```rust
pub struct QueryCache {
    cache: Arc<RwLock<LruCache<u64, CachedResult>>>,
    // NEW: Track which keys belong to which domain
    domain_index: Arc<RwLock<HashMap<String, HashSet<u64>>>>,
    metrics: Arc<RwLock<CacheMetrics>>,
    // ... existing fields
}
```

### 2. Update on Cache Operations

```rust
impl QueryCache {
    pub fn put(&self, key: CacheKey, episodes: Vec<Episode>) {
        let hash = key.compute_hash();

        // Track domain association
        if let Some(ref domain) = key.domain {
            let mut index = self.domain_index.write().unwrap();
            index.entry(domain.clone())
                .or_insert_with(HashSet::new)
                .insert(hash);
        }

        // ... existing put logic
    }

    pub fn invalidate_domain(&self, domain: &str) {
        let mut cache = self.cache.write().unwrap();
        let mut index = self.domain_index.write().unwrap();

        if let Some(hashes) = index.remove(domain) {
            for hash in hashes {
                cache.pop(&hash);
            }
        }
    }
}
```

### 3. Update Complete Episode Logic

```rust
// memory-core/src/memory/learning.rs
let episode_domain = episode.context.domain.clone();
self.query_cache.invalidate_domain(&episode_domain);
info!(
    episode_id = %episode_id,
    domain = %episode_domain,
    "Invalidated domain-specific cache entries"
);
```

## Implementation Estimate

**Effort**: 2-3 hours
- Extend CacheKey and QueryCache structures (45 min)
- Implement invalidate_domain logic (30 min)
- Add integration tests (30 min)
- Update documentation (30 min)
- Add benchmarks (30 min)

**Testing Strategy**:
1. Unit test: Invalidate domain A, verify domain B unaffected
2. Integration test: Complete episode in domain A, query domain B (should hit cache)
3. Benchmark: Compare invalidate_all() vs invalidate_domain() performance
4. Load test: High episode rate with multiple domains

## Risks & Mitigation

**Risk 1: Memory Growth**
- **Issue**: `domain_index` HashMap grows with unique domains
- **Mitigation**: Use bounded cache (already have LRU eviction)
- **Monitor**: Track `domain_index.len()` in metrics

**Risk 2: Stale Cache Entries**
- **Issue**: If episode changes domain mid-execution, cache might have stale data
- **Mitigation**: Episodes have immutable domains (set at creation)
- **Fallback**: Keep `invalidate_all()` as public API for manual clearing

**Risk 3: Complex Domain Matching**
- **Issue**: Queries might not exactly match episode domain (substrings, wildcards)
- **Mitigation**: Start with exact matching, add fuzzy matching in v0.1.14+ if needed

## Monitoring & Validation

**Success Metrics** (post-implementation):
1. Cache hit rate increases by ≥10% for multi-domain workloads
2. No correctness regressions (cached results == fresh results)
3. Invalidation latency <100µs (vs ~10ms for full invalidation with 1000 entries)
4. Memory overhead <10MB (domain_index is small)

**Monitoring Plan**:
```rust
// Add to cache metrics
pub struct CacheMetrics {
    pub domain_invalidations: HashMap<String, u64>,
    pub domain_hit_rates: HashMap<String, f64>,
    // ... existing fields
}
```

## Related Work

- Original implementation: PR #XXX (v0.1.12 query caching)
- Analysis-swarm review: Identified this as optimization opportunity
- Similar patterns: Redis key prefixes, Memcached namespace invalidation

## Open Questions

1. Should we support wildcards? (e.g., `invalidate_domain("web-*")` clears all web domains)
2. What if episode spans multiple domains? (rare, but possible in complex workflows)
3. Should invalidation be async to avoid blocking episode completion?
4. Do we need domain-based metrics, or is overall hit rate sufficient?

## References

- [v0.1.12 Implementation](../memory-core/src/retrieval/cache.rs)
- [Analysis-swarm Review](./ANALYSIS_SWARM_v0112_REVIEW.md)
- [Query Cache Benchmarks](../benches/query_cache_benchmark.rs)
- [Cache Design Doc](./CACHE_DESIGN_v0112.md)

---

**Created**: 2025-12-29
**Status**: Draft (to be filed after v0.1.12 ships)
**Assigned**: TBD
**Milestone**: v0.1.13 or later (based on monitoring data)
