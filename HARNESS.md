# Harness Engineering

> Agent = Model + Harness. This is the harness map for rust-self-learning-memory.
> Based on: https://martinfowler.com/articles/harness-engineering.html

## Mental Model

Two axes:
- **Feedforward (guides):** What to read *before* writing code
- **Feedback (sensors):** What fires *after* writing code

Two modes:
- **Computational:** Deterministic (clippy, tests, deny) — always trust output
- **Inferential:** LLM-based (skill docs, agent context) — direction, not commands

## Feedforward Guides

### Inferential (read before coding)

| Guide | Path | Purpose |
|---|---|---|
| Agent contract | `AGENTS.md` | Coding conventions, change workflow, quality gates |
| Skills index | `.agents/SKILLS.md` | Available executable task knowledge |
| Harness overview | `HARNESS.md` (this file) | How guides and sensors connect |
| Clippy intent | `.clippy.toml` | Linting philosophy and allowed exceptions |
| Dependency rules | `deny.toml` | Crate layering: `memory-types → memory-core → memory-storage-* → memory-mcp/memory-cli` |
| Architecture | `plans/adr/` | Architecture Decision Records |
| Perf-PR review gate | `.agents/skills/perf-pr-guardrails/` | Verify perf claims against before-code, demand benchmark evidence, check heuristic branch coverage + commitlint before approving perf PRs |

### Computational (structural constraints)

| Constraint | File | Enforced by |
|---|---|---|
| Crate layering | `deny.toml` | `cargo deny check` |
| No unsafe code | `Cargo.toml` `[workspace.lints.rust]` | `rustc` |
| No `#[allow(...)]` | `Cargo.toml` `allow_attributes = "deny"` | `clippy` |
| Conventional commits | `commitlint.config.cjs` | `commitlint` pre-commit hook |

## Feedback Sensors

Signal sets are defined in `do-harness.toml` (pinned harness version `v0.1.0`,
the number tracked in `scripts/do-harness-version.txt`):
- `feedback`: `fmt`, `loc`, `check`, `commit-ci-tokens`, `agents-md-ids` (fast local edit loop)
- `verification`: `fmt`, `check`, `clippy`, `loc`, `deny`, `lockfile-sync`, `changelog-headings`, `shell-counters`, `lesson-distillation`, `commit-ci-tokens`, `agents-md-ids`, `merge-method`, `release-blockers`, `harness-compat`, `harness-doctrine`, `docs`, `test`, `csm-feature`, `ignored-tests`, `lint-suppressions`, `yaml-frontmatter`, `storage-backend-capabilities`, `quality-gates` (canonical local evidence boundary)

The same two probes are measured green against the published binaries up to 0.4.1,
which is what `validated_through` in `scripts/do-harness-version.txt` records. The
prose above is not a claim: `scripts/check-do-harness-compat.sh` compares the version
each routing doc states against the tracked reference and goes red on disagreement, so
raising the number is a deliberate change that has to carry a recorded compat run.

Local evidence boundary command:
```bash
do-harness verify --set verification --strict --record && do-harness status --set verification
```

### Computational (deterministic — always trust)

