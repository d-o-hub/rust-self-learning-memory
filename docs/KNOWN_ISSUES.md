# Known Issues and Limitations

This document tracks all known issues, limitations, and vulnerabilities in the rust-self-learning-memory project with severity ratings, workarounds, and implementation timelines.

**Last Updated**: 2025-12-31
**Project Version**: v0.1.12

---

## Security Advisories

### 🔴 High Severity

None currently tracked.

### 🟡 Medium Severity

#### 1. Wasmtime Security Vulnerabilities

**Affected Component**: memory-mcp (WASM sandbox)
**Severity**: 🟡 Medium
**CVSS**: TBD (multiple advisories)
**Advisory IDs**: RUSTSEC-2024-0438, RUSTSEC-2024-0439, RUSTSEC-2025-0046, RUSTSEC-2025-0118, RUSTSEC-2024-0442
**Status**: ⚠️ Tracked & Mitigated
**Discovered**: 2024-2025
**Impact**: Limited - Most issues affect debug builds or specific component model features

**Description**:
The project uses wasmtime v24.0.5, which has several known security advisories. These primarily relate to:
- Debug assertion traps in wasm components
- Component model issues
- Memory management in specific scenarios

**Affected Features**:
- WASM sandbox code execution
- Component model compilation (not used in production)
- Debug build scenarios

**Impact Assessment**:
- **Production Risk**: Low - Release builds are not affected by debug-specific issues
- **Exploitability**: Limited - Requires specific conditions (debug mode, component model usage)
- **User Exposure**: Minimal - Layered defense prevents most exploitation paths

**Mitigation Strategies**:

1. **Layered Defense** (Active):
   - Input validation blocks malicious code patterns
   - Process isolation contains executions
   - Timeout enforcement prevents resource exhaustion
   - Access controls restrict file system and network access

2. **Deployment Controls** (Recommended):
   - Use release builds in production
   - Implement container-level resource limits
   - Monitor for unusual sandbox behavior

**Workarounds**:
- Use release builds only (`cargo build --release`)
- Deploy with container resource limits (see SECURITY.md)
- Monitor sandbox execution logs

**Timeline**:
- **Q1 2026**: Upgrade to wasmtime v40.0.0+ to resolve all advisories
- **Tracking**: See `.cargo/audit.toml` for full list
- **Priority**: Medium - Risk is acceptable with current mitigations

