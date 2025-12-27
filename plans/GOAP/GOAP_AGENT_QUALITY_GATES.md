# GOAP Agent Quality Gates

Use these gates for any GOAP-driven plan or coordination effort in this repository.

## Baseline Repository Gates
- Build: `cargo build --all`
- Tests: `cargo test --all` (maintain coverage as per docs/QUALITY_GATES.md)
- Lint: `cargo fmt --all -- --check` and `cargo clippy --all -- -D warnings`
- Security: existing security workflow (dependency-review, gitleaks) green

## GOAP-Specific Gates
- Plan Validity: Each plan must include Objective, Constraints, Strategy, Phases, Quality Gates, Risks, and Rollback.
- Coordination Strategy: Explicitly state Parallel/Sequential/Hybrid/Loop and justify choice.
- Agent Routing: Name agents and deliverables for each phase; include testability.
- Metrics & Feedback: At least one feedback loop using memory-mcp tools:
  - `health_check` must pass
  - Optional: `get_metrics` snapshot included in the plan execution notes
  - Optional: `advanced_pattern_analysis` or `analyze_patterns` when data available

## CI Integration Suggestions
- Add a job step to render plan summaries and run actionlint/yamllint on workflow-related plans.
- Ensure plans remain <500 lines.

## Exit Criteria
A GOAP plan is considered complete when:
- All phases’ success criteria met
- All gates pass in CI
- A short “Execution Summary” is appended to the plan or linked follow-up.
