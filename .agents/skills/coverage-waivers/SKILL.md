---
name: coverage-waivers
description: "Classify and answer Codecov patch-coverage findings on a PR: which uncovered lines need tests, which are structurally uncoverable, and how to reply with per-file evidence. Use when a PR shows missing-lines/low-patch Codecov comments, when deciding whether to add tests before merge, or when writing a coverage waiver."
---

# Coverage Waivers

Answer Codecov patch-coverage findings with evidence instead of blanket dismissals — and know which classes this repository's pipeline cannot cover at all.

## When to Use

- A Codecov conversation comment reports missing lines / low patch % on a PR.
- `./scripts/check-pr-readiness.sh` prints `⚠️ Codecov reports missing coverage`.
- Deciding between adding tests, waiving with evidence, or filing a pipeline gap.

## Pipeline Facts (verify before classifying)

These decide whether a line *can* be covered by CI:

| Fact | Where to verify |
|---|---|
| Coverage instruments **one fixed feature set** | `coverage.yml:98` runs `cargo llvm-cov --workspace --exclude do-memory-benches --exclude do-memory-examples --exclude test-utils --features do-memory-mcp/oauth --lcov ...` — that single `--features` is the whole measured universe |
| `csm` **tests do run in CI, but are not instrumented** | `ci.yml:206-212` (job "CSM Feature Tests", added for #1045) runs `cargo nextest run --profile ci -p do-memory-core --features csm`. So a csm line is *proven to work* by a green CI job yet still reports 0% to Codecov — cite the job, then explain the measurement gap |
| Same gap for the Turso optional features | `do-memory-storage-turso` gates real code on `#[cfg(feature = "turso_multi_dimension")]` / `"compression"`, which `coverage.yml` never enables. Nightly runs them (`--features turso_multi_dimension … --run-ignored all`), coverage cannot see them |
| `codecov/patch` can be **green while the comment warns** | read the check-run title (`gh api .../commits/<sha>/check-runs` → `codecov/patch`, e.g. `62.98% of diff hit (target 50.00%)`). Above target ⇒ conclusion `success`, so the `:x:` comment is a thread to answer with evidence, **not** a CI blocker |
| "Needs a live Turso instance" is **not** a waiver reason by itself | the crate has non-ignored local tests: `TursoStorage::new("file:…")` + `libsql::Builder::new_local` in `tests/pool_integration_test.rs` and `tests/capacity_enforcement_test.rs`. Count `#[ignore]` before claiming a line is unreachable in CI |
| Waivable classes were accepted here before | PR #1041 comment, PR #1042 comment |
| A red **Code Coverage** job is often the *formatting* gate, not coverage | `e2e-tests/tests/quality_gates.rs::quality_gate_formatting` runs `cargo fmt --check` in the same suite — read the failing test name before the percentage (LESSON-039) |

## Classification

| Class | Detection | Action |
|---|---|---|
| **Coverable** | Any ordinary branch/error/helper line the tests skip | Add a test. Prefer **ungated** tests — if the code needs a feature, first check whether the line itself is feature-gated |
| **`tracing`/`log` field spans** | Multi-line `info!(`/`debug!(` field expressions (`outcome = %x.as_str()`, `count = y.len()`) | Fields evaluate only when a subscriber consumes the event; with no test subscriber they stay at zero. Waive with the cited line numbers |
| **Unreachable defensive arm** | Branch guarded by a prior exhaustive check (ID-set equality, duplicate check, validated config) | Waive; cite the guard that makes it unreachable (prefer converting to `let-else` with a comment) |
| **Feature-gated, CI-instruments-elsewhere** | `#[cfg(feature = "csm")]`, `#[cfg(feature = "turso_multi_dimension")]`, `#[cfg(feature = "compression")]` source paths | Waive in-thread by naming the feature **and** the coverage invocation that omits it (`coverage.yml:98`), then cite the CI job that does run it (`ci.yml:206-212` for csm; nightly for the Turso features). Do not add more gated tests to chase the percentage |

Rule of thumb: if a plain, ungated test can execute the line in the default-feature build, it is not waivable — write the test.

## Local Reproduction (no extra CI cycle)

Re-slice the data you already have instead of re-running the whole suite:

```bash
# Instrumented run for the crate you changed (default features, like CI)
cargo llvm-cov nextest -p <crate> --lib <module-filter>

# Per-line zero hits (lcov is the least surprising view)
cargo llvm-cov report --lcov -p <crate> | grep -E '^(SF|DA:<line>,0)'

# Or re-slice the JSON report from the same profdata (segments = [line, col, count, has, ...])
cargo llvm-cov report --json -p <crate> > /tmp/cov.json
```

Do **not** re-run `cargo llvm-cov` just to read a report — `report` re-slices existing profdata in seconds.

## Procedure

1. Read the Codecov comment; list every file with missing lines.
2. Classify each per the table.
3. Write tests for every coverable line — ungated, default-feature-runnable.
4. Push the tests; re-check the patch report on the new head.
5. Reply on the Codecov thread using the template below, then re-run the readiness gate.

## Reply Template

```markdown
Coverage follow-up for this report; the gap split is material, so documenting it precisely.

**Covered since this report** (<sha>):
- <file> — <what the new tests execute>

**Waived with evidence (not coverable in the measured pipeline):**
1. <file> — `tracing` field spans at lines X–Y: fields evaluate only with a
   subscriber; the branch itself is covered (asserted via telemetry deltas).
2. <file> — unreachable defensive arm after the <guard>; cite <guard>.
3. <feature-gated> — `#[cfg(feature = "<csm|turso_multi_dimension|compression>")]`
   path. It is exercised in CI (`ci.yml:206-212` runs `--features csm`; nightly runs
   the Turso features) but `coverage.yml:98` instruments only
   `--features do-memory-mcp/oauth`, so Codecov cannot see it either way.

**Local verification on head:** <commands + counts>.
```

Never claim a line is covered without a test that runs under `coverage.yml`'s fixed feature set; never waive a line a plain, ungated test can reach.

## Anti-Patterns

- ❌ "CI green, ready to merge" while an unanswered Codecov comment stands — the repository treats bot findings as actionable.
- ❌ Adding feature-gated tests to close a patch-coverage gap (`csm`, `turso_multi_dimension`, `compression`) — the coverage job instruments only `do-memory-mcp/oauth`, so they cannot move the patch number.
- ❌ Waiving a Turso line as "needs a live instance" when `tests/pool_integration_test.rs`-style local `new_local` tests already run it in CI.
- ❌ Chasing 100% by testing defensive arms with contrived inputs.
- ❌ Waiving without citing line numbers and the guard/subscriber reason.

## See Also

- `pr-readiness` skill — the full merge-readiness procedure this feeds into.
- Issue #1045 — the csm gap, now **closed for tests** by `ci.yml`'s CSM Feature Tests job; what remains open is that `coverage.yml` does not instrument csm.
- `nightly-tests.yml` — where the `turso_multi_dimension` paths actually execute.
