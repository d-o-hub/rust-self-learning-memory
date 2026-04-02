# GOAP Goals Index

- **Last Updated**: 2026-04-02 (v0.1.27 current)
- **Source ADR**: ADR-037
- **Status**: Active

## v0.1.28 Sprint Goals (Planned)

1. **WG-089**: DyMoE-inspired pattern routing-drift protection
   - Priority: P1
   - Owner: feature-implement
   - Target: Add PatternAffinityClassifier + EpisodeAssignmentGuard to prevent ambiguous episodes from corrupting established patterns
   - Source: Issue #419
   - Status: 🔵 Planned

2. **WG-090**: Dual reward scoring (stability + novelty)
   - Priority: P2
   - Owner: feature-implement
   - Target: Split reward scoring into stability_score + novelty_score for smarter cluster management
   - Source: Issue #419
   - Status: 🔵 Planned

3. **WG-091**: Merge AI spam detector PR #406
   - Priority: P2
   - Owner: ci-engineer
   - Target: Merge ai-slop-detector workflows, close issue #401
   - Status: 🔵 Planned

4. **WG-092**: Resolve open Dependabot security alerts
   - Priority: P1
   - Owner: deps
   - Target: Address rustls-webpki (medium), lru (low), libsql-sqlite3-parser (low) alerts
   - Status: 🔵 Planned

5. **WG-093**: Fix CodeQL cleartext logging alert #60
   - Priority: P1
   - Owner: code-quality
   - Target: Resolve cleartext logging in memory-cli/src/commands/feedback/core.rs
   - Status: 🔵 Planned

---

## Completed Sprint Summary

| Sprint | WGs | Status | Key Deliverables |
|--------|-----|--------|------------------|
| v0.1.27 | WG-073,075,077-079,084-085 | ✅ All Complete | Bayesian ranking, Episode GC, MMR diversity, MCP Server Card, spawn_blocking audit, GH Pages, llms.txt |
| v0.1.26 | WG-086-088 | ✅ All Complete | Crate renaming do-memory-*, crates.io publish, GitHub Release |
| v0.1.24 | WG-059-067,080-083 | ✅ All Complete | Test stability, dependency updates, CHANGELOG backfill, tag+release |
| v0.1.23 | WG-051-058 | ✅ All Complete | Durable attribution/checkpoints, MCP contract, docs refresh, CI coverage, disk hygiene |
| v0.1.22 | WG-040-050 | ✅ All Complete | Doctests, file splits, dead_code, snapshots, property tests, MCP parity, git-cliff |
| v0.1.20 | WG-022-024,026-027,030 | ✅ All Complete | redb compilation, ignored test fixes, coverage improvement, codecov config |
| v0.1.19 | WG-012-021 | ✅ All Complete | Nightly filter, changelog workflow, dead_code audit, stale TODOs |
| v0.1.18 | WG-008-011 | ✅ All Complete | Ignored test triage, batch MCP tools, error handling, dep dedup |
| v0.1.17 | WG-001-007 | ✅ All Complete | Docs integrity, release wrapper, GOAP index, Dependabot merges |

---

## Partially Complete / Backlog

1. **WG-025**: Un-ignore fixable tests
   - Status: 🟡 Partial — 119→118 (pattern CLI e2e un-ignored); 6 sandbox/WASM tests still pending

2. **WG-028**: Property test expansion
   - Status: 🟡 Partial — ACT-030 (serialization) and ACT-031 (calculator) complete; ACT-032 (fuzz) pending

3. **WG-029**: Integration coverage
   - Status: 🟠 Pending — ACT-033, ACT-034, ACT-035 not started
