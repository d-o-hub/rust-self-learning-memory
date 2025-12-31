# Plans Navigation Guide

**Purpose**: Help contributors quickly find planning and roadmap documents
**Last Updated**: 2025-12-31
**Version**: v0.1.12

---

## Quick Start (3 min)

**I want to...**
- **Understand current project status**: See [STATUS/PROJECT_STATUS_UNIFIED.md](STATUS/PROJECT_STATUS_UNIFIED.md)
- **See what's being built next**: See [ROADMAPS/ROADMAP_ACTIVE.md](ROADMAPS/ROADMAP_ACTIVE.md)
- **Learn about architecture**: See [ARCHITECTURE/ARCHITECTURE_CORE.md](ARCHITECTURE/ARCHITECTURE_CORE.md)
- **Find implementation details**: Check relevant crate docs in `../docs/`
- **Search for specific topics**: Run `./scripts/search-plans.sh "keyword"`

---

## Active Planning Documents

### Current Status & Progress
- **[STATUS/PROJECT_STATUS_UNIFIED.md](STATUS/PROJECT_STATUS_UNIFIED.md)** - Single source of truth for project status
- **[ROADMAPS/ROADMAP_ACTIVE.md](ROADMAPS/ROADMAP_ACTIVE.md)** - Current development roadmap (v0.1.x → v1.0)

### Architecture & Design
- **[ARCHITECTURE/ARCHITECTURE_CORE.md](ARCHITECTURE/ARCHITECTURE_CORE.md)** - System architecture overview
- **[ARCHITECTURE/ARCHITECTURE_PATTERNS.md](ARCHITECTURE/ARCHITECTURE_PATTERNS.md)** - Design patterns used
- **[ARCHITECTURE/ARCHITECTURE_INTEGRATION.md](ARCHITECTURE/ARCHITECTURE_INTEGRATION.md)** - Component integration
- **[ARCHITECTURE/ARCHITECTURE_DECISION_RECORDS.md](ARCHITECTURE/ARCHITECTURE_DECISION_RECORDS.md)** - Key architectural decisions
- **[ARCHITECTURE/API_DOCUMENTATION.md](ARCHITECTURE/API_DOCUMENTATION.md)** - API reference (split into modules)

### Configuration
- **[CONFIGURATION/CONFIG_VALIDATION_GUIDE.md](CONFIGURATION/CONFIG_VALIDATION_GUIDE.md)** - Configuration validation rules
- **[CONFIGURATION/CONFIG_UX_GUIDE.md](CONFIGURATION/CONFIG_UX_GUIDE.md)** - Configuration user experience
- **[EMBEDDINGS_INTEGRATION_ANALYSIS.md](EMBEDDINGS_INTEGRATION_ANALYSIS.md)** - Multi-provider embeddings guide

### Feature-Specific Plans
- **[OPTIMIZATION_ANALYSIS_2025-12-29.md](OPTIMIZATION_ANALYSIS_2025-12-29.md)** - Performance optimization roadmap
- **[VECTOR_SEARCH_OPTIMIZATION.md](VECTOR_SEARCH_OPTIMIZATION.md)** - Vector search improvements
- **[CIRCUIT_BREAKER_CONFIGURATION_GUIDE.md](CIRCUIT_BREAKER_CONFIGURATION_GUIDE.md)** - Circuit breaker setup
- **[CIRCUIT_BREAKER_INCIDENT_RUNBOOK.md](CIRCUIT_BREAKER_INCIDENT_RUNBOOK.md)** - Troubleshooting guide

### Quality & Testing
- **[QUALITY_METRICS_TOOL.md](../docs/QUALITY_METRICS_TOOL.md)** - Quality measurement framework
- **[QUALITY_GATES.md](../docs/QUALITY_GATES.md)** - CI/CD quality gates

---

## Archive Structure

The `archive/` folder contains historical documents organized by category:

### Completed Work
- `archive/completed/` - Implementation summaries and completion reports
- `archive/completed/quality-gates/` - Quality gate validation reports

### GOAP Plans
- `archive/goap-plans/` - Historical GOAP execution plans
- Organized by date (e.g., `2025-12/`)

### Research
- `archive/research/` - Completed research findings and reports

### Legacy
- `archive/legacy/` - Historical planning framework (Phase 0-7)
- Preserved for historical reference

### Version-Specific
- `archive/v0.1.0/`, `archive/v0.1.2/`, etc. - Version-specific roadmaps and plans
- `archive/v0.1.7-roadmap/` - Archived v0.1.7-v0.1.8 planning

### Archive Index
- **[archive/ARCHIVE_INDEX.md](archive/ARCHIVE_INDEX.md)** - Complete archive index

---

## Search Tips

### Using the Search Script
```bash
# Search for plans about embeddings
./scripts/search-plans.sh "embeddings" --active

# Search in archive only
./scripts/search-plans.sh "circuit breaker" --archive

# Search across all plans
./scripts/search-plans.sh "optimization"
```

### Finding Specific Types of Documents

**Execution Plans**: Look in `GOAP/` folder
**Status Reports**: Look in `STATUS/` folder
**Research**: Look in `research/` (active) or `archive/research/` (historical)
**Roadmaps**: Look in `ROADMAPS/` (active) or `archive/` (historical)

### Understanding Document Status

Active documents are in the root or active subdirectories:
- `ARCHITECTURE/`, `CONFIGURATION/`, `ROADMAPS/`, `STATUS/`, `GOAP/`, `research/`

Archived documents are in `archive/`:
- Completed work, superseded versions, historical plans

---

## Common Questions

**Q: Where do I find what features are being worked on?**  
A: Check [ROADMAPS/ROADMAP_ACTIVE.md](ROADMAPS/ROADMAP_ACTIVE.md) for current priorities

**Q: How do I know if a document is current or outdated?**  
A: Documents in root directories are current. Documents in `archive/` are historical. Check the "Last Updated" date at the top of each file.

**Q: Where can I learn about the system architecture?**  
A: Start with [ARCHITECTURE/ARCHITECTURE_CORE.md](ARCHITECTURE/ARCHITECTURE_CORE.md) for overview, then dive into specific documents as needed.

**Q: How do I find past decisions about a specific topic?**  
A: Search with `./scripts/search-plans.sh "topic"` or check [ARCHITECTURE/ARCHITECTURE_DECISION_RECORDS.md](ARCHITECTURE/ARCHITECTURE_DECISION_RECORDS.md) for key architectural decisions.

**Q: Where do I find implementation details for specific features?**  
A: Check the `../docs/` folder and `../agent_docs/` for implementation guides. The `plans/` folder focuses on planning and roadmap documentation.

**Q: What if I can't find what I'm looking for?**  
A: Try the search script, check the [archive/ARCHIVE_INDEX.md](archive/ARCHIVE_INDEX.md), or ask the team.

---

## Document Lifecycle

1. **Active Planning**: Documents in root folders (current work)
2. **Implementation**: Moved to `docs/` or implemented in code
3. **Completion**: Summarized and archived in `archive/completed/`
4. **Retention**: Kept for 1 year, then reviewed for deletion
5. **Preservation**: "Keep Forever" documents retained indefinitely

See [ARCHIVE_POLICY.md](ARCHIVE_POLICY.md) for full retention rules.

---

## Getting Help

- **Navigation**: Use this guide + search script
- **Technical questions**: Check `../docs/` and `../agent_docs/`
- **Project status**: See [STATUS/PROJECT_STATUS_UNIFIED.md](STATUS/PROJECT_STATUS_UNIFIED.md)
- **Team coordination**: Ask in project discussions

---

**Last Updated**: 2025-12-31
**Maintained By**: Project Team
