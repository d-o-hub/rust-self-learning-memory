# Skills Consolidation - Visual Overview

## Before Consolidation

```
Current State (Fragmented)
==========================

.opencode/skill/                    .claude/skills/
├── agent-coordination/             ├── agent-coordination/
│   └── SKILL.md (200 lines)        │   └── SKILL.md (150 lines)
├── analysis-swarm/                 ├── agentdb-advanced/
│   └── SKILL.md (515 lines)        │   └── SKILL.md
├── goap-agent/                     ├── agentdb-learning/
│   └── SKILL.md (893 lines)        │   └── SKILL.md
├── build-rust/                     ├── analysis-swarm/
│   └── SKILL.md                    │   ├── SKILL.md (41 lines)
├── ... (14 more)                   │   ├── discourse.md
                                    │   ├── examples.md
Total: 18 skills                    │   ├── orchestration.md
                                    │   └── personas.md
                                    ├── agentdb-*
                                    ├── v3-*
                                    ├── codebase-*
                                    └── ... (60+ more)

                                    Total: 69 skills

Issues:
- Duplicate skills (15 exist in both)
- Different structures
- No single source of truth
- Difficult to maintain
```

## After Consolidation

```
Unified State (.agents/skills/)
================================

.agents/skills/
├── INVENTORY.md
│
├── [Simple Skills - Single File or Symlink]
│   ├── agent-coordination/
│   │   └── SKILL.md → .claude/skills/agent-coordination/SKILL.md
│   ├── clean-code-developer/
│   │   └── SKILL.md → .claude/skills/clean-code-developer/SKILL.md
│   ├── code-quality/
│   │   └── SKILL.md → .claude/skills/code-quality/SKILL.md
│   ├── git-worktree-manager/
│   │   └── SKILL.md → .claude/skills/git-worktree-manager/SKILL.md
│   ├── github-release-best-practices/
│   │   └── SKILL.md → .claude/skills/github-release-best-practices/SKILL.md
│   ├── perplexity-researcher-pro/
│   │   └── SKILL.md → .claude/skills/perplexity-researcher-pro/SKILL.md
│   ├── perplexity-researcher-reasoning-pro/
│   │   └── SKILL.md → .claude/skills/perplexity-researcher-reasoning-pro/SKILL.md
│   │
│   ├── build-rust/
│   │   └── SKILL.md → .opencode/skill/build-rust/SKILL.md
│   │
│   └── ... (simple unique skills)
│
├── [Rich Skills - Multiple Files, Pending Merge]
│   ├── analysis-swarm/
│   │   ├── SKILL.md
│   │   ├── discourse.md
│   │   ├── examples.md
│   │   ├── orchestration.md
│   │   ├── personas.md
│   │   └── MERGE_NOTES.md ← Action required
│   │
│   ├── architecture-validation/
│   │   ├── SKILL.md
│   │   ├── compliance.md
│   │   ├── dimensions.md
│   │   ├── extraction.md
│   │   ├── self-learning.md
│   │   ├── workflow.md
│   │   └── MERGE_NOTES.md ← Action required
│   │
│   ├── debug-troubleshoot/
│   │   ├── SKILL.md
│   │   ├── issues.md
│   │   ├── logging.md
│   │   ├── techniques.md
│   │   ├── tokio-console.md
│   │   └── MERGE_NOTES.md ← Action required
│   │
│   ├── feature-implement/
│   │   ├── SKILL.md
│   │   ├── patterns.md
│   │   ├── process.md
│   │   ├── quality.md
│   │   ├── structure.md
│   │   └── MERGE_NOTES.md ← Action required
│   │
│   ├── github-workflows/
│   │   ├── SKILL.md
│   │   ├── advanced-features.md
│   │   ├── caching-strategies.md
│   │   ├── release-management.md
│   │   ├── troubleshooting.md
│   │   └── MERGE_NOTES.md ← Action required
│   │
│   └── goap-agent/
│       ├── SKILL.md
│       ├── agents.md
│       ├── examples.md
│       ├── execution-strategies.md
│       ├── methodology.md
│       ├── patterns.md
│       ├── skills.md
│       └── MERGE_NOTES.md ← Action required
│
├── [Unique Skills from .claude]
│   ├── agentdb-advanced/SKILL.md
│   ├── agentdb-learning/SKILL.md
│   ├── agentdb-memory-patterns/SKILL.md
│   ├── agentdb-optimization/SKILL.md
│   ├── agentdb-vector-search/SKILL.md
│   ├── codebase-analyzer/SKILL.md
│   ├── codebase-consolidation/
│   │   ├── SKILL.md
│   │   ├── analysis-dimensions.md
│   │   ├── consolidation-patterns.md
│   │   └── report-templates.md
│   ├── codebase-locator/SKILL.md
│   ├── context-retrieval/SKILL.md
│   ├── episode-complete/SKILL.md
│   ├── episode-log-steps/SKILL.md
│   ├── episode-start/SKILL.md
│   ├── episodic-memory-testing/
│   │   ├── SKILL.md
│   │   └── resources/
│   │       ├── episode-lifecycle.md
│   │       ├── pattern-extraction.md
│   │       └── reward-scoring.md
│   ├── github-code-review/SKILL.md
│   ├── github-multi-repo/SKILL.md
│   ├── github-project-management/SKILL.md
│   ├── github-release-management/SKILL.md
│   ├── github-workflow-automation/SKILL.md
│   ├── hooks-automation/SKILL.md
│   ├── do-memory-cli-ops/
│   │   ├── SKILL.md
│   │   └── troubleshooting.md
│   ├── pair-programming/SKILL.md
│   ├── parallel-execution/SKILL.md
│   ├── plan-gap-analysis/SKILL.md
│   ├── playwright-cli /SKILL.md
│   ├── quality-unit-testing/
│   │   ├── SKILL.md
│   │   ├── .test-quality.toml.example
│   │   ├── checklists/
│   │   ├── reference/
│   │   ├── scripts/
│   │   └── templates/
│   ├── release-guard/
│   │   ├── SKILL.md
│   │   └── validation.md
│   ├── reasoningbank-agentdb/SKILL.md
│   ├── reasoningbank-intelligence/SKILL.md
│   ├── rust-async-testing/
│   │   ├── SKILL.md
│   │   └── resources/
│   ├── skill-builder/SKILL.md
│   ├── skill-creator/
│   │   ├── SKILL.md
│   │   ├── description.md
│   │   ├── examples.md
│   │   ├── naming.md
│   │   ├── structure.md
│   │   ├── templates.md
│   │   └── validation.md
│   ├── sparc-methodology/SKILL.md
│   ├── storage-sync/SKILL.md
│   ├── stream-chain/SKILL.md
│   ├── swarm-advanced/SKILL.md
│   ├── swarm-orchestration/SKILL.md
│   ├── task-decomposition/SKILL.md
│   ├── test-fix/SKILL.md
│   ├── test-optimization/
│   │   ├── SKILL.md
│   │   └── resources/
│   ├── test-runner/SKILL.md
│   ├── v3-cli-modernization/SKILL.md
│   ├── v3-core-implementation/SKILL.md
│   ├── v3-ddd-architecture/SKILL.md
│   ├── v3-integration-deep/SKILL.md
│   ├── v3-mcp-optimization/SKILL.md
│   ├── v3-memory-unification/SKILL.md
│   ├── v3-performance-optimization/SKILL.md
│   ├── v3-security-overhaul/SKILL.md
│   ├── v3-swarm-coordination/SKILL.md
│   ├── verification-quality/SKILL.md
│   └── web-search-researcher/
│       ├── SKILL.md
│       └── (4 supporting files)
│
└── [Backups - Temporary]
    ├── .claude/skills.backup.20260213_094200/
    └── .opencode/skill.backup.20260213_094200/

Total: 72 skills
```

