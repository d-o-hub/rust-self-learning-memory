# Gap Analysis — v0.1.21 Sprint

**Generated**: 2026-03-15
**Method**: Analysis Swarm (RYAN + FLASH + SOCRATES)
**Scope**: ADR-044, ADR-041, ADR-045

---

## Executive Summary

| Area | Status | Gap Count | Priority |
|------|--------|-----------|----------|
| **ADR-045 Publishing** | ✅ Implemented | 0 | Complete |
| **ADR-044 Features** | ⏳ Proposed | 3 features | P0-P1 |
| **ADR-041 Test Health** | ⏳ 85% Complete | 4 tasks | P2-P3 |
| **Documentation Sync** | ⚠️ Stale | 2 files | P1 |

---

## Multi-Perspective Analysis

### RYAN's View (Methodical Analyst)

**Security & Scalability Concerns**:
1. ADR-044 Feature 2 (Attribution) requires new storage schema — migration risk
2. Playbook quality depends on existing pattern/summary quality — need validation tests
3. Attribution requires explicit agent feedback — potential UX friction

**Evidence-Based Findings**:
- ADR-041 Phase 1-3: ✅ Verified complete (build passes, clippy clean)
- ADR-041 Phase 4-5: 4 tasks remain (T4.2, T4.3, T5.2, T5.3)
- ADR-044: 0% implemented (status: Proposed)
- ADR-045: 100% implemented (verified infrastructure exists)

**Recommendation**: Complete ADR-041 P2 tasks before starting ADR-044 to ensure stable test infrastructure.

### FLASH's View (Rapid Innovator)

**Quick Wins Available**:
1. Fix sandbox timing tests (T4.2) — simple timeout wrappers
2. Fix WASM binary data tests (T4.3) — use `from_utf8_lossy`
3. Add nightly trend tracking (T5.2) — artifact upload

**Impact Analysis**:
- ADR-044 Feature 1 (Playbooks): **Highest impact** — closes usability gap
- ADR-044 Feature 2 (Attribution): **High impact** — closes feedback gap
- ADR-044 Feature 3 (Handoff): **Medium impact** — niche use case

**Recommendation**: Ship ADR-044 Features 1+2 first (P0), defer Feature 3 to v0.1.22.

### SOCRATES' View (Questioning Facilitator)

**Critical Questions**:

1. **Sprint Scope Drift**: ROADMAP_ACTIVE.md says v0.1.21 is "publishing infrastructure" but we're discussing ADR-044 features. Which is it?
   - *Answer*: ADR-045 publishing is complete. ADR-044 features are proposed for the sprint.

2. **Documentation Consistency**: STATUS/CURRENT.md says "Released Version: v0.1.19" but ROADMAP_ACTIVE.md says "Released Version: v0.1.20". Which is correct?
   - *Answer*: v0.1.20 is released. STATUS/CURRENT.md is stale.

3. **ADR Status Mismatch**: ADR-044 and ADR-041 show "Proposed" and "Partially Implemented" but some tasks are complete. Should status be updated?
   - *Answer*: Yes, update ADR status fields to reflect actual state.

**Recommendation**: Sync documentation before committing to new feature work.

---

## Detailed Gap Inventory

### ADR-044: High-Impact Features (Proposed)

| Feature | Priority | Status | Effort | Dependencies |
|---------|----------|--------|--------|--------------|
| 1. Actionable Playbooks | P0 | ❌ Not Started | 3-5 days | Feature 2 (attribution) |
| 2. Recommendation Attribution | P0 | ❌ Not Started | 3-4 days | None |
| 3. Episode Checkpoints/Handoff | P1 | ❌ Not Started | 4-6 days | None |

**Implementation Files Required**:
```
memory-core/src/memory/playbook/mod.rs       (NEW)
memory-core/src/memory/playbook/generator.rs (NEW)
memory-core/src/memory/attribution/mod.rs    (NEW)
memory-core/src/memory/attribution/tracker.rs (NEW)
memory-core/src/memory/checkpoint/mod.rs     (NEW)
```

**MCP Tools to Add**:
- `recommend_playbook`
- `explain_pattern`
- `record_recommendation_feedback`
- `checkpoint_episode`
- `get_handoff_pack`
- `resume_from_handoff`

**CLI Commands to Add**:
- `playbook recommend`
- `playbook explain`
- `feedback record`
- `episode checkpoint`
- `episode handoff`

### ADR-041: Test Health Remediation (85% Complete)

| Phase | Task | Status | Effort | Blocker |
|-------|------|--------|--------|---------|
| 1 | Fix Build | ✅ Complete | - | - |
| 2 | Fix Stale Ignores | ✅ Complete | - | - |
| 3 | Nightly Refactor | ✅ Complete | - | - |
| 4 | T4.1 Pattern CLI e2e | ✅ Complete | - | - |
| 4 | T4.2 Sandbox timing tests | ⏳ Pending | 2h | None |
| 4 | T4.3 WASM binary data tests | ⏳ Pending | 2h | None |
| 5 | T5.1 Ceiling script | ✅ Complete | - | - |
| 5 | T5.2 Nightly trend tracking | ⏳ Pending | 1h | None |
| 5 | T5.3 libsql version monitor | ⏳ Pending | 1h | None |

