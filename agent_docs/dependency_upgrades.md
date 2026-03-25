# Dependency Major Version Upgrades

**CRITICAL**: Always check docs.rs for breaking changes before upgrading.

## redb 3.x
- `begin_read()` moved to `ReadableDatabase` trait - must import:
  ```rust
  use redb::ReadableDatabase;
  ```
- `begin_write()` remains on `Database` struct

## rand 0.10
- `rand::thread_rng()` → `rand::rng()`
- `Rng::gen()` → `RngExt::random()`
- `Rng::gen_range()` → `RngExt::random_range()`
- `Rng::gen_bool()` → `RngExt::random_bool()`
- Import `RngExt` trait: `use rand::RngExt;`
- Keep `rand` and `rand_chacha` versions aligned (both 0.10)

## Upgrade Process
1. Check docs.rs for changelog/breaking changes
2. Run `cargo build` to identify errors
3. Search codebase for old API usage
4. Fix imports in ALL files
5. Run `./scripts/code-quality.sh clippy --workspace`
6. Run `cargo nextest run --all`

## Transitive Security Advisory Handling

When `cargo audit` reports a vulnerability in a transitive dependency:

### Decision Flowchart
1. **Can it be fixed with `cargo update -p <crate>`?** (semver-compatible update)
   - YES → Run update, verify build, commit
   - NO → Continue to step 2

2. **Is there a newer major version of the parent dependency?**
   - YES → Check if upgrading parent is feasible (API changes, breaking changes)
   - NO → Continue to step 3

3. **Can we `[patch]` the vulnerable crate in Cargo.toml?**
   - Only if the patch version is semver-compatible with the constraint
   - `^0.102.8` cannot be patched to `0.103.x` (0.x treats minor as major)
   - YES → Add `[patch]` section, verify build
   - NO → Continue to step 4

4. **Document the advisory ignore in config files** (NOT in CI commands)
   - `.cargo/audit.toml` for `cargo audit`
   - `deny.toml` for `cargo-deny`
   - Include: dependency chain, semver constraint, upstream tracking link, mitigation assessment

### Key Rule: Config Files, Not CLI Flags
- **NEVER** use `--ignore RUSTSEC-XXXX-XXXX` in CI workflow commands
- **ALWAYS** use `.cargo/audit.toml` and `deny.toml` as single source of truth
- Config files provide auditable, reviewable documentation of known issues