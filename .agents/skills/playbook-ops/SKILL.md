---
name: playbook-ops
description: Generate task playbooks and explain patterns using learned strategies from episodic memory
---

# Playbook Operations

## When to Use

- Generating step-by-step playbooks for a new task
- Getting actionable recommendations based on past success
- Explaining what a pattern means in human-readable form
- Understanding when/when-not to apply a strategy

## CLI Commands

| Command | Purpose |
|---------|---------|
| `do-memory-cli playbook recommend [TASK]` | Generate playbook for a task |
| `do-memory-cli playbook explain <PATTERN_ID>` | Explain a pattern |

## MCP Tools

| Tool | Parameters | Purpose |
|------|-----------|---------|
| `recommend_playbook` | task, domain, task_type, max_steps, language, framework, tags | Generate actionable playbook |
| `explain_pattern` | pattern_id | Human-readable pattern explanation |
| `search_patterns` | query, domain, limit | Find relevant patterns |
| `recommend_patterns` | task_type, domain, context | Get pattern recommendations |

## Generating a Playbook

```bash
# Basic recommendation
do-memory-cli playbook recommend "implement rate limiting middleware"

# With context
do-memory-cli playbook recommend "add caching layer" \
  --domain "rust-backend" \
  --task-type "code_generation" \
  --language "rust" \
  --framework "axum" \
  --max-steps 7
```

### Playbook Output Includes

- **Task match score**: How well the playbook fits your request
- **Confidence**: Based on past episode success rates
- **Steps**: Ordered actions with tool hints and expected results
- **Pitfalls**: Known failure modes to avoid
- **When to apply / When NOT to apply**: Applicability conditions
- **Expected outcome**: What success looks like

## Explaining Patterns

```bash
do-memory-cli playbook explain "pat-uuid-here"
```

Returns a natural-language explanation of:
- What the pattern does
- When it was discovered
- Success rate and contexts where it works

## Task Types

Valid `--task-type` values:
- `code_generation` — Building new features
- `debugging` — Finding and fixing bugs
- `refactoring` — Restructuring code
- `testing` — Writing or fixing tests
- `analysis` — Code review and analysis
- `documentation` — Writing docs

## Integration

Playbooks work best when the memory system has episodes:
1. Complete episodes with outcomes and steps
2. System extracts patterns from successful episodes
3. `recommend_playbook` uses patterns to generate actionable steps
4. Feedback loop (via `recommendation-feedback` skill) improves recommendations
