# OAuth 2.1 Implementation Plan

**Document Type**: GOAP Execution Plan
**Version**: 1.0
**Created**: 2025-12-25
**Priority**: P3 (Security Enhancement)
**Estimated Effort**: 40-60 hours
**Duration**: 2-3 weeks

---

## Executive Summary

Implement OAuth 2.1 authentication and authorization for the memory system, providing secure access control with **incremental scope consent**. This enhancement adds granular permissions for memory operations while maintaining compatibility with existing anonymous access patterns.

**Primary Goal**: Secure memory access with user-controlled permissions
**Security Impact**: High (authentication, authorization, audit trail)
**Migration Impact**: Low (opt-in enhancement, backward compatible)

---

## GOAP Analysis

### Task Complexity
**Complexity**: Medium
**Reasoning**: Requires OAuth 2.1 provider integration, scope design, and authorization middleware. No breaking changes to existing functionality.

### Dependencies
- MCP 2025-11-25 OAuth enhancements research findings
- Existing Memory-MCP infrastructure
- OAuth 2.1 provider (e.g., Auth0, Keycloak, or custom)
- Existing storage layer (Turso + redb)

### Success Criteria
- [ ] OAuth 2.1 authentication working
- [ ] Scope-based authorization for memory operations
- [ ] Client ID Metadata Documents implemented
- [ ] All existing tests still passing (backward compatible)
- [ ] Security audit passing
- [ ] Documentation complete

---

## Execution Plan

### Phase 1: Design and Architecture (Week 1, 15-20 hours)

#### Task 1.1: Scope Model Design (5 hours)
**Agent**: feature-implementer
**Dependencies**: None
**Description**: Design OAuth 2.1 scopes for memory operations

**Scopes Definition**:
```
memory:read       - Query episodes, patterns, and metrics
memory:write      - Create, update episodes and patterns
memory:delete     - Delete episodes and patterns
memory:analyze    - Run pattern analysis and analytics
memory:admin      - Full system administration
```

**Deliverables**:
- Scope model document (`docs/OAUTH_SCOPES.md`)
- Permission matrix for each MCP tool
- Scope validation rules

**Success Criteria**:
- Scopes defined for all memory operations
- Permission matrix complete
- Validation rules documented

---

#### Task 1.2: OAuth Provider Selection (5 hours)
**Agent**: feature-implementer
**Dependencies**: Task 1.1
**Description**: Evaluate and select OAuth 2.1 provider

**Options**:
1. **Auth0**: Managed service, fast implementation
2. **Keycloak**: Self-hosted, full control
3. **Custom**: Using `oauth2` crate, maximum flexibility

**Evaluation Criteria**:
- OAuth 2.1 compliance
- Incremental scope consent support
- Integration complexity
- Operational overhead
- Cost implications

**Deliverables**:
- Provider comparison matrix
- Selection recommendation
- Integration architecture diagram

**Success Criteria**:
- Provider selected with justification
- Integration plan documented
- OAuth 2.1 compliance verified

---

#### Task 1.3: Authorization Middleware Design (5-10 hours)
**Agent**: feature-implementer
**Dependencies**: Tasks 1.1, 1.2
**Description**: Design authorization middleware for Memory-MCP

**Components**:
- Token validation middleware
- Scope extraction and validation
- Permission enforcement layer
- Audit logging for authorization decisions

**Architecture**:
```
Request → OAuth Middleware → Scope Check → Tool Handler → Response
                      ↓
                 Audit Log
```

**Deliverables**:
- Middleware design document
- Sequence diagrams for authorization flow
- Error handling strategy
- Audit log format specification

**Success Criteria**:
- Middleware architecture designed
- Authorization flow documented
- Error handling complete
- Audit logging specified

---

### Phase 2: Implementation (Week 2, 20-25 hours)

#### Task 2.1: OAuth 2.1 Client Integration (8 hours)
**Agent**: feature-implementer
**Dependencies**: Phase 1 complete
**Description**: Implement OAuth 2.1 client integration

**Implementation**:
```rust
// memory-mcp/src/auth/oauth.rs
pub struct OAuthClient {
    client_id: String,
    client_secret: String,
    auth_url: Url,
    token_url: Url,
    scopes: Vec<String>,
}

impl OAuthClient {
    pub async fn validate_token(&self, token: &str) -> Result<TokenInfo> {
        // Validate JWT token
        // Extract scopes
        // Verify expiration
    }

    pub fn has_scope(&self, token_info: &TokenInfo, scope: &str) -> bool {
        token_info.scopes.contains(&scope.to_string())
    }
}
```

