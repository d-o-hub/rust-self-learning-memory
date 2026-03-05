# GOAP Plan: Chaotic Semantic Memory Workflow Gap Adoption (2026-03-05)

- **Status**: Complete (Phases A/B/C executed)
- **Date**: 2026-03-05
- **Method**: GOAP (ANALYZE -> DECOMPOSE -> STRATEGIZE -> COORDINATE -> EXECUTE -> SYNTHESIZE)
- **Source Comparison**: `https://github.com/d-o-hub/chaotic_semantic_memory`
- **Related ADRs**: ADR-022, ADR-029, ADR-034, ADR-037 (proposed)

## 1) ANALYZE

### Local baseline (`rust-self-learning-memory`)

Strengths already present:

1. Rich script layer (`scripts/build-rust.sh`, `scripts/code-quality.sh`, `scripts/quality-gates.sh`)
2. Strong skill inventory in `.agents/skills/` (40+ skills)
3. Explicit Skill+CLI-first policy in `AGENTS.md`
4. Existing GOAP and ADR discipline (`plans/GOAP_*.md`, `plans/adr/ADR-022-*.md`)

### External comparison highlights (`chaotic_semantic_memory`)

Potentially transferable workflow patterns:

1. Docs integrity automation (`check-docs-links.sh`, `sync-docs.sh`, `sync-version.sh`)
2. Unified release operation script (`release-manager.sh`, pre-release gate orchestration)
3. Fast local hook path (`pre-commit.sh`, `setup-hooks.sh`)
4. Structured plan/goal manager script (`plans-manager.sh`) and generated context artifacts

### Gap verdict

Do not import broad tooling wholesale. Import only targeted workflow capabilities not already covered locally.

### Prioritized recommendations

| Priority | Recommendation | Benefit | Effort | Risk | ADR Link |
|----------|----------------|---------|--------|------|----------|
| P1 | Add docs integrity gate script (`check-docs-integrity.sh`) | Prevents docs/link/version drift before merge | M | Low | ADR-037, ADR-029 |
| P1 | Add release operations wrapper (`release-manager.sh`) | Single operator command for validate/prepare/publish/rollback | M | Low-Med | ADR-037, ADR-034 |
| P2 | Add lightweight GOAP state index (`GOALS.md`, `ACTIONS.md`, `GOAP_STATE.md`) | Reduces planning drift and improves handoffs | S | Low | ADR-037, ADR-022 |
| P2 | Add machine-readable context contract (`docs/architecture/context.yaml`) | Faster deterministic agent bootstrap and context reuse | S-M | Low | ADR-037, ADR-022 |

## 2) DECOMPOSE

### High-value gaps to address

1. **Docs integrity gate gap**
   - Missing a single script to validate links, command snippets, and version references across docs and plans.

2. **Release execution ergonomics gap**
   - ADR-034 defines release modernization, but there is no single operator-facing command wrapper for validate/prepare/publish/rollback orchestration.

3. **GOAP state index gap (lightweight)**
   - Plans are comprehensive but fragmented. A minimal canonical state layer would reduce plan drift.

4. **Machine-readable architecture context gap**
   - No standard `docs/architecture/context.yaml` equivalent for deterministic agent bootstrap.

### Non-goals

1. No domain-specific algorithm/process imports from external repo.
2. No npm/wasm publishing workflow adoption unless this repository starts shipping npm artifacts.
3. No replacement of existing quality gate scripts.

## 3) STRATEGIZE

Use a **hybrid strategy**:

- **Sequential**: define ADR and scope boundaries first.
- **Parallel**: implement docs gate + release wrapper + context contract drafts.
- **Sequential**: integrate into `AGENTS.md` and quality gate workflows.

## 4) COORDINATE

### Work packages

| ID | Work Package | Owner Skill/Agent | Depends On | Exit Criteria |
|----|--------------|-------------------|------------|---------------|
| WP1 | Docs integrity script design | `code-quality` + `documentation` | ADR-037 | Script spec complete + command contract documented |
| WP2 | Release manager wrapper design | `github-release-best-practices` + `release-guard` | ADR-037 | Script subcommands defined + ADR-034 cross-link |
| WP3 | GOAP state index design | `goap-agent` + `architecture-validation` | ADR-037 | Minimal files and schema agreed |
| WP4 | Context contract draft | `codebase-analyzer` + `yaml-validator` | ADR-037 | `context.yaml` schema draft complete |

