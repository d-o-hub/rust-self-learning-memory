# 500 LOC Compliance Refactoring Plan

## Overview
Split 7 files exceeding 500 LOC into modular, single-responsibility components.

## Target Files

| File | Current LOC | Split Strategy |
|------|-------------|----------------|
| `storage/batch.rs` | 1,419 | 5 modules (config, episode, pattern, combined, query) |
| `lib.rs` | 1,091 | Move methods to trait_impls.rs |
| `metrics/collector.rs` | 710 | 3 modules (types, core, collector) |
| `storage/episodes.rs` | 670 | 4 modules (compression, crud, query, row) |
| `compression/mod.rs` | 574 | 2 modules (algorithms, stats) |
| `pool/keepalive.rs` | 592 | 2 modules (pool, monitoring) |
| `cache/adaptive_ttl.rs` | 887 | 4 modules (config, state, stats, main) |

## Module Structure After Refactoring

```
memory-storage-turso/src/
├── lib.rs                    # 200 LOC (exports + struct def)
├── turso_config.rs           # 150 LOC (TursoConfig)
├── storage/
│   ├── mod.rs                # 100 LOC (re-exports)
│   ├── episodes.rs           # 200 LOC (main impls)
│   ├── episodes_crud.rs      # 150 LOC (CRUD operations)
│   ├── episodes_query.rs     # 150 LOC (query builders)
│   ├── batch.rs              # 200 LOC (main batch)
│   ├── batch_episodes.rs     # 200 LOC (episode batch)
│   ├── batch_patterns.rs     # 200 LOC (pattern batch)
│   ├── batch_combined.rs     # 150 LOC (combined batch)
│   └── patterns.rs           # existing
├── metrics/
│   ├── mod.rs                # 100 LOC (re-exports)
│   ├── types.rs              # 200 LOC (OperationType, LatencyStats, etc)
│   ├── core.rs               # 150 LOC (TursoMetrics)
│   └── collector.rs          # 250 LOC (MetricsCollector)
├── compression/
│   ├── mod.rs                # 150 LOC (main + stats)
│   ├── algorithms.rs         # 250 LOC (compression algorithms)
│   └── stats.rs              # 100 LOC (CompressionStatistics)
├── pool/
│   ├── mod.rs                # 100 LOC (re-exports + config)
│   ├── keepalive.rs          # 200 LOC (KeepAlivePool)
│   └── monitoring.rs         # 150 LOC (monitoring helpers)
├── cache/
│   ├── mod.rs                # 100 LOC (re-exports)
│   ├── adaptive_ttl.rs       # 300 LOC (main cache impl)
│   ├── adaptive_config.rs    # 150 LOC (AdaptiveTtlConfig)
│   ├── adaptive_state.rs     # 150 LOC (state management)
│   └── adaptive_stats.rs     # 150 LOC (AdaptiveTtlStats)
└── trait_impls.rs            # existing
```

## Execution Order

1. Split `batch.rs` (largest file, 1419 LOC)
2. Split `lib.rs` (1091 LOC)
3. Split `collector.rs` (710 LOC)
4. Split `episodes.rs` (670 LOC)
5. Split `compression/mod.rs` (574 LOC)
6. Split `keepalive.rs` (592 LOC)
7. Split `adaptive_ttl.rs` (887 LOC)

## Quality Gates

- All files ≤ 500 LOC
- `cargo build --all` passes
- `cargo test --all` passes (maintain >90% coverage)
- `cargo clippy --all -- -D warnings` passes
- No API surface changes (backward compatible)

## Success Metrics

- **LOC Reduction**: From ~6,000 LOC to ~2,500 LOC in oversized files
- **Module Count**: 7 files → 25 modules
- **Single Responsibility**: Each module has one clear purpose
- **Maintainability**: Easier to navigate and modify
