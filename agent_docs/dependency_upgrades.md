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

## sysinfo 0.40
- Every constructor is now fallible — `System::new()`, `System::new_all()`,
  `Components::new()`, `Disks::new()`, `Networks::new()`, `Users::new()`,
  `Groups::new()` all return `Result<Self, Error>`:
  ```rust
  if let Ok(mut system) = System::new_all() {
      system.refresh_all();
  }
  ```
- `SystemExt` is gone (the trait split landed earlier); `IS_SUPPORTED_SYSTEM`,
  `set_open_files_limit`, and the `list[_mut]` methods on `Users`, `Components`,
  `Disks`, `Networks` were removed.
- `Networks` interface names moved `String` → `OsString`.
- Degrade to the call site's existing fallback rather than inventing a machine
  profile: the CLI `suggest_*` helpers already clamp to their minimums, so
  `(0, 0)` memory and `unwrap_or(1)` cores are the conservative answer.
- Migrated call sites: MCP `memory_usage_mb` (`memory-mcp/.../time_series.rs`),
  CLI config defaults (`memory-cli/src/config/types/defaults.rs`), environment
  fitness validation (`memory-cli/src/config/validator/rules/environment.rs`),
  soak memory capture (`tests/soak/stability_test.rs`).

## In-Tree Alignment Constraints

A version comment in `Cargo.toml` is a constraint, not decoration, and Dependabot
does not read it.

| Crate | Requirement | Reason |
|---|---|---|
| `libsql` | one version per graph; today `0.9.30` | `chaotic_semantic_memory` 0.3.8 requires `0.9.x`; two libsql versions duplicate libsql's exported C symbols when the `csm` feature links |

- A `cargo check` cannot detect the conflict (rlibs do not link). Detect it from
  the lockfiles: `./scripts/check-lockfile-sync.sh`, harness sensor
  `lockfile-sync`.
- When a group bump includes such a crate, drop that member from the group, keep
  the rest, and re-resolve both lockfiles. Do not add a waiver to make the split
  acceptable.

## Both Lockfiles Must Stay Resolved

`fuzz` is `exclude`d from the root workspace and has its own `fuzz/Cargo.lock`.
Any change to a requirement reachable from `do-memory-core` / `do-memory-mcp`
also needs the fuzz graph re-resolved:

```bash
cd fuzz && cargo metadata --format-version 1 >/dev/null
```

Otherwise `cargo deny --manifest-path fuzz/Cargo.toml`, run by
`scripts/check-dependency-audit.sh`, rewrites the stale lockfile mid-scan and the
Supply Chain Audit fails with "Lockfiles were modified during the audit scan!"
while reporting 0 vulnerabilities. `cargo metadata --locked` proves freshness
without touching files. See LESSON-077.

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