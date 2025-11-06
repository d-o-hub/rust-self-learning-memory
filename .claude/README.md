# Claude Code Setup for Rust Self-Learning Memory

This directory contains Claude Code configuration for AI-assisted development of the Rust self-learning memory project.

## Overview

This Claude Code setup provides:
- **Skills**: Reusable knowledge modules for common development tasks
- **Agents**: Specialized sub-agents for focused workflows
- **Commands**: Quick slash commands for common operations
- **Hooks**: Automated quality checks and workflows

## Directory Structure

```
.claude/
├── README.md              # This file
├── skills/                # Reusable knowledge modules
│   ├── episode-start.md
│   ├── episode-complete.md
│   ├── episode-log-steps.md
│   ├── storage-sync.md
│   ├── context-retrieval.md
│   ├── test-runner.md
│   ├── test-fix.md
│   ├── build-compile.md
│   ├── code-quality.md
│   ├── feature-implement.md
│   ├── debug-troubleshoot.md
│   ├── task-decomposition/
│   │   └── SKILL.md
│   ├── agent-coordination/
│   │   └── SKILL.md
│   ├── parallel-execution/
│   │   └── SKILL.md
│   └── skill-creator/
│       └── SKILL.md
├── agents/                # Specialized sub-agents
│   ├── test-runner.md
│   ├── code-reviewer.md
│   ├── feature-implementer.md
│   ├── refactorer.md
│   ├── debugger.md
│   ├── goap-agent.md
│   └── agent-creator.md
└── commands/              # Slash commands
    ├── test.md
    ├── build.md
    ├── check-quality.md
    ├── pre-commit.md
    ├── implement-feature.md
    ├── fix-tests.md
    ├── debug-issue.md
    ├── refactor.md
    └── review-code.md
```

## Skills

Skills are reusable knowledge modules that provide detailed guidance for specific tasks.

**Format**: Skills are organized as directories containing a `SKILL.md` file with YAML frontmatter:
```yaml
---
name: skill-name
description: What this skill does and when to use it
---
```

Claude autonomously invokes skills based on the description when relevant to the task.

### Episode Management
- **episode-start**: Start a new learning episode
- **episode-complete**: Complete and score an episode
- **episode-log-steps**: Log execution steps during episodes

### Storage Operations
- **storage-sync**: Sync between Turso and redb
- **context-retrieval**: Retrieve relevant episodic context

### Development
- **test-runner**: Run and manage tests
- **test-fix**: Diagnose and fix test failures
- **build-compile**: Build and compilation guidance
- **code-quality**: Code quality tools and standards
- **feature-implement**: Systematic feature implementation
- **debug-troubleshoot**: Debug runtime issues

### Planning & Coordination
- **task-decomposition**: Break down complex tasks into atomic, actionable goals
- **agent-coordination**: Coordinate multiple agents through various execution strategies
- **parallel-execution**: Manage parallel agent execution with synchronization

### Meta Development
- **skill-creator**: Create new Claude Code skills with proper format and structure

## Agents

Specialized agents handle complex, multi-step tasks autonomously. Each agent operates in its own context window with a custom system prompt and tool access.

**Format**: Agents are Markdown files with YAML frontmatter:
```yaml
---
name: agent-name
description: When to invoke this agent
tools: Tool1, Tool2, Tool3  # Optional, inherits all tools if omitted
model: sonnet  # Optional
---
```

Agents are invoked explicitly using the Task tool when their specialized capabilities are needed.

### Test Runner Agent
**Purpose**: Execute tests and fix failures

**Usage**: Invoke when you need to:
- Run full test suite
- Debug failing tests
- Verify test coverage

**Capabilities**:
- Run unit, integration, and doc tests
- Diagnose async/await issues
- Fix race conditions
- Verify fixes

### Code Reviewer Agent
**Purpose**: Review code for quality and correctness

**Usage**: Invoke when you need to:
- Review code changes before commit
- Check adherence to project standards
- Verify test coverage and documentation

**Capabilities**:
- Run quality checks (fmt, clippy)
- Review architecture and design
- Check correctness and performance
- Verify AGENTS.md compliance

### Feature Implementer Agent
**Purpose**: Implement new features systematically

**Usage**: Invoke when you need to:
- Add new functionality
- Create new modules
- Integrate features into main API

**Capabilities**:
- Design feature architecture
- Implement with proper structure
- Add comprehensive tests
- Write documentation
- Verify quality

### Refactorer Agent
**Purpose**: Improve code quality through refactoring

**Usage**: Invoke when you need to:
- Split large files (> 500 LOC)
- Extract duplicate code
- Simplify complex functions
- Optimize performance

**Capabilities**:
- Systematic refactoring
- Preserve functionality
- Maintain test coverage
- Improve maintainability

### Debugger Agent
**Purpose**: Diagnose and fix runtime issues

**Usage**: Invoke when you need to:
- Debug production issues
- Fix performance problems
- Resolve deadlocks or race conditions

