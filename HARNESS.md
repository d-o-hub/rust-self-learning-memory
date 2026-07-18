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

### Computational (structural constraints)

| Constraint | File | Enforced by |
|---|---|---|
| Crate layering | `deny.toml` | `cargo deny check` |
| No unsafe code | `Cargo.toml` `[workspace.lints.rust]` | `rustc` |
| No `#[allow(...)]` | `Cargo.toml` `allow_attributes = "deny"` | `clippy` |
| Conventional commits | `commitlint.config.cjs` | `commitlint` pre-commit hook |

## Feedback Sensors

### Computational (deterministic — always trust)

| Sensor | Trigger | Config | LLM Fix Hint |
|---|---|---|---|
| `cargo fmt --check` | pre-commit | `.pre-commit-config.yaml` | Run `cargo fmt --all` |
| `cargo clippy -D warnings` | pre-commit + CI | `.clippy.toml` | Fix all warnings; see `.clippy.toml` for exceptions |
| `cargo deny check` | pre-commit + CI | `deny.toml` | Check crate layering diagram in `Cargo.toml` comments |
| `cargo nextest run` | CI (`ci.yml`) | `Cargo.toml` | Fix failing tests before opening PR |
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

## Steering Loop

When any sensor fires **repeatedly** (>2 times in one sprint):
1. Identify the root cause category (maintainability / architecture / behaviour)
2. Update the corresponding **feedforward guide** to prevent recurrence
3. If no guide exists, create one in `.agents/skills/`
4. Document the update in `CHANGELOG.md`

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
