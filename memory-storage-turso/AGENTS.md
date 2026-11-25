# Storage Crate Agent Notes

- Cache unit tests under `src/cache/tests/` only compile when `src/cache/mod.rs` declares `#[cfg(test)] mod tests;`. Treat a focused `cargo test` success as evidence only when it reports a nonzero executed-test count (LESSON-086).
