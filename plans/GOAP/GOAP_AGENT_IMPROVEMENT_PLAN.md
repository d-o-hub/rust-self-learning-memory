# GOAP Agent Improvement Plan

## Objective
Enhance the GOAP (Goal-Oriented Action Planning) agent usage in this repository to deliver clearer plans, tighter coordination, measurable quality gates, and smooth integration with memory-mcp and the Rust crates.

## Current State (Summary)
- GOAP guidance exists in `.claude/agents/goap-agent.md` and `.opencode/agent/goap-agent.md` and several plans under `plans/archive/goap-plans/`.
- Recent GOAP plans focus on GitHub Actions workflows in `plans/GOAP_EXECUTION_PLAN_*.md` (CI, quick-check, benchmarks, security, yaml-lint, release).
- No concrete, reusable GOAP orchestration entrypoint is present in code; coordination is performed manually via agent guidance.

## Constraints / Assumptions
- Keep plans <= 500 lines per file (repo policy).
- No breaking changes to Rust crates; improvements should be additive.
- Use existing quality gates (quality-gates.sh, tests, clippy, fmt) and expand with GOAP-specific checks where useful.

## Strategy
- Hybrid: Document a reusable GOAP execution protocol and provide concrete implementation hooks (scripts + MCP-backed memory) to enable iterative planning and verification.

## Phases
1) Define Standard GOAP Templates (Parallel)
- Task: Introduce minimal templates for plan creation, decision logs, and quality gates.
- Deliverables: `GOAP_AGENT_QUALITY_GATES.md`, `GOAP_AGENT_EXECUTION_TEMPLATE.md` (embedded in this repo usage).
- Success: New plans follow the template and pass lint checks.

2) Establish Execution Hooks (Sequential)
- Task: Provide lightweight scripts/commands to run GOAP checks locally and in CI.
- Deliverables: Script stubs to validate plans, run targeted tests, and collect MCP metrics.
- Success: `./scripts/quality-gates.sh` includes an optional GOAP step; CI shows pass/fail.

3) Integrate Memory-MCP for Feedback (Sequential)
- Task: Use memory-mcp tools to summarize patterns across GOAP episodes (where available) and surface guidance.
- Deliverables: Guidance on `query_memory`, `analyze_patterns`, `advanced_pattern_analysis` usage during plans.
- Success: Plans include a short “feedback loop” step that runs without failures.

4) Codify Coordination Heuristics (Parallel)
- Task: Record routing heuristics for test-runner, code-reviewer, feature-implementer, debugger, refactorer, loop-agent.
- Deliverables: Updated `GOAP_AGENT_EXECUTION_TEMPLATE.md` embedded section with routing examples.
- Success: At least one plan uses a parallel + sequential split effectively.

5) Validation and Rollout (Sequential)
- Task: Apply the template to one active area (e.g., security workflow).
- Deliverables: A new plan using the template and passing the quality gates.
- Success: Plan PR merged; CI green.

## Quality Gates
- Lint: yamllint/actionlint for workflow changes; markdownlint basics (no trailing spaces, headings sequence).
- Build/Test: `cargo build --all` and `cargo test --all` (with >90% coverage per docs/QUALITY_GATES.md).
- Static: `cargo clippy --all -- -D warnings`, `cargo fmt --all -- --check`.
- Security: `.github/workflows/security.yml` pass; dependency-review and gitleaks.
- MCP: `memory-mcp health_check` OK; optional `get_metrics` summary produced.

## Risks & Mitigations
- Plan sprawl: Keep under 500 lines and consolidate in `plans/` root; archive old variants.
- Tooling drift: Pin action SHAs and schedule quarterly updates.
- Missing episodes: Use synthetic data in `advanced_pattern_analysis` until production episodes exist.

## Rollout Checklist
- [ ] Add GOAP templates and quality gates file
- [ ] Wire optional GOAP checks into quality-gates script
- [ ] Add an exemplar plan using the template (e.g., Security workflow improvements)
- [ ] Validate CI and update documentation

## Success Criteria
- New plans consistently use the template, pass gates, and include a feedback loop referencing memory-mcp tools.
