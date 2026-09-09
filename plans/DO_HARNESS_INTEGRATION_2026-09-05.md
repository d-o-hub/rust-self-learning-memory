# do-harness Integration Plan (2026-09-05)

## Purpose

`do-harness` (github.com/d-o-hub/do-harness) is a compiled Rust CLI that operationalizes the
exact "Agent = Model + Harness" model this repo already documents in `HARNESS.md`: feedforward
guides (`AGENTS.md`, `.agents/skills/`) plus feedback sensors (computational checks), with a
local libSQL state DB (`.do-harness/agent_state.db`) for sensor beats, fail-fast strikes, task
gating, traces, and skill-eval history. It does **not** read `HARNESS.md`; it reads
`do-harness.toml` (sensor definitions, hook staging) and `plans/invariants.json` (seed data).
`HARNESS.md` stays our conceptual map; do-harness adds the executable sensor + state layer.

## Install (already satisfied)

- **No GitHub releases, no tags, no prebuilt binaries.** `cargo install` from source is the only path.
- **Already installed**: `/home/do/.cargo/bin/do-harness` v0.1.0 (commit `3df96c2`, 2026-09-03),
  built from `/home/do/git/do-bookster/vendor/do-harness/crates/do-harness`. Upstream HEAD is
  `d7b8348` (2026-09-04) — a few commits ahead; no pinned versions exist upstream.
- Rebuild/upgrade: `cargo install --path <checkout>/crates/do-harness` (small dep tree: clap,
  tokio, libsql, serde, axum — builds in ~1-2 min). Do NOT build into this repo's shared `target/`.

## Command Surface (cheatsheet)

| Command | Purpose |
|---|---|
| `do-harness init [--language rust]` | Scaffold `do-harness.toml`, `plans/invariants.json`, skills, `scripts/check-loc.sh`; existing files untouched unless `--force` |
| `do-harness list` | List sensor names from `do-harness.toml` |
| `do-harness verify [--only <n>]... [--record] [--task <id>] [--format json]` | Run sensors; `--record` persists beats + error signatures |
| `do-harness doctor` | Binary resolution, git-hook health, state-DB migration skew |
| `do-harness hook install\|status\|uninstall` | Write pre-commit/pre-push/commit-msg hooks into `.git/hooks/` |
| `do-harness task add\|advance\|done\|fail` | Task gate: `done` refuses until `verify --record` passes the named sensor |
| `do-harness trace add` / `distill --from-trace` | Record resolved sensor fixes; distill heuristics into a skill |
| `do-harness metrics` | Sensor stats, strikes, eval pass-rate history |
| `do-harness errors list\|clear` / `audit-chain` | Fail-fast signatures / hash-chain integrity |

## Recommended Sensor Pack (`do-harness.toml` — mirrors existing gates, no duplication)

Keep sensors aligned with the repo's own commands (post-init edits):

```toml
language = "rust"
[hooks]
pre-commit = ["fmt", "loc"]   # cheap subset only
pre-push = []

[[sensors]] name = "fmt"     argv = ["cargo", "fmt", "--all", "--", "--check"]
[[sensors]] name = "clippy"  argv = ["cargo", "clippy", "--workspace", "--", "-D", "warnings"]
[[sensors]] name = "test"    argv = ["cargo", "nextest", "run", "--all"]   # repo standard
[[sensors]] name = "deny"    argv = ["cargo", "deny", "check"]
[[sensors]] name = "loc"     argv = ["bash", "scripts/check-loc.sh"]       # enforces ≤500 LOC invariant
```

## Proposed AGENTS.md Diff (exact markdown)

1. Add one row to the Quick Reference table (after the Quality Gates row):

```markdown
| **Agent Harness** | `do-harness` | `do-harness verify --record` / `do-harness doctor` |
```

2. Add a new section directly above `## Steering Loop`:

```markdown
## Dev Harness (do-harness)

Computational sensor runner + local agent state DB (`.do-harness/agent_state.db`, gitignored).
Sensors are defined in `do-harness.toml`; `HARNESS.md` maps them to guides.

- `do-harness verify --record` — run the sensor suite and persist beats (run before commit)
- `do-harness verify --only <sensor>` — targeted re-run after a failure
- `do-harness task done <id>` — task gate; refuses until its sensor passed via `verify --record`
- `do-harness doctor` — after upgrading the binary (checks hook/DB migration skew)
- Sensor fired? Fix the firing sensor first (`verify --only <name>`), then commit. Same
  self-correction protocol as HARNESS.md; beats recorded by `--record` feed `do-harness metrics`.
```

3. Change Workflow: insert as new step 10 (renumber `git status` to 11):
   `10. do-harness verify --record`

## Recommended Workflow Hook + Rationale

- **Hook point**: Change Workflow step (above) — NOT `scripts/quality-gates.sh` and NOT CI.
  - `quality-gates.sh` already owns coverage/pattern thresholds; layering do-harness there
    couples two failure domains. Keep verify as a separate, explicit step.
  - `scripts/harness-check.sh` remains; do-harness adds state (beats/strikes) it lacks.
- **Git hooks**: DEFER `do-harness hook install`. This repo's `.pre-commit-config.yaml` (pre-commit
  framework) and do-harness both write `.git/hooks/pre-commit` — last installer wins. Current clone
  has no hooks installed, so there is no clash today, but pick one owner before installing.
- **CI**: AGAINST for now. CI already runs fmt/clippy/nextest/deny directly; there are no prebuilt
  binaries, so CI would pay a build (~2 min) to re-run identical checks. Revisit only if JSON
  evidence artifacts (`verify --format json`) become a requirement.
- **Adoption steps (on a branch)**: run `do-harness init` → review diff → trim (see risks) →
  edit sensors to the pack above → `do-harness init-db && do-harness seed` → `do-harness verify`.

## Risks

1. **`init` scaffolds 6 skill dirs** (`harness`, `htn-planner`, `spike-runner`, `skill-distiller`,
   `event-modeler`, `skill-creator`) with their own `evals/`. `.agents/skills/skill-creator`
   **collides by name** with our existing skill (its `SKILL.md` is safe — per-file write-if-absent —
   but `scripts/init_skill.py` etc. would be injected). Post-init: delete unwanted dirs, keep at
   most `harness`, and check `./scripts/run-evals.sh --fixtures` still passes (Skill Evals CI).
2. **`.gitignore` pollution**: init appends `.agents/events/` — but this repo COMMITS those files
   (Steering Loop metrics events). Remove that line after init (keep `.do-harness/`).
3. **Sensor duplication**: `check-commitlint.sh` scaffold duplicates `commitlint.config.cjs` +
   pre-commit; drop the scaffold or the pre-commit commitlint hook — do not run both.
4. **Version drift**: no upstream tags/releases; installed binary comes from the do-bookster
   vendored copy and is already behind HEAD. Decide a source of truth (vendor here like do-bookster,
   or rebuild from upstream on demand) before depending on it.
5. **Vacuous pass**: if `do-harness.toml` lists zero sensors, `verify` exits 0 meaninglessly —
   keep the pack above populated.

## Smoke-Test Evidence (2026-09-05)

`--help`, `version`, `list`, `doctor`, `metrics` all green against installed binary; `init` ran
clean in `/tmp/do-harness-init-test` (artifact list above verified). No repo files were modified.
