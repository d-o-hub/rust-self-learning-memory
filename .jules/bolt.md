## 2026-03-27 - [Zero-copy Episodic Memory Retrieval Caching]
**Learning:** The episodic memory retrieval cache was previously performing deep clones of `Episode` structs (including strings, maps, and vectors) on both cache hits and insertions. Storing `Arc<[Arc<Episode>]>` instead of `Arc<[Episode]>` enables true zero-copy retrieval by only cloning atomic pointers, which is significantly more efficient for large result sets.
**Action:** When implementing caches for shared data structures, prefer storing collections of `Arc`s to avoid expensive deep clones during retrieval.
