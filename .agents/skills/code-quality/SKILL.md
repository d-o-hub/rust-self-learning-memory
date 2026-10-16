---
name: code-quality
description: "Maintain high code quality through formatting, linting, static analysis, and clean code principles. Use for rustfmt, clippy, cargo audit, code reviews, refactoring, and quality gates."
---

# Code Quality

Unified skill for Rust code quality and clean code development.

## Quick Commands

```bash
./scripts/code-quality.sh fmt                    # Format check
./scripts/code-quality.sh clippy --workspace     # Lint with clippy
./scripts/code-quality.sh audit                  # Security audit
./scripts/code-quality.sh check                  # Run all quality gates
./scripts/quality-gates.sh                       # Full gates (coverage floor 70%, target 90%)
```

## Quality Gates

| Check | Command | Target |
|-------|---------|--------|
| Format | `./scripts/code-quality.sh fmt` | 100% compliant |
| Lint | `./scripts/code-quality.sh clippy --workspace` | Zero warnings |
| Audit | `cargo audit` | No known vulnerabilities |
| Coverage | `cargo llvm-cov --html` | >=70% floor (90% target) |
| Docs | `cargo doc --no-deps` | All public APIs |

## Rust Quality Dimensions

| Dimension | Focus |
|-----------|-------|
| Structure | Files <500 LOC, clear modules |
| Error Handling | Custom Error, Result<T>, `?` operator (no unwrap) |
| Async | spawn_blocking for CPU work, no blocking in async |
| Testing | >=70% coverage floor (90% target), AAA pattern, cargo nextest + doctests |
| Security | Parameterized SQL, env vars, no hardcoded secrets |

## Clean Code Principles

**SOLID**: Single Responsibility, Open-Closed, Liskov Substitution, Interface Segregation, Dependency Inversion

**DRY/KISS/YAGNI**: Extract common code, simple solutions, build what's needed now

## Before Every Push

- Run `cargo fmt --all -- --check` as the **last** step, including after conflict
  resolution and after hand-indenting merged blocks. `tests/quality_gates.rs::quality_gate_formatting`
  re-runs the same check inside the Storage Matrix *and* Coverage jobs, so one
  formatting miss reds three jobs (LESSON-039).
- A crate-root `#[expect(lint, reason = "...")]` must be **fulfilled** in every
  configuration that compiles the crate; an expectation that never fires fails
  `cargo clippy --workspace --tests -- -D warnings` as
  `unfulfilled_lint_expectations`. Dead `cfg(test)` suppressions are the usual
  cause — delete them or narrow the `cfg_attr` (LESSON-037).
- **Never act on a clippy finding from a `-p <crate>` run.** The authoritative gate is
  `cargo clippy --workspace --tests -- -D warnings -A clippy::expect_used
  -A clippy::uninlined_format_args -A clippy::unwrap_used`
  (`scripts/code-quality.sh:144,158`; CI calls it at `ci.yml:94`). Feature unification
  (the expectation only fires once the workspace's unified feature set compiles the code
  path it gates), and can miss findings the workspace run catches. Observed on both
  `memory-storage-turso/src/lib.rs` and `test-utils/src/lib.rs` (`do-memory-test-utils`)
  while writing PR #1180's tests — neither crate needed touching, and the workspace gate
  was green.
  Re-run the exact workspace command before editing another crate's `lib.rs`. Two more
  traps: `-A clippy::expect_used` on the command line cannot silence a crate-level
  `#![expect(...)]` (the attribute wins), and clippy's own suggestions on new test code
  must be applied as rewrites, not suppressions — `&vec[..].to_vec()`-style clones want
  `std::slice::from_ref(&x)`, and fields set after `Default::default()` want a struct
  literal with `..Default::default()` (LESSON-060).
- A new defaulted `StorageBackend` method also needs a classification in
  `scripts/storage-backend-capabilities.toml` (see `feature-implement`).

## Anti-Patterns

