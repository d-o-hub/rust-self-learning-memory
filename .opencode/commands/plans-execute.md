---
description: Execute plans/ folder with swarm of specialist agents, handoff coordination, atomic commits, CI loop until green
agent: general
subtask: true
---

# GOAP Plans Execution Command

Execute actionable plans in `plans/` using swarm coordination with specialist agents.

## Phase 0: Prerequisites & Safety Checks

### Step 0.1: Verify Tool Availability
Check all required tools before starting:
```bash
# Required tools
command -v git && command -v cargo && command -v gh
cargo nextest --version
rustup component list --installed | grep -E 'rustfmt|clippy'
gh auth status
```

If any tool is missing, report and stop. Do NOT proceed without prerequisites.

### Step 0.2: Workspace State Check
```bash
# Ensure clean working tree and not on protected branch
git status --porcelain
git branch --show-current
git fetch origin
```

**Rules**:
- NEVER work directly on `main` or `master` — create or switch to a feature branch
- If working tree is dirty with unrelated changes, warn the user before proceeding
- Check if behind upstream: `git log HEAD..origin/$(git branch --show-current) --oneline`

### Step 0.3: Discover Workspace Members
Read workspace crate list dynamically from `Cargo.toml`:
```bash
grep -A 20 '^\[workspace\]' Cargo.toml | grep '"' | tr -d ' ",'
```
Use these as valid `module` names for commits and targeted test runs.

## Phase 1: Discovery & Analysis

### Step 1.1: Discover Active Plans
Dynamically find plans — never hardcode filenames:

1. **Read plan index** (find the canonical one):
   ```bash
   ls plans/INDEX.md plans/**/INDEX.md 2>/dev/null
   ```

2. **Find active roadmap**:
   ```bash
   ls plans/ROADMAPS/ROADMAP_ACTIVE.md plans/ROADMAPS/ROADMAP*.md 2>/dev/null
   ```

3. **Find newest GOAP execution plan**:
   ```bash
   ls -t plans/GOAP_EXECUTION_PLAN_*.md plans/GOAP_*_EXECUTION_*.md 2>/dev/null | head -5
   ```

4. **Discover ADRs**:
   ```bash
   ls plans/adr/ADR-*.md 2>/dev/null
   ls plans/ARCHITECTURE/*.md 2>/dev/null
   ```

5. **Exclude stale/archived plans**:
   - Skip everything under `plans/archive/`
   - Skip `plans/research/` unless explicitly requested
   - Flag plans not referenced by the active roadmap or index

Read the discovered files, starting with the index and active roadmap.

### Step 1.2: Identify Pending Tasks
From the discovered plans, extract:
1. **P0 Critical** — blocking releases or CI
2. **P1 High priority** — user-facing impact
3. **P2 Medium priority** — quality of life
4. Tasks with dependencies already met (ready to start)

Present a summary and **wait for confirmation** before executing. Never auto-execute all plans blindly.

### Step 1.3: Plan Hygiene Validation
Before executing any plan, verify it contains:
- [ ] Clear objective / goal statement
- [ ] Validation criteria / success metrics
- [ ] Risk assessment
- [ ] Rollback strategy

Warn (non-blocking) if a plan is missing these sections.
Verify that any file paths, script references, or ADR links in the plan actually exist.

## Phase 2: ADR Compliance Check

### Step 2.1: Dynamic ADR Discovery
```bash
ls plans/adr/ADR-*.md 2>/dev/null
```

Do NOT hardcode ADR numbers. Instead:
- Match ADR titles/filenames to the current task domain keywords
- Read ADRs referenced in the selected plan files (grep for `ADR-` links)
- Always read "baseline" ADRs when touching their domain:
  - **Build/disk** → search for ADRs mentioning "disk", "build", "optimization"
  - **Testing** → search for ADRs mentioning "test", "CI", "stability"
  - **Release** → search for ADRs mentioning "release", "semver", "engineering"
  - **Dependencies** → search for ADRs mentioning "dependency", "deduplication"

### Step 2.2: Apply ADR Constraints
- Note architectural decisions and constraints from matching ADRs
- Check ADR status (Accepted/Active vs Deprecated/Superseded)
- If a new architectural decision is made during execution → create a new ADR

## Phase 3: Swarm Execution Strategy

