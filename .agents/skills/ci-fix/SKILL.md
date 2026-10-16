---
name: ci-fix
description: "Diagnose and fix GitHub Actions CI failures for Rust projects. Use when CI fails, tests timeout, or linting issues occur. Captures common patterns from CLAUDE_INSIGHTS_REPORT.md."
---

# CI Fix Skill

Diagnose and fix GitHub Actions CI failures using patterns from past CI issues.

## Quick Diagnosis Pattern

1. Get CI status:
```bash
gh run list --limit 5 --json status,conclusion,name,headBranch
```

2. Get failed job logs:
```bash
gh run view <run_id> --log --job <job_name>
```

3. Categorize failure:
- **lint**: fmt/clippy warnings
- **test**: test failures or timeouts
- **build**: compilation errors
- **security**: cargo audit/deny failures
- **coverage**: coverage threshold missed
- **deprecated**: old action versions (common!)

## Common Fixes (From History)

### Deprecated GitHub Actions
```bash
# Detect
grep -r "actions/checkout@v1" .github/workflows/
grep -r "actions-rs" .github/workflows/

# Fix: Update to v2+
```

### Optional Dependency Issues (libclang)
```bash
# Pattern: --all-features triggered optional dep issues
# Fix: build with the specific features you need
cargo build --workspace --features "turso,redb"
```

### Clippy Lint Allow-List
```bash
# See new warnings
./scripts/code-quality.sh clippy --workspace 2>&1 | grep "warning:"

# Fix: Add #[allow(...)] comments
```

### Coverage Threshold
```bash
# Generate coverage
cargo tarpaulin --workspace

# Check threshold in .github/workflows/
```

### Benchmark Timeout
```bash
# Increase timeout-minutes in workflow YAML
```

### Check Quick Check Status CANCELLED after 15m
```bash
# Symptom: YAML Lint (or CI) "Check Quick Check Status" cancelled; real jobs skipped
# Cause: wait-on-check timeout-minutes:15 < Quick Check wall time (~15–20m)
# Permanent fix:
#  1. Remove wait gate from cheap workflows (yaml-lint.yml)
#  2. Raise remaining wait jobs to timeout-minutes: 40
#  3. Raise Quick Check job to timeout-minutes: 25
#  4. running-workflow-name must match this workflow's name:
# See LESSON-021, agent_docs/github_actions_patterns.md
```

### Insta snapshot fails only on macOS
```bash
# Cause: f32/f64 {:?} Debug differs by platform
# Fix: format!("{:.4}", value) before assert_snapshot!
# See LESSON-022
```

### RustSec advisory failure on `main` (Cargo Deny / Dependency Audit)
```bash
# Symptom: Cargo Deny, Dependency Audit, or Quality Gates goes red with no code
#          change — a new advisory landed against the existing lock.
# TRAP: the step runs TWO blocking tools. Fixing what `cargo deny` printed
#       leaves `cargo audit --json` (vulnerabilities.count > 0) red, because
#       they consult different advisory DB clones. Scope from the union.
# TRAP: local `cargo deny` reports a FALSE GREEN when its DB clone
#       (~/.cargo/advisory-dbs/) is stale; check the checkout date, or verify
#       with `cargo audit` (~/.cargo/advisory-db) which refreshes separately.
cargo deny check advisories
cargo audit --json > /tmp/audit.json && jq -r '.vulnerabilities.count' /tmp/audit.json
# Fix: bump to the minimal patched version from the advisory's
#      [versions] patched range, not the newest:
cargo update -p <pkg> --precise <min-patched>
# Then re-run the WASM gate if the bump moved getrandom across a major.
# Waivers go in BOTH deny.toml and .cargo/audit.toml, with verified evidence.
# See .agents/skills/code-quality/SKILL.md, LESSON-046..048
```

### Drift check (`Check for unreleased changes`) will not clear on re-run
```bash
# Symptom: fails while cadence is critical; the workflow auto-adds
#          `release-preparation`, but `gh run rerun` fails again.
# Cause: the re-run replays the ORIGINAL event payload (label-less), and the
#        bot-applied label (via GITHUB_TOKEN) cannot trigger a new run.
gh pr edit <n> --remove-label release-preparation
gh pr edit <n> --add-label release-preparation   # human actor => `labeled` run
# See LESSON-049 (and LESSON-044 for the synchronize-event variant)
```

