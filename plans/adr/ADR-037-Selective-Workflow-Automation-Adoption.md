# ADR-037: Selective Workflow Automation Adoption from External Repository Patterns

- **Status**: Proposed
- **Date**: 2026-03-05
- **Deciders**: Project maintainers
- **Supersedes**: None
- **Related**: ADR-022 (GOAP Agent System), ADR-029 (GitHub Actions Modernization), ADR-034 (Release Engineering Modernization)

## Context

This repository already has strong developer workflow automation:

1. Build and quality scripts are standardized (`scripts/build-rust.sh`, `scripts/code-quality.sh`, `scripts/quality-gates.sh`)
2. Skill ecosystem is broad (`.agents/skills/`)
3. Planning and ADR governance already exist (`plans/`, `plans/adr/`)

A comparison against `d-o-hub/chaotic_semantic_memory` identified a small set of operational workflow patterns that may improve developer productivity without duplicating existing capabilities.

The key question is whether to import external workflow assets wholesale or selectively.

## Decision

Adopt a **selective import policy**:

1. **Import only high-value workflow patterns that fill real gaps**
2. **Do not import domain-specific constraints or redundant tooling**
3. **Implement imported patterns using local conventions and existing scripts/skills**

### In-scope adoption targets

1. **Documentation integrity automation**
   - Add a script-level gate for doc links, command references, and version consistency.

2. **Release operation wrapper**
   - Add a single operator-friendly release script that orchestrates validation and release steps aligned with ADR-034.

3. **Lightweight GOAP state index**
   - Add minimal canonical state tracking docs to reduce drift across multiple GOAP plan files.

4. **Machine-readable architecture context contract**
   - Add a `context.yaml` style artifact for deterministic agent bootstrap and lower-friction context sharing.

### Explicit exclusions

1. No wholesale copy of external repository scripts
2. No adoption of external domain-specific runtime constraints
3. No npm publish workflow adoption unless npm artifacts become first-class deliverables here
4. No replacement of existing quality gate/build scripts

## Rationale

1. Protects local architecture and conventions while still learning from proven patterns
2. Avoids tool sprawl and duplicated workflows
3. Aligns with ADR-022 GOAP discipline and ADR-034 release modernization
4. Keeps change surface area small and auditable

## Consequences

### Positive

1. Better documentation reliability and lower drift
2. Cleaner release operations UX
3. Stronger planning state visibility for GOAP execution
4. Improved agent onboarding through machine-readable context

### Negative

1. Additional maintenance for new scripts/contracts
2. Minor process overhead for teams adopting new gates

### Risks

1. Over-engineering low-value automation
2. Divergence between plan docs and implemented scripts if rollout is partial

## Implementation Notes

1. Start with docs integrity automation and release wrapper (highest ROI)
2. Keep GOAP state index minimal; avoid duplicating full plan content
3. Validate all additions through existing quality gates and clippy/test policy
4. Update `AGENTS.md` references once implementation lands

## Adoption Plan

### Phase 1 (P1)

1. Implement `scripts/check-docs-integrity.sh`
2. Implement `scripts/release-manager.sh` aligned with ADR-034 phases

### Phase 2 (P2)

1. Add `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md` as lightweight canonical planning index
2. Add `docs/architecture/context.yaml` contract and validation

## Acceptance Criteria

1. New scripts are documented in `AGENTS.md` and discoverable from `plans/README.md`
2. Existing validation flow remains green with no regressions
3. No overlap replaces `build-rust.sh`, `code-quality.sh`, or `quality-gates.sh`
4. External import remains selective (no wholesale copy)

## Progress (2026-03-05)

1. `scripts/check-docs-integrity.sh` created
2. `scripts/release-manager.sh` created
3. GOAP index files created: `plans/GOALS.md`, `plans/ACTIONS.md`, `plans/GOAP_STATE.md`
4. Pending: `docs/architecture/context.yaml` and quality-gate integration

## References

- `plans/GOAP_CSM_WORKFLOW_GAP_ADOPTION_2026-03-05.md`
- `plans/adr/ADR-022-GOAP-Agent-System.md`
- `plans/adr/ADR-034-Release-Engineering-Modernization.md`
- External comparison source: `https://github.com/d-o-hub/chaotic_semantic_memory`
