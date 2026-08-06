---
name: perf-pr-guardrails
description: "Review performance-optimization PRs (especially auto-generated ones like Jules bot PRs) before approving, merging, or closing. Verifies complexity/memory claims against the pre-change code, demands benchmark or measurement evidence, checks branch coverage of heuristics (longer-first, asymmetric, unicode/multibyte), rejects tests written only for dead defensive branches, and enforces commitlint compliance. Use when reviewing perf PRs, judging close-vs-merge decisions, or adding guard rails after a perf PR review found overstated claims."
---

# Perf-PR Guard Rails

Prevent low-impact, overstated performance PRs from churning the codebase.
Triggered by any PR whose title/body claims speed, memory, allocation, or
complexity improvements — and **especially** auto-generated perf PRs (Jules /
`google-labs-jules[bot]`).

## When to Use

- Reviewing a `perf(...)` or optimization PR before approving/merging
- Deciding whether a perf PR has enough impact to keep vs. close
- Auditing a perf PR's PR-body + doc-comment claims
- Adding guard rails after a perf-PR review found inflated claims (see
  `plans/GOAP_PR925_REVIEW_AND_PERF_PR_GUARDRAILS_2026-08-06.md`)

## The Checklist (all 7 items)

### 1. Baseline diff — read the OLD code first
Before believing any claim, read the pre-change implementation and compute the
*true* asymptotic before and after.

- **Red flag**: "reduces space from O(N·M) to O(min(N,M))" when the old code
  already used a rolling buffer (2 rows) = already O(min(N,M)). That is a
  constant-factor change, not an asymptotic one. (This exact false claim
  shipped in PR #925.)

### 2. Claims audit
Every complexity / allocation / memory claim in the PR body **and** in the new
doc comments must be verified against the actual diff.

- Red flags: "eliminates O(M) swaps" (a `std::mem::swap` of two `Vec`s is O(1));
  "significant savings" with no measurement; any claim about the *old* code
  that the old code's own comments contradict.

### 3. Benchmark evidence (or an explicit disclaimer)
- Require a benchmark/measurement of the **changed path**, or an explicit
  "not benchmarked" statement in the PR body.
- A passing generic benchmark suite that does not touch the changed code is
  **not** evidence.
- If no benchmark exists, the honest impact estimate must come from a
  call-site trace (see #7), not from the diff alone.

### 4. Branch coverage of heuristics
Any short/long, min/max, or ordering heuristic must be tested on **both**
branches:

- first-argument-longer **and** first-argument-shorter
- asymmetric lengths (both directions)
- unicode/multibyte inputs where **byte length != char count** (e.g. `"ééé"`
  is 3 chars / 6 bytes) — a byte-length heuristic silently picks the wrong
  string to buffer, invalidating "O(min)" space claims

### 5. Differential tests for DP / algorithm rewrites
Rolling-buffer → single-row DP rewrites are classic off-by-one /
diagonal-overwrite bug territory. Require a test that cross-checks against a
**naive reference implementation** (e.g. full-matrix Levenshtein) over empty,
asymmetric, and unicode inputs. "It passes the existing tests" is insufficient
if the existing tests never hit the changed branch.

### 6. No tests for dead defensive branches
- Defensive `if x.is_empty()` guards that are unreachable through the public
  path (because a caller already filters empties) are **dead code**.
- Do not add tests for dead code to chase Codecov — prefer **removing** the
  redundant guard, or converting it to a `debug_assert!`.
- Codecov patch % can be gamed with dead-branch tests; a low-but-real number
  on reachable branches beats a perfect number with fabricated coverage.

### 7. Commit hygiene
- Conventional commit type, header ≤ 100 chars (commitlint
  `header-max-length`); the *only* CI check a bot PR fails is usually this.
- Squash auto-generated commit chains (5 commits / 71 lines with 3 duplicated
  messages is a smell).
- One logical change per PR; no mixed code+docs+meta sprawl.

## Real-World Impact Trace

Before accepting "significant impact", trace the call sites:

```bash
# find callers
rg -n "similarity_score|string_similarity|edit_distance" memory-core/src --type rust
```

Estimate frequency × input size. For pattern similarity, inputs are short tool
names / error types over small pattern sets — sub-microsecond allocation
savings per call. Match the claim to the trace: "micro-optimization" is honest,
"significant memory allocation savings" is not.

## Close-vs-Merge Decision

| Finding | Verdict |
|---|---|
| Correct + honest claims + measured or traced impact | Merge |
| Correct but near-zero impact, claims overstated | Recommend **close**, optionally offer a corrected PR |
| Correctness bug or regression risk | Close / request fixes (never merge) |

When closing, post the full review as the closing comment (evidence, not
opinion) and reference this skill + the plans doc so the lessons persist.

## Case Study: PR #925 (2026-08-06)

A Jules auto-generated PR claimed O(N·M)→O(min(N,M)) and O(N+M)→O(min(N,M))
reductions in `memory-core/src/patterns/similarity.rs`. The old code was
**already** O(min(N,M)) (2-row rolling buffer); `mem::swap` was O(1); the
byte-length heuristic misorders multibyte strings; the added tests covered
unreachable `len1 == 0` guards; and 2 of 5 commits had 113-char headers
(failing Commit Message Lint — the only red check). Correctness was verified,
impact was near-zero, so the PR was closed and a corrected PR (char-count
heuristic, differential + unicode + longer-first tests, honest docs, one
squashed commit) was opened instead.
