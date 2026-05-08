
## 2026-04-17 — SQL Injection in Metadata Queries
**Vulnerability:** Unsanitized user input was interpolated into SQL using `format!` in `query_episodes_by_metadata`.
**Learning:** Even specialized functions like `json_extract` support parameterized paths in `libsql`/SQLite.
**Prevention:** Avoid `format!` for any SQL string construction; always prefer `libsql::params!`.

## 2026-05-08 — Missing Input Bounds in MCP Core Tools
**Vulnerability:** Unbounded 'limit' and 'min_success_rate' parameters in public MCP tools (query_memory, analyze_patterns) allowed for potential resource exhaustion (DoS).
**Learning:** Public MCP tool parameters must be bounded both in the JSON schema (for client-side validation) and in the implementation logic (for server-side enforcement).
**Prevention:** Always use named constants for bounds and enforce them via clamping (e.g., `.min()`) in tool handlers.