### Step 3.1: Dynamic Skill Discovery
Discover available skills from the codebase:
```bash
ls .agents/skills/*/SKILL.md 2>/dev/null | sed 's|.agents/skills/||;s|/SKILL.md||'
```

Map tasks to skills by domain keyword matching. Use the `task-decomposition` skill for complex task breakdowns.

### Step 3.2: Agent Groups (Spawn Based on Task Needs)

**Group A: Code Quality Swarm**
- Skill: `code-quality` → Format, lint, clippy fixes
- Skill: `rust-code-quality` → Rust-specific quality review
- Skill: `quality-unit-testing` → Test quality assessment
- Skill: `clean-code-developer` → Code standards enforcement
- Script: `./scripts/code-quality.sh`

**Group B: Testing Swarm**
- Skill: `test-runner` → Run tests, report failures
- Skill: `test-fix` → Diagnose and fix test failures
- Skill: `test-optimization` → Optimize test performance
- Skill: `rust-async-testing` → Async test patterns
- Skill: `episodic-memory-testing` → Domain-specific tests
- Script: `./scripts/check-doctests.sh`

**Group C: CI/CD Swarm**
- Skill: `ci-fix` → Diagnose and fix CI failures
- Skill: `github-workflows` → Workflow optimization
- Skill: `yaml-validator` → Validate workflow YAML
- Skill: `release-guard` → Release safety gates

**Group D: Architecture Swarm**
- Skill: `architecture-validation` → Validate against plans
- Skill: `plan-gap-analysis` → Identify implementation gaps
- Skill: `codebase-analyzer` → Trace data flow
- Skill: `codebase-locator` → Find implementation files
- Skill: `analysis-swarm` → Multi-perspective analysis

**Group E: Feature Swarm (if plan requires new features)**
- Skill: `feature-implement` → Implement new features
- Skill: `debug-troubleshoot` → Fix runtime issues
- Skill: `memory-mcp` → MCP server work
- Skill: `memory-cli-ops` → CLI operations
- Skill: `storage-sync` → Storage layer sync

**Group F: Coordination Swarm**
- Skill: `goap-agent` → Complex multi-step planning
- Skill: `agent-coordination` → Multi-agent orchestration
- Skill: `task-decomposition` → Task breakdown
- Skill: `parallel-execution` → Parallel task management
- Skill: `loop-agent` → Iterative refinement

Only activate groups relevant to the selected tasks. Not all groups run every time.

### Step 3.3: Handoff Coordination Pattern

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  Discovery       │────→│  Planning        │────→│  Execution       │
│  (dynamic)       │     │  (task-decomp)   │     │  (specialists)   │
└──────────────────┘     └──────────────────┘     └──────────────────┘
                                                         │
         ┌───────────────────────────────────────────────┘
         ▼
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  Validation      │────→│  Commit          │────→│  CI Monitor      │
│  (quality gates) │     │  (atomic)        │     │  (loop-agent     │
└──────────────────┘     └──────────────────┘     │   until green)   │
                                                  └──────────────────┘
```

## Phase 4: Self-Learning Episode Integration

### Step 4.1: Start Episode
Before executing any task group, start a learning episode:
```
Skill: episode-start
Context: {language: "rust", domain: "<task-domain>", tags: ["goap", "plan-execution"]}
Description: "Plan execution: <task summary>"
```

### Step 4.2: Log Milestones
At each significant milestone, log the step:
```
Skill: episode-log-steps
Log: decision made, change applied, validation result, CI outcome
```

### Step 4.3: Retrieve Past Context
Before starting complex tasks, check for relevant past experience:
```
Skill: context-retrieval
Query: similar tasks, past decisions, known patterns
```

## Phase 5: Execution Loop

### Step 5.1: Execute Tasks
For each selected pending task:
1. Load appropriate skill(s) from `.agents/skills/`
2. Load relevant scripts from `scripts/`
3. Spawn specialist agent with `Task` tool (only for parallelizable work)
4. Prefer doing sequential work directly (better context retention)
5. Monitor progress and capture results
6. Hand off to next agent in sequence

### Step 5.2: Quality Gates (Before Commit)
Run ALL quality checks. Prefer scripts when they exist:
```bash
# Build (use project script)
./scripts/build-rust.sh check

