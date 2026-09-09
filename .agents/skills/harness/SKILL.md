---
name: harness
description: >
  Map the harness-engineering feedforward guides and feedback sensors, and run
  the self-correction protocol when a computational sensor fires. Use when a
  sensor fails (do-harness verify), before making code changes, or when setting
  up agent context for a new task. Triggers: "harness", "sensor fire",
  "CI failure", "self-correction".
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
| Sensor | Command | Stage |
|--------|---------|-------|
| verify | `do-harness verify` | pre-commit (subset) + pre-push + CI (full) |
| individual | `do-harness verify --only <name>` | targeted re-runs |

Sensors are defined in `do-harness.toml`; run `do-harness list` to see them.

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