## Symlink Architecture

```
Symlink Strategy (Relative)
============================

.agents/skills/agent-coordination/SKILL.md
  └─→ ../../.claude/skills/agent-coordination/SKILL.md

.agents/skills/build-rust/SKILL.md
  └─→ ../../.opencode/skill/build-rust/SKILL.md

Benefits:
✓ Portable (works across machines)
✓ Maintains single source of truth
✓ Allows independent updates
✓ Easy to break for migration

Future Migration:
1. Copy file content (break symlink)
2. Update content
3. Remove source directory
```

## Merge Workflow for Rich Skills

```
Rich Skill Merge Process (Example: analysis-swarm)
===================================================

Step 1: Current State After Consolidation
------------------------------------------
.agents/skills/analysis-swarm/
├── SKILL.md                (41 lines, from .claude)
├── discourse.md            (from .claude)
├── examples.md             (from .claude)
├── orchestration.md        (from .claude)
├── personas.md             (from .claude)
└── MERGE_NOTES.md          (instructions)

.opencode/skill/analysis-swarm/
└── SKILL.md                (515 lines, comprehensive)

Step 2: Manual Merge
--------------------
1. Read .opencode/skill/analysis-swarm/SKILL.md
   - Contains all content inline
   - 515 lines of comprehensive documentation

2. Identify sections:
   - Personas (RYAN, FLASH, SOCRATES)
   - Orchestration protocol
   - Response format
   - Use case examples
   - etc.

3. Map to .claude structure:
   - Personas → personas.md
   - Orchestration → orchestration.md
   - Examples → examples.md
   - Discourse → discourse.md

4. Merge content:
   - Enhance personas.md with detailed definitions
   - Add examples to examples.md
   - Update orchestration.md
   - Enhance discourse.md

5. Update SKILL.md:
   - Keep as entry point
   - Add cross-references
   - Include quick reference

Step 3: Final State
-------------------
.agents/skills/analysis-swarm/
├── SKILL.md                (merged entry point)
├── discourse.md            (enhanced with .opencode content)
├── examples.md             (enhanced with .opencode content)
├── orchestration.md        (enhanced with .opencode content)
├── personas.md             (enhanced with .opencode content)
└── MERGE_NOTES.md          (DELETE when complete)

Step 4: Validation
-----------------
✓ All internal links work
✓ Content preserved from both sources
✓ Skill loads correctly
✓ No duplicate content

Time: 30-45 minutes per skill
```