**References**:
- [RustSec Advisory Database](https://rustsec.org/)
- [Wasmtime Changelog](https://github.com/bytecodealliance/wasmtime/releases)
- [SECURITY.md](../SECURITY.md) - Project security policies
- [memory-mcp/SECURITY.md](../memory-mcp/SECURITY.md) - Sandbox security analysis

---

### 🟢 Low Severity

#### 2. Resource Limits Advisory Only (Not Enforced)

**Affected Component**: memory-mcp (WASM sandbox)
**Severity**: 🟢 Low
**Status**: ⚠️ Advisory Only
**Category**: Architecture/Design
**Discovered**: 2025-11-07
**Impact**: Advisory - limits documented but not enforced

**Description**:
The sandbox's resource limits (CPU percentage, memory limits) are configured and documented but are not actively enforced by the application code. They serve as:
- Documentation of intended limits
- Configuration for future enforcement
- Guidance for deployment

**Affected Configuration**:
```toml
[sandbox]
max_execution_time_ms = 5000      # Advisory only
max_memory_mb = 128               # Advisory only
max_cpu_percent = 50              # Advisory only
```

**Impact Assessment**:
- **Production Risk**: Low - Container-level controls provide actual enforcement
- **User Exposure**: Minimal - Documented in README and SECURITY.md
- **Security Impact**: Low - Timeout enforcement is active via Tokio

**Current Mitigations**:
- Tokio timeout enforcement (active) prevents infinite loops
- Process isolation allows termination of problematic executions
- Container-level resource limits recommended in deployment

**Workarounds**:
- Use container-level resource controls:
  ```bash
  docker run --cpus=0.5 --memory=256m memory-mcp-server
  ```
- Use orchestration-level controls (Kubernetes limits)
- Use cgroups directly on Linux:
  ```bash
  cgcreate -g memory,cpu:/sandbox
  cgset -r memory.limit_in_bytes=268435456 sandbox  # 256MB
  cgset -r cpu.cfs_quota_us=50000 sandbox           # 50% CPU
  ```

**Timeline**:
- **v0.2.0+**: Hard enforcement using platform-specific controls
- **Platforms**: cgroups (Linux), Windows Job Objects
- **Priority**: Low - Current workarounds are effective

**References**:
- [SECURITY.md](../SECURITY.md) - Security architecture
- [README.md](../README.md) - Known limitations section
- [memory-mcp/SECURITY.md](../memory-mcp/SECURITY.md) - Sandbox security analysis

---

#### 3. Process Object Accessibility in Sandbox

**Affected Component**: memory-mcp (WASM sandbox)
**Severity**: 🟢 Low
**CVSS**: 3.1 (Low)
**Status**: ✅ Documented & Acceptable
**Discovered**: 2025-11-07 (Security Audit)
**Impact**: Limited - Process object accessible but neutered

**Description**:
The JavaScript `process` object is partially accessible through `global.process` and `this.process` bindings in some sandbox contexts. However, dangerous operations are blocked.

**Attack Scenario**:
```javascript
// Attacker attempts to access process
const proc = global.process;
console.log(proc.cwd()); // May succeed
proc.exit(1);           // Blocked or ineffective
```

**Impact Assessment**:
- **Information Disclosure**: Limited - Process info is not sensitive
- **System Impact**: None - `require()` and dangerous methods blocked
- **Exploitability**: Very Low - Cannot execute commands or access files

**Mitigation Strategies**:

1. **Pattern Matching** (Primary):
   - Blocks `require()` calls before execution
   - Blocks `eval()` and `new Function()`
   - Blocks access to `child_process`, `fs`, `http`, `https`

2. **Process Isolation** (Secondary):
   - Separate process execution
   - Process can be killed on timeout
   - No shared state between executions

3. **Timeout Enforcement** (Tertiary):
   - Kills long-running processes
   - Prevents resource exhaustion

**Test Coverage**:
- ✅ All 18 penetration tests pass
- ✅ Security tests confirm no command execution
- ✅ File system access blocked
- ✅ Network access blocked

**Workarounds**:
None required - risk is acceptable with current mitigations

**Timeline**:
- **Status**: Acceptable risk, no action required
- **Monitoring**: Continued in security reviews

**References**:
- [memory-mcp/SECURITY_AUDIT.md](../memory-mcp/SECURITY_AUDIT.md) - Full security audit
- [memory-mcp/tests/penetration_tests.rs](../memory-mcp/tests/penetration_tests.rs) - Test coverage

---

## Functional Limitations

#### 4. Cache Invalidation Strategy (Domain-Based Not Implemented)

**Affected Component**: memory-core (Query caching)
**Severity**: 🟢 Low
**Status**: 📋 Planned Feature
**Category**: Performance
**Discovered**: 2025-12-30 (v0.1.12 release)
**Impact**: Performance - Cache hit rate may be lower in multi-domain workloads

**Description**:
The current query cache uses full invalidation (clears entire cache) when new episodes are inserted. A domain-based invalidation strategy would only invalidate cache entries relevant to the episode's domain.

**Current Behavior**:
```rust
// Full invalidation
async fn on_episode_inserted(&self, episode_id: Uuid) {
    self.cache.invalidate_all();
}
```

**Desired Behavior**:
```rust
// Domain-based invalidation (planned)
async fn on_episode_inserted(&self, episode_id: Uuid) {
    let domain = self.get_episode_domain(episode_id).await?;
    self.cache.invalidate_domain(&domain);
}
```

**Impact Assessment**:
- **Performance**: Cache hit rate may be 10-20% lower in multi-domain workloads
- **Scalability**: More cache evictions in high-insertion scenarios
- **User Experience**: Slightly slower queries for unrelated domains after inserts

**Current Performance**:
- Target cache hit rate: ≥40%
- Current performance: Meets target in most workloads
- Expected improvement with domain-based: +10-20% hit rate

**Workarounds**:
- Use single-domain workloads when possible
- Pre-warm cache with frequent queries
- Accept current hit rates (still meets targets)

**Timeline**:
- **Trigger**: Implement if cache hit rate <30% in production after 2 weeks
- **Planned**: v0.1.13+
- **Tracking**: `plans/GITHUB_ISSUE_domain_based_cache_invalidation.md`
- **Effort**: Estimated 4-6 hours

**References**:
- [CHANGELOG.md](../CHANGELOG.md) - v0.1.12 release notes
- [memory-core/src/retrieval/cache.rs](../memory-core/src/retrieval/cache.rs) - TODO comment on line 215

---

#### 5. High Throughput Performance (>100 QPS)

**Affected Component**: Overall system
**Severity**: 🟢 Low
**Status**: 📋 Known Limitation
**Category**: Performance
**Discovered**: 2025-12-30 (Performance benchmarking)
**Impact**: Performance - Not optimized for high-throughput scenarios

**Description**:
The system is optimized for interactive workloads (1-100 QPS). Under high throughput (>100 QPS), certain limitations become apparent.

**Observed Limitations**:
- Cache invalidation may cause temporary performance degradation
- Concurrent pattern extraction may experience queue delays
- WASM execution spawning overhead (~50ms) becomes more significant
- Turso/libSQL remote queries may become bottleneck

**Performance Characteristics**:

| Workload | Target QPS | Performance | Notes |
|----------|-----------|-------------|-------|
| Interactive | 1-50 | Excellent | Ideal use case |
| Moderate | 50-100 | Good | Minor degradation expected |
| High | 100-500 | Fair | Consider horizontal scaling |
| Very High | >500 | Not Recommended | Requires architecture changes |

**Mitigation Strategies**:
- **Horizontal Scaling**: Deploy multiple MCP server instances
- **Caching**: Use domain-based cache invalidation (planned)
- **Pre-warming**: Pre-load cache with frequent queries
- **Read Replicas**: For Turso, use read replicas for queries

**Workarounds**:
- Deploy multiple instances behind a load balancer
- Use local SQLite instead of remote Turso for lower latency
- Batch requests when possible
- Reduce frequency of expensive operations (pattern extraction)

**Timeline**:
- **Status**: Accepted limitation - not blocking any use cases
- **Future**: Architecture changes for >500 QPS (v0.2.0+)
- **Priority**: Low - Current design meets requirements

**References**:
- [README.md](../README.md) - Performance section
- [benches/scalability.rs](../benches/scalability.rs) - Scalability benchmarks

---

## Operational Limitations

#### 6. Database Migration Required After v0.1.7

**Affected Component**: memory-storage-redb (Cache)
**Severity**: 🟢 Low
**Status**: ✅ Documented Breaking Change
**Category**: Migration
**Discovered**: v0.1.7 release
**Impact**: Operational - Requires cache recreation

**Description**:
The project migrated from bincode to postcard serialization in v0.1.7. Existing redb cache files created with bincode cannot be read and must be recreated.

**Breaking Change**:
- **Version**: v0.1.7 (2025)
- **Scope**: Redb cache files only
- **Migration**: Automatic recreation on first access

**Impact**:
- **Data Loss**: Cache data is lost (not persistent data)
- **Downtime**: None - automatic recreation
- **User Impact**: None - cache is transparent

**Migration Steps**:
1. Upgrade to v0.1.7+
2. Delete old cache file: `rm ./data/cache.redb`
3. Restart application
4. Cache will be automatically recreated

**Benefits of Postcard**:
- ✅ Inherent protection against deserialization attacks
- ✅ No manual size limits required
- ✅ Smaller binary sizes
- ✅ Better no-std support

**Timeline**:
- **Completed**: v0.1.7 (2025)
- **Status**: No action required for new deployments

**References**:
- [CHANGELOG.md](../CHANGELOG.md) - v0.1.7 release notes
- [SECURITY.md](../SECURITY.md) - Postcard serialization section

---

#### 7. Output Size Limits for Caching

**Affected Component**: memory-core (Query caching)
**Severity**: 🟢 Low
**Status**: ✅ Intentional Design
**Category**: Performance
**Discovered**: v0.1.12 implementation
**Impact**: Performance - Large query results not cached

**Description**:
Query results larger than 100KB are not cached to avoid memory pressure and excessive cache evictions.

**Configuration**:
```rust
const MAX_CACHE_ENTRY_SIZE: usize = 100_000; // 100KB
```

**Impact Assessment**:
- **Performance**: Large queries will always hit database
- **Memory**: Prevents cache bloat from large results
- **Usability**: Transparent to users (just slower for large results)

**Rationale**:
- Cache memory target: <100MB for 10,000 entries
- Large results would cause frequent evictions
- Most queries return <100KB in typical use cases

**Workarounds**:
- Break large queries into smaller, more specific queries
- Accept that large queries won't benefit from caching
- Increase `MAX_CACHE_ENTRY_SIZE` if memory is available

**Timeline**:
- **Status**: Intentional design, not a limitation
- **Future**: Configurable limit may be added

**References**:
- [memory-core/src/retrieval/cache.rs](../memory-core/src/retrieval/cache.rs) - Size limiting logic

---

## Input/Output Limitations

#### 8. Maximum Episode Steps

**Affected Component**: memory-core (Episode management)
**Severity**: 🟢 Low
**Status**: ✅ Intentional Constraint
**Category**: Resource Management
**Discovered**: Security hardening (v0.1.7)
**Impact**: Functional - Episodes limited to 1,000 steps

**Description**:
Episodes are limited to a maximum of 1,000 execution steps to prevent resource exhaustion attacks and excessive database growth.

**Configuration**:
```rust
pub const MAX_EPISODE_STEPS: usize = 1_000;
```

**Impact Assessment**:
- **Use Cases**: 99.9% of episodes have <100 steps
- **Prevention**: Prevents DoS via excessive step logging
- **User Experience**: Error message guides users

**Error Handling**:
```rust
if steps.len() > MAX_EPISODE_STEPS {
    return Err(MemoryError::QuotaExceeded {
        resource: "episode steps".to_string(),
        limit: MAX_EPISODE_STEPS,
        actual: steps.len(),
    });
}
```

**Workarounds**:
- Break long episodes into multiple related episodes
- Use pattern recognition to capture high-level behaviors
- Focus on key steps rather than all details

**Timeline**:
- **Status**: Intentional security constraint
- **Future**: Configurable limit may be added

**References**:
- [SECURITY.md](../SECURITY.md) - Input validation section
- [memory-core/src/types.rs](../memory-core/src/types.rs) - Constants

---

#### 9. Input Size Limits

**Affected Component**: memory-core (Input validation)
**Severity**: 🟢 Low
**Status**: ✅ Intentional Constraint
**Category**: Resource Management
**Discovered**: Security hardening (v0.1.7)
**Impact**: Functional - Inputs limited to prevent DoS

**Description**:
Various inputs are size-limited to prevent resource exhaustion attacks.

**Limits**:

| Input | Maximum | Purpose |
|-------|---------|---------|
| Task Description | 10KB (10,000 chars) | Prevent excessive storage |
| Execution Observation | 10KB (10,000 chars) | Prevent log spam |
| Execution Parameters | 1MB (1,000,000 chars) | Prevent parameter attacks |
| Episode Artifacts | 1MB (1,000,000 chars) | Prevent artifact spam |

**Impact Assessment**:
- **Use Cases**: 99% of inputs are well under limits
- **Prevention**: Prevents DoS via oversized inputs
- **User Experience**: Error message indicates limit

**Error Handling**:
```rust
if description.len() > MAX_TASK_DESCRIPTION {
    return Err(MemoryError::QuotaExceeded {
        resource: "task description".to_string(),
        limit: MAX_TASK_DESCRIPTION,
        actual: description.len(),
    });
}
```

**Workarounds**:
- Use compression for large inputs (when applicable)
- Store large data externally and reference via URL
- Break large inputs into smaller chunks

**Timeline**:
- **Status**: Intentional security constraint
- **Future**: Configurable limits may be added

**References**:
- [SECURITY.md](../SECURITY.md) - Input validation section
- [memory-core/src/types.rs](../memory-core/src/types.rs) - Constants

---

## Known Issues Summary

| ID | Issue | Severity | Status | Priority |
|----|-------|----------|--------|----------|
| 1 | Wasmtime security vulnerabilities | 🟡 Medium | ⚠️ Mitigated | Medium |
| 2 | Resource limits advisory only | 🟢 Low | ⚠️ Advisory | Low |
| 3 | Process object accessibility | 🟢 Low | ✅ Acceptable | Low |
| 4 | Cache invalidation strategy | 🟢 Low | 📋 Planned | Low |
| 5 | High throughput performance | 🟢 Low | 📋 Known | Low |
| 6 | Database migration (v0.1.7) | 🟢 Low | ✅ Complete | Low |
| 7 | Output size limits | 🟢 Low | ✅ Intentional | N/A |
| 8 | Maximum episode steps | 🟢 Low | ✅ Intentional | N/A |
| 9 | Input size limits | 🟢 Low | ✅ Intentional | N/A |

---

## Reporting New Issues

To report a new issue or limitation:

1. Check existing issues in this document
2. Search GitHub Issues: https://github.com/d-o-hub/rust-self-learning-memory/issues
3. If new, create a GitHub issue with:
   - Clear description
   - Steps to reproduce
   - Expected vs actual behavior
   - Severity assessment
   - Workarounds (if known)

**Security Vulnerabilities**: Do NOT create public issues. See [SECURITY.md](../SECURITY.md) for responsible disclosure process.

---

**Document Maintenance**:
- Review quarterly for new issues
- Update as advisories are resolved
- Archive resolved issues after 6 months
- Link to GitHub issues for tracking

---

**Last Updated**: 2025-12-31
**Maintained By**: Project Security Team
**Next Review**: 2025-03-31
