---
name: cargo-mutants-workspace
description: "Configure and run cargo-mutants correctly in Cargo workspaces. Prevents the silent 0-mutant failure caused by --file path resolution relative to workspace root, not crate root."
---

# cargo-mutants Workspace Skill

Correctly configure `cargo-mutants` in a Cargo workspace. The most common silent failure is **0 mutants found** due to incorrect `--file` path prefixes.

---

## ⚠️ Critical Rule: --file Paths Are Workspace-Relative

`cargo-mutants` **always** resolves `--file` glob patterns relative to the **workspace root**, regardless of which `--package` is specified.

### ❌ Wrong (silent failure — 0 mutants)
```bash
cargo mutants \
  --package do-memory-core \
  --file 'src/reward/**/*.rs'   # resolved as <workspace-root>/src/reward/ -- doesn't exist!
```

### ✅ Correct
```bash
cargo mutants \
  --package do-memory-core \
  --file 'memory-core/src/reward/**/*.rs'  # full path from workspace root
```

**Rule:** Always prefix `--file` patterns with the crate's subdirectory relative to the workspace root.

---

## Diagnosis: Why 0 Mutants?

When you see:
```
WARN No mutants found under the active filters
Found 0 mutants to test
```

Check in this order:

1. **Wrong `--file` prefix** (most common) -- see rule above
2. **All functions are skipped** -- check `#[mutants::skip]` annotations or `skip` config
3. **`--package` name mismatch** -- verify with `cargo metadata --no-deps | jq '.packages[].name'`
4. **Files excluded via `.cargo-mutants.toml`** -- check for overly broad `exclude` patterns

### Quick Verify
```bash
# Dry-run to list what cargo-mutants WOULD test without running tests
cargo mutants --package do-memory-core \
  --file 'memory-core/src/reward/**/*.rs' \
  --list
```

If `--list` returns 0 lines, the `--file` filter is wrong.

---

## Workspace Structure Pattern

For this repo's workspace layout:

```
<workspace-root>/
|-- Cargo.toml              <- workspace manifest
|-- memory-core/            <- crate dir (package name: do-memory-core)
|   `-- src/
|       |-- reward/
|       |-- retrieval/
|       |-- retry/
|       `-- patterns/
|-- memory-cli/             <- crate dir (package name: do-memory-cli)
|-- memory-mcp/             <- crate dir (package name: do-memory-mcp)
`-- ...
```

**File path formula:** `<crate-dir>/src/<module>/**/*.rs`

| Package | Crate Dir | Example --file |
|---------|-----------|----------------|
| `do-memory-core` | `memory-core/` | `memory-core/src/reward/**/*.rs` |
| `do-memory-cli` | `memory-cli/` | `memory-cli/src/**/*.rs` |
| `do-memory-mcp` | `memory-mcp/` | `memory-mcp/src/**/*.rs` |

---

## Correct mutants.yml Pattern

```yaml
- name: Verify mutation target files exist
  run: |
    # Guard: fail fast if --file paths resolve to nothing (LESSON-MUT-001)
    FILE_COUNT=$(find \
      memory-core/src/reward \
      memory-core/src/retrieval \
      memory-core/src/retry \
      memory-core/src/patterns \
      -name '*.rs' 2>/dev/null | wc -l)
    if [ "$FILE_COUNT" -eq 0 ]; then
      echo "::error::No .rs files found. Check --file paths are workspace-root-relative."
      exit 1
    fi
    echo "Preflight OK: found $FILE_COUNT .rs files"

- name: Run mutation testing
  run: |
    cargo mutants \
      --package do-memory-core \
      --timeout "${TIMEOUT}" \
      --file 'memory-core/src/reward/**/*.rs' \
      --file 'memory-core/src/retrieval/**/*.rs' \
      --file 'memory-core/src/retry/**/*.rs' \
      --file 'memory-core/src/patterns/**/*.rs' \
      --output mutants.out \
      -- --lib
```

---

## Lessons Learned

### LESSON-MUT-001: cargo-mutants --file is workspace-root-relative
- **Context:** `mutants.yml` used `--file 'src/reward/**/*.rs'` with `--package do-memory-core`
- **Symptom:** `WARN No mutants found under the active filters` / `Found 0 mutants`
- **Cause:** `cargo-mutants` resolves `--file` from workspace root, not crate root. `src/reward/` does not exist at workspace root.
- **Fix:** Prefix with crate directory: `memory-core/src/reward/**/*.rs`
- **Prevention:** Always use `--list` to dry-run before scheduling in CI; add file-count guard step
- **Discovered:** 2026-07-26 (issue #898)

---

## Related

- Issue [#898](https://github.com/d-o-hub/rust-self-learning-memory/issues/898) -- original bug report
- Issue [#747](https://github.com/d-o-hub/rust-self-learning-memory/issues/747) -- mutation testing phase plan
- [cargo-mutants documentation](https://mutants.rs/)