| Sensor | Trigger | Config | LLM Fix Hint |
|---|---|---|---|
| `cargo fmt --check` | feedback/verification | `do-harness.toml` | Run `cargo fmt --all` |
| `cargo clippy -D warnings` | verification + CI | `.clippy.toml` | Fix all warnings; see `.clippy.toml` for exceptions |
| `cargo deny check` | verification + CI | `deny.toml` | Check crate layering diagram in `Cargo.toml` comments |
| `check-lockfile-sync.sh` (`lockfile-sync`) | verification | `scripts/check-lockfile-sync.sh` | Stale `Cargo.lock`/`fuzz/Cargo.lock`: re-resolve that graph with `cargo metadata`; a 2-version `libsql` means the workspace drifted off `chaotic_semantic_memory`'s constraint |
| `check-changelog-headings.sh` (`changelog-headings`) | verification | `scripts/check-changelog-headings.sh` | Duplicate `## [x.y.z]` heading or a non-descending order: parse-changelog rejects the file, so merge the sections under one heading |
| `check-shell-counters.sh` (`shell-counters`) | verification | `scripts/shell-counters-baseline.txt` | New bare `((i++))`: write `i=$((i + 1))`; a 0-valued increment exits 1 and truncates the script under `set -e` |
| `check-lesson-distillation.sh` (`lesson-distillation`) | verification | `scripts/lesson-distillation-baseline.txt` | Cite the new lesson id from the file that carries the rule (AGENTS.md, a skill, or the enforcing script); never reuse an existing id |
| `check-commit-ci-tokens.sh` (`commit-ci-tokens`) | feedback + verification + commit-msg hook | `scripts/check-commit-ci-tokens.sh` | Reword the message so the bracketed token never appears, `git commit --amend -F <file>`, prove it with `git rev-parse 'HEAD^{tree}'` |
| `check-agents-md-ids.sh` (`agents-md-ids`) | feedback + verification | `scripts/check-agents-md-ids.sh` | Move the raw API id to `agent_docs/LESSONS.md` and cite the CI run number in `AGENTS.md`; never add `.codacy.yml` to silence it |
| `check-merge-method.sh` (`merge-method`) | verification | `scripts/check-merge-method.sh` | `merge-pr.sh` defaults to a method the base ruleset refuses, dropped its async stacked-PR path or its `$HEAD_SHA` pin, dropped the `--method merge` refusal, or requests `bypass_rules`: `main-protection` has `required_linear_history`, so keep `squash` and never bypass (LESSON-081) |
| `check-release-blockers.sh --self-test` (`release-blockers`) | verification | `scripts/check-release-blockers.sh` | Offline fixtures only: this proves the blocker classifier still fires. The live read runs inside `release-manager.sh ship`, which refuses while any pull request or issue is open; the auto-maintained `release-drift` issue is excluded by label so the gate cannot deadlock |
| `check-do-harness-compat.sh` (`harness-compat`) | verification (`--offline`) + CI (`harness-compat` job in `skill-evals.yml`) | `scripts/do-harness-version.txt` | Resolves the binary independently of PATH (a second `do-harness` on PATH is a violation), then checks the stated version against the tracked reference, the prose pin in every routing doc, the subcommands and flags this repo's own commands cite, `do-harness.toml` set/sensor parity, and that each script-backed sensor's path exists. Exit 2 means nothing was verified — an absent binary is never a pass. Fix: install a version in the validated range, or raise the pin in a PR that carries a recorded compat run |
| `check-do-harness-doctrine.sh` (`harness-doctrine`) | verification | `scripts/check-do-harness-doctrine.sh` | Runs the status contract against throwaway fixture sensors in a temp root, so a real green run is exercised rather than asserted: 0 = ran and passed, 1 = ran and failed, 2 = could not run; an unspawnable sensor must report `exit_code: null`, `--strict` must refuse weak evidence, and `--record` then `status --set` must report green. Repo-independent, so it also works on a fresh clone. Its pack input contract then asserts this workspace's own `do-harness.toml`: every sensor must declare `inputs`, and every declared glob must match at least one non-ignored file. A sensor without usable inputs records a **null** digest through the `input:git-index-and-nonignored-untracked` fallback, which makes `do-harness status --set <set>` report `stale (inputs_changed_or_unknown)` for every change — including a no-op on a clean tree — so the local evidence boundary line below could never confirm green. Fix: declare the sensor's real input surface, or `inputs = []` if it reads no file content |
| `check-ignored-tests.sh` (`ignored-tests`) | verification | `scripts/check-ignored-tests.sh` | An added `#[ignore]` needs its reason recorded; run the ignored set per feature (`--run-ignored only` force-runs only ignored tests) |
| `check-lint-suppressions.sh` (`lint-suppressions`) | verification | `scripts/lint-suppressions-baseline.txt` | New crate-root `#![allow]`: prefer a per-site `#[expect]` with a reason; run `--update` only after review |
| `check-yaml-frontmatter.sh` (`yaml-frontmatter`) | verification | `scripts/check-yaml-frontmatter.sh` | Give the skill/doc its required `---` frontmatter block |
| `check-storage-backend-capabilities.sh` (`storage-backend-capabilities`) | verification | `scripts/check-storage-backend-capabilities.sh --check` | Keep declared backend capabilities and the feature-gated implementations in sync |
| `quality-gates.sh` (`quality-gates`) | verification | `scripts/quality-gates.sh` | Runs the consolidated gate suite; doctests and LOC are skipped here because `docs` and `loc` own them |
| `cargo test --doc` / `doc` (`docs`) | verification + CI | `scripts/check-doctests.sh` | Wrap URLs in `<...>`, re-export public types, fix broken doctests |
| `cargo nextest run` (`test`) | verification + CI | `.config/nextest.toml` | Fix failing unit/integration tests before opening PR |
| `csm` feature tests (`csm-feature`) | verification + CI | `do-memory-core` | Fix failing CSM cascade/rerank/evidence tests |
| `check-loc.sh` (`loc`) | feedback/verification | `scripts/check-loc.sh` | Keep Rust source files <=500 LOC |
| `cargo mutants` | CI weekly (`mutants.yml`) | `Cargo.toml` metadata | If score < threshold, add targeted unit tests |
| `shellcheck` | pre-commit | `.pre-commit-config.yaml` | Fix shell script issues at severity=warning |
| `gitleaks` | CI (`security.yml`) | `.gitleaks.toml` | Remove secrets; use env vars |
| Architecture fitness | `tests/arch_fitness.rs` | dev-deps | HARNESS VIOLATION message includes fix instructions |
| Snapshot/behaviour | `tests/behaviour_harness.rs` | `insta` | Run `cargo insta review` to approve new baselines |

