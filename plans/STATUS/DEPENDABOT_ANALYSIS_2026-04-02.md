# Dependabot Alert Analysis — 2026-04-05

## Summary

All three open Dependabot alerts are **transitive dependencies** that cannot be
directly patched in this repository. They require upstream fixes.

## Alerts

| Alert # | Dependency | Severity | Chain | Patch Status |
|---------|------------|----------|-------|--------------|
| 12 | rustls-webpki | Medium | libsql → hyper-rustls → rustls → rustls-webpki | v0.103.10 available, blocked by rustls 0.22 constraint |
| 2 | lru | Low | augurs-changepoint → changepoint → rv → lru@0.9.0 | Patch available but transitive |
| 1 | libsql-sqlite3-parser | Low | libsql → libsql-sqlite3-parser | No patch available yet |

## Actions Taken (2026-04-05)

1. Updated libsql from 0.9.29 to 0.9.30
2. Ran `cargo update` for latest compatible versions
3. Removed wasmtime advisory ignores from `.cargo/audit.toml` (WASM removed in v0.1.29)
4. Updated audit.toml documentation for remaining transitive vulnerabilities

## Resolution

These are accepted risks due to transitive dependency constraints.
The low/medium severity and the fact that these are in development/test
dependencies (not exposed to production traffic) makes them acceptable.

### Blocking Factors

- **rustls-webpki**: Requires libsql to migrate from hyper-rustls 0.25 (hyper 0.14) to 0.27 (hyper 1.x)
- **lru**: Requires augurs-changepoint to update its changepoint/rv dependencies
- **libsql-sqlite3-parser**: Requires libsql to update the parser dependency

Revisit when upstream patches are available.
