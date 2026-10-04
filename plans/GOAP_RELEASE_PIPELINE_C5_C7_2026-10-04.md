# GOAP: Release-pipeline C5–C7 (issue #1109)

- **Date**: 2026-10-04
- **Orchestrator**: goap-agent skill (ANALYZE → DECOMPOSE → STRATEGIZE → COORDINATE → EXECUTE → SYNTHESIZE)
- **Scope**: issue #1109 §3 C5 (Pages boundary + "Verify a release" page), C6 (slim the
  release path), C7 (secret/permission hygiene). C1–C4 landed in #1123/#1121.
- **Status**: executing (parallel slices).

## ANALYZE — constraints from the affected ADRs

- **ADR-072 §4** (one release authority): tag-only releases; the tag must be `v<workspace version>`;
  `release.yml` preflight already enforces tag↔Cargo.toml and tag↔`origin/main` ancestry.
- **ADR-079 §6** (truthful triggers): release stays tag-only; publication must be explicit,
  fail closed, with a safe dry-run mode.
- **ADR-079 §3** (actor parity) and the live ruleset `9591004`: required contexts are exactly
  `Codacy Static Code Analysis` + `CI / Required`, and `CI / Required` aggregates only the
  `ci.yml` jobs (`fast-gate, commitlint, test, csm-tests, mcp-build, multi-platform, quality-gates`).
  `release.yml`'s PR-run jobs (`preflight`, `plan`, `build-*`) are **not** required.
- **ADR-045** (publishing best practices): `permissions` least-privilege per job, SHA-pinned
  actions, `persist-credentials: false` where the token is not needed.
- **ADR-058**: tag pushes are what close drift issues — do not change the tag trigger semantics
  beyond tightening the glob.

## DECOMPOSE — slices, ownership, contracts

| Slice | Owner | Files (exclusive) | Deliverable |
|---|---|---|---|
| W1 book/docs | agent `BookVerifyPage` | `book/src/verify-a-release.md` (new), `book/src/SUMMARY.md`, `SECURITY.md`, `.github/SECURITY.md` (pointer only) | "Verify a release" page: checksums (`.sha256`), `gh attestation verify` (build provenance + `--predicate-type` SBOM), `gh release verify`, minimum CLI version, links; SECURITY.md gains the same commands |
| W2 CI surface | agent `ReleaseCiHygiene` | `.github/workflows/release.yml`, `.github/workflows/publish-crates.yml`, `scripts/release-manager.sh`, `.agents/skills/release-guard/SKILL.md`, `agent_docs/github_actions_patterns.md` | C6 trigger slimming + C7 hygiene + Pages-decoupling comment + release-notes link to the book page + `--skip-local-tests` CI-parity wording |
| W3 verify | orchestrator | — | validators, actionlint/yamllint, mdbook build, adversarial review, PR |

**Contracts (fixed now, do not renegotiate in-flight):**

1. Book page path: `book/src/verify-a-release.md`; published URL
   `https://d-o-hub.github.io/rust-self-learning-memory/verify-a-release.html`.
2. SUMMARY entry: `- [Verify a Release](./verify-a-release.md)` inserted directly after
   `- [Getting Started](./getting-started.md)`.
3. `release.yml` release-notes prefix: prepend one line linking the published page to
   `$RUNNER_TEMP/notes.txt` right before it is written, keeping the file-based body handling.
4. `release.yml` PR trigger: keep `pull_request` but gate it to
   `paths: ['.github/workflows/release.yml', 'dist-workspace.toml']`, with a comment that the
   resulting checks are non-required (path-filtered workflows leave checks pending only for
   *required* contexts — GitHub docs, "Skipping workflow runs").
5. `publish-crates.yml`: all four checkouts get `persist-credentials: false`.
6. No change to required-check names, tag semantics, or the preflight checks.

## STRATEGIZE

- W1 and W2 are file-disjoint → run in parallel (one agent each), then a single verification
  phase (validators + CI + adversarial review) by the orchestrator.
- W3 includes a real doc build (`mdbook build`) plus the PR's `pages.yml` PR run as CI proof.

## Decisions

- **Versioned docs (C5 optional)**: deferred. A per-tag `vX.Y.Z/` snapshot needs a book build on
  tag pushes plus artifact merging in `pages.yml`; no acceptance criterion requires it, and the
  "verify a release" page covers the user-facing need. Recorded as a follow-up candidate.
- **Tag glob tightening**: `push.tags: ['**[0-9]+.[0-9]+.[0-9]+*']` (dist default) does not
  require the `v` prefix that ADR-072 and `release-manager.sh` mandate, so a stray `0.1.46` tag
  runs the whole release path only to fail preflight. Tighten to `'v[0-9]+.[0-9]+.[0-9]+*'` with
  a comment (prereleases like `v0.1.44-rc.1` still match).
- **C7 permissions**: `release.yml` already keeps workflow-level `contents: read` and raises
  write scopes inside `host` (contents/id-token/attestations) and `dispatch-publish` (actions);
  no change needed — the PR records this as evidence instead of churn.

## VERIFY (acceptance mapping)

- §4 "SECURITY.md + book 'Verify a release' page describe the attestation and checksum
  verification commands" → W1 (+ local `mdbook build` + CI `pages.yml` PR run).
- §4 "release.yml no longer runs a full `dist plan` on unrelated PRs, or the trigger is
  explicitly justified in a comment" → W2 (both: `paths:` gate **and** comment).
- §4 "Pages still deploys only from `book/**` main pushes; no release job depends on it" → W2
  comment in `release.yml` + unchanged `pages.yml` triggers (verified by reading the file).
- C7 → evidence + `persist-credentials: false` on the publish path; validator run.
- All: `test-release-workflow.sh`, `validate-gate-contract.sh`, `validate-plans.sh`, `check-loc.sh`,
  `run-evals.sh --fixtures`, `actionlint`, `yamllint`, `mdbook build`.
