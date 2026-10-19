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
- `feedback`: `fmt`, `loc`, `check` (fast local edit loop)
- `verification`: `fmt`, `check`, `clippy`, `loc`, `deny`, `docs`, `test`, `csm-feature`, `ignored-tests`, `lint-suppressions`, `yaml-frontmatter`, `storage-backend-capabilities`, `quality-gates` (canonical local evidence boundary)

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

Run all sensors locally with structured output:

```bash
bash scripts/harness-check.sh <fmt|clippy|deny|test|arch|all>
```

Each failure emits a `HARNESS VIOLATION:` prefix with an agent-parseable fix hint.
