# Feature Spike Decision Artifacts

Decision records for GO/NO-GO feature pilots (F4.*) and feasibility spikes (e.g. S1.1c).

## Layout

| Path | Role |
|------|------|
| `plans/spikes/<ID>.toml` | Spike config (inputs/thresholds for the runner) |
| `plans/STATUS/spikes/<ID>.json` | Decision artifact (validated output) |
| `scripts/run-feature-spike.sh` | Producer (when present) |
| `scripts/validate-feature-spike.sh` | Validator (when present) |

If the runner is missing, artifacts may be written manually with the same schema. Prefer the runner when present:

```bash
./scripts/run-feature-spike.sh F4.1 --config plans/spikes/F4.1.toml --output plans/STATUS/spikes/F4.1.json
./scripts/validate-feature-spike.sh plans/STATUS/spikes/F4.1.json
```

**TOML caveat**: `run-feature-spike.sh` uses a minimal line parser — put `required_files` / `reviewers` / `commands` arrays on **one line**.

## Schema

Required fields (from `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md` §6.1):

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Child package id (`F4.1`, `S1.1c`, …) |
| `commit` | string | Full git SHA at decision time |
| `timestamp` | string | ISO-8601 UTC |
| `owner` | string | Agent or human owner |
| `commands` | string[] | Evidence commands run or intended |
| `metrics` | object | Measured values (numeric or boolean) |
| `preapproved_thresholds` | object | Thresholds fixed **before** measurement |
| `result` | string | Outcome summary (`pass`, `fail`, …) |
| `decision` | string | `GO` or `NO-GO` |
| `reviewers` | string[] | Who accepted the decision |

Recommended optional fields:

| Field | Description |
|-------|-------------|
| `title` | Human label |
| `branch` / `pr` | Integration context |
| `rationale` | Why GO or NO-GO |
| `producer` | Runner or `manual (...)` |
| `evidence_paths` | Inside `metrics` or top-level |
| `blocks` / `follows_from` | Dependency edges |

## Decision rules

- **Missing fields**, post-hoc thresholds, or unmet `GO` thresholds → validation **fails**.
- `GO` means pilot proceeds (or code already lands on the branch under that pilot).
- `NO-GO` means stay on the prior safe state (e.g. S1.1c NO-GO → keep fail-closed execution; do not start S1.1d).
- V5.1 / convergence reads **validated** decision artifacts only — never ad-hoc choices.

## Current artifacts (2026-07-18 F4 remainder)

| ID | Decision | Notes |
|----|----------|-------|
| F4.1 | **GO** | Provenanced retrieval API on branch (PR #874) |
| F4.2 | **GO** | Operation journal on branch |
| F4.3 | **GO** | Local model digests/size pins (S1.5b) |
| F4.4 | **GO** | Skill contract compiler + generated catalog |
| S1.1c | **NO-GO** | WASI/Wasmtime remain out; fail-closed retained |

**Policy for this sprint**: all missing implemented, **no release** (workspace stays `0.1.36` unreleased; tag remains `v0.1.35`).