**Capabilities**:
- Reproduce issues
- Add instrumentation
- Diagnose root cause
- Apply and verify fixes

### GOAP Agent (Goal-Oriented Action Planning)
**Purpose**: Intelligent task planning and multi-agent coordination

**Usage**: Invoke when you need to:
- Plan and coordinate complex multi-step tasks
- Optimize execution through parallel/sequential/swarm strategies
- Manage multiple specialized agents for comprehensive workflows
- Dynamic task distribution and resource optimization

**Capabilities**:
- **Goal Decomposition**: Break complex tasks into atomic, actionable goals
- **Dependency Mapping**: Identify task relationships and optimal execution sequences
- **Multi-Agent Coordination**: Orchestrate specialized agents through parallel, sequential, swarm, or hybrid execution
- **Quality Assurance**: Implement validation checkpoints and success criteria monitoring
- **Dynamic Optimization**: Adjust plans based on real-time execution feedback
- **Result Synthesis**: Aggregate outputs from multiple agents into comprehensive deliverables

**Coordination Strategies**:
- **Parallel Execution**: Independent tasks run simultaneously for maximum throughput
- **Sequential Execution**: Dependent tasks with proper handoffs and quality gates
- **Swarm Coordination**: Multiple perspectives for complex problem solving
- **Hybrid Execution**: Mixed strategies optimized for complex workflows

**Example Invocations**:
- "Plan and coordinate implementation of batch update feature with full testing and documentation"
- "Organize a comprehensive pre-release quality audit using multiple agents"
- "Coordinate debugging and fixing of performance degradation across the system"
- "Plan and execute phased refactoring of storage layer with minimal disruption"

### Agent Creator
**Purpose**: Create new Claude Code agents with proper format and structure

**Usage**: Invoke when you need to:
- Build a new specialized sub-agent
- Create autonomous task executors
- Extend Claude Code capabilities
- Design agents with custom system prompts and tool access

**Capabilities**:
- **Proper Formatting**: Creates agents with correct YAML frontmatter (name, description, tools)
- **System Prompt Design**: Writes comprehensive, focused system prompts
- **Tool Selection**: Recommends appropriate tool access based on agent purpose
- **Template Library**: Provides templates for execution, analysis, and coordination agents
- **Integration Guidance**: Documents how agent works with skills and other agents
- **Validation**: Ensures agent follows all naming and format conventions

**When to Create an Agent**:
- Task requires complex multi-step execution
- Need isolated context window separate from main agent
- Task benefits from custom system prompt and specialized focus
- Different tool access requirements than main agent
- Autonomous execution provides significant value

**Example Invocations**:
- "Create a new agent for deploying Rust applications to production"
- "Build an agent specialized in GraphQL API integration and testing"
- "Design a security audit agent for reviewing code vulnerabilities"
- "Create a documentation generation agent for Rust projects"

## Slash Commands

Quick commands for common workflows.

### Testing
- `/test` - Run all tests
- `/fix-tests [name]` - Fix failing tests

### Building
- `/build` - Build entire project
- `/check-quality` - Run all quality checks
- `/pre-commit` - Pre-commit verification

### Development
- `/implement-feature [desc]` - Implement new feature
- `/refactor [target]` - Refactor code
- `/debug-issue [desc]` - Debug an issue
- `/review-code [path]` - Review code changes

## Common Workflows

### Creating New Claude Code Agents and Skills

```bash
# Creating a new skill
User: "I need a skill for GraphQL schema validation"

# Claude will use skill-creator skill to:
# 1. Create proper directory structure (.claude/skills/graphql-schema-validation/)
# 2. Generate SKILL.md with YAML frontmatter
# 3. Write comprehensive content with examples
# 4. Validate format and naming conventions

# Creating a new agent
User: "Create an agent for managing Docker deployments"

# Invoke agent-creator explicitly:
# 1. Define agent purpose and scope
# 2. Generate YAML frontmatter (name, description, tools)
# 3. Write focused system prompt
# 4. Include templates and examples
# 5. Document integrations with other agents/skills
# 6. Validate format and test agent
```

### Complex Multi-Step Tasks with GOAP Agent

```bash
# Use GOAP agent for complex tasks requiring coordination
User: "Implement comprehensive caching system with performance benchmarks, tests, and documentation"

# GOAP agent will:
# 1. Decompose task into atomic goals
# 2. Map dependencies and create execution plan
# 3. Coordinate multiple agents (feature-implementer, test-runner, code-reviewer)
# 4. Execute in optimal strategy (hybrid: parallel analysis → sequential implementation → parallel validation)
# 5. Aggregate results and provide comprehensive report

The GOAP agent handles the entire coordination autonomously.
```

### Starting a New Feature

```bash
# Option 1: Use slash command
/implement-feature Add pattern caching to improve retrieval performance

# Option 2: Manual steps
1. Read feature-implement skill
2. Plan architecture
3. Implement following project conventions
4. Add tests
5. Run /check-quality
6. Commit
```

### Fixing Failing Tests

