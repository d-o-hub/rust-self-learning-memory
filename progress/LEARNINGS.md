
## 2026-04-17 — SQL Injection in Metadata Queries
**Vulnerability:** Unsanitized user input was interpolated into SQL using `format!` in `query_episodes_by_metadata`.
**Learning:** Even specialized functions like `json_extract` support parameterized paths in `libsql`/SQLite.
**Prevention:** Avoid `format!` for any SQL string construction; always prefer `libsql::params!`.

## 2026-05-23 — Resource Exhaustion via Unbounded Field Projection
**Vulnerability:** Public MCP tools accepted unbounded arrays of field names for JSON projection, leading to potential CWE-770 (Resource Exhaustion).
**Learning:** Security bounds must be applied not just to scalar 'limit' parameters, but also to collection sizes (vectors/arrays) provided by users.
**Prevention:** Use `.truncate(MAX_CONSTANT)` for user-provided lists and ensure all numeric/floating-point inputs are clamped to safe ranges.

## 2026-03-06 — Unbounded top_k in HierarchicalReranker::rerank_with_query
**Vulnerability:** `HierarchicalReranker::rerank_with_query` accepted an unbounded `top_k: usize` parameter which was passed directly to `Vec::with_capacity(top_k)` in `select_diverse`, leading to potential out-of-memory panics (DoS) when called with large `top_k` values like `usize::MAX`.
**Learning:** Public retrieval APIs accepting limits/top_k parameters must bound user-provided values prior to vector allocations.
**Prevention:** Clamp caller-provided `top_k` / `limit` parameters with existing constants like `crate::storage::MAX_QUERY_LIMIT` before allocation.

## 2026-10-01 — Unbounded limit in SpatiotemporalIndex::query_range
**Vulnerability:** `SpatiotemporalIndex::query_range` accepted an unbounded `limit: usize` parameter which caused full time hierarchy tree traversal and unbounded vector accumulation when passed large values like `usize::MAX`.
**Learning:** Index range query functions must clamp `limit` before embarking on multi-node tree traversals to prevent high CPU and memory resource consumption.
**Prevention:** Clamp caller-provided `limit` using `crate::storage::MAX_QUERY_LIMIT` at the entry point of query functions.