**Deliverables**:
- OAuth client implementation
- Token validation logic
- Scope extraction
- Unit tests

**Success Criteria**:
- Token validation working
- Scopes extracted correctly
- All unit tests passing
- Error handling comprehensive

---

#### Task 2.2: Authorization Middleware (8 hours)
**Agent**: feature-implementer
**Dependencies**: Task 2.1
**Description**: Implement authorization middleware

**Implementation**:
```rust
// memory-mcp/src/auth/middleware.rs
pub struct AuthMiddleware {
    oauth_client: Arc<OAuthClient>,
}

impl AuthMiddleware {
    pub async fn authorize(&self, request: &ToolRequest) -> Result<AuthContext> {
        let token = self.extract_token(request)?;
        let token_info = self.oauth_client.validate_token(&token).await?;

        let required_scope = self.get_required_scope(request.tool)?;
        if !self.oauth_client.has_scope(&token_info, &required_scope) {
            return Err(Error::InsufficientScope(required_scope));
        }

        Ok(AuthContext {
            user_id: token_info.user_id,
            scopes: token_info.scopes,
        })
    }
}
```

**Deliverables**:
- Middleware implementation
- Scope-to-tool mapping
- Authorization context struct
- Unit tests

**Success Criteria**:
- Middleware authorizes requests correctly
- Scope enforcement working
- All unit tests passing
- Error messages clear

---

#### Task 2.3: MCP Tool Integration (6-7 hours)
**Agent**: feature-implementer
**Dependencies**: Task 2.2
**Description**: Integrate authorization into all MCP tools

**Integration Pattern**:
```rust
// memory-mcp/src/tools/query_memory.rs
pub async fn query_memory(
    request: ToolRequest,
    auth: AuthContext,
    memory: Arc<SelfLearningMemory>,
) -> Result<ToolResponse> {
    // Authorization handled by middleware
    // Proceed with query
}
```

**Scope Mappings**:
- `query_memory` → `memory:read`
- `complete_episode` → `memory:write`
- `delete_episode` → `memory:delete`
- `analyze_patterns` → `memory:analyze`

**Deliverables**:
- All MCP tools integrated
- Scope mappings configured
- Integration tests
- Updated JSON schemas

**Success Criteria**:
- All tools require authorization
- Scope mappings correct
- Integration tests passing
- Anonymous access still works (opt-in)

---

#### Task 2.4: Client ID Metadata Documents (3 hours)
**Agent**: feature-implementer
**Dependencies**: Task 2.1
**Description**: Implement Client ID Metadata Documents

**Specification**:
```json
{
  "client_id": "memory-mcp-client",
  "client_name": "Memory System MCP",
  "client_uri": "https://example.com",
  "logo_uri": "https://example.com/logo.png",
  "policy_uri": "https://example.com/privacy",
  "tos_uri": "https://example.com/terms"
}
```

**Deliverables**:
- Client ID metadata endpoint
- Metadata document structure
- Documentation

**Success Criteria**:
- Metadata document accessible
- OAuth 2.1 specification compliant
- Documented

---

### Phase 3: Testing & Security (Week 3, 10-15 hours)

#### Task 3.1: Security Testing (5 hours)
**Agent**: code-reviewer
**Dependencies**: Phase 2 complete
**Description**: Comprehensive security testing

**Test Coverage**:
- [ ] Token validation attacks (invalid, expired, malformed)
- [ ] Scope bypass attempts
- [ ] Authorization header injection
- [ ] CSRF protection
- [ ] Rate limiting per user
- [ ] Audit log integrity

**Deliverables**:
- Security test suite
- Penetration test results
- Security recommendations

**Success Criteria**:
- All security tests passing
- No critical vulnerabilities
- Audit logging comprehensive

---

#### Task 3.2: Integration Testing (3 hours)
**Agent**: test-runner
**Dependencies**: Phase 2 complete
**Description**: End-to-end integration testing

**Test Scenarios**:
- [ ] Successful OAuth flow
- [ ] Scope enforcement per tool
- [ ] Token refresh workflow
- [ ] Error handling for unauthorized access
- [ ] Backward compatibility with anonymous access

**Deliverables**:
- Integration test suite
- Test results
- Bug fixes (if any)

**Success Criteria**:
- All integration tests passing
- Anonymous access still works
- OAuth flow complete

---

