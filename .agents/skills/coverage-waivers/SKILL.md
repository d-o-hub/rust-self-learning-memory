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
| Coverage runs **default features only** — no `csm`, no `--all-features` | `.github/workflows/coverage.yml` (`cargo llvm-cov --workspace ...`, no feature flags) |
| **No CI job enables `csm`**, so `#[cfg(feature = "csm")]` tests never execute in CI | grep all workflows for `--features csm`; see issue #1045 |
| Tests gated behind a feature that CI never builds are dead weight for coverage | the `#[cfg(feature = ...)]` attribute on the test module |
| Waivable classes were accepted here before | PR #1041 comment, PR #1042 comment |

## Classification

| Class | Detection | Action |
|---|---|---|
| **Coverable** | Any ordinary branch/error/helper line the tests skip | Add a test. Prefer **ungated** tests — if the code needs a feature, first check whether the line itself is feature-gated |
| **`tracing`/`log` field spans** | Multi-line `info!(`/`debug!(` field expressions (`outcome = %x.as_str()`, `count = y.len()`) | Fields evaluate only when a subscriber consumes the event; with no test subscriber they stay at zero. Waive with the cited line numbers |
| **Unreachable defensive arm** | Branch guarded by a prior exhaustive check (ID-set equality, duplicate check, validated config) | Waive; cite the guard that makes it unreachable (prefer converting to `let-else` with a comment) |
| **Feature-gated, CI-never-builds** | `#[cfg(feature = "csm")]` source paths | Waive in-thread **and** reference the pipeline gap (#1045); do not add more csm-only tests without also noting they run locally only |

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
3. <feature-gated> — csm path; tests exist and run locally via
   `cargo nextest run -p do-memory-core --features csm`, but no CI job builds
   `csm` (see #1045).

**Local verification on head:** <commands + counts>.
```

Never claim a line is covered without a test that runs in the default-feature pipeline; never waive a line a plain test can reach.

## Anti-Patterns

- ❌ "CI green, ready to merge" while an unanswered Codecov comment stands — the repository treats bot findings as actionable.
- ❌ Adding csm-gated tests to close a patch-coverage gap (they do not run in the coverage pipeline).
- ❌ Chasing 100% by testing defensive arms with contrived inputs.
- ❌ Waiving without citing line numbers and the guard/subscriber reason.

## See Also

- `pr-readiness` skill — the full merge-readiness procedure this feeds into.
- Issue #1045 — the csm CI gap.
