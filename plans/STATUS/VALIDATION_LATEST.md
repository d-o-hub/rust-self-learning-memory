# Validation Latest — 2026-11-15 (plans compaction + harness pin)

**Workspace**: `0.1.50` (development) · **Released tag**: `v0.1.49` (`16d92d22`) ·
**Main baseline**: `fa3c561f`
**Scope**: plan compaction, single agent contract, do-harness pin `0.4.6`, R-F backlog truth.
**Previous full record**: [`archive/2026/VALIDATION_2026-11-01.md`](archive/2026/VALIDATION_2026-11-01.md)

## Verified in this change

| Check | Command | Result |
|-------|---------|--------|
| Plan hygiene | `./scripts/validate-plans.sh --all` | ✅ `dated_root=0`, `cargo=0.1.50 tag=0.1.49`, tracker drift `prs=0 issues=0 mismatches=0` |
| Docs integrity | `./scripts/check-docs-integrity.sh` | ✅ links, script references, version consistency, fail-closed contract all pass |
| Gate contract | `./scripts/validate-gate-contract.sh --ci-parity` | ✅ ruleset `9591004` requires `CI / Required`; local coverage floor 70% |
| Release cadence | `./scripts/check-release-drift.sh` | ✅ `severity=clean reason=within_cadence` (6 commits, 0 feat, 2 fix, 3 days) |
| Harness pin | `./scripts/check-do-harness-compat.sh --offline --expect-version 0.4.6` | ✅ `Compatibility Passed (0.4.6)`; binary clean, prose pins agree, flags/sets valid |
| Harness doctrine | `./scripts/check-do-harness-doctrine.sh --expect-version 0.4.6` | ✅ exit tiers `0/1/2`, `.unrunnable` + `exit_code:null`, strict refusal, pack-input contract |
| Harness contract fixtures | `./scripts/test-do-harness-compat.sh` | ✅ **31 cases** passed (mutations still turn the probes red) |
| Feature spike schema | `./scripts/validate-feature-spike.sh plans/STATUS/spikes/*.json` | ✅ 17/17 artifacts valid (R-F backlog now `NO-GO`, schema-valid) |
| Feedback sensors | `do-harness verify --set feedback` | ✅ `fmt`, `loc`, `check`, `commit-ci-tokens`, `agents-md-ids` |
| Fast verification sensors | `do-harness verify --only <s> --strict` for `changelog-headings`, `shell-counters`, `lesson-distillation`, `commit-ci-tokens`, `agents-md-ids`, `merge-method`, `release-blockers`, `ignored-tests`, `lint-suppressions`, `yaml-frontmatter`, `storage-backend-capabilities` | ✅ all pass |
| Ignored-test inventory | `./scripts/check-ignored-tests.sh` | ✅ `160` attributes within ceiling `200`; inventory matches `159` entries |
| Skill routes | `./scripts/validate-skill-routes.sh` | ✅ 44 defined / 44 routed |
| Skill eval schema | `./scripts/run-evals.sh --fixtures` | ✅ all schema fixtures passed |
| YAML / shell syntax | `yamllint -c .yamllint.yml --strict .pre-commit-config.yaml`; `bash -n` on changed scripts | ✅ clean |
| Agent contract deletion | `git ls-files .jules GEMINI.md` | ✅ empty; file-structure and root-doc allowlists updated |

## Release-prep gate (intentionally not green)

`./scripts/verify-release-state.sh --check-unreleased` exits 1 during a development bump
because it expects the `Released Version` docs and a CHANGELOG section to name the
workspace version (`0.1.50`). That is the release-prep step, not a code gap: `v0.1.49`
is the released tag and `0.1.50` has no release. Run it after the release-prep PR lands,
immediately before `release-manager.sh ship`.

## Prior release evidence (pointer)

| Artifact | Evidence |
|----------|----------|
| `v0.1.49` | tag `16d92d22`; GitHub Release 2026-11-12T17:23:40Z with 37 assets; four registered crates at `0.1.49` on crates.io |
| Released-CLI dogfood | asset digest matched; `--version` = `0.1.49`; `pattern list` total equals `storage stats` total in a fresh process |
| `v0.1.48` / `v0.1.47` / `v0.1.46` | released and dogfooded in the same chain; publish guard fix #1179 held for all three |
| ADR-079 stage 4 | probe #1229: failing `CI / Required` → `mergeStateStatus=BLOCKED`, `merge-pr.sh` exit 1 |
| Nightly | isolated Turso native suite 20/20, 0 skipped on the last recorded scheduled run |

Full historical rows: [`archive/2026/`](archive/2026/).

## Method

- Claims come from the command output above, not from tracker text.
- Every plan statement names its evidence file or PR; dated history is archived, not
  restated as current.
- A red release-prep gate is reported as red, with the reason and the step that clears it.
- The harness pin is only "measured" when the binary, tracked file, prose and CI matrix
  agree **and** both probes pass against the pinned version.
