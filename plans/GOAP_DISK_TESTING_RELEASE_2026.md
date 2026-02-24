# GOAP Execution Plan: Disk Space, Testing & Release Modernization (2026)

- **Date**: 2026-02-21
- **Goal**: Reduce disk footprint, adopt 2026 testing best practices, modernize release engineering
- **Strategy**: Sequential phases with parallel sub-tasks where independent

## Current State Analysis

| Metric | Current (2026-02-24 Rebaseline) | Target | Status | Verification |
|--------|----------------------------------|--------|--------|--------------|
| target/ size | 75G (`du -sh target`) | < 2 GB | 🔴 Pending | `du -sh target` |
| Duplicate deps | 121 roots | < 80 | 🟡 Monitoring only | `cargo tree -d | rg -c "^[a-z]"` |
| Rust edition | 2024 (workspace) | 2024 | ✅ Done | `rg 'edition\s*=\s*"2024"' Cargo.toml` |
| Test runner | nextest in CI/nightly; doctests via `cargo test --doc` | nextest everywhere (except doctests) | ✅ Done | `.github/workflows/*.yml` |
| Mutation testing | Nightly `cargo mutants` for `memory-core` | cargo-mutants on core | ✅ Done | `.github/workflows/nightly-tests.yml` |
| Property testing | Partial (`proptest` exists, not expanded across all crates) | proptest on invariants across crates | 🟡 Partial | `rg 'proptest' **/*.rs` |
| Snapshot testing | 13 snapshot files (`memory-cli`, `memory-mcp`) | >=3 core snapshot suites | ✅ Baseline met | `glob '**/snapshots/*.snap'` |
| Release automation | `release.toml` present; cargo-release flow documented | cargo-release + cargo-dist | 🟡 Partial | `ls release.toml` + workflow checks |
| Semver checking | Enabled in CI (`cargo semver-checks check-release`) | cargo-semver-checks in CI | ✅ Done | `.github/workflows/ci.yml` |
| node_modules/ | Absent | 0 MB | ✅ Done | `test -d node_modules` |

### Phase Status Rebaseline (ADR-Linked)

- **ADR-032**: Partial. Mold + profile optimization landed; `target/` footprint remains major open item.
- **ADR-033**: Partial/strong baseline. nextest and mutants are active; property-test expansion remains open.
- **ADR-034**: Partial. semver checks + release config are present; release evidence loop remains process-dependent.
- **ADR-035**: Complete. Workspace edition is `2024`.
- **ADR-036**: Monitoring phase only. Baseline duplicate count tracked; cleanup deferred until current blockers close.

## GOAP Task Decomposition

### Phase 1: Quick Wins (Week 1) — Parallel

```
┌─────────────────────────────────┐
│  1A. Build Profile Optimization │  ADR-032 Phase 1
│  - debug = "line-tables-only"   │  Expected: -3 GB
│  - deps debug = false           │
│  - proc-macro opt-level = 3     │
└─────────────────────────────────┘
┌─────────────────────────────────┐
│  1B. Edition Migration          │  ADR-035
│  - cargo fix --edition          │  Expected: 1-2 hours
│  - edition = "2024"             │
└─────────────────────────────────┘
┌─────────────────────────────────┐
│  1C. Cleanup Orphans            │  ADR-032 Phase 6
│  - Remove node_modules/         │  Expected: -89 MB
│  - Add clean-artifacts script   │
└─────────────────────────────────┘
```

### Phase 2: Testing Foundation (Week 2-3) — Sequential

```
2A. Standardize nextest ──→ 2B. Add nextest.toml profiles ──→ 2C. Update all CI workflows
     ADR-033 Phase 1              ADR-033 Phase 1                   ADR-033 Phase 1
```

### Phase 3: Dependency & Linker (Week 3-4) — Parallel

```
┌─────────────────────────────────┐
│  3A. cargo-machete audit        │  ADR-036 Tier 1
│  3B. cargo-shear audit          │
│  3C. cargo-unused-features      │  ADR-036 Tier 2
└─────────────────────────────────┘
┌─────────────────────────────────┐
│  3D. mold linker setup          │  ADR-032 Phase 2 ✅ DONE
│  - ✅ Install mold              │     Installed and active
│  - ✅ Configure .cargo/config.toml │  2-5x faster link times
└─────────────────────────────────┘
```

### Phase 4: Release Engineering (Week 4-6) — Sequential

```
4A. cargo-semver-checks in CI ──→ 4B. release.toml + cargo-release ──→ 4C. Evaluate cargo-dist
     ADR-034 Phase 1                  ADR-034 Phase 2                      ADR-034 Phase 3
```

### Phase 5: Advanced Testing (Week 6-8) — Parallel

```
┌─────────────────────────────────┐
│  5A. cargo-mutants pilot        │  ADR-033 Phase 4
│  - memory-core first            │
│  - Nightly CI job               │
└─────────────────────────────────┘
┌─────────────────────────────────┐
│  5B. proptest integration       │  ADR-033 Phase 5
│  - Serialization roundtrips     │
│  - State machine transitions    │
└─────────────────────────────────┘
┌─────────────────────────────────┐
│  5C. insta snapshot tests       │  ADR-033 Phase 6
│  - MCP tool responses           │
│  - CLI output                   │
└─────────────────────────────────┘
```

### Phase 6: Upstream Tracking (Ongoing)

```
6A. Monitor dependency updates    │  ADR-036 Tier 3
6B. Track cargo build-dir rework  │  Rust Project Goal 2025H2
6C. Cranelift backend (optional)  │  When production-ready
```

## Dependency Graph

```
Phase 1 (Quick Wins) ─────┐
                           ├──→ Phase 2 (Testing Foundation)
                           │         │
                           │         ├──→ Phase 5 (Advanced Testing)
                           │         │
Phase 3 (Deps/Linker) ────┤
                           │
                           └──→ Phase 4 (Release Engineering)
                                      │
                                      └──→ Phase 6 (Ongoing)
```

## ADR Reference Map

| ADR | Title | Phases |
|-----|-------|--------|
| [ADR-032](adr/ADR-032-Disk-Space-Optimization.md) | Disk Space Optimization | 1A, 1C, 3D |
| [ADR-033](adr/ADR-033-Modern-Testing-Strategy.md) | Modern Testing Strategy | 2A-2C, 5A-5C |
| [ADR-034](adr/ADR-034-Release-Engineering-Modernization.md) | Release Engineering | 4A-4C |
| [ADR-035](adr/ADR-035-Rust-2024-Edition-Migration.md) | Rust 2024 Edition | 1B |
| [ADR-036](adr/ADR-036-Dependency-Deduplication.md) | Dependency Deduplication | 3A-3C, 6A |

## Success Criteria

- [ ] `target/` < 2 GB after clean dev build
- [ ] All CI uses `cargo nextest run` (except doctests)
- [ ] nextest profiles configured for default/ci/nightly
- [ ] cargo-semver-checks in CI pipeline
- [ ] cargo-release workflow documented and tested
- [ ] Mutation testing running on memory-core (nightly)
- [ ] ≥ 5 proptest property tests in memory-core
- [ ] ≥ 3 insta snapshot tests in memory-mcp or memory-cli
- [ ] Edition 2024 across all workspace crates
- [ ] Duplicate dep roots < 80
- [ ] node_modules/ removed
- [ ] Quality gates script tracks dep count