## 5) EXECUTE

### Phase A (Immediate: planning only)

1. Create ADR-037 for selective workflow adoption policy.
2. Keep implementation out of scope for this planning cycle.

### Phase B (Implementation proposal)

1. Add `scripts/check-docs-integrity.sh` (new)
2. Add `scripts/release-manager.sh` (new, wrapper style)
3. Add `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md` (minimal index only)
4. Add `docs/architecture/context.yaml` and validation step

### Phase B progress update (2026-03-05)

1. `scripts/check-docs-integrity.sh` - Implemented
2. `scripts/release-manager.sh` - Implemented (dry-run default)
3. `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md` - Added
4. `docs/architecture/context.yaml` - Added

### Phase C progress update (2026-03-05)

1. Docs integrity script integrated in `scripts/quality-gates.sh` (non-blocking mode)
2. Usage references added in `AGENTS.md` and `agent_docs/README.md`
3. Dry-run release validation path confirmed via `./scripts/release-manager.sh validate`
4. Initial verification signals recorded in `plans/STATUS/VALIDATION_LATEST.md`

### Phase D monitoring update (2026-03-05, PR #334)

1. PR merge state currently `UNSTABLE` due to CI check failures.
2. Root causes identified via GH CLI:
   - Formatting drift (`tests/e2e/cli_workflows.rs`) in `Essential Checks (format)` and `Quick PR Check`.
   - YAML lint failures in `.github/workflows/changelog.yml` (`truthy` + missing newline at EOF).
   - `Check Quick Check Status` failing as expected downstream dependency.
3. Additional issue tracked: `codecov/patch` failing.
4. Follow-up actions captured in `plans/ACTIONS.md` under WG-005.

### Phase C (Validation and rollout)

1. Wire docs integrity script into `scripts/quality-gates.sh` as non-destructive check first, then blocking after stabilization.
2. Add usage references in `AGENTS.md` and `agent_docs/README.md`.
3. Run validation sequence:
   - `./scripts/code-quality.sh fmt`
   - `./scripts/code-quality.sh clippy`
   - `./scripts/build-rust.sh check`
   - `cargo nextest run --all`
   - `./scripts/quality-gates.sh`
4. Capture initial effectiveness metrics in `plans/STATUS/VALIDATION_LATEST.md`.

## 6) SYNTHESIZE

### Recommended adoption set

Adopt now (planning accepted):

1. Docs integrity automation
2. Release operation wrapper
3. Lightweight GOAP state index
4. Machine-readable context contract

### Final synthesis

1. All planned phases (A/B/C) completed for ADR-037 rollout scope.
2. Integration is intentionally low-risk: docs checks are non-blocking during stabilization.
3. Next hardening step: graduate docs integrity to blocking mode once baseline drift is cleared.

### GOAP learnings (2026-03-05)

1. **Check-trigger coupling matters**: after remediation, a plans-only follow-up commit can leave PR check rollup empty for required checks depending on workflow trigger/path configuration.
2. **Operational sequencing improvement**: avoid adding plans-only trailing commits before required CI workflows attach/publish statuses for remediation commits.
3. **Monitoring requirement**: include explicit GH CLI checkpoint after push (`gh pr view --json statusCheckRollup,mergeStateStatus`) and treat empty rollup on required checks as a blocker signal.
4. **ADR alignment insight**: ADR-037 automation rollout is valid, but merge stability also depends on CI trigger topology (ADR-029/ADR-034 adjacent concern).

Reject for now:

1. NPM publish workflow import
2. WASM-size specific gate import
3. Domain-specific constraints from external project

### Success metrics

1. Documentation drift defects found pre-merge (>=1 meaningful catch in first month)
2. Release workflow command count reduced to one top-level operator command
3. GOAP plan status synchronization effort reduced (subjective team feedback)
4. Agent bootstrap context available in one machine-readable file

### Timeline proposal

| Week | Scope | Deliverable |
|------|-------|-------------|
| W1 | P1 docs integrity gate | `scripts/check-docs-integrity.sh` + quality gate integration |
| W1-W2 | P1 release wrapper | `scripts/release-manager.sh` + ADR-034 workflow alignment |
| W2 | P2 GOAP state index | `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md` |
| W2-W3 | P2 context contract | `docs/architecture/context.yaml` + validation hook |
