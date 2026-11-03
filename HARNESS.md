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

Signal sets are defined in `do-harness.toml` (pinned harness version `v0.1.0`):
- `feedback`: `fmt`, `loc`, `check`, `commit-ci-tokens`, `agents-md-ids` (fast local edit loop)
- `verification`: `fmt`, `check`, `clippy`, `loc`, `deny`, `lockfile-sync`, `changelog-headings`, `shell-counters`, `lesson-distillation`, `commit-ci-tokens`, `agents-md-ids`, `merge-method`, `docs`, `test`, `csm-feature`, `ignored-tests`, `lint-suppressions`, `yaml-frontmatter`, `storage-backend-capabilities`, `quality-gates` (canonical local evidence boundary)

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
| `check-merge-method.sh` (`merge-method`) | verification | `scripts/check-merge-method.sh` | `merge-pr.sh` defaults to a method the base ruleset refuses, lost its async stacked-PR path or its `$HEAD_SHA` pin, or requests `bypass_rules`: `main-protection` has `required_linear_history`, so keep `squash` and never bypass (LESSON-081) |
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
2. Identify category: fmt / lint / test / arch / security
3. Apply the minimal fix (do not refactor unrelated code)
4. Re-run the specific sensor: `cargo clippy`, `cargo nextest run`, etc.
5. Only commit when the sensor is green

## Harness Check Script

A structured-output wrapper for the same categories, useful when you want
`HARNESS VIOLATION: [category] — fix hint` lines to parse:

```bash
bash scripts/harness-check.sh <fmt|clippy|deny|test|arch|security|all>
```

Each failure emits a `HARNESS VIOLATION:` prefix with an agent-parseable fix hint.

**Its `clippy` category is not the gate.** `harness-check.sh clippy` runs
`cargo clippy --lib --tests` under **27** `-A clippy::…` suppressions (cast losses,
`doc_markdown`, `manual_let_else`, `if_not_else`, …), while the `clippy` sensor in
`do-harness.toml` runs `scripts/code-quality.sh clippy --workspace`, which passes
`--workspace --tests` and suppresses only three lints (`expect_used`,
`uninlined_format_args`, `unwrap_used`). So `harness-check.sh clippy` can be green while
CI's clippy fails — on both scope (`--lib` per package vs `--workspace`) and strictness.
Use it for the parseable hints, never as completion evidence; the authoritative pair is:

```bash
./scripts/code-quality.sh clippy --workspace   # == sensor "clippy"; this is the gate
cargo fmt --all -- --check                     # == sensor "fmt"
```

`harness-check.sh` is not registered as a do-harness sensor, so `do-harness status` will
not reflect it. Collapsing the two suppression lists is deliberately out of scope here —
aligning them would change what the wrapper reports, which needs its own measured change.
