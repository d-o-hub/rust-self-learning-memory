
## 2026-04-17 — SQL Injection in Metadata Queries
**Vulnerability:** Unsanitized user input was interpolated into SQL using `format!` in `query_episodes_by_metadata`.
**Learning:** Even specialized functions like `json_extract` support parameterized paths in `libsql`/SQLite.
**Prevention:** Avoid `format!` for any SQL string construction; always prefer `libsql::params!`.

## 2026-05-23 — Resource Exhaustion via Unbounded Field Projection
**Vulnerability:** Public MCP tools accepted unbounded arrays of field names for JSON projection, leading to potential CWE-770 (Resource Exhaustion).
**Learning:** Security bounds must be applied not just to scalar 'limit' parameters, but also to collection sizes (vectors/arrays) provided by users.
**Prevention:** Use `.truncate(MAX_CONSTANT)` for user-provided lists and ensure all numeric/floating-point inputs are clamped to safe ranges.

## 2026-07-18 — Unbounded top_k in Hierarchical Reranker
**Vulnerability:** `HierarchicalReranker::rerank_with_query` accepted an unguarded `top_k: usize` parameter, causing unbounded allocation in `Vec::with_capacity(k)`.
**Learning:** High-level reranking / retrieval APIs that take `top_k` must clamp values against system max bounds even if callers are expected to pass reasonable limits.
**Prevention:** Always clamp `top_k` using `top_k.min(MAX_QUERY_LIMIT)` at the entry point of public retrieval APIs.