### Nightly is red — or "nightly didn't run"
```bash
# Symptom: no nightly run at the expected time, or nightly counts moved with no
#          push. Do NOT conclude "nightly didn't run" from the cron line.
# Cause: `0 2 * * *` in .github/workflows/nightly-tests.yml is a subscription,
#        not a clock — scheduled runs sit in the queue (observed starts
#        06:30–06:49Z) and the same workflow ALSO fires on `push` of `v*.*.*`.
#        What you are reading is decided by event + timing, never by the cron.
gh run list --workflow nightly-tests.yml --limit 10 \
  --json number,event,createdAt,headSha,status,conclusion
gh run view <run_id> --json event,createdAt,runStartedAt,conclusion
# Fix (diagnosis): workflow_dispatch is the only reliable clock for "nightly
# now" — dispatch it and name the run id in any claim about it having run.
gh workflow run nightly-tests.yml
# Never quote a nightly pass/fail count without the feature set that produced it
# (next entry). An always-red nightly re-attaches to the release SHA and blocks
# the ship via ci-check: narrow that gate only by explicit decision, never by
# silently excluding schedule runs or deleting runs (LESSON-051, LESSON-052).
```

### An ignored test failed / the test count changed
```bash
# Symptom: nightly "reports" a schema mismatch from a #[ignore]d Turso test, or
#          the total count moves between two runs of the same commit.
# Cause 1: `[[test]] required-features` is a BUILD-level gate — with the feature
#          off the target never compiles, so tests silently vanish from totals.
#          Counts are only comparable inside ONE feature matrix.
# Cause 2: `--run-ignored only` runs (and thus FORCE-RUNS) ONLY the ignored
#          tests; `--run-ignored all` runs normal + ignored. Force-running a
#          test that is ignored because its schema/feature is absent produces a
#          TEST CONFIGURATION mismatch — it is not a storage bug.
cargo nextest run -p do-memory-storage-turso --run-ignored only
#   => 86 passed / 0 failed, but ONLY against the default schema
cargo nextest run -p do-memory-storage-turso --features turso_multi_dimension \
  --test multi_dimension_routing --test phase1_validation --run-ignored all
#   => 20 passed / 0 skipped
# 86 + 20 = 106 = the pre-fix total => nothing broke; one number had been mixing
# two configurations. Fix = split the run per feature set, never "fix" the schema.
```

### Green audit + green deny, but Dependabot alerts are open
```bash
# Symptom: cargo audit and cargo deny are green in CI while the Security tab
#          still lists advisories. This is a NORMAL state, not a broken gate —
#          the two tools and Dependabot read three different registers.
# Classify the register BEFORE touching any gate:
gh api "/repos/{owner}/{repo}/dependabot/alerts?state=open" \
  -q '.[] | [.security_advisory.cve_id, .dependency.manifest_path,
              .security_advisory.severity] | @tsv'
# Inspect `.security_advisory | keys` before trusting a jq projection — a wrong
# key silently returns empty columns; and count DISTINCT GHSA ids, not rows.
# Case A (invisible to BOTH gates): lru RUSTSEC-2026-0002 is `informational = "unsound"`.
#          cargo-audit routes that class to `.warnings.unsound`, never to
#          `.vulnerabilities`, and the CI step fails only on `.vulnerabilities.count`,
#          so it cannot redden audit; `cargo deny` does not gate informational ids
#          either. Measured: in-repo `cargo audit --no-fetch --json` => rc 0,
#          `.vulnerabilities.list` empty, yet 5 `.warnings.unsound` + 1 `.unmaintained`
#          survive. Run the same lock from /tmp (so `.cargo/audit.toml` ignores do not
#          apply) => rc 1 with 8 `.vulnerabilities` (rsa 2023-0071, rustls-webpki
#          2026-0049/0098/0099/0104, h2 2026-0258, rustls 2026-0346/0349).
#          Lesson: a green audit means "nothing un-ignored reached .vulnerabilities" —
#          always name the ignore list and the .warnings count when citing it.
cargo audit --no-fetch --json > /tmp/audit.json; echo "rc=$?"
jq '{vulns: (.vulnerabilities.count // 0), unsound: ((.warnings.unsound // [])|length), unmaintained: ((.warnings.unmaintained // [])|length)}' /tmp/audit.json
grep -n 'informational' ~/.cargo/advisory-dbs/*/crates/*/RUSTSEC-*.md | head
# Case B (neither): libsql-sqlite3-parser CVE-2025-47736 / GHSA was never adopted
#          into RustSec (no crates/libsql-sqlite3-parser in the advisory DB) =>
#          no local gate can see it; only Dependabot does. It stays an open alert
#          until upstream ships a patch — classify it upstream-blocked.
# Hard rule: never "fix" this by waiving, skipping, or bypassing a gate to match
#          the alert count, and never suppress a check because Dependabot
#          disagrees with cargo. The only sanctioned record is an evidence-backed
#          waiver in BOTH deny.toml and .cargo/audit.toml (RustSec entry above).
```

## Diagnosing "failures" that are not failures

- `gh pr checks` prints **cancelled** rows as `fail` with a duration. Confirm the
  real conclusion before diagnosing: `gh api repos/{owner}/{repo}/actions/jobs/<job_id>
  --jq '.status, .conclusion'` (`completed/cancelled`). Cancelling runs also keeps
  `mergeStateStatus` away from `CLEAN`, so never cancel a PR you intend to merge
  (LESSON-041).
- **`check-pr-readiness.sh` reports `N fail` while `gh pr checks` shows none**:
  `statusCheckRollup` retains *every* check run for the head SHA, so a re-run
  (label toggle, `gh run rerun`) leaves stale `FAILURE` rows that the readiness
  counter counts. Confirm with `gh api repos/{owner}/{repo}/commits/<sha>/check-runs`
  (compare `completed_at` per name) and fix it on a **fresh SHA** —
  `git commit --allow-empty -m "chore: re-trigger CI after <reason>" && git push`.
  Deleting workflow runs to quiet a gate is a bypass (LESSON-052).
- A red `Code Coverage` or `Storage Matrix` job is often the *formatting* gate:
  both run `e2e-tests/tests/quality_gates.rs`, whose `quality_gate_formatting`
  test runs `cargo fmt --check` (LESSON-039). Find the failing test name before
  looking at coverage numbers.
- A scheduled job (fuzz/nightly) failing with no repository change is usually a
  floating toolchain: `nightly` broke the transitive `branches` crate via
  `core::intrinsics` (LESSON-040). Pin the toolchain in the workflow.
- The same story on `stable`, and it looks like a lint regression: a clippy-only
  red, or a red `quality_gate_formatting`, with nothing of yours changed. New
  deny-by-default lints appear between releases (`assert_is_empty`,
  `redundant_explicit_links` in 1.99). Diagnose by `rustup update stable` and
  reproducing on **untouched `main`** before bisecting your branch; if `main`
  reproduces it, it is not your PR's failure. The fix is always the lint itself,
  never a suppression and never relaxing the gate (LESSON-039, LESSON-040).
- A row stuck at `skipping` forever on a PR is not a skipped gate.
  `Maintain canonical drift issue` is guarded by
  `if: github.event_name != 'pull_request'`, so it only ever runs on schedule
  (LESSON-049).
- **The ruleset, not branch protection, decides what blocks a merge.** `main` is
  guarded by ruleset `main-protection`; `/branches/main/protection` 404s. Green
  rows outside `.rules[].parameters.required_status_checks` don't unblock
  anything, and an unresolved review thread can keep `mergeStateStatus` at
  `BLOCKED` with every check passing (LESSON-050).

## Fix Commands

```bash
# Linting
cargo fmt --all
cargo clippy --workspace --fix --allow-dirty

# Testing
cargo test --workspace -- --nocapture

# Security
cargo audit
cargo deny check

# Build
cargo build --workspace
```

## Success Criteria
- All CI jobs pass
- No new warnings introduced

## Codecov Patch Coverage Gap Fix

When Codecov reports "Patch coverage is X% with N lines missing coverage":

### Step 1: Parse the Report
Identify files with missing lines from the Codecov comment:
```
| File | Patch % | Missing Lines |
|------|---------|---------------|
| path/to/file.rs | 0.00% | 24 Missing |
```

### Step 2: Locate Uncovered Code
For each file, read the specific lines identified. They are typically:
- New functions without tests
- Extracted submodules (refactored code moved to new files)
- Changed branches not exercised by existing tests

### Step 3: Write Targeted Tests
Add tests to the existing `#[cfg(test)] mod tests` in each file, or to a sibling test file:
- For new public functions: unit test exercising the function directly
- For new structs: test Default, Debug, Clone impls + key methods
- For refactored code: test that the extracted module works identically to before
- For branch coverage: test each match arm / if-else branch

### Step 4: Verify
```bash
cargo nextest run -p <crate> -E 'test(<test_name_pattern>)'
cargo clippy --workspace -- -D warnings
```

### Step 5: Commit
Single atomic commit: `test(coverage): add tests for Codecov patch gaps in N files`

### Common Patterns
- **Extracted submodule** (e.g. `stats.rs` from `wrapper.rs`): test the new module's public API directly
- **New monitoring metric**: test the recording method + snapshot/getter
- **New heuristic function**: test boundary conditions (empty input, short/long input, edge cases)
- **New StorageType enum**: test Debug, Clone, and pattern matching
- Changes committed if needed