#### Task 3.3: Documentation (4-5 hours)
**Agent**: feature-implementer
**Dependencies**: Phase 2 complete
**Description**: Complete documentation

**Documentation**:
- [ ] OAuth 2.1 setup guide (`docs/OAUTH_SETUP.md`)
- [ ] Scope reference (`docs/OAUTH_SCOPES.md`)
- [ ] Client registration process
- [ ] Troubleshooting guide
- [ ] API documentation updates

**Deliverables**:
- Setup guide
- Scope reference
- Troubleshooting documentation
- Updated README and AGENTS.md

**Success Criteria**:
- Setup guide complete with examples
- All scopes documented
- Troubleshooting guide helpful
- README mentions OAuth capability

---

## Quality Gates

### Phase 1 Quality Gates
- [ ] Scope model approved by security review
- [ ] OAuth provider selected with justification
- [ ] Middleware architecture reviewed

### Phase 2 Quality Gates
- [ ] OAuth client validates tokens correctly
- [ ] Middleware enforces scopes properly
- [ ] All MCP tools integrated
- [ ] Client ID metadata compliant

### Phase 3 Quality Gates
- [ ] Security tests passing (100%)
- [ ] Integration tests passing (100%)
- [ ] No breaking changes to existing functionality
- [ ] Documentation complete and accurate

### Final Quality Gate
- [ ] Security audit passing
- [ ] Performance impact <5% (authorization overhead)
- [ ] Test coverage >90%
- [ ] Backward compatibility maintained

---

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| **OAuth provider downtime** | High | Low | Fallback to anonymous access |
| **Scope bypass vulnerability** | Critical | Low | Security testing + code review |
| **Performance degradation** | Medium | Low | Benchmark authorization |
| **Complexity for users** | Medium | High | Clear documentation + examples |
| **Breaking changes** | High | Low | Opt-in, maintain backward compatibility |

---

## Timeline

| Week | Tasks | Hours | Key Deliverables |
|------|-------|-------|------------------|
| **Week 1** | Phase 1: Design | 15-20 | Scope model, provider selected, middleware design |
| **Week 2** | Phase 2: Implementation | 20-25 | OAuth client, middleware, tool integration |
| **Week 3** | Phase 3: Testing | 10-15 | Security tests, integration tests, documentation |

**Total Effort**: 45-60 hours
**Duration**: 2-3 weeks

---

## Agent Coordination Strategy

**Execution Strategy**: Sequential (phases must complete in order)

**Rationale**: Each phase depends on the previous phase's outputs.

**Agent Assignments**:
- **feature-implementer**: Design and implementation (45 hours)
- **code-reviewer**: Security testing and review (5 hours)
- **test-runner**: Integration testing (3 hours)

**Coordination**:
1. Phase 1: feature-implementer designs architecture
2. Phase 2: feature-implementer implements OAuth integration
3. Phase 3: code-reviewer and test-runner validate in parallel

---

## Deliverables Summary

### Phase 1 Deliverables
- Scope model document (`docs/OAUTH_SCOPES.md`)
- Provider comparison and selection
- Middleware architecture design
- OAuth 2.1 integration plan

### Phase 2 Deliverables
- OAuth client implementation (`memory-mcp/src/auth/oauth.rs`)
- Authorization middleware (`memory-mcp/src/auth/middleware.rs`)
- MCP tool integration with authorization
- Client ID metadata endpoint
- Unit tests for all components

### Phase 3 Deliverables
- Security test suite and results
- Integration test suite
- Setup guide (`docs/OAUTH_SETUP.md`)
- Troubleshooting documentation
- Updated project documentation

---

## Success Metrics

| Metric | Baseline | Target | Status |
|--------|----------|--------|--------|
| **Security** | Anonymous only | OAuth 2.1 + scopes | ⏳ TBD |
| **Test Coverage** | Current | >90% | ⏳ TBD |
| **Performance** | Baseline | <5% overhead | ⏳ TBD |
| **Backward Compatibility** | N/A | 100% | ⏳ TBD |
| **Documentation** | Basic | Comprehensive | ⏳ TBD |

---

## Next Steps

1. **Review this GOAP execution plan** with technical team
2. **Approve OAuth provider selection** (Week 1)
3. **Begin Phase 1 implementation** upon approval
4. **Schedule security audit** for Phase 3 completion

---

**Document Status**: ✅ Ready for Execution
**Next Review**: After Phase 1 completion (Week 2)
**Priority**: P3 - Enhancement (defer to Q2 2026 after MCP 2025-11-25 integration)
