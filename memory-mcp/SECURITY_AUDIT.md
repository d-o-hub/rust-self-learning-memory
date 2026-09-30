# MCP Code Execution Security Review

**Date**: 2026-09-30
**Status**: Reconciled with current fail-closed posture (supersedes the
2025-11-07 "MCP Sandbox Security Audit Report")
**Scope**: Agent code execution and MCP security boundaries

---

## Executive Summary

The MCP code-execution **sandbox no longer exists in production**. The
WASM/Wasmtime/Javy backend and its feature names were removed in v0.1.29
(ADR-052). `execute_agent_code` is **fail-closed**: it is not advertised by
`tools/list`, direct and batch calls are rejected, and the handler audit-logs
the attempt and returns "no longer available".

This document replaces the earlier audit that rated a production sandbox
"approved for production use". That report described code and configuration
(`sandbox/{isolation,fs,network}.rs`) that is now compiled only behind the
non-default `sandbox-dev` feature and is **not** wired into any execution path
in a production build.

### Current security score

No production sandbox exists to score. The relevant boundaries are the MCP
server itself: stdio transport, optional OAuth 2.1 authentication, per-client
rate limiting, redacted audit logging, and parameterized storage access.

---

## 1. Current Security Boundaries (production)

### 1.1 Transport

- JSON-RPC 2.0 over **stdio** (`src/bin/memory-mcp-server.rs`). No network
  listener is bound by the default binary.
- Malformed JSON is answered with JSON-RPC errors, not panics.

### 1.2 Authentication & Authorization

- OAuth 2.1 bearer-token validation (`src/bin/server_impl/oauth.rs`), compiled
  only with the `oauth` feature; configured via `MCP_OAUTH_*`.
- When the feature is disabled the server logs that OAuth is disabled.

### 1.3 Rate Limiting

- Per-client token-bucket rate limiting (`src/server/rate_limiter/`) with
  separate read/write limits, configured via `MCP_RATE_LIMIT_*`.

### 1.4 Audit Logging

- Structured JSON audit logging (`src/server/audit/`) for authentication,
  rate-limit violations, security violations, configuration changes, episode
  deletion, and rejected code-execution attempts.
- Recursive key-based redaction (`src/server/audit/redaction.rs`), configured
  via `AUDIT_LOG_REDACT_FIELDS`.

### 1.5 Input Validation & Storage

- Parameterized SQL on all backends (Turso/libSQL, redb). No string
  concatenation of queries.
- Tool parameters validated against schemas with size/range limits.
- Malformed episode/pattern identifiers rejected consistently across
  core/MCP/CLI.

---

## 2. Agent Code Execution (fail-closed)

| Path | State |
|------|-------|
| `tools/list` | `execute_agent_code` **not advertised** |
| `tools/call` (direct) | rejected — error `-32000`, "Tool execution failed" |
| `batch/execute` | rejected identically |
| Handler `handle_execute_code` | audit-logs attempt, returns "no longer available" |
| WASM/Wasmtime/Javy backend | **removed** (v0.1.29, ADR-052) |
| Feature names | `wasmtime-backend`, `javy-backend`, `wasm-rquickjs` **do not exist** |

### 2.1 `sandbox-dev` — trusted local experimentation only

A legacy Node.js `CodeSandbox` (`src/sandbox/`) is compiled **only** with the
non-default `sandbox-dev` feature. It is **not** a production sandbox and MUST
NOT receive untrusted input.

**Enforced:**

- Execution timeout (`max_execution_time_ms`) via Tokio timeout, with
  `kill_on_drop(true)` terminating the child.
- Maximum code length (100 KB) checked before execution.
- Static regex source screening (filesystem/network/subprocess/malicious).
- Global shadowing/deletion in the JavaScript wrapper.

**Not enforced:**

- OS-level isolation. `src/sandbox/isolation.rs` exposes `apply_isolation`
  (privilege drop, `ulimit`, namespaces), but the execution path never calls it.
- Memory (`max_memory_mb`) and CPU (`max_cpu_percent`/`max_cpu_seconds`) limits —
  configuration only.
- Output sanitization — stdout/stderr returned as-is.
- Runtime capability enforcement for filesystem/network/subprocess; the
  `fs.rs`/`network.rs` restrictions are source-pattern checks, not runtime
  controls.

---

## 3. OWASP Top 10 (2021) — current surface

| Risk | Status | Implementation |
|------|--------|----------------|
| A01: Broken Access Control | ✅ | OAuth 2.1 (opt-in) + rate limiting + fail-closed tools |
| A02: Cryptographic Failures | ✅ | TLS to Turso; credentials via env only |
| A03: Injection | ✅ | Parameterized SQL; validated identifiers |
| A04: Insecure Design | ✅ | Fail-closed execution; no untrusted-code path |
| A05: Security Misconfiguration | ✅ | Sandbox-dev not built by default; env-driven config |
| A06: Vulnerable Components | ✅ | `cargo audit` in CI |
| A07: Identification/Authentication | ✅ | OAuth 2.1 when `oauth` enabled |
| A08: Software/Data Integrity | ✅ | No production code execution path |
| A09: Security Logging/Monitoring | ✅ | Structured audit logging with redaction |
| A10: Server-Side Request Forgery | ✅ | No network listener; no egress from server |

---

## 4. Recommendations

### Immediate (High Priority)

- Keep `execute_agent_code` fail-closed and out of tool discovery.
- Do **not** enable `sandbox-dev` in production builds.

### Short Term (Medium Priority)

1. Ensure OAuth and rate limiting are enabled and tuned for shared deployments.
2. Configure audit destinations and redaction field lists.
3. Keep the reachability/docs-integrity checks green
   (`scripts/check-source-reachability.sh`, `scripts/check-docs-integrity.sh`).

### Long Term (Low Priority)

1. If code execution is ever reintroduced, adopt an approved capability-enforced
   backend per ADR-073 (WASI/component model with enforced capabilities).
2. Add runtime behavior analysis and resource enforcement only together with a
   real backend.

---

## 5. Conclusion

There is **no production sandbox** for agent code. The MCP server's security
rests on its transport, authentication, rate limiting, audit logging, and
parameterized storage access. The legacy Node executor is trusted-local only and
implements only timeout/length/regex controls — no OS isolation, no enforced
memory/CPU limits, no output sanitization.

**Superseded finding**: the earlier "APPROVED FOR PRODUCTION USE" rating applied
to a sandbox that no longer exists and must not be relied upon.

---

## 6. Related Documents

- [README.md](README.md) — fail-closed code-execution contract and `sandbox-dev` limits
- [SECURITY.md](SECURITY.md) — current security boundaries
- [../plans/adr/ADR-073-Capability-Enforced-Agent-Code-Execution.md](../plans/adr/ADR-073-Capability-Enforced-Agent-Code-Execution.md)
- [../plans/adr/ADR-052-Comprehensive-Analysis-v0.1.29.md](../plans/adr/ADR-052-Comprehensive-Analysis-v0.1.29.md)

---

**End of Security Review**
