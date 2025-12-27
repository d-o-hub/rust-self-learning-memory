# GOAP Archive Recommendations (Non-Destructive)

This list identifies candidates for archival or consolidation.

## ✅ Completed Actions (2025-12-24)

### 1. Deprecated Files Moved
The following files have been moved to `plans/archive/goap-plans/deprecated/` with historical notes:
- ~~`plans/archive/goap-plans/goap-execution-plan-cleanup.md`~~ → **moved to deprecated/**
  - Superseded by GOAP_AGENT_IMPROVEMENT_PLAN.md and GOAP_AGENT_QUALITY_GATES.md
- ~~`plans/archive/goap-plans/PLANS_FOLDER_ANALYSIS_AND_RECOMMENDATIONS.md`~~ → **moved to deprecated/**
  - Covered by GOAP_AGENT_CODEBASE_VERIFICATION.md with clearer next actions
- ~~`plans/archive/goap-plans/goap-execution-schedule.md`~~ → **moved to deprecated/**
  - Roadmap now standardized in GOAP_AGENT_ROADMAP.md

Each deprecated file now has a historical note at the top explaining:
- When it was archived (2025-12-24)
- Why it was deprecated
- Reference to the canonical replacement document

### 2. plans/README.md Index
The `plans/README.md` already prefers the new canonical GOAP docs:
- GOAP_AGENT_IMPROVEMENT_PLAN.md
- GOAP_AGENT_QUALITY_GATES.md
- GOAP_AGENT_EXECUTION_TEMPLATE.md
- GOAP_AGENT_CODEBASE_VERIFICATION.md
- GOAP_AGENT_ROADMAP.md

### 3. Keep (Reference Value)
- `plans/archive/goap-plans/goap-verification-final-report.md` (historical validation summary)
- `plans/archive/goap-plans/goap-memory-mcp-verification.md` (useful MCP context)
- `plans/archive/goap-plans/deprecated/*` (archived superseded plans, with notes)
- `plans/archive/releases/*` (release-specific histories)

## Ongoing Maintenance
- **Periodically review** for duplication and line-count creep (>500 lines per AGENTS.md)
- **Archive old plans** when superseded by new canonical documentation
- **Add historical notes** to archived files explaining their deprecation reason