### Inferential (LLM-based — use for direction)

| Sensor | Path | Purpose |
|---|---|---|
| Codacy quality review | CI | Code quality suggestions |
| Codecov coverage | `codecov.yml`, CI | Coverage regression detection |
| AI slop detector | `.github/workflows/ai-slop-detector.yml` | Detect low-quality AI-generated code patterns |
| Perf-claim accuracy | `.agents/skills/perf-pr-guardrails/` (PR review swarm) | Catch overstated complexity/memory claims, dead-code tests, and commitlint failures on auto-generated perf PRs |

## Steering Loop

When any sensor fires **repeatedly** (>2 times in one sprint):
1. Identify the root cause category (maintainability / architecture / behaviour)
2. Update the corresponding **feedforward guide** to prevent recurrence
3. If no guide exists, create one in `.agents/skills/`
4. Document the update in `CHANGELOG.md`

Perf-claim inaccuracy counts as a sensor: a perf PR whose stated complexity/
memory impact does not hold against the pre-change code (e.g. PR #925, whose
O(N·M)→O(min(N,M)) claim was already true before) must be routed through
`.agents/skills/perf-pr-guardrails/` before approve/merge/close.

## Self-Correction Protocol for Agents

When a computational sensor fires:
1. Read the full error message — it includes a fix hint
2. Identify the sensor by its `do-harness.toml` name (`fmt`, `clippy`, `test`,
   `lockfile-sync`, …) — `arch` and `security` are diagnostics, not sensors
3. Apply the minimal fix (do not refactor unrelated code)
4. Re-run that sensor: `do-harness verify --only <sensor> --strict`
5. Only commit when the sensor is green

## Harness Check Script (a shim, not a second harness)

`scripts/harness-check.sh` exists so older agent docs and muscle memory keep working.
Every subcommand delegates, so there is exactly one definition of green:

```bash
bash scripts/harness-check.sh <fmt|clippy|deny|test|arch|security|all>
```

| Subcommand | What actually runs |
|---|---|
| `fmt`, `clippy`, `deny`, `test` | `do-harness verify --only <sensor> --strict` — the same sensors CI and the completion set use |
| `all` | `do-harness verify --set verification --strict --record` — the canonical completion contract |
| `arch` | `cargo nextest run -p e2e-tests --test arch_fitness` — a focused subset of the `test` sensor |
| `security` | `gitleaks detect --config .gitleaks.toml …` — a local diagnostic; the `Security` workflow owns the authoritative scan |

Three tiers exist, and only one is completion evidence:

| Tier | Command | Meaning |
|---|---|---|
| feedback | `do-harness verify --set feedback` | cheap edit loop; never completion |
| **verification** | `do-harness verify --set verification --strict --record` then `do-harness status --set verification` | **the completion contract** |
| diagnostics | `harness-check.sh arch`, `harness-check.sh security`, `./scripts/quality-gates.sh` | a focused look at one property; a pass here is not a merge verdict |

Exit codes are `0` the delegated check passed, `1` it ran and failed, `2` it could not run
at all (CLI or tool absent). The `2` is deliberate: this script used to print `deny: PASS`
with no `cargo-deny` installed and `security: PASS` with no `gitleaks`, and its `clippy`
was a private `cargo clippy --lib --tests` under 27 suppressions while the `clippy` sensor
runs `scripts/code-quality.sh clippy --workspace` under three. Two notions of green was the
bug (issue #1246) — `harness-check.sh clippy` could be green while CI's clippy was red, on
scope (`--lib` per package vs `--workspace`) and strictness at the same time. The
suppression lists were not merged; the weaker command was removed, so nothing can report it.

`scripts/test-harness-entrypoint.sh` pins this contract with stub binaries on a private
PATH (47 cases: the exact delegated argv, propagated failures, absent CLI, absent
gitleaks, the `arch` package name, and doc↔handler parity for the subcommand list). It
needs no Rust compile and runs in CI under `.github/workflows/skill-evals.yml`.
