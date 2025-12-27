# GOAP Agent Roadmap (Q1–Q2)

## Vision
Operationalize a repeatable GOAP planning-and-execution loop across the repo with clear quality gates, memory-driven feedback, and minimal overhead.

## Themes
- Templates and quality gates (docs-first, code-light)
- Optional CI integration (non-blocking to start)
- Memory-MCP feedback loop integration
- Periodic hygiene: pinning, permissions, archives

## Milestones
1) Templates and Gates (Week 1)
- Adopt: GOAP_AGENT_EXECUTION_TEMPLATE.md, GOAP_AGENT_QUALITY_GATES.md
- Add plan links in plans/README.md
- Success: New plans conform; CI unchanged

2) CI Hooks (Week 2–3)
- Add optional GOAP checks to scripts/quality-gates.sh (non-fatal)
- Validate on PRs touching plans/workflows
- Success: GOAP step runs, reports summary

3) Memory Feedback (Week 3–4)
- Document memory-mcp health_check/get_metrics usage within plans
- Provide sample advanced_pattern_analysis inputs
- Success: Plans include a short feedback loop that runs

4) Consolidation (Week 5–6)
- Archive stale or redundant plans to plans/archive/
- Link the canonical GOAP docs from README
- Success: Plans folder is lean; <500 lines per doc

5) Continuous Improvement (Quarterly)
- Refresh action SHAs, revisit permissions; update templates if needed
- Review metrics/reporting usefulness and adjust gates
- Success: Low friction, high signal planning and execution

## Risks & Mitigations
- Plan drift: keep concise, leverage templates
- CI noise: keep GOAP checks optional initially
- Missing episode data: allow synthetic analysis for now

## KPIs
- Time-to-Plan (<30 min), Plan completeness rate, CI pass rate, Security findings addressed, Coverage maintained
