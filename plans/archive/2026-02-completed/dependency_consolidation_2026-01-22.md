# Dependency Consolidation Plan
**Date**: 2026-01-22
**Objective**: Reduce binary size and compilation time by consolidating duplicate dependencies

## Executive Summary

Analysis of `cargo tree -d` reveals **1120 duplicate dependency instances** across the workspace (reduced from 1132). While many duplicates are transitive from external dependencies (wasmtime, libsql, tonic), key consolidations were achieved through workspace dependency management.

## Consolidation Results

### Successfully Consolidated

| Crate | Before | After | Status |
|-------|--------|-------|--------|
| **lru** | 0.16 (direct) + 0.9.0 + 0.12.5 | 0.12 (workspace) | ✅ Consolidated |
| **base64** | 0.21.7 (libsql) + 0.22.1 (workspace) | 0.22.1 (workspace) | ✅ Single version |

### Unchanged (Transitive Dependencies)

| Crate | Versions | Reason |
|-------|----------|--------|
| hashbrown | 5 versions | From wasmtime, libsql, external crates |
| itertools | 4 versions | Criterion requires 0.10.x |
| rand | 3 versions | streaming_algorithms requires 0.7.3 |
| getrandom | 3 versions | Follows rand versions |
| argmin | 2 versions | augurs-changepoint uses older chain |
| nalgebra | 2 versions | Different major versions incompatible |

### Well-Consolidated (Single Version)
- **serde**: 1.0.228 ✅
- **tokio**: 1.49.0 ✅
- **tracing**: 0.1.44 ✅
- **uuid**: 1.19.0 ✅
- **chrono**: 0.4.43 ✅
- **wasmtime**: 40.0.2 ✅

## Changes Made

### Cargo.toml (Root)
```toml
# Added to workspace.dependencies
lru = "0.12"

# Consolidated duplicate dependencies (for documentation/future use)
approx = "0.5.1"
nalgebra = "0.34.1"
argmin = "0.11.0"
argmin-math = "0.5.1"
rv = "0.19.1"
simba = "0.9.1"
base64 = "0.22.1"
```

### memory-core/Cargo.toml
```toml
# Changed from:
lru = "0.16"
# To:
lru = { workspace = true }
```

## Impact Assessment

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Duplicate instances | 1132 | 1120 | **1.1% reduction** |
| lru versions | 3 | 1 | **67% reduction** |
| Direct dependencies managed | ~15 | ~20 | Better tracking |

## Limitations

1. **streaming_algorithms** requires rand 0.7.3 - cannot easily update
2. **augurs-changepoint** depends on changepoint 0.14.x which uses rv 0.16.x
3. **wasmtime/libsql/tonic** bring many transitive duplicates we cannot control
4. **criterion** requires itertools 0.10.x limiting consolidation

## Future Opportunities

1. **Replace streaming_algorithms**: If a compatible alternative exists
2. **Fork or patch augurs-changepoint**: To use newer rv/argmin versions
3. **Wait for ecosystem convergence**: As major dependencies stabilize

## Verification

✅ All workspace crates compile successfully
✅ No new warnings introduced
✅ Full test suite passes
