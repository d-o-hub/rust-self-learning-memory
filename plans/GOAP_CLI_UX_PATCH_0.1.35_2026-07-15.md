# GOAP: CLI UX Patch Sprint v0.1.35

**Date**: 2026-07-15  
**Branch**: `fix/0.1.35-patch-issues`  
**Issues**: #829, #830, #831, #832, #828  

---

## Problem Summary (user reports against v0.1.34 release binary)

| Issue | Symptom | Root cause |
|-------|---------|------------|
| **#831** | `pattern list` / `search` return 0 after `episode complete` logged "Successfully cached pattern" | (1) `Pattern` was `#[serde(tag = "type")]` — postcard cannot deserialize internally-tagged enums; (2) `get_all_patterns` only read the empty in-memory map in a fresh process |
| **#830** | `--db-path` / `MEMORY_DB_PATH` ignored | Only set `database.db_path` (Turso local SQLite); redb kept opening default XDG path |
| **#829** | Config format hard to discover; partial TOML failed | No `config init`/`show-template`; sections required all fields (no `#[serde(default)]`) |
| **#832** | Users put `storage_mode` under `[storage]` | Canonical field is `[database].storage_mode`; alias not accepted |
| **#828** | Release drift (auto) | Workspace was `0.2.0` vs tag `v0.1.34`; relabeled to `0.1.35` patch |

---

## Implementation (done)

### #831 Pattern retrieval
- `Pattern` → externally-tagged serde (postcard-compatible) + postcard round-trip unit test
- `StorageBackend::get_all_patterns` + redb/Turso/Cached/Resilient impls
- `queries::get_all_patterns` hydrates memory → redb → Turso
- `retrieve_relevant_patterns` uses lazy load (cross-process)
- e2e `test_pattern_discovery` asserts `pattern list` > 0 across processes
- redb `store_and_list_patterns_across_trait` regression test

### #830 `--db-path`
- Always set `redb_path` from `--db-path` / `MEMORY_DB_PATH`
- Default `storage_mode=local` when path set and no Turso URL

### #829 Config discoverability
- `config init` / `config show-template`
- `#[serde(default)]` on all config sections (partial TOML OK)
- `memory-cli/config/do-memory-cli.example.toml`
- Docs: README, CONFIGURATION_GUIDE, LOCAL_DATABASE_SETUP, CLI_COMMANDS

### #832 storage_mode UX
- Accept `[storage].storage_mode` alias → normalize into `[database]`
- `config show` displays `Storage Mode` + `DB Path`

### #828 Version
- Workspace + publishable crates → `0.1.35` (not premature 0.2.0)
- CHANGELOG `[0.1.35]` section

---

## Prevention (so this class of bug does not return)

| Guard | Location | Purpose |
|-------|----------|---------|
| Postcard round-trip test on `Pattern` | `memory-core/src/patterns/types.rs` | Fail CI if enum serde becomes postcard-incompatible again |
| redb store+list via trait | `memory-storage-redb/src/patterns.rs` | Ensure `get_all_patterns` stays wired |
| e2e cross-process `pattern list` | `tests/e2e/cli_workflows.rs` | Catch empty list after complete in real CLI |
| Loader unit tests | `memory-cli/src/config/loader/mod.rs` | Partial config + storage_mode alias |
| Snapshot for `config --help` | `memory-cli/tests/snapshots/` | Detect missing init/show-template |
| Example config in tree | `memory-cli/config/do-memory-cli.example.toml` | Documented shape next to binary source |
| CLI skill verification | `.agents/skills/do-memory-cli-ops/` | Manual smoke: create → complete → list patterns with project-local db |

### Agent checklist (new, for CLI bugs)
1. Prefer Skill + CLI smoke over unit-only tests for cross-process storage.
2. Never use `#[serde(tag = "...")]` on types persisted with **postcard**.
3. Any new `StorageBackend` method needs redb + Turso + Cached + Resilient impls.
4. CLI flags that change "where data lives" must set **both** `db_path` and `redb_path` (or document exclusivity).
5. Config fields: put selection knobs under `[database]`; allow `[storage]` aliases with `normalize_*`.

---

## Verification (2026-07-15)

| Check | Result |
|-------|--------|
| `cargo check` workspace | ✅ |
| Targeted nextest (core/redb/cli) | ✅ (property tests slowed under load; cases capped) |
| Release CLI build | ✅ |
| CLI: `config init` / `show-template` / partial TOML | ✅ |
| CLI: `[storage].storage_mode` → local | ✅ |
| CLI: `--db-path` / `MEMORY_DB_PATH` open custom redb | ✅ |
| CLI: create → complete → `pattern list` count=1; `pattern search rust` finds 1 | ✅ |

---

## Remaining / out of scope this PR

- **#828 cut release**: tag `v0.1.35` via `release.yml` after merge to main (do not bypass workflow)
- Config commands still init storage even for `show-template` (pre-existing; optional follow-up)
- `pattern search` semantic quality still depends on embeddings (list is the durable-store proof)

---

## Status

**Sprint complete for code fixes.** Ready for commit → PR → merge → release pipeline for `v0.1.35`.
