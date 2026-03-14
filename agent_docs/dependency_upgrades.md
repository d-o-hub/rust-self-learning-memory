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
5. Run `cargo clippy --all -- -D warnings`
6. Run `cargo nextest run --all`