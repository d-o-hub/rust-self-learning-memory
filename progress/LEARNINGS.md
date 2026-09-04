
## 2026-04-17 — SQL Injection in Metadata Queries
**Vulnerability:** Unsanitized user input was interpolated into SQL using `format!` in `query_episodes_by_metadata`.
**Learning:** Even specialized functions like `json_extract` support parameterized paths in `libsql`/SQLite.
**Prevention:** Avoid `format!` for any SQL string construction; always prefer `libsql::params!`.

## 2026-05-23 — Resource Exhaustion via Unbounded Field Projection
**Vulnerability:** Public MCP tools accepted unbounded arrays of field names for JSON projection, leading to potential CWE-770 (Resource Exhaustion).
**Learning:** Security bounds must be applied not just to scalar 'limit' parameters, but also to collection sizes (vectors/arrays) provided by users.
**Prevention:** Use `.truncate(MAX_CONSTANT)` for user-provided lists and ensure all numeric/floating-point inputs are clamped to safe ranges.

## 2026-07-18 — Unbounded Graph Traversal via max_depth
**Vulnerability:** `SelfLearningMemory::build_relationship_graph` accepted an unbounded `max_depth` parameter, enabling potential denial-of-service or high resource consumption via deep graph traversal.
**Learning:** Public API parameters specifying recursion or graph traversal depth must be bounded at entry.
**Prevention:** Always clamp graph/tree traversal depths using named constants (e.g. `max_depth.min(MAX_RELATIONSHIP_DEPTH)`).
