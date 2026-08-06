# GOAP PR #925 Review — Perf-PR Guard Rails (2026-08-06)

## Goal

Review and resolve PR #925 (`perf(patterns): optimize edit distance and string
similarity using single-row DP and streamed character iteration`): address PR
comments, fix failing CI, review/roast the PR, close it if it has no impact,
and add guard rails to prevent the same failure modes.

## Outcome

- **PR #925 closed** as near-zero impact with the full review posted as the
  closing comment. The optimization was *correct* but its advertised impact did
  not survive contact with the code.
- **New PR opened** with the corrected implementation (char-count heuristic,
  honest docs, differential + unicode + longer-first tests, squashed ≤100-char
  commit).
- **Guard rails added**: new `perf-pr-guardrails` skill, HARNESS.md sensor +
  guide rows, this plan, and a steering-loop event log.

## Evidence (swarm review)

Two independent reviewers + call-site analysis + web research on Levenshtein
best practices. Consensus findings:

1. **False headline claim**: "Reduces space complexity from O(N·M) to
   O(min(N,M))" — the old code was **already O(min(N,M))** (2-row rolling
   buffer). Constant-factor win (1 fewer `Vec`), not asymptotic.
2. **"Eliminates O(M) std::mem::swap operations"** — `mem::swap` of two `Vec`s
   is O(1); the old comment even called them "O(1) row transitions".
3. **Byte-length heuristic misorders multibyte strings**: `s1.len()` compares
   bytes, the DP runs over chars; `"ééé"` (3 chars / 6 bytes) vs `"xyzw"`
   (4 chars / 4 bytes) collects the longer-in-chars string. Correctness safe
   (Levenshtein symmetric), space claim only strict for ASCII.
4. **Tests for dead, self-redundant defensive code**: `len1 == 0` guards are
   provably redundant (single-row DP handles the empty shorter side
   naturally: `dp=[0]`, `dp[0]` accumulates `len2`), and the `is_empty()`
   guards in `string_similarity`/`sequence_similarity` make them unreachable
   via the public path. The one reachable-but-uncovered branch (the `else`
   branch when `s1` is longer) was exactly the Codecov missing line (97.56%).
5. **No benchmark evidence**: the perf suite does not measure
   `similarity.rs`; "significant memory allocation savings" unverified.
6. **Impact trace**: `Pattern::similarity_score` → `PatternClusterer::
   deduplicate_patterns` / `find_similar_patterns` — short strings (tool
   names, error types, conditions) over small pattern sets. Sub-microsecond
   savings per call → **near-zero** codebase impact.
7. **CI**: only failure was Commit Message Lint — 2 of 5 commits had 113-char
   headers (limit 100); 5 commits / 71 lines, 3 duplicated messages.

## Corrected New PR (perf/pattern-similarity-single-row)

- Single-row DP kept (verified correct: `pre_dp` diagonal handling).
- `string_similarity` now picks short/long by **char count**
  (`chars().count()`), not byte length → strict O(min) char storage for UTF-8.
- Redundant `len1 == 0` guards removed; empty-side behavior locked in by
  behavioral tests.
- Tests: differential vs. naive full-matrix reference (empty, asymmetric,
  unicode, classic `kitten`/`sitting`), longer-first (else-branch) coverage,
  unicode byte-len inversion cases, symmetry assertions.
- Honest doc comments + honest PR body (constant-factor, not asymptotic).
- One commit, header well under commitlint's 100 chars.

## Guard Rails

- `.agents/skills/perf-pr-guardrails/SKILL.md` — 7-point perf-PR review
  checklist (baseline diff, claims audit, benchmark evidence, heuristic branch
  coverage, differential tests, no dead-code tests, commit hygiene) +
  close-vs-merge matrix + PR #925 case study.
- `HARNESS.md` — added `perf-pr-guardrails` to the inferential feedforward
  guides and the inferential feedback sensors; perf-claim inaccuracy now
  counts toward the steering-loop sensor.
- `.agents/events/2026/08/06/perf-claims-overstated-925-*.json` — structured
  sensor event.

## Follow-ups

- Monitor new-PR CI to green (quick-check, tests, skill schema + evals).
- If the corrected PR merges, `perf-pr-guardrails` should be invoked on any
  future perf PR (including future Jules perf PRs) before merge.
