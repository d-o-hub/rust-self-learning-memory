# Turso Cache Wrapper Benchmark Analysis

## Summary
The `CachedTursoStorage` wrapper provides a transparent caching layer for episodes, patterns, and heuristics. By utilizing an adaptive TTL cache, it significantly reduces database I/O for frequently accessed items.

## Benchmark Methodology
Benchmarks were implemented using Criterion with `iter_batched` to isolate the performance of cache logic from expensive setup tasks such as database initialization and filesystem I/O.

### Measured Scenarios
1.  **Cold Reads**: First access to an item after storage, triggering a cache miss and fill.
2.  **Warm Reads**: Subsequent accesses to items already present in the cache.
3.  **Mixed Access (80/20)**: Realistic workload simulation with 80% hits and 20% misses.
4.  **Payload Size Comparison**: Overhead of caching small episodes vs. large episodes (100 steps).
5.  **Uncached Baseline**: Direct storage access for comparison.

## Preliminary Findings

### Performance Gains
*   **Latency Reduction**: Warm cache reads are orders of magnitude faster than direct database reads, as they avoid serialization overhead and SQL execution.
*   **Adaptive Efficiency**: The adaptive TTL logic ensures that "hot" items stay in memory longer while "cold" items are evicted, optimizing memory usage.

### Cost Analysis
1.  **Cache Logic**: Minimal overhead. Atomic operations for statistics and LRU management are very efficient.
2.  **Serialization**: The primary bottleneck for misses. `postcard` is used for efficient serialization, but large episodes still incur non-trivial CPU costs during the miss-and-fill phase.
3.  **Storage I/O**: The dominant cost for cold reads. Network or local disk latency for SQL queries outweighs all other factors.

## Recommendations for Optimization
*   **Zero-Copy Deserialization**: Investigate if the cache can store items in a way that minimizes cloning or re-serialization.
*   **Batch Priming**: For scenarios with known access patterns, pre-filling the cache can hide the "cold read" latency.
*   **Negative Caching**: Consider caching `None` results for a short duration to prevent repeated database queries for non-existent IDs in high-miss scenarios.

## Conclusion
The `CachedTursoStorage` wrapper is a critical performance component. These benchmarks establish a reliable baseline for measuring future improvements and ensuring that the retrieval path remains efficient as payload complexity increases.
