# Playbooks and Checkpoints: Actionable Memory

This guide explains how to use the new Playbook and Checkpoint features introduced in v0.1.22. These features transform passive episodic memory into active, actionable guidance and state management.

## 1. Actionable Playbooks

Playbooks are step-by-step guides generated from successful past episodes. Instead of just showing you *what* was done, a playbook tells you *how* to do it again.

### How it Works
1.  **Pattern Extraction**: The system identifies successful tool sequences and decision points from your history.
2.  **Synthesis**: Multiple similar successful episodes are synthesized into a coherent set of instructions.
3.  **Refinement**: Steps are ordered and deduplicated to create a clean "recipe" for the task.

### Using Playbooks via MCP
Use the `recommend_playbook` tool to get guidance for a new task:

```json
{
  "query": "Implement a JWT authentication flow in Rust",
  "domain": "security",
  "task_type": "code_generation",
  "max_playbooks": 1,
  "max_steps": 10
}
```

### Using Playbooks via CLI
```bash
memory-cli playbook recommend "Implement a JWT authentication flow" --domain security
```

---

## 2. Episode Checkpoints and Handoff

Checkpoints allow you to save the state of a long-running task. This is essential for:
- **Resuming Tasks**: If an agent hits a token limit or needs to restart.
- **Agent Handoff**: Passing a complex task from one specialized agent to another.
- **Branching**: Trying different approaches from the same starting point.

### Creating a Checkpoint
A checkpoint captures the current state, including:
- Task progress
- Key findings
- Pending actions
- Required context

**Via MCP:**
```json
{
  "episode_id": "uuid-of-current-episode",
  "checkpoint_name": "Setup Complete",
  "state_summary": "Database schema initialized, models created.",
  "pending_actions": ["Implement CRUD handlers", "Add validation middleware"]
}
```

**Via CLI:**
```bash
memory-cli ep checkpoint <episode-id> --name "Setup Complete" --summary "..."
```

### Performing a Handoff
When you need to pass a task to another agent, generate a "Handoff Pack":

```bash
memory-cli ep handoff <episode-id>
```

The handoff pack includes the most recent checkpoint, the episode history, and relevant patterns to help the new agent start with full context.

---

## 3. Recommendation Feedback

To improve the quality of future recommendations, you can provide feedback on the playbooks and patterns suggested by the system.

### Providing Feedback
**Via MCP (`provide_feedback`):**
```json
{
  "recommendation_id": "uuid-from-playbook-or-pattern",
  "relevance_score": 0.9,
  "is_useful": true,
  "comment": "The step about middleware was exactly what I needed."
}
```

**Via CLI:**
```bash
memory-cli feedback add <recommendation-id> --score 0.9 --useful true --comment "..."
```

The system uses this feedback to boost the ranking of highly-rated patterns and suppress unhelpful ones in future queries.
