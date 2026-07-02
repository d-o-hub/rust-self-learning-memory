# Open PR Remediation Plan — 2026-07-02

Analysis of all open GitHub PRs, their review comments, merge conflicts, and CI
failures, with a concrete resolution for each. Scope of this document is
**analysis + solution design only** (per request, only `plans/` was modified;
no source code was changed).

Base of analysis: `origin/main` @ `91839c2f` ("Strict JSON-RPC Content-Length
validation (#708)").

## Snapshot

| PR | Title | Mergeable | CI | Blocking issue |
|----|-------|-----------|----|----------------|
| #718 | feat(patterns): AbstentionPattern extractor + `abstention_score` | BEHIND (no conflict) | ❌ FAIL | `Cargo.lock`: `sysinfo` specified twice |
| #715 | refactor(mcp): remove deprecated marker from `handle_shutdown` (ADR-060) | CONFLICTING | ✅ mostly pass | add/add conflict on 1 test file |
| #711 | docs: align documentation with architecture | BEHIND (no conflict) | ✅ pass | none — just needs rebase/update |
| #709 | Resolve flat file conflicts / consolidate modules (`episodic`→`episode`, `pattern`→`patterns`) | CONFLICTING | ❌ FAIL | dangling `mod` decls + `Cargo.toml` conflict |

### PR comments (all four PRs)

There are **no human or inline review-thread comments** on any of the four PRs.
Every comment is from a bot and requires no code action:

- `google-labs-jules` — automated "reporting for duty" intro.
- `codacy-production` — "Up to standards ✅", 0 new issues.
- `codecov` — "All modified and coverable lines are covered".
- `github-actions` — benchmark result tables.

No requested changes exist to address; "addressing comments" here reduces to
making CI green and resolving conflicts.

---

## PR #718 — AbstentionPattern extractor

**State:** `MERGEABLE` but `BEHIND` main. CI red.

**Root cause of CI failure** (`Quick PR Check (Format + Clippy)` → *Run clippy on lib*):

```
error: failed to parse lock file at: .../Cargo.lock
Caused by:
  package `sysinfo` is specified twice in the lockfile
```

This is a lockfile/manifest inconsistency inherited from `main`, not something
the abstention feature introduced. The three workspace crates disagree on the
`sysinfo` requirement:

| Crate | `sysinfo` requirement (main) |
|-------|------------------------------|
| `memory-cli/Cargo.toml`  | `0.39` |
| `tests/Cargo.toml`       | `0.39` |
| `memory-mcp/Cargo.toml`  | `0.38`  ← inconsistent |

Because `"0.38"` and `"0.39"` are incompatible caret ranges, Cargo resolves two
separate `sysinfo` entries. `main`'s committed `Cargo.lock` already contains two
`sysinfo` blocks (both `0.39.5`), and when PR #718's branch (which pinned a
single `0.38.4`) is merge-tested against main, the combined lock ends up with
both `0.38.4` and `0.39.5` → "specified twice".

**Solution:**

1. Unify the `sysinfo` requirement across the workspace to a single version.
   Set `memory-mcp/Cargo.toml` `sysinfo = "0.39"` to match `memory-cli` and
   `tests` (do **not** downgrade the others to `0.38`).
2. Regenerate the lockfile: `cargo update -p sysinfo` (or delete the duplicate
   block and run `cargo build`) so exactly one `sysinfo` entry remains.
3. Rebase the branch on `origin/main` to clear the `BEHIND` state.
4. Re-run: `./scripts/code-quality.sh fmt && ./scripts/code-quality.sh clippy --workspace`.

**Note:** The `sysinfo` duplication is a **latent bug on `main` itself**
(`origin/main:Cargo.lock` has two identical `sysinfo 0.39.5` blocks and
`memory-mcp` pins `0.38` while the lock resolves `0.39.5`, which violates the
`"0.38"` caret range). Fixing it on `main` first — a one-line manifest change
plus lock regeneration — would unblock #718 automatically and prevent the same
failure recurring on future PRs. This should be a standalone `fix(deps)` commit.

Otherwise the abstention change itself is clean: Codacy reports 0 issues, adds a
new `abstention_pattern.rs` extractor and a new `Abstained` `TaskOutcome`
variant with call sites updated across the crate.

---

## PR #715 — remove deprecated marker from `handle_shutdown` (ADR-060)

**State:** `CONFLICTING`. CI (on branch head) mostly green.

**Merge conflict:** exactly one file —

```
CONFLICT (add/add): memory-mcp/tests/jsonrpc_shutdown_integration_test.rs
```

Both sides added this file:
- `main` added it via PR #708 (with a crate-level `#![allow(deprecated)]`
  because `handle_shutdown` was still `#[deprecated]`).
- PR #715 added its own version that **drops** `#![allow(deprecated)]` (its whole
  purpose is to remove the deprecation marker, so the allow is no longer needed).