# Format
./scripts/code-quality.sh fmt

# Lint
cargo clippy --all -- -D warnings

# Full build
cargo build --all

# Tests (nextest for unit/integration)
cargo nextest run --all

# Doctests (nextest does NOT run these — required separately)
cargo test --doc

# Full quality gates
./scripts/quality-gates.sh
```

**Additional checks based on changed area**:
- If MCP touched → `./scripts/diagnose-mcp.sh` or `./scripts/test-mcp-tools.sh`
- If CLI touched → `./scripts/verify_cli_usage.sh`
- If storage touched → `./scripts/verify_storage_backends.sh`
- If unwrap policy relevant → `./scripts/check-unwrap-production.sh`
- If YAML/workflows touched → use `yaml-validator` skill
- If performance-sensitive → `./scripts/check_performance_regression.sh`

### Step 5.3: Atomic Commit
If all quality gates pass:
```bash
git status
git diff --stat
```

Create atomic commit following conventions:

**Format**: `type(module): description`

| Type | Use |
|------|-----|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code restructuring |
| `test` | Test changes only |
| `docs` | Documentation only |
| `chore` | Build/tooling/config |
| `ci` | CI/CD changes |

**Module**: Must be a workspace member (from Step 0.3) or one of: `ci`, `docs`, `plans`, `scripts`

**Commit body** should include when applicable:
- `Plan: plans/<relevant-plan>.md`
- `ADR: ADR-XXX`

Stage only files directly related to the current task — **never** `git add -A` or `git add .`.

### Step 5.4: Push & Monitor CI
```bash
git push origin $(git branch --show-current)
```

Check CI status:
```bash
gh run list --limit 5
```

### Step 5.5: CI Fix Loop
If CI fails, use the `loop-agent` pattern with a retry cap:

1. Use `ci-fix` skill to diagnose failure
2. Apply fix
3. Re-run quality gates (Step 5.2)
4. Commit and push
5. Check CI again
6. **Max 3 retries** — if still failing after 3 attempts, stop and report the issue

### Step 5.6: Rollback Strategy
If a change causes cascading failures:
1. `git revert <commit>` for the problematic commit
2. Push the revert
3. Document the failure in the episode log
4. Re-analyze the task with different approach

## Phase 6: Self-Learning Completion

### Step 6.1: Complete Episode
After task execution (success or failure):
```
Skill: episode-complete
Score based on: goal achievement, efficiency, quality gate results, CI outcome
```

### Step 6.2: Analyze Patterns
1. Identify what worked well
2. Identify what failed or was slow
3. If a pattern was used 3+ times → consider creating a new skill
4. If new skill needed → use `skill-creator` skill

### Step 6.3: Update Memory
If MCP server is available, optionally persist:
- Execution summary
- Decisions made
- Metrics snapshot
- Links to commits and PRs

## Phase 7: Update Plans

After completion:
1. Update task status in the active roadmap
2. Create ADR if an architectural decision was made
3. Update `plans/INDEX.md` with progress
4. Document any blockers, decisions, or follow-up tasks
5. Archive completed plan documents if fully done

## Execution Summary Format

Return a summary containing:
1. **Tasks Completed**: List with plan references
2. **Tasks Skipped/Blocked**: With reasons
3. **Commits Made**: Count, hashes, and messages
4. **Quality Gate Results**: Pass/fail per gate
5. **CI Status**: Final workflow results
6. **Skills Used**: Which skills were effective
7. **Scripts Used**: Which scripts were run
8. **Episode ID**: Learning episode reference
9. **ADRs Referenced/Created**: List
10. **New Skills Created**: Any new patterns documented
11. **Next Steps**: Recommended follow-up tasks

## Constraints

- NEVER force push to main/master
- NEVER work directly on main/master — use feature branches
- NEVER skip quality gates
- NEVER commit secrets or credentials
- NEVER auto-execute all plans — require confirmation after discovery
- ALWAYS use parameterized SQL
- ALWAYS follow ADR constraints
- ALWAYS update relevant `plans/` files after changes
- ALWAYS run doctests separately (`cargo test --doc`)
- ALWAYS stage only related files (never `git add -A`)
- MAX 3 CI fix retries before stopping
- MAX 1 task in_progress at a time per agent group
