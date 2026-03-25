# Disk Hygiene

Practical disk-management guidance for local development.

## Why This Exists

This workspace can grow large quickly due to Rust build artifacts, coverage output, and optional JS dependencies. Use script-first cleanup to keep local environments healthy and consistent.

## Script-First Cleanup

Prefer `scripts/clean-artifacts.sh` over ad-hoc `rm -rf` commands.

```bash
# Default cleanup (incremental + release + coverage artifacts)
./scripts/clean-artifacts.sh

# Fast cleanup for iteration loops
./scripts/clean-artifacts.sh quick

# Full cleanup (cargo clean + coverage artifacts)
./scripts/clean-artifacts.sh full

# Include optional JS dependency cleanup
./scripts/clean-artifacts.sh standard --node-modules

# Preview actions without deleting
./scripts/clean-artifacts.sh standard --dry-run
```

## `CARGO_TARGET_DIR` Usage

Use `CARGO_TARGET_DIR` to offload build artifacts to a larger/faster disk.

```bash
# One-off command
CARGO_TARGET_DIR=/mnt/fastssd/rslm-target ./scripts/build-rust.sh dev

# One-off cleanup against that same location
CARGO_TARGET_DIR=/mnt/fastssd/rslm-target ./scripts/clean-artifacts.sh standard
```

Notes:
- If `CARGO_TARGET_DIR` is set, cleanup script uses it automatically.
- Relative target dirs are resolved from repository root.
- Keep the same target-dir value across build/test/cleanup commands for predictable results.

## Coverage Artifact Hygiene

Coverage runs can leave substantial artifacts. `standard` and `full` cleanup modes remove:
- `target/llvm-cov-target` (or equivalent under `$CARGO_TARGET_DIR`)
- `coverage/`, `coverage-html/`
- `*.profraw`, `*.profdata`
- `lcov.info`, `cobertura.xml`

## `node_modules` Reality

`node_modules/` may appear locally for auxiliary tooling/workflows. Cleanup is **opt-in** (`--node-modules`) so JS dependencies are not removed unexpectedly.

## Recommended Routine

1. Daily/after heavy test cycles: `./scripts/clean-artifacts.sh quick`
2. Before large branch switches or low-disk warnings: `./scripts/clean-artifacts.sh standard`
3. Before deep rebuild/debug reset: `./scripts/clean-artifacts.sh full`
4. If JS tooling is not needed locally: rerun with `--node-modules`
