# Dependabot Alert Analysis — 2026-04-02

## Summary

All three open Dependabot alerts are **transitive dependencies** that cannot be
directly patched in this repository. They require upstream fixes.

## Alerts

| Alert # | Dependency | Severity | Chain | Patch Status |
|---------|------------|----------|-------|--------------|
| 12 | rustls-webpki | Medium | libsql → hyper-rustls → rustls → rustls-webpki | v0.103.10 available, needs libsql update |
| 2 | lru | Low | augurs-changepoint → changepoint → rv → lru@0.9.0 | Patch available but transitive |
| 1 | libsql-sqlite3-parser | Low | libsql → libsql-sqlite3-parser | No patch available yet |

## Actions

1. **rustls-webpki (Medium)**: Monitor libsql releases for rustls update
2. **lru (Low)**: Monitor augurs-changepoint releases for dependency updates
3. **libsql-sqlite3-parser (Low)**: Monitor libsql releases

## Resolution

These are accepted risks due to transitive dependency constraints.
The low/medium severity and the fact that these are in development/test
dependencies (not exposed to production traffic) makes them acceptable.

Revisit when upstream patches are available.
