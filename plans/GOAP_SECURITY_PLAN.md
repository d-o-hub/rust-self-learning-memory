# GOAP: Security Improvement Plan

**Date**: 2026-05-16
**Type**: Security Hardening Plan
**Priority**: P1 - System security
**WG**: WG-145

---

## Executive Summary

**Goal**: Systematically audit and harden security across the memory storage system, including dependency security, OAuth/JWT handling, and production deployment security.

**Current State**: Basic security measures in place (parameterized SQL, env vars, dependency audits).
**Target**: Full security coverage with automated verification.

---

## Phase 1: ANALYZE

### Current Security Posture

| Area | Status | Risk Level |
|------|--------|------------|
| Parameterized SQL | ✅ Implemented | Low |
| OAuth/JWT verification | ✅ Implemented (jsonwebtoken) | Low |
| Dependency audits | ✅ cargo-audit in CI | Low |
| Environment variable management | ✅ Implemented | Low |
| Input validation | ⚠️ Partial review needed | Medium |
| Rate limiting | ⚠️ Needs tuning | Medium |
| Audit logging | ⚠️ Needs verification | Medium |
| MCP tool access control | ⚠️ Needs review | Medium |

### Threat Model

| Threat | Vector | Severity | Mitigation |
|--------|--------|----------|------------|
| SQL injection | Storage input | HIGH | Parameterized SQL ✅ |
| Token theft | MCP server | HIGH | JWT verification ✅ |
| Supply chain | Dependencies | MEDIUM | cargo-audit ✅ |
| Embedding injection | OpenAI API | MEDIUM | Input validation ⚠️ |
| Denial of service | MCP tools | LOW | Rate limiting ⚠️ |
| Data leakage | Error messages | LOW | Review error handling ⚠️ |

---

## Phase 2: DECOMPOSE

### WG Tasks

| WG | Task | Priority | Dependencies |
|----|------|----------|--------------|
| WG-145.1 | Complete dependency audit (cargo-deny) | HIGH | None |
| WG-145.2 | Verify OAuth/JWT production settings | HIGH | None |
| WG-145.3 | Review input validation across MCP tools | MEDIUM | None |
| WG-145.4 | Tune rate limiting for production | MEDIUM | WG-145.3 |
| WG-145.5 | Verify audit logging works end-to-end | MEDIUM | None |
| WG-145.6 | Run gitleaks scan on repository | HIGH | None |
| WG-145.7 | Document security procedures in SECURITY.md | MEDIUM | All above |

---

## Phase 3: EXECUTE

### Sprint 1: Audit

```
Week 1: WG-145.1 (dependency audit) + WG-145.2 (JWT verification)
Week 2: WG-145.4 (rate limiting) + WG-145.5 (audit logging)
Week 3: WG-145.3 (input validation) + WG-145.6 (gitleaks scan)
Week 4: WG-145.7 (documentation)
```

### Audit Commands

```bash
# Run security audits
cargo audit
cargo deny check

# Run gitleaks
gitleaks detect --verbose

# Verify OAuth config
grep -r "MCP_OAUTH_TOKEN_SECRET" memory-mcp/
```

---

## Quality Gates

| Milestone | Check | Target |
|-----------|-------|--------|
| Dependency audit | cargo-audit + cargo-deny pass | All clear |
| JWT verification | Production config verified | HMAC signature |
| Input validation | MCP tool input review | All tools reviewed |
| Audit logging | End-to-end test | Events logged |
| Gitleaks scan | No secrets committed | Zero findings |

---

## Cross-References

- SECURITY.md: Security guidelines
- DEPLOYMENT_SECURITY.md: Deployment security
- AUDIT_LOGGING_SETUP.md: Audit logging setup
- SECURITY_OPERATIONS.md: Security operations
- GOAP_STATE.md: Current GOAP state tracking
