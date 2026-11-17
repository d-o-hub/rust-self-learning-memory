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

**TOML parsing**: `run-feature-spike.sh` parses the config with Python `tomllib`
(`tomli` fallback), so multi-line arrays and normal TOML are fine. The producer still
only accepts the fields it knows; keep unknown keys in prose.

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

## Current artifacts (2026-11-15)

| ID | Decision | Notes |
|----|----------|-------|
| F4.1 | **GO** | Provenanced retrieval API (PR #874) — shipped |
| F4.2 | **GO** | Operation journal — shipped |
| F4.3 | **GO** | Local model digests/size pins (S1.5b) — shipped |
| F4.4 | **GO** | Skill contract compiler + generated catalog — shipped |
| S1.1c | **NO-GO** | WASI/Wasmtime remain out; fail-closed retained |
| R-F8 | **GO** | CLI relationship panel — shipped (#893) |
| R-F9 | **GO** | HNSW persistence + capacity eviction — shipped (#893) |
| R-F10 | **GO** | crates.io trusted publishing — shipped (ADR-078) |
| R-F2 | **GO** | Opt-in OTLP tracing — `R-F2-otel-2026-11-17.json`, ADR-084, shipped |
| R-F1, R-F3…R-F7 | **NO-GO** | Deferred by `R-F-product-backlog-2026-11-15.json`; the 2026-07-28 `forced_go` records are feasibility notes with empty `commands`, not approval |
| R-F-product-backlog | **NO-GO** | Authoritative product gate; supersedes the 2026-07-20 DEFER record and R-F1…R-F7 |

**Policy**: a `GO` artifact is approval to implement, not a claim that implementation
exists. `force_decision` artifacts without measured `commands` are notes only. The
active plan is `plans/ROADMAPS/ROADMAP_ACTIVE.md`; do not promote an R-F item from an
artifact alone. The 2026-07-20 register is archived at
`plans/archive/2026-11-consolidation/GOAP_COMPREHENSIVE_RECOMMENDATIONS_2026-07-20.md`.
