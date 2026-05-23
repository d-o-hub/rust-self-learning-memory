
## 2026-04-17 — SQL Injection in Metadata Queries
**Vulnerability:** Unsanitized user input was interpolated into SQL using `format!` in `query_episodes_by_metadata`.
**Learning:** Even specialized functions like `json_extract` support parameterized paths in `libsql`/SQLite.
**Prevention:** Avoid `format!` for any SQL string construction; always prefer `libsql::params!`.

## 2026-05-02 — Resource Exhaustion via Unbounded Field Projection
**Vulnerability:** Public MCP tools accepted unbounded arrays of field names for JSON projection, leading to potential CWE-770 (Resource Exhaustion).
**Learning:** Security bounds must be applied not just to scalar 'limit' parameters, but also to collection sizes (vectors/arrays) provided by users.
**Prevention:** Use `.truncate(MAX_CONSTANT)` for user-provided lists and ensure all numeric/floating-point inputs are clamped to safe ranges.
