# OpenCode Configuration

This directory contains OpenCode-specific configuration for the Rust self-learning memory project.

## Structure

- `agent/` - Specialized agent configurations for autonomous task execution
- `skills/` - Skill definitions providing reusable knowledge and procedures for development tasks
- `.gitignore` - Git ignore patterns for OpenCode-related files

## Agents

Specialized agents that handle complex, multi-step tasks autonomously:

### Core Development Agents
- `analysis-swarm.md` - Multi-persona code analysis with RYAN, FLASH, and SOCRATES
- `architecture-validator.md` - Generic architecture validation against plan files
- `code-reviewer.md` - Code quality, correctness, and standards compliance review
- `debugger.md` - Runtime issue diagnosis and performance problem fixing
- `feature-implementer.md` - New feature design, implementation, testing, and integration

### Coordination Agents
- `goap-agent.md` - Goal-Oriented Action Planning for complex multi-agent coordination
- `loop-agent.md` - Iterative workflow execution for progressive refinement

### Specialized Agents
- `create-agent.md` - Create new opencode agents with proper format and configuration
- `git-worktree-manager.md` - Manage git worktrees for efficient multi-branch development
- `github-action-editor.md` - Edit and create GitHub Actions workflows
- `memory-mcp-tester.md` - Test memory-mcp server integration and functionality

### Research Agents
- `perplexity-researcher-pro.md` - Complex research with multi-step reasoning
- `perplexity-researcher-reasoning-pro.md` - High-stakes research requiring expert analysis
- `websearch-researcher.md` - Systematic web research for technical documentation

## Skills

Reusable knowledge and procedures for development tasks:

- `analysis-swarm.md` - Collective code intelligence framework
- `architecture-validation.md` - Dynamic architecture compliance validation
- `feature-implement.md` - Systematic feature implementation process

## Skill Source of Truth

Skills are defined in `.agents/skills/` and shared across all CLI tools.
Agent files in this directory reference those skills. See `.agents/skills/_consolidated/README.md`
for the consolidation history.

## Usage

These agents and skills are automatically available when using OpenCode in this project directory. Agents can be invoked directly by name, while skills provide supporting knowledge and procedures for various development workflows.

## Project Integration

All agents and skills are designed specifically for the Rust self-learning memory project, incorporating:
- AGENTS.md conventions (500 LOC limit, async patterns)
- Turso/libSQL and redb storage patterns
- Tokio async runtime usage
- Comprehensive error handling with `anyhow::Result`
- Security-first approach with input validation and parameterized queries