**Ignored Test Breakdown**:
| Category | Count | Fixability |
|----------|-------|------------|
| Turso libsql bug (upstream) | 70 | ❌ Blocked |
| Slow integration tests | 29 | ⚠️ By design |
| WASM/sandbox | 9 | 🟡 Partial |
| Flaky CI | 5 | 🟡 Fixable |
| E2E/process | 3 | 🟢 Fixable |
| Requires backends | 2 | ⚠️ By design |
| Local embeddings | 1 | ❌ Blocked |
| **Total** | **118** | - |

### ADR-045: Publishing Best Practices (Implemented)

| Phase | Task | Status |
|-------|------|--------|
| 1 | Cargo.toml metadata | ✅ Complete |
| 2 | verify-crate-metadata.sh | ✅ Complete |
| 3 | supply-chain.yml workflow | ✅ Complete |
| 4 | deny.toml configuration | ✅ Complete |
| 5 | publish-crates.yml workflow | ✅ Complete |
| 6 | OIDC trusted publishing | ✅ Configured |

**Verification Commands**:
```bash
./scripts/verify-crate-metadata.sh  # Passes
cargo deny check                     # Passes
cargo cyclonedx --all               # Generates SBOM
```

---

## Prioritized Action Items

### P0: Documentation Sync (Immediate)

1. Update `plans/STATUS/CURRENT.md`:
   - Change "Released Version: v0.1.19" → "Released Version: v0.1.20"
   - Change "Next Version: v0.1.20" → "Next Version: v0.1.21"

2. Update `plans/adr/ADR-041-Test-Health-Remediation-v0.1.20.md`:
   - Change status to "Accepted (Mostly Implemented)"

3. Update `plans/adr/ADR-044-High-Impact-Features-v0.1.20.md`:
   - Keep status as "Proposed" (features not implemented)

### P1: Complete ADR-041 Remaining Tasks (4-6 hours)

| Task | File | Action |
|------|------|--------|
| T4.2 | `memory-mcp/src/sandbox/tests.rs` | Add `tokio::time::timeout` wrappers |
| T4.3 | `memory-mcp/src/unified_sandbox/tests.rs` | Use `from_utf8_lossy` or base64 |
| T5.2 | `.github/workflows/nightly-tests.yml` | Add artifact upload for test results |
| T5.3 | `scripts/check-libsql-version.sh` | Create version monitor script |

### P2: ADR-044 Feature Implementation (1-2 weeks)

**Week 1: Attribution + Playbooks**
```
Day 1-2: Feature 2 (Attribution)
  - Create memory-core/src/memory/attribution/mod.rs
  - Create RecommendationTracker struct
  - Add record_recommendation_feedback MCP tool

Day 3-5: Feature 1 (Playbooks)
  - Create memory-core/src/memory/playbook/mod.rs
  - Create PlaybookGenerator
  - Add recommend_playbook MCP tool
```

**Week 2: Handoff (Optional)**
```
Day 1-3: Feature 3 (Handoff)
  - Create memory-core/src/memory/checkpoint/mod.rs
  - Add checkpoint_episode, get_handoff_pack MCP tools
```

---

## Pre-Existing Issues

### Issue 1: Dependabot Vulnerabilities

**Status**: 5 vulnerabilities reported (3 high, 2 low)
**Location**: https://github.com/d-o-hub/rust-self-learning-memory/security/dependabot

**Recommended Action**: Review and address high-priority vulnerabilities before release.

### Issue 2: `execute_agent_code` MCP Tool Disabled

**Location**: `memory-mcp/src/handlers.rs:72-91`
**Reason**: "WASM sandbox compilation issues"
**Status**: Registered conditionally but returns error

**Recommended Action**: Either fix the WASM sandbox issues or remove the tool registration entirely.

### Issue 3: Broken Markdown Links (89 count)

**Status**: Mostly in archived files
**Target**: 0

**Recommended Action**: Run `./scripts/check-docs-integrity.sh` and fix critical links.

### Issue 4: `#[allow(dead_code)]` Annotations (110 count)

**Status**: 37 files have annotations
**Target**: ≤50

**Recommended Action**: Systematic dead code removal or add proper `#[cfg]` conditions.

---

## Quality Gates for v0.1.21 Release

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --tests -- -D warnings` passes
- [ ] `cargo build --all` succeeds
- [ ] `cargo nextest run --all` passes (excluding ignored)
- [ ] `cargo test --doc --all` passes
- [ ] `./scripts/quality-gates.sh` passes
- [ ] `./scripts/verify-crate-metadata.sh` passes
- [ ] `cargo deny check` passes
- [ ] Documentation updated (CURRENT.md, ROADMAP_ACTIVE.md)
- [ ] ADR status fields updated

---

## Cross-References

- **ADR-041**: [Test Health Remediation](../adr/ADR-041-Test-Health-Remediation-v0.1.20.md)
- **ADR-044**: [High-Impact Features](../adr/ADR-044-High-Impact-Features-v0.1.20.md)
- **ADR-045**: [Publishing Best Practices](../adr/ADR-045-Publishing-Best-Practices-2026.md)
- **Current Status**: [CURRENT.md](CURRENT.md)
- **Active Roadmap**: [ROADMAP_ACTIVE.md](../ROADMAPS/ROADMAP_ACTIVE.md)