| Bad | Fix |
|-----|-----|
| Excessive clone | Use borrowing or Arc |
| Unnecessary unwrap | Use `?` operator |
| Deep nesting | Extract methods |
| Large functions | Split (<50 LOC) |
| Deadlocks | Release locks before `.await` |
| `.py`/`.sh` in root | Move to `scripts/` or delete if one-off |

## Dependency Monitoring

```bash
cargo tree -d | grep -cE "^[a-z]"   # Count duplicates (target: <100)
cargo machete && cargo shear       # Find unused deps
```

## Security Advisory Bumps (read before scoping a fix)

**Two blocking tools, one step.** The Quality Gates job runs `cargo deny check
advisories` *and* `cargo audit --json` (failing on `.vulnerabilities.count > 0`).
They read different advisory DB clones and report different sets, so scope from
the union (LESSON-046):

```bash
cargo deny check advisories
cargo audit --json > /tmp/audit.json; jq -r '.vulnerabilities.count' /tmp/audit.json
jq -r '.vulnerabilities.vulnerabilities[] | .package.name + " " + .advisory.id' /tmp/audit.json
```

**Three registers, one row per lockfile.** Dependabot alerts are a third source of
truth, independent of both local gates, and they emit **one row per advisory per
lockfile**: 12 open alerts were 6 advisories × (`Cargo.lock` +
`fuzz/Cargo.lock`). `Cargo.toml:14` sets `exclude = ["fuzz"]`, so **no** local gate
ever reads `fuzz/Cargo.lock`. Before promising a fix, name the register the id
lives in (GHSA / RUSTSEC-under-audit / RUSTSEC-under-deny) and the lockfile each
gate actually reads, and dedupe by advisory id — never by row count:

| Source | Lockfiles read | Blind to |
|--------|----------------|----------|
| `cargo deny check advisories` (the **blocking** gate) | `Cargo.lock` | `informational` advisories (beyond `unmaintained = "workspace"`), GHSAs not adopted into RustSec, every id in `deny.toml` `ignore` |
| `cargo audit` (structured report; CI fails only on `.vulnerabilities.count`) | `Cargo.lock` | `informational` advisories — they land in `.warnings`, never in `.vulnerabilities` — plus GHSAs not adopted into RustSec and every id in `.cargo/audit.toml` `ignore` |
| Dependabot alerts | `Cargo.lock` + `fuzz/Cargo.lock` | nothing — but it closes only on a lockfile change |

```bash
gh api "/repos/{owner}/{repo}/dependabot/alerts?state=open" -q '.[0].security_advisory | keys'
# Keys FIRST: a wrong key silently yields empty columns and a register that looks
# bigger or smaller than it is.
gh api "/repos/{owner}/{repo}/dependabot/alerts?state=open" \
  -q '[.[].security_advisory.ghsa_id] | unique | length'    # advisories, not rows
gh api "/repos/{owner}/{repo}/dependabot/alerts?state=open" \
  -q '.[] | [.security_advisory.ghsa_id, .dependency.manifest_path, .security_advisory.severity] | @tsv'
```

**Two ways an advisory hides from a gate — neither means fixed.**
- *`informational` severity*: `lru` **RUSTSEC-2026-0002** is
  `informational = "unsound"` in the DB
  (`~/.cargo/advisory-dbs/<db>/crates/lru/RUSTSEC-2026-0002.md`:
  `aliases = ["GHSA-rhfx-m35p-ff5j"]`, `patched = [">= 0.16.3"]`,
  `unaffected = ["< 0.9.0"]`). cargo-audit routes severities of that class to
  `.warnings.unsound`, **not** `.vulnerabilities`, so it cannot redden the CI audit
  step (which fails only when `.vulnerabilities.count > 0`) and it is not reported by
  `cargo deny` either. Measured on the same lock, twice:
  ```bash
  cargo audit --no-fetch --json                      # in-repo: .cargo/audit.toml applies
  # rc=0, .vulnerabilities.list = []  BUT .warnings.unsound = 5, .warnings.unmaintained = 1
  cargo audit --no-fetch --json -f "$REPO/Cargo.lock" # from /tmp: ignores do NOT apply
  # rc=1, .vulnerabilities.list = 8 (rsa 2023-0071, rustls-webpki 2026-0049/0098/0099/0104,
  # h2 2026-0258, rustls 2026-0346/0349) and .warnings.unsound gains lru 2026-0002
  ```
  So a green `cargo audit` proves only "nothing un-ignored reached `.vulnerabilities`" —
  name the ignore list and the surviving `.warnings` count whenever you cite it. The
  audit.toml entry for 2026-0002 silences a *warning* line; it was never the thing
  keeping the step green.
- *Never adopted*: `libsql-sqlite3-parser` **CVE-2025-47736 / GHSA** has no
  `crates/libsql-sqlite3-parser` directory in the advisory DB (only `libsqlite3-sys`),
  so it fails **neither** local gate — Dependabot is the only observer.

"audit green + deny green + Dependabot alerts open" is therefore a **normal
state**, not a broken gate. Check (a) severity and (b) RustSec adoption before
concluding the gate is wrong.

**Local deny can lie.** A stale clone under `~/.cargo/advisory-dbs/` reports
"advisories ok" against a lock CI proves red; check its checkout date and
cross-check with `cargo audit` (DB at `~/.cargo/advisory-db`). Investigate a
leftover `db.lock` instead of deleting it (LESSON-047). Per-advisory facts:
`~/.cargo/advisory-db/crates/<pkg>/<ID>.md` or `rustsec.org/advisories/<ID>.html`
(`api.rustsec.io` / `crates.io/api/v1` return empty bodies to `curl`).

**Bump the floor, not the tip.**

```bash
cargo update -p <pkg> --precise <minimal-patched-from-[versions]-patched>
```

**Prove a bump by resolution, not by release notes.** A changelog claim that
version X drops an advisory is a hypothesis; the proof is a before/after
`cargo tree -e no-dev` diff. A scratch crate with the workspace's feature set
is enough:

```bash
cargo new /tmp/probe && cd /tmp/probe
# Cargo.toml: libsql = { version = "<candidate>", features = ["core","replication","remote","sync","tls"] }
cargo update && cargo tree -e no-dev
```

`libsql 0.10.0` failed exactly this test: it resolves `hyper-rustls 0.25.0 ->
rustls 0.22.4 -> rustls-webpki 0.102.8` and `libsql-sqlite3-parser 0.13.0` —
identical to `0.9.30`, so the bump closes nothing. When resolution shows the pin,
**name the pinning chain** (`libsql 0.9.30 -> hyper-rustls 0.25 -> rustls 0.22 ->
rustls-webpki 0.102.8`) and record the advisory as upstream-blocked instead of
opening a PR that changes nothing.

When a bump moves `getrandom` across a major, re-run the WASM gate locally with
its real flags: `cargo check -p do-memory-core --target wasm32-unknown-unknown
--no-default-features --features wasm32` with
`RUSTFLAGS='-D warnings --cfg getrandom_backend="wasm_js"'` (LESSON-048).

**Waivers.** Keep advisory PRs lockfile-only. Every ignore needs the same
evidence-backed entry in **both** `deny.toml` (`[advisories] ignore`) and
`.cargo/audit.toml`, stating: the advisory, the dependency chain, the fix range
(or why no in-place patch exists), and the mitigation verified against source —
not asserted. The one legitimate asymmetry is an `informational` id, which only
`cargo audit` can see (see *Two ways an advisory hides* above). Fix, don't
suppress: a waiver is only for chains this workspace cannot patch. An ignore
clears a **gate**, never an **alert**: PR #1174 made both local gates green and
closed zero Dependabot rows. A row closes only when a lockfile stops resolving the
affected version — and `fuzz/Cargo.lock` rows need a change Dependabot can see but
no local gate can.

## Code Review Output Format

```markdown
# Code Quality Report
Score: X/100 | Critical Issues: N | Warnings: M
- Structure: [Status] | Error Handling: [Status] | Testing: [Status]
1. [Issue] - File:line - Fix: [Recommendation]
```

## References

quality-dimensions.md, ADR-036 (Dependency Deduplication), ADR-032 (Disk Space)
