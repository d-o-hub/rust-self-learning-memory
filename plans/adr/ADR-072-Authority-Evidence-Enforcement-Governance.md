# ADR-072: Authority, Evidence, and Enforcement Governance

- **Status**: Proposed
- **Date**: 2026-07-14
- **Deciders**: Project maintainers
- **Related**: ADR-022, ADR-034, ADR-039, ADR-042, ADR-045, ADR-058, ADR-059
- **Plan**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md`

## Context

The repository currently has multiple contradictory descriptions of the same facts:

- Cargo metadata reports workspace version `0.2.0`, while canonical plans report `0.1.33` or `0.1.34`.
- AGENTS describes 90% coverage, `quality-gates.sh` and its Rust tests default to 70%, and Codecov has separate targets.
- AGENTS requires automated releases, while active plans and release skills still prescribe direct `gh release create`.
- Workflow prose claims publishing improvements that are absent from `publish-crates.yml`.
- ADR-039 defines a small canonical active plan set, but dated analyses and historical status remain active.
- ADR numbers 025 and 054 are duplicated, ADR-061 is referenced but absent, and WG identifiers have been reused.
- Test, coverage, performance, and release claims often omit the command, scope, commit, time, and artifact that produced them.

Documentation cannot override executable behavior, but executable behavior can also be incorrect. The project needs a declared authority hierarchy, explicit evidence requirements, one release path, and blocking drift checks.

## Decision

### 1. Authority matrix

When sources disagree, use this order and reconcile lower authorities promptly:

| Fact | Authoritative source | Non-authoritative mirrors |
|---|---|---|
| Workspace package/version set | `cargo metadata` derived from Cargo manifests | README, plans, generated VERSION text |
| Feature flags/dependencies | Owning crate `Cargo.toml` plus compile checks | README and skill summaries |
| CLI contract | Clap definitions and executable help snapshots | README/skills examples |
| MCP tool contract | Tool registry/definitions and protocol tests | README/skills summaries |
| CI behavior | Checked-in workflow/action/script executed at a commit | Status prose and old run links |
| Local quality command | Versioned canonical scripts and their tests | Copied commands in docs/skills |
| Released version | Matching immutable git tag plus successful automated GitHub release for the same commit | Workspace version alone |
| Published crate version | crates.io API/index evidence for exact package/version | Release plan claims |
| Metrics | Generated evidence artifact with required metadata | Hand-written totals or estimates |
| Architecture intent | Accepted ADR | Proposed ADR or roadmap item |
| Implementation state | Current code and executable tests | ADR status text and completed checkboxes |

A `VERSION` file, if retained for consumers, is generated from Cargo metadata and is never co-canonical.

### 2. Evidence schema

Every canonical metric or completion claim must include or link to:

```text
value/result
command or workflow job
scope/features/platform
commit SHA
UTC timestamp
tool version when material
artifact or durable run URL
```

Claims without this data are labeled `target`, `estimate`, `historical`, or `unverified`; they are not presented as current measurements.

### 3. Separate current, blocking, and target values

Coverage, performance, ignored-test count, LOC, and similar controls expose three fields:

- **Measured**: latest reproducible result.
- **Required**: the currently blocking floor/ceiling.
- **Target**: the planned ratchet objective.

ADR-042’s 70→75→80 coverage progression remains a target sequence until a fresh baseline and blocking implementation are recorded. A 90% aspiration must not be described as the current default unless the executable gate enforces it and the workspace passes it.

### 4. One release authority

The only supported release-creation path is:

1. approve and merge through normal branch protection;
2. validate the exact release commit with required gates;
3. bump Cargo workspace version and changelog/status as required;
4. push an approved matching tag; and
5. let `.github/workflows/release.yml` create the GitHub release.

Humans and agents do not call `gh release create` directly. Its use inside the controlled release workflow is an implementation detail, not a manual fallback. Release and skill documentation must reference this path. Publishing is a separate automated stage with explicit package policy, `--locked`, exact-version dependency propagation, and blocking preflight.

### 5. Canonical plan roles

ADR-039’s consolidation intent is retained and clarified:

- `plans/STATUS/CURRENT.md`: concise current state and evidence links.
- `plans/ROADMAPS/ROADMAP_ACTIVE.md`: future work only.
- `plans/GOALS.md`, `ACTIONS.md`, `GOAP_STATE.md`: active execution state only.
- one latest validation report and one latest gap/codebase analysis.
- dated GOAP documents are working snapshots and are archived after synthesis or supersession.
- ADRs record decisions, not sprint status journals.

Canonical files that describe one sprint/state are updated atomically in the same change.

### 6. Identifier registry

Create a machine-readable or generated registry for ADR and work identifiers.

- New numeric ADR IDs are globally unique and allocated before file creation.
- Historical duplicate files are not silently renumbered. The registry assigns explicit historical aliases (for example, `ADR-025A`/`ADR-025B` and `ADR-054A`/`ADR-054B`) and requires full filename links until maintainers choose a migration.
- Missing ADR references fail validation. Existing ADR-061 references must be removed, redirected to an actual decision, or satisfied by a deliberately authored ADR; they cannot remain implied.
- New work IDs are namespaced by plan (for example, `CBI-2026-07-14-S1.1`) or allocated from a registry. Bare reused `WG-NNN` identifiers are prohibited.

### 7. Lifecycle and revalidation

ADRs use: `Proposed -> Accepted -> Implemented -> Superseded/Rejected`.

- `Accepted` records decision date and deciders.
- `Implemented` links code/workflow evidence and validation.
- Material implementation drift moves the ADR to `Accepted (drift detected)` or triggers a superseding ADR; it is not left falsely implemented.
- High-impact ADRs are revalidated on major/minor releases or when their governing implementation changes.

### 8. Blocking drift validation

A required plans/docs check validates changed active content for:

- Cargo version disagreement in canonical current-state fields;
- duplicate/new ADR or work identifiers;
- missing ADR/local links;
- manual release instructions outside historical archives/workflow implementation;
- stale `LATEST` records beyond the agreed service level;
- multiple active documents claiming to be canonical/latest;
- active dated plans superseded by a newer synthesis;
- metrics without evidence metadata; and
- README feature flags/commands that do not exist in manifests or generated contracts.

The validator itself has executable fixtures and cannot reduce failures to warnings in required CI.

## Migration

1. Capture a local and remote truth snapshot at one commit.
2. Reconcile whether workspace `0.2.0` is released, unreleased, or a release candidate.
3. Update all canonical files atomically and archive superseded snapshots.
4. Add the identifier registry and historical duplicate aliases.
5. Remove or repair ADR-061 references.
6. Reconcile ADR-034/045/058 release text with the automated-only path.
7. Implement changed-plan validation as reporting, resolve baseline defects, then make it required.

## Consequences

### Positive

- Executable reality and architecture intent have explicit, noncompeting roles.
- Release, version, and metric claims become reproducible.
- Plan drift, duplicate identifiers, and stale skills/docs fail early.
- Historical ADRs remain auditable rather than being silently rewritten.

### Negative

- Canonical updates and release preparation require evidence artifacts.
- Initial migration will expose and require cleanup of substantial stale planning content.
- Some generated indexes and checks add maintenance/tooling cost.

### Neutral

- This ADR does not select a new coverage percentage; it governs how the measured value, blocking floor, and target are represented and enforced.
- It does not establish remote release state; that requires remote evidence.

## Alternatives considered

1. **Treat plans as authoritative**: rejected because plans currently contradict manifests and workflows.
2. **Treat code as the only documentation**: rejected because architecture intent, release policy, and user contracts need stable human-readable decisions.
3. **Adopt a hand-maintained `VERSION` as co-canonical**: rejected because two writable version authorities create drift.
4. **Renumber duplicate historical ADR files immediately**: rejected because it breaks historical links and obscures provenance; use registry aliases first.
5. **Keep drift checks nonblocking**: rejected as an end state because current nonblocking checks allowed the present contradictions.

## Acceptance criteria

- One verified workspace/release state appears in all canonical plans.
- No active instruction permits manual GitHub release creation.
- Every active ADR reference resolves and every new ADR/work ID is unique.
- Duplicate historical ADRs have registry aliases and full-filename links.
- Current quality metrics distinguish measured, required, and target values.
- Required plan/docs validation fails on representative drift fixtures.
- ADR-039, ADR-034, ADR-045, and ADR-058 contain supersession notes where this decision changes their governance or release guidance.
