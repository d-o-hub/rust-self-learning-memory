# Security Analysis: Memory MCP Server

## Overview

This document describes the security posture of the `do-memory-mcp` server as
it exists today. It is a security analysis of the **live** boundaries —
transport, authentication, authorization, rate limiting, audit logging,
input validation, and the fail-closed code-execution policy — not of a removed
sandbox.

> **Code execution is fail-closed.** `execute_agent_code` is unavailable in
> production: it is not advertised by `tools/list`, direct calls are rejected,
> and there is no WASM/Wasmtime/Javy execution backend. See
> [ADR-073](../plans/adr/ADR-073-Capability-Enforced-Agent-Code-Execution.md)
> and [ADR-052](../plans/adr/ADR-052-Comprehensive-Analysis-v0.1.29.md).
> An earlier WASM sandbox was removed in v0.1.29; no production sandbox
> behavior is promised or implemented.

## Threat Model

### Attacker Capabilities

We assume an attacker can:

- Send arbitrary MCP JSON-RPC requests to the server's transport.
- Attempt to invoke tools with malformed, oversized, or hostile arguments.
- Attempt to invoke `execute_agent_code` or other unavailable tools.
- Attempt to exhaust CPU, memory, or storage through query/ingest pressure.
- Attempt to inject SQL or otherwise tamper with storage queries.
- Attempt to exfiltrate secrets through logs or tool responses.

### Assets to Protect

1. **Stored memory**: episodes, patterns, tags, and relationships.
2. **Host system**: files, processes, network, and environment.
3. **Credentials**: OAuth JWKS/secret material and embedding provider keys.
4. **System resources**: CPU, memory, disk, and network bandwidth.
5. **Audit integrity**: trustworthy, tamper-resistant security event records.

## Security Boundaries

### 1. Transport

- The server communicates over **stdio using JSON-RPC 2.0**
  (`src/bin/memory-mcp-server.rs`). There is no network listener bound by the
  default binary.
- Requests are parsed defensively; malformed JSON yields JSON-RPC error
  responses rather than panics.

### 2. Authentication & Authorization (OAuth 2.1, opt-in)

- OAuth 2.1 bearer-token validation lives in `src/bin/server_impl/oauth.rs` and
  is compiled only with the `oauth` feature.
- Configuration is read from environment variables:
  `MCP_OAUTH_ENABLED`, `MCP_OAUTH_AUDIENCE`, `MCP_OAUTH_ISSUER`,
  `MCP_OAUTH_SCOPES`, `MCP_OAUTH_JWKS_URI`, `MCP_OAUTH_TOKEN_SECRET`.
- When enabled, tokens are validated (JWT signature verification, issuer,
  audience, and scope checks) and failures produce `WWW-Authenticate`
  challenges.
- When the `oauth` feature is disabled, the server logs that OAuth is disabled
  and continues; deployments that require authentication MUST enable it.

### 3. Code Execution (fail-closed)

- `execute_agent_code` is **not** a working execution backend. It is absent from
  tool discovery and direct calls are rejected
  (`src/bin/server_impl/handlers/call_tool.rs`,
  `src/bin/server_impl/handlers/batch_execute.rs`).
- The handler in `src/bin/server_impl/tools/memory_handlers.rs` audit-logs the
  attempt and returns a "no longer available" error.
- No WASM, Wasmtime, Javy, or rquickjs dependency or feature exists in this
  crate; the `wasmtime-backend`, `javy-backend`, and `wasm-rquickjs` names were
  removed in v0.1.29.

#### `sandbox-dev` (trusted local experimentation only)

A legacy Node.js `CodeSandbox` (`src/sandbox/`) remains compiled **only** behind
the non-default `sandbox-dev` feature. It is **not** a production sandbox and
MUST NOT receive untrusted input.

**Enforced:** execution timeout with `kill_on_drop`, 100 KB code-length cap,
static regex source screening (filesystem/network/subprocess/malicious), and
global shadowing/deletion in the generated JavaScript wrapper.

**Not enforced:** OS-level isolation (`src/sandbox/isolation.rs` exposes
`apply_isolation`, but the execution path never calls it), memory and CPU
limits (configuration-only), output sanitization, and runtime capability
enforcement. Regex screening is heuristic and bypassable via runtime
obfuscation.

### 4. Rate Limiting

- Token-bucket per-client rate limiting (`src/server/rate_limiter/`) protects the
  server from DoS. Limits are configurable via `MCP_RATE_LIMIT_*` environment
  variables (read/write RPS and burst, cleanup interval, stale threshold).
- Read and write operations are limited separately.

### 5. Audit Logging

- Structured JSON audit logging (`src/server/audit/`) records security-relevant
  events: authentication, rate-limit violations, security violations,
  configuration changes, episode deletion, and rejected code-execution attempts.
- Sensitive fields are recursively redacted by key
  (`src/server/audit/redaction.rs`); the redaction field list is configurable
  via `AUDIT_LOG_REDACT_FIELDS`.
- Configuration is read from `AUDIT_LOG_*` environment variables (enable,
  destination, file path, rotation, level).

### 6. Input Validation & Storage

- All database access uses **parameterized SQL** via the Turso/libSQL and redb
  storage backends; queries are never built by string concatenation.
- Tool parameters are validated against their schemas, including length and
  range constraints, before use.
- Episode and pattern identifiers are validated and malformed IDs rejected
  consistently across core, MCP, and CLI.

### 7. Data Protection

- Credentials are read from environment variables and are never persisted or
  echoed in responses, warnings, errors, or audit fields.
- Embedding activation is atomic; failed activation leaves the prior provider
  unchanged.
- Transport security for external storage (Turso) is handled by TLS at the
  client layer.

## Attack Scenarios

### 1. SQL Injection

**Attack**: Supply crafted episode IDs or filters to alter queries.

**Defense**: Parameterized statements across all backends; identifiers are
validated. ✅ **MITIGATED**

### 2. Unauthorized Tool Use

**Attack**: Invoke `execute_agent_code` or other privileged operations without
authorization.

**Defense**: `execute_agent_code` is rejected fail-closed; when OAuth is
enabled, tools require a valid bearer token with the appropriate scope.
✅ **MITIGATED** (with `oauth` enabled for authenticated deployments)

### 3. Resource Exhaustion

**Attack**: Flood the server with requests or large payloads.

**Defense**: Per-client token-bucket rate limiting, input size limits, and
schema constraints. ⚠️ **PARTIAL** — limit tuning is deployment-specific.

### 4. Secret Leakage via Logs or Responses

**Attack**: Cause secrets to be written to logs or returned in tool output.

**Defense**: Credentials are read from env and never stored or echoed; audit
metadata is redacted by key. ⚠️ **PARTIAL** — redaction depends on configured
field names.

### 5. Prompt Injection via Stored Memory

**Attack**: Store adversarial content in episodes that later influences an
agent.

**Defense**: Out of scope for the server's authentication boundary; memory
content is treated as untrusted by clients. ⚠️ **NOT MITIGATED** at the server
layer — consumers must treat retrieved memory as untrusted data.

## Deployment Recommendations

### Container Hardening

Run the server with a non-root user, dropped capabilities, a read-only root
filesystem, and explicit resource limits:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: do-memory-mcp-server
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
  containers:
  - name: mcp-server
    image: do-memory-mcp:latest
    resources:
      limits:
        memory: "256Mi"
        cpu: "500m"
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop: ["ALL"]
```

Because there is no in-process sandbox for arbitrary code, resource limits are
enforced by the deployment substrate (cgroups, container limits), not by the
MCP server.

### Configuration

- Enable OAuth (`oauth` feature + `MCP_OAUTH_*`) for any shared deployment.
- Keep rate limiting enabled and tuned (`MCP_RATE_LIMIT_*`).
- Enable audit logging to a durable, access-controlled destination
  (`AUDIT_LOG_*`) and set `AUDIT_LOG_REDACT_FIELDS`.
- Do **not** enable `sandbox-dev` in production.

## Security Checklist

Before deploying:

- [ ] OAuth enabled with correct issuer/audience/scopes (if shared)
- [ ] Rate limiting enabled and tuned
- [ ] Audit logging enabled with redaction fields configured
- [ ] Dependencies audited (`cargo audit`)
- [ ] No hardcoded secrets; credentials via environment variables
- [ ] TLS configured for Turso connections
- [ ] Container runs non-root with dropped capabilities and resource limits
- [ ] `sandbox-dev` feature disabled
- [ ] Log rotation configured
- [ ] Incident response plan documented
- [ ] Backup/recovery tested

## Incident Response

If a security breach is suspected:

1. **Contain**: restrict access to the server and its transports.
2. **Isolate**: quarantine affected host and storage.
3. **Analyze**: review audit logs and execution history (respecting redaction).
4. **Remediate**: patch and redeploy.
5. **Monitor**: watch for similar patterns.
6. **Report**: document the incident and lessons learned.

## Responsible Disclosure

Security vulnerabilities should be reported privately via the repository's
GitHub Security Advisory workflow. Do NOT create public issues for security
vulnerabilities.

## Conclusion

The server's production security posture rests on a stdio-only transport,
optional OAuth 2.1 authentication, per-client rate limiting, structured and
redacted audit logging, parameterized storage access, and a **fail-closed**
code-execution policy. There is no production sandbox for arbitrary agent code;
deployments that need to run untrusted code must do so in an external,
separately hardened runner.

**Last Updated**: 2026-09-30
