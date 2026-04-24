
## 2026-04-17 — SQL Injection in Metadata Queries
**Vulnerability:** Unsanitized user input was interpolated into SQL using `format!` in `query_episodes_by_metadata`.
**Learning:** Even specialized functions like `json_extract` support parameterized paths in `libsql`/SQLite.
**Prevention:** Avoid `format!` for any SQL string construction; always prefer `libsql::params!`.

## 2026-04-24 — Unbounded Parameters and LIMIT Interpolation
**Vulnerability:** Unsanitized user input reached SQL `LIMIT` clauses via `format!` across multiple storage modules. Retrieval limits and tag counts were also unbounded.
**Learning:** LibSQL safely supports parameterized `LIMIT ?` clauses when bound with `i64` values. Strict input bounds at the API level (e.g., `MAX_TAGS_PER_QUERY`) are necessary for resource safety.
**Prevention:** Use `apply_query_limit` for all retrieval operations and ensure all SQL clauses, including `LIMIT`, use parameter placeholders.
