# Playbooks and Checkpoints: Actionable Memory

This guide explains how to use the Playbook and Checkpoint capabilities. These features transform passive episodic memory into active, actionable guidance and state management.

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
do-memory-cli playbook recommend "Implement a JWT authentication flow" --domain security
```

---

## 2. Episode Checkpoints and Handoff

Checkpoints allow you to save the state of a long-running task. This is required for:
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
  "reason": "Long-running task pause",
  "note": "Setup Complete"
}
```

**Via CLI:**
```bash
do-memory-cli episode checkpoint <episode-id> --reason "Long-running task pause" --note "Setup Complete"
```

### Performing a Handoff
When you need to pass a task to another agent, generate a "Handoff Pack".

**Via CLI (compact by default, max 8 KB / ~2k tokens):**
```bash
do-memory-cli episode handoff <checkpoint-id>
do-memory-cli episode handoff <checkpoint-id> --max-bytes 4096
do-memory-cli episode handoff <checkpoint-id> --full   # unbounded, audit/debug
do-memory-cli episode resume <checkpoint-id>           # resumes the compact pack
```

**Via MCP (`get_handoff_pack`, compact by default):**
```json
{
  "checkpoint_id": "checkpoint-id-from-checkpoint_episode",
  "mode": "compact",
  "max_bytes": 4096
}
```
```json
{
  "checkpoint_id": "checkpoint-id-from-checkpoint_episode",
  "mode": "full"
}
```
Resume a compact pack with `resume_from_compact` (takes `compact_handoff`);
resume a full pack with `resume_from_handoff` (takes `handoff_pack`).

The compact pack carries the objective, status/progress, verified findings,
decisions, pending actions, ID-only pattern/heuristic references, and the most
recent step excerpts — plus an `omitted` receipt with exact counts of everything
left out and a pointer to the full pack (`get_handoff_pack` in `full` mode).

---

## 3. Recommendation Feedback

To improve the quality of future recommendations, you can provide feedback on the playbooks and patterns suggested by the system.

### Providing Feedback
**Via MCP (`record_recommendation_feedback`):**
```json
{
  "session_id": "session-id-from-record_recommendation_session",
  "applied_pattern_ids": ["pattern-123"],
  "consulted_episode_ids": ["episode-456"],
  "outcome": "success",
  "message": "The middleware step was exactly what I needed.",
  "agent_rating": 0.9
}
```

**Via CLI:**
```bash
do-memory-cli feedback record-session --episode-id <episode-id> --patterns <pattern-id-1,pattern-id-2>
do-memory-cli feedback record-feedback --session <session-id> --outcome success --message "Worked well" --rating 0.9
```

The system records this feedback as attribution statistics (adoption rate,
success-after-adoption rate, and precision metrics). It does not yet use
feedback to boost or suppress the ranking of future pattern recommendations —
feedback-to-ranking adaptation remains deferred to a follow-up ADR.
