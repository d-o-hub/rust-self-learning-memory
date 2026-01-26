# Dependency Update Plan

## Overview
Update all Rust workspace packages to the latest compatible versions while maintaining:
- cargo clippy: 0 warnings
- cargo build: success
- cargo test: all passing
- Quality gates: >90% coverage maintained

## Current State Analysis

### Workspace Root (Cargo.toml)
| Dependency | Current | Status |
|------------|---------|--------|
| tokio | 1.49 | Needs update |
| async-trait | 0.1 | Check update |
| anyhow | 1.0 | Check update |
| thiserror | 2.0 | Check update |
| serde | 1.0 | Check update |
| serde_json | 1.0 | Check update |
| postcard | 1.1.3 | Check update |
| libsql | 0.9 | Needs update |
| redb | 2.6 | Check update |
| uuid | 1.19 | Updated to 1.20.0 |
| chrono | 0.4 | Check update |
| tracing | 0.1 | Check update |
| tracing-subscriber | 0.3 | Check update |
| rquickjs | 0.11.0 | Check update |
| wasmtime | 40.0.2 | Check update |
| clap | 4.5 | Check update |

### Individual Packages
| Package | Key Dependencies |
|---------|------------------|
| memory-core | ort 2.0.0-rc.11, tokenizers 0.22, augurs-changepoint 0.10.1 |
| memory-storage-turso | lz4_flex 0.12, zstd 0.13, flate2 1.1 |
| memory-mcp | javy 6.0.0, augurs 0.10.1, deep_causality 0.13.2 |
| memory-cli | toml 0.9, serde_yaml 0.9, dirs 6.0 |

## Task Phases

### Phase 1: Discovery (Completed)
- [x] Analyze workspace Cargo.toml files
- [x] Check current dependency versions
- [x] Identify update candidates
- [x] Run cargo update to see available compatible updates

### Phase 2: Research
- [ ] Research latest stable versions of key dependencies
- [ ] Check for breaking changes in major version bumps
- [ ] Identify compatibility requirements (e.g., ort -> ndarray version)

### Phase 3: Update (Parallel Tasks)
- [ ] **Agent 1: Core Dependencies** - tokio, async-trait, thiserror, anyhow, serde, serde_json, postcard
- [ ] **Agent 2: Database Dependencies** - libsql, redb, compression libs
- [ ] **Agent 3: Web/Server Dependencies** - tracing, tracing-subscriber, reqwest
- [ ] **Agent 4: Advanced Dependencies** - wasmtime, rquickjs, augurs, ort, tokenizers

### Phase 4: Verify
- [ ] Run cargo clippy --all -- -D warnings
- [ ] Run cargo build --all
- [ ] Run cargo test --all
- [ ] Run quality gates script
- [ ] Verify coverage maintained >90%

## Known Constraints & Compatibility Issues
1. ort 2.0.0-rc.x requires specific ndarray version matching
2. javy versions must align with wasmtime
3. argmin/argmin-math versions come from transitive dependencies
4. Some hashbrown/itertools duplicates are from external dependencies

## Update Strategy
1. Start with workspace root dependencies
2. Update individual package dependencies
3. Test each major category in parallel where possible
4. Roll back if any major breaking change occurs

## Success Criteria
- [ ] All dependencies updated to latest compatible versions
- [ ] cargo clippy: 0 warnings
- [ ] cargo build: success
- [ ] cargo test: all passing (>95% pass rate)
- [ ] Quality gates: >90% coverage maintained