## Timeline

```
Consolidation Timeline
======================

Phase 1: Execution (Day 1)
---------------------------
☐ Run consolidation script (5 min)
☐ Review inventory (5 min)
☐ Create merge plan (15 min)

Phase 2: Manual Merges (Days 2-3)
----------------------------------
☐ analysis-swarm (45 min)
☐ architecture-validation (45 min)
☐ debug-troubleshoot (45 min)
☐ feature-implement (45 min)
☐ github-workflows (45 min)
☐ goap-agent (60 min)

Total: ~5 hours

Phase 3: Validation (Day 3)
----------------------------
☐ Run validation script
☐ Fix any issues
☐ Test skill loading
☐ Verify all 72 skills accessible

Phase 4: Migration (6 months later)
------------------------------------
☐ Break symlinks
☐ Copy all content
☓ Deprecate source directories
☐ Update references

Phase 5: Cleanup (12 months later)
------------------------------------
☐ Remove .claude/skills/
☐ Remove .opencode/skill/
☐ Single source: .agents/skills/
```

## Risk Assessment

```
Risk Analysis
=============

Risk: Breaking existing functionality
Impact: HIGH
Probability: LOW
Mitigation: ✓ Backups created
         ✓ Original directories preserved
         ✓ Easy rollback
         ✓ Symlinks optional

Risk: Content loss during merge
Impact: HIGH
Probability: LOW
Mitigation: ✓ Side-by-side comparison
         ✓ MERGE_NOTES.md guides process
         ✓ Source files unchanged
         ✓ Manual review required

Risk: Symlinks break across systems
Impact: MEDIUM
Probability: LOW
Mitigation: ✓ Use relative symlinks
         ✓ Test on target systems
         ✓ Can skip symlinks entirely

Risk: Time-consuming manual merges
Impact: LOW
Probability: HIGH (certain)
Mitigation: ✓ Only 6 skills require merge
         ✓ Clear merge instructions
         ✓ Can be done incrementally
         ✓ ~5 hours total

Overall Risk: LOW (with backups and rollback plan)
```

## Success Metrics

```
Success Criteria
=================

Structural:
☑ 72 skills in .agents/skills/
☑ All skills have SKILL.md
☑ Zero broken symlinks
☑ All MERGE_NOTES.md resolved

Functional:
☑ Skills load correctly with Skill tool
☑ All internal links work
☑ Frontmatter valid for all SKILL.md
☑ No content loss from sources

Quality:
☑ Improved organization
☑ Single canonical location
☑ Clear documentation
☑ Future-proof structure

Validation:
☑ ./scripts/validate-consolidation.sh passes
☑ Sample skills tested
☑ User acceptance testing complete
```

## File Sizes

```
Storage Requirements
====================

Current (Before):
.opencode/skill/:     ~2 MB  (18 skills, single files)
.claude/skills/:      ~15 MB (69 skills, some multi-file)
Total:                ~17 MB

After Consolidation:
.agents/skills/:      ~17 MB (72 skills, copies)
.backups/:            ~17 MB (duplicates)
Total:                ~34 MB (temporary)

After Migration (6 months):
.agents/skills/:      ~17 MB (72 skills, only copy)
.backups/:            ~0 MB  (removed)
Total:                ~17 MB

Peak Usage: ~34 MB (during transition)
Final Usage: ~17 MB (after cleanup)
```

This visual overview provides a complete picture of the consolidation process, from current state through execution to final migration.
