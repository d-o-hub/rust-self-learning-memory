# Plans Folder - Navigation & Organization

**Last Updated**: 2025-12-23  
**Purpose**: Working directory for active planning and project management  
**Status**: Documentation consolidation in progress

## 📋 Quick Navigation

### 🔴 Active Planning Documents (Current Work)
These files contain active development work, roadmaps, and current implementation plans:

| Document | Purpose | Last Updated | Priority |
|----------|---------|--------------|----------|
| **[PROJECT_STATUS_UNIFIED.md](./PROJECT_STATUS_UNIFIED.md)** | **SINGLE SOURCE OF TRUTH** - Current project status, quality gates, implementation progress | 2025-12-23 | 🔴 **CRITICAL** |
| **[ROADMAP.md](./ROADMAP.md)** | Master roadmap, version history, and future planning | 2025-12-20 | 🔴 **CRITICAL** |
| **[CURRENT_ARCHITECTURE_STATE.md](./CURRENT_ARCHITECTURE_STATE.md)** | Detailed technical architecture documentation | 2025-12-21 | 🟡 **HIGH** |
| **[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)** | Missing implementations and technical specifications | 2025-12-20 | 🟡 **HIGH** |

### 🟡 Planning Resources (Reference Materials)
Supporting documentation for planning and optimization:

| Document | Purpose | Last Updated | Priority |
|----------|---------|--------------|----------|
| **[CONFIGURATION_OPTIMIZATION_STATUS.md](./CONFIGURATION_OPTIMIZATION_STATUS.md)** | Configuration complexity resolution status | 2025-12-22 | 🟡 **HIGH** |
| **[EMBEDDINGS_REFACTOR_DESIGN.md](./EMBEDDINGS_REFACTOR_DESIGN.md)** | Semantic embeddings implementation design | 2025-12-21 | 🟡 **MEDIUM** |
| **[QUALITY_SYSTEMS_ANALYSIS.md](./quality_systems_analysis.md)** | Quality gates and systems analysis | 2025-12-20 | 🟡 **MEDIUM** |
| **[CONFIG_VALIDATION_STRATEGY.md](./CONFIG_VALIDATION_STRATEGY.md)** | Configuration validation framework | 2025-12-20 | 🟡 **MEDIUM** |

### 🟢 Archive Index (Historical Reference)
Completed planning work and historical documents:

- **[archive/](./archive/)** - Organized by version and theme
  - **archive/completed/** - Finished implementation summaries
  - **archive/goap-plans/** - GOAP execution planning documents
  - **archive/legacy/** - Historical planning framework
  - **archive/releases/** - Version-specific documentation
  - **archive/research/** - Research findings and analysis
  - **archive/v0.1.7-prep/** - v0.1.7 release preparation materials

**See**: **[archive/v0.1.7-prep/ARCHIVE_INDEX.md](./archive/v0.1.7-prep/ARCHIVE_INDEX.md)** for complete archive navigation.

## 🎯 How to Use This Folder

### For Current Development
1. **Start here**: [PROJECT_STATUS_UNIFIED.md](./PROJECT_STATUS_UNIFIED.md) for current state
2. **Roadmap**: Check [ROADMAP.md](./ROADMAP.md) for future plans
3. **Architecture**: Reference [CURRENT_ARCHITECTURE_STATE.md](./CURRENT_ARCHITECTURE_STATE.md) for technical details
4. **Implementation**: See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for missing features

### For Historical Context
1. **Archive access**: Navigate to [archive/](./archive/) folder
2. **Version history**: Check [CHANGELOG.md](../CHANGELOG.md) in root
3. **Release notes**: See [archive/releases/](./archive/releases/) for version-specific documentation

### For Planning New Work
1. **Review current status**: [PROJECT_STATUS_UNIFIED.md](./PROJECT_STATUS_UNIFIED.md)
2. **Check roadmap**: [ROADMAP.md](./ROADMAP.md) for alignment
3. **Reference architecture**: [CURRENT_ARCHITECTURE_STATE.md](./CURRENT_ARCHITECTURE_STATE.md)
4. **Document in plans/**: Add new planning documents here
5. **Archive when complete**: Move finished planning to [archive/](./archive/)

## 📊 Document Categories

### By Function
| Category | Documents | Purpose |
|----------|-----------|---------|
| **Status Tracking** | PROJECT_STATUS_UNIFIED.md | Single source of truth for current state |
| **Strategic Planning** | ROADMAP.md | Long-term vision and roadmap |
| **Technical Planning** | IMPLEMENTATION_PLAN.md, CURRENT_ARCHITECTURE_STATE.md | Implementation details and architecture |
| **Optimization Work** | CONFIGURATION_OPTIMIZATION_STATUS.md, EMBEDDINGS_REFACTOR_DESIGN.md | Ongoing optimization efforts |
| **Historical** | archive/ folder | Completed work and historical context |

### By Priority Level
| Priority | Documents | Status |
|----------|-----------|--------|
| **🔴 Critical** | PROJECT_STATUS_UNIFIED.md, ROADMAP.md | Active, frequently updated |
| **🟡 High** | CURRENT_ARCHITECTURE_STATE.md, IMPLEMENTATION_PLAN.md | Reference materials |
| **🟢 Medium** | Configuration and optimization docs | Supporting materials |
| **🔵 Archive** | All files in archive/ | Historical reference |

## 🔄 Document Lifecycle

### Active Documents
- Created for active development work
- Updated frequently during implementation
- Linked from root README.md and other key docs
- **Lifecycle**: Active → Reference → Archive

### Reference Documents
- Provide detailed technical context
- Updated periodically
- Linked from active documents
- **Lifecycle**: Reference → Archive (when superseded)

### Archive Documents
- Completed planning and implementation work
- Organized by version and theme
- Preserved for historical context and lookup
- **Lifecycle**: Permanent reference

## 🛠️ Maintenance Guidelines

### Adding New Planning Documents
1. **Check existing docs first**: Don't duplicate information
2. **Use clear naming**: `YYYY-MM-DD-descriptive-name.md`
3. **Link appropriately**: Cross-reference existing documents
4. **Set review date**: Add "Review by:" date for maintenance

### Updating Existing Documents
1. **Update timestamps**: Always update "Last Updated" when making changes
2. **Maintain consistency**: Ensure cross-references remain valid
3. **Archive when complete**: Move finished work to appropriate archive folder
4. **Review regularly**: Check for outdated information

### Archiving Completed Work
1. **Move to archive/**: Organize by theme (releases/, goap-plans/, research/, etc.)
2. **Update archive index**: Add entries to [archive/v0.1.7-prep/ARCHIVE_INDEX.md](./archive/v0.1.7-prep/ARCHIVE_INDEX.md)
3. **Update references**: Ensure active docs reference archived materials appropriately
4. **Clean up**: Remove or consolidate outdated active documents

## 📝 Style Guide

### Document Headers
All planning documents should include:
```markdown
# Document Title

**Last Updated**: YYYY-MM-DD
**Purpose**: Brief description
**Status**: [Draft/In Progress/Complete/Deprecated]
**Owner**: [Team/Person responsible]
```

### Cross-References
- Use relative paths: `[Document Name](./document-name.md)`
- Avoid absolute paths when possible
- Include descriptive link text
- Update links when moving/renaming files

### Timestamps
- Use ISO format: `YYYY-MM-DDTHH:MM:SSZ`
- Update timestamps on any substantive changes
- Include timezone information for clarity

## 🚨 Important Notes

### Single Source of Truth
**[PROJECT_STATUS_UNIFIED.md](./PROJECT_STATUS_UNIFIED.md)** replaces all previous status documents:
- ✅ IMPLEMENTATION_STATUS_2025-12-20.md (deprecated)
- ✅ V0.2.0_COMPLETION_STATUS.md (deprecated)  
- ✅ QUALITY_GATES_CURRENT_STATUS.md (deprecated)

### Active vs. Archive
- **plans/ folder**: Active planning and current work
- **archive/ folder**: Completed work and historical context
- **docs/ folder**: Permanent documentation (not planning)

### Navigation Help
If you can't find information:
1. Check [PROJECT_STATUS_UNIFIED.md](./PROJECT_STATUS_UNIFIED.md) for current state
2. Look in [archive/](./archive/) for historical context
3. Review [ROADMAP.md](./ROADMAP.md) for future plans
4. Check root [README.md](../README.md) for project overview

---

**Document Purpose**: Navigation guide for plans folder  
**Last Updated**: 2025-12-23  
**Next Review**: 2025-12-30  
**Owner**: Project maintainers

---

*This navigation document helps developers and contributors find the right planning information quickly. For current project status, always start with PROJECT_STATUS_UNIFIED.md.*