```bash
# Option 1: Use slash command
/fix-tests

# Option 2: Invoke test-runner agent
Invoke test-runner agent to diagnose and fix all failing tests

# Option 3: Manual
1. Run cargo test --all
2. Identify failures
3. Use test-fix skill to diagnose
4. Apply fixes
5. Verify
```

### Code Review Before Commit

```bash
# Use pre-commit command
/pre-commit

# Or invoke code-reviewer agent
Invoke code-reviewer agent to review all changes since last commit
```

### Debugging Runtime Issue

```bash
# Use debug command
/debug-issue Program hangs when processing concurrent episodes

# Or invoke debugger agent
Invoke debugger agent to debug deadlock in concurrent episode processing
```

### Refactoring Large File

```bash
# Use refactor command
/refactor Split patterns.rs into submodules

# Or invoke refactorer agent
Invoke refactorer agent to split patterns.rs (650 LOC) into submodules
```

## Hooks Configuration

### Recommended Hooks

Configure in your local Claude Code settings:

#### Pre-Commit Hook
```bash
# Run before every commit
cargo fmt && cargo clippy --all -- -D warnings && cargo test --all
```

#### Pre-Push Hook
```bash
# Run before every push
cargo build --release && cargo test --all && cargo doc --no-deps
```

#### Tool-Use Hook (Edit)
```bash
# Run after file edits
cargo check
```

### Setting Up Hooks

1. Open Claude Code settings
2. Navigate to Hooks configuration
3. Add hooks for:
   - `pre-commit`: Quality checks before commit
   - `pre-push`: Full build before push
   - `tool-use-edit`: Quick check after file changes

## Project Standards

All code must follow standards defined in `AGENTS.md`:

- **File size**: ≤ 500 LOC per file
- **Error handling**: Use `anyhow::Result` for top-level functions
- **Async/Tokio**: All I/O operations are async
- **Storage**: Turso (durable) + redb (cache)
- **Testing**: Unit tests + integration tests (>80% coverage)
- **Documentation**: All public APIs documented
- **Code quality**: Pass `cargo fmt` and `cargo clippy`

## Best Practices

### When to Use Skills
- Learning how to do something
- Following established patterns
- Reference for best practices

### When to Use Agents
- Complex multi-step tasks
- Need systematic approach
- Want autonomous execution

### When to Use Commands
- Quick common operations
- Workflow automation
- Consistent task execution

## Examples

### Example 1: Implement New Feature

```
User: Add batch pattern update functionality

Claude: I'll implement this feature systematically using the feature-implementer agent.

[Invokes feature-implementer agent]

Agent:
1. Requirements: Batch update patterns in single transaction
2. Design: Add save_batch method to PatternStorage
3. Implementation:
   - Created src/patterns/batch.rs (245 LOC)
   - Added batch_save_turso and batch_save_redb methods
   - Implemented transaction handling
4. Tests: Added 8 unit tests, 2 integration tests
5. Documentation: Full API docs with examples
6. Quality: All checks pass ✓

Ready for review and commit.
```

### Example 2: Fix Failing Tests

```
User: /fix-tests

Claude: Running test suite...

[Invokes test-runner agent]

Agent:
Found 3 failing tests:
1. test_episode_creation - Missing .await on line 45
2. test_concurrent_writes - Race condition in pattern update
3. test_invalid_input - Wrong error type assertion

Fixing...
1. ✓ Added .await
2. ✓ Added Mutex for thread-safety
3. ✓ Updated error type assertion

All tests passing ✓ (45/45)
```

### Example 3: Code Review

```
User: /review-code

Claude: Reviewing recent changes...

[Invokes code-reviewer agent]

Agent: Code Review Report

Summary: Request Changes

Critical Issues:
- src/storage.rs:78 - Potential deadlock (lock held across await)
- src/patterns.rs:45 - unwrap() in library code

Suggestions:
- Consider batching database operations for better performance
- Add doc comment examples to main API functions

Tests: ✓ Good coverage
Documentation: ⚠ Missing examples

Please address critical issues before commit.
```

## Troubleshooting

### Skills Not Loading
- Ensure `.claude/skills/` directory exists
- Check file names end with `.md`
- Verify markdown format is correct

### Agents Not Available
- Ensure `.claude/agents/` directory exists
- Check agent files are in correct format
- Verify agent names are referenced correctly

### Commands Not Working
- Ensure `.claude/commands/` directory exists
- Check command file names match usage
- Verify command format is correct

## Contributing

When adding new skills, agents, or commands:

1. Follow existing format and structure
2. Include clear documentation
3. Add examples where helpful
4. Keep focused and specific
5. Test thoroughly

## References

- Project Guidelines: `AGENTS.md`
- Claude Code Docs: https://docs.claude.com/en/docs/claude-code
- Rust Self-Learning Memory: Main project README

## Support

For issues with:
- **Project code**: See main project README
- **Claude Code setup**: Check this README
- **Claude Code platform**: See official docs

---

This Claude Code setup is designed to accelerate development while maintaining high code quality and consistency with project standards.
