---
name: harness
description: "Map the harness-engineering feedforward guides and feedback sensors, and run the self-correction protocol when a computational sensor fires. Use when a sensor fails (do-harness verify), before making code changes, or when setting up agent context for a new task. Triggers: harness, sensor fire, self-correction, harness doctor."
license: MIT
metadata:
  version: "0.1.0"
  tags: harness sensors verify self-correction
---

# Harness Skill

## What Is the Harness
Agent = Model + Harness. The harness is the system of feedforward guides (what
to do before coding) and feedback sensors (what catches violations after
coding).

- **Feedforward**: `AGENTS.md` and `.agents/skills/` — context, constraints,
  conventions read before coding.
- **Feedback**: `do-harness verify` — automated checks that fire after code
  changes. Computational output strictly supersedes LLM self-assessment.

## Feedback Sensors
Pinned compatible harness version: `v0.1.0`. Signal sets defined in `do-harness.toml`:

| Signal Set / Mode | Command | Stage |
|-------------------|---------|-------|
| feedback | `do-harness verify --set feedback` | cheap edit loop (`fmt`, `loc`, `check`, `commit-ci-tokens`, `agents-md-ids`) |
| verification boundary | `do-harness verify --set verification --strict --record` | pre-completion contract (all quality gates) |
| verification status | `do-harness status --set verification` | confirm green evidence boundary |
| individual | `do-harness verify --only <name>` | targeted re-runs |
| diagnostics | `bash scripts/harness-check.sh arch` / `… security` | focused look at one property; never completion evidence |
| harness binary | `./scripts/check-do-harness-compat.sh --offline` | which binary resolves, prose pin vs `scripts/do-harness-version.txt`, documented flags, sensor/set parity |
| harness status contract | `./scripts/check-do-harness-doctrine.sh` | exit-code doctrine and JSON fields, exercised on fixture sensors in a temp root; then this workspace's own pack: every sensor declares a non-empty `inputs` list (`inputs = []` records a null digest like omitting it), every declared glob matches a file, and a sensor claiming `*.rs` also claims every file those sources embed with `include_str!`/`include_bytes!` — including the `harness-doctrine` sensor itself, whose rule reads those sources |

`scripts/harness-check.sh` is a shim, not a second harness: `fmt|clippy|deny|test`
delegate to those sensors with `--strict`, `all` runs the canonical verification set, and
`arch`/`security` are labelled diagnostics. Exit `2` means nothing was checked (the CLI or
the tool is absent) — never read it as a pass. It used to run its own weaker copies of the
gates (a private `--lib --tests` clippy with 27 suppressions, `PASS` for skipped
`cargo-deny`/`gitleaks`), which is how an agent could report CI readiness against a red
CI; that is issue #1246 and `scripts/test-harness-entrypoint.sh` pins the contract now.

Sensors are defined in `do-harness.toml`; run `do-harness list` to see them.

The version above is not a guess and not a hope: `scripts/do-harness-version.txt`
carries the pin plus the newest release the probes have actually been measured green
against, and the `harness-compat` sensor goes red the moment the installed binary, that
file, and any routing doc disagree. CI re-runs the same two probes against published
binaries (`.github/workflows/skill-evals.yml`, `harness-compat` job), each
checksum-verified from the release's `checksums.txt`, and `scripts/test-do-harness-compat.sh`
proves with 18 fixture mutations that neither probe can pass vacuously. Upgrading the
CLI is therefore a deliberate change: run the probe against the new binary, record the
result, and only then raise the number.

## Self-Correction Protocol
1. Read the full error output — it includes the failing check.
2. Classify the failure: formatting / compile / lint / test / config.
3. Apply the minimal fix — do not refactor unrelated code.
4. Re-run the specific sensor (`do-harness verify --only <name>`).
5. Proceed only when the sensor is green.

## Fail-Fast Policy
If the same subtask fails a sensor 3 consecutive times, halt, record the error
signature in `.do-harness/agent_state.db`, and surface a diagnostic.

## New-Repo Adoption (Dogfood Rule)
- `do-harness init` (rust pack) scaffolds a minimal crate when no
  `Cargo.toml` exists, so `init && verify` exits 0 on an empty tree;
  existing crates are never touched, not even with `--force`.
- The generic pack ships zero sensors: its `verify` pass is vacuous until
  real `[[sensors]]` are configured — not evidence.
- Never assume the harness works in this repo: re-prove with
  `do-harness verify --format json` and read the per-sensor exit codes.

## Gotchas
- Never trust LLM self-assessment over a computational sensor's exit code.
- Fix the sensor that fired; do not refactor unrelated code in the same pass.
- An empty sensor suite passes vacuously; that is not evidence.
- Exit codes carry the doctrine: **0 = ran and passed, 1 = ran and failed, 2 = could
  not run**. A gate that never started must never be readable as a pass, so the harness
  probes exit 2 (`HARNESS BLOCKED …`) when no binary resolves.
- An unspawnable sensor argv is reported by `do-harness` as a *failed* sensor with
  `exit_code: null` — the exit code alone cannot tell "gate red" from "gate never ran",
  so read the JSON, not the number.

## Steering Loop

The steering loop turns repeated sensor violations into harness improvements
instead of one-off fixes. The protocol:

1. **Trigger** — any computational sensor fires more than **2 times in one
   sprint** (across PRs/commits).
2. **Classify** — maintainability, architecture, behaviour, or security.
3. **Locate or create the guide** — find the feedforward guide in `HARNESS.md`;
   if none covers the pattern, scaffold a skill with
   `.agents/skills/skill-creator/` in `.agents/skills/<category>/`. The skill
   must *prevent* the violation, not just describe it.
4. **Update the sensor table** — point the sensor row in `HARNESS.md` at the
   guide.
5. **Log it** — add the entry to `CHANGELOG.md` under the current sprint.

### Metrics events (audit trail)

Every resolved sensor firing writes a structured event:

Path: `.agents/events/YYYY/MM/DD/<sensor>-<timestamp>.json`

```json
{
  "timestamp": "2026-07-18T12:00:00Z",
  "sensor": "clippy::too_many_arguments",
  "category": "maintainability",
  "violation_count": 3,
  "root_cause": "Function signatures grown beyond manageable parameter count",
  "guide_updated": "HARNESS.md#feedforward-guides",
  "skill_created": false,
  "resolution": "Refactored to use config struct"
}
```

`.agents/events/` is committed on purpose — it is the searchable record future
agents use to avoid repeating known violations.