The two versions differ only in the doc comment wording, the removed
`#![allow(deprecated)]` line, and a comment ("real handler" vs "real
(deprecated) handler").

**Solution:** Resolve by taking **PR #715's version wholesale** (`git checkout
--theirs` for this file during the merge/rebase). This is consistent with #715's
intent: once `#[deprecated]` is removed from `handle_shutdown`, the
`#![allow(deprecated)]` attribute would itself trigger an
`unused_attributes`/`useless_deprecated`-style warning under `-D warnings`, so it
*must* be dropped. Steps:

1. Rebase #715 onto `origin/main`.
2. On the conflict, keep the branch's file (drops `#![allow(deprecated)]`).
3. Verify no other `#[allow(deprecated)]` remains referencing `handle_shutdown`
   (PR already removes them in `server_impl/core.rs` and `server_impl/jsonrpc.rs`).
4. `cargo clippy --workspace --all-targets -- -D warnings` and run the MCP
   integration tests listed in the PR body.

No further comment action required.

---

## PR #711 — documentation alignment

**State:** `MERGEABLE`, `BEHIND`. CI green.

**No merge conflict, no CI failure.** Files touched are docs + `scripts/code-quality.sh`
+ one benchmark assertion. Codecov and Codacy pass.

**Solution:** Update the branch (merge/rebase `origin/main`) to clear `BEHIND`,
then it is ready to merge. Before merge, sanity-check that the `scripts/code-quality.sh`
edits (8 add / 8 del) do not weaken any quality gate — the change is described as
tone/wording sanitization, so confirm it did not alter clippy lint sets or
thresholds. No comments to address.

---

## PR #709 — flat-file conflict resolution / module consolidation

**State:** `CONFLICTING`. CI red. Largest and riskiest PR (renames `episodic`→
`episode`, `pattern`→`patterns`, plus many file moves; Codacy: 103 complexity,
50 duplication).

### Problem 1 — CI failure (`cargo fmt` step, before clippy)

```
Error writing files: failed to resolve mod `cache`:
  memory-mcp/src/cache.rs does not exist
Error writing files: failed to resolve mod `tests`:
  memory-storage-turso/src/transport/compression/types/tests.rs does not exist
```

The refactor left **dangling `mod` declarations** pointing at files that were
moved/renamed/deleted. `cargo fmt` (and compilation) cannot resolve them.

**Solution:**
- In `memory-mcp/src/` find the `mod cache;` declaration and either restore
  `cache.rs`/`cache/mod.rs` or remove the stale declaration to match the new
  layout.
- In `memory-storage-turso/src/transport/compression/types/` remove or restore
  the `mod tests;` declaration so it matches the actual file tree.
- Run `cargo build --workspace` locally to surface every remaining unresolved
  module — a rename PR of this size likely has more than these two.

### Problem 2 — merge conflict

```
CONFLICT (content): memory-mcp/Cargo.toml
```

Three-way differences vs `main`:

| Line | `main` | PR #709 | Correct resolution |
|------|--------|---------|--------------------|
| `do-memory-core` dep | `version = "0.1.33"` | adds `features = ["agentfs"]` | **keep PR's** `features = ["agentfs"]` |
| `which = "4"` | present | **removed** | **keep main's** — required |
| `regex = { workspace = true }` | present | **removed** | **keep main's** — required |
| `sysinfo` | `"0.38"` | `"0.38"` | no conflict (but see #718 — should become `"0.39"`) |

**Critical:** PR #709 removes `which` and `regex` from `memory-mcp`, but
`main` (via PR #706 "Harden Node.js sandbox") added
`memory-mcp/src/sandbox.rs` which uses **both**:

```
memory-mcp/src/sandbox.rs:75: use regex::RegexSet;
memory-mcp/src/sandbox.rs:81: use which::which;
```

Removing those deps after merging `main` would break compilation. The merged
`memory-mcp/Cargo.toml` must therefore **retain `which` and `regex`** while also
**keeping PR #709's `agentfs` feature** on `do-memory-core`.

**Solution (ordered):**
1. Rebase #709 onto `origin/main`.
2. Resolve `memory-mcp/Cargo.toml`: union — `do-memory-core` with
   `features = ["agentfs"]` **and** keep `which = "4"` + `regex = { workspace = true }`.
   Set `sysinfo = "0.39"` (align with #718 fix).
3. Fix all dangling `mod` declarations (Problem 1).
4. `cargo build --workspace` → `./scripts/code-quality.sh fmt` →
   `clippy --workspace --all-targets -- -D warnings` → `cargo nextest run --all`.
5. Because this PR renames modules that #718's new `abstention_pattern.rs` lives
   under (`patterns/extractors/`), sequence matters — see below.

---

## Recommended merge order

Merge conflicts compound: #709 renames the very modules #718 and #715 touch.
Land in dependency order, rebasing each survivor after every merge:

```diagram
╭──────────────────────────────────────────────────────────╮
│ 0. fix(deps) on main: unify sysinfo → 0.39, dedupe lock   │
│    (unblocks #718 and prevents recurrence)                 │
╰───────────────────────────┬──────────────────────────────╯
                            ▼
   ╭─────────────╮   ╭─────────────╮   these are small & low-risk
   │ #711 docs   │   │ #715 mcp    │   → merge first
   ╰──────┬──────╯   ╰──────┬──────╯
          ╰────────┬────────╯
                   ▼
            ╭─────────────╮
            │ #718 feature│   → rebase after sysinfo fix, merge
            ╰──────┬──────╯
                   ▼
            ╭─────────────╮
            │ #709 refactor│  → merge LAST (largest surface,
            ╰─────────────╯     rebase over everything above)
```

Rationale:
- **#711** and **#715** are tiny and nearly ready; land them first.
- **#718** only needs the `sysinfo` fix + rebase.
- **#709** is a sweeping rename; landing it last means it rebases once over the
  final tree instead of forcing every other PR to re-resolve rename conflicts.

## Verification checklist (apply to each PR before merge)

- [ ] `./scripts/code-quality.sh fmt`
- [ ] `./scripts/code-quality.sh clippy --workspace`
- [ ] `./scripts/build-rust.sh check`
- [ ] `cargo nextest run --all` + `cargo test --doc`
- [ ] `cargo metadata` shows a single `sysinfo` version
- [ ] `git status` clean / all changes staged
```

