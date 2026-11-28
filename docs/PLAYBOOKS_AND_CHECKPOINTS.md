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

### Citing the evidence behind a step

Every emitted step names the pattern it was synthesized from, in `step_sources`,
keyed by step order:

```json
{
  "ordered_steps": [
    { "order": 1, "action": "Use read_file tool", "tool_hint": "read_file", "expected_result": "Execute read file operation" },
    { "order": 2, "action": "Evaluate: test_coverage < 80%", "tool_hint": null, "expected_result": "Then: add_more_tests" }
  ],
  "step_sources": [
    { "order": 1, "pattern_id": "0f1c8c1e-…", "episode_ids": [] },
    { "order": 2, "pattern_id": "7a2b4d90-…", "episode_ids": [] }
  ],
  "supporting_pattern_ids": ["0f1c8c1e-…", "7a2b4d90-…"],
  "supporting_episode_ids": ["…"]
}
```

`supporting_pattern_ids` lists every pattern the generator considered; `step_sources`
answers the narrower question *which pattern produced this step*. A step refused by
`max_steps` during synthesis has no entry — a truncated step is never cited.
`episode_ids` is filled only where the source pattern links episode evidence directly
(context patterns); tool sequences, decision points, and error recovery patterns carry a
pattern-only reference. Older records and older clients simply have no `step_sources`, and
an empty attribution is omitted from JSON instead of being written as an empty array.

Both surfaces expose the same mapping:

- MCP `recommend_playbook` — the `step_sources` field of each returned playbook.
- CLI `playbook recommend` — `source` on every entry of `steps` in JSON/YAML output, and a
  `Source pattern:` line in the human-readable rendering.

### End-to-end: recommend, cite a step, report feedback

```bash
EPISODE_ID=$(do-memory-cli episode start "Add retry to the HTTP client" --domain networking)

# 1. Recommend with attribution, so the call records a session you can quote.
do-memory-cli --format json playbook recommend "Add retry to the HTTP client" \
  --domain networking --episode-id "$EPISODE_ID" > playbook.json

SESSION=$(jq -r '.session.session_id' playbook.json)
# 2. Read the pattern behind the step you actually followed.
CITED=$(jq -r '.playbooks[0].step_sources[] | select(.order == 1) | .pattern_id' playbook.json)

# 3. Report feedback for exactly that step's pattern.
do-memory-cli feedback record-feedback --session "$SESSION" \
  --applied "$CITED" --outcome success --message "Followed step 1"
```

`record-feedback` accepts a cited ID because the session records the playbook's
`supporting_pattern_ids`, and a step's source pattern is always one of them. An ID that was
never recommended for the session is still rejected. Via MCP the same flow is
`recommend_playbook` with `episode_id`, then `record_recommendation_feedback` on the
returned `session.session_id`.

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
success-after-adoption rate, and precision metrics). Current recommendation ranking uses the
learned ADR-082 Wilson weight derived from this attributed feedback to boost or suppress the
ranking of future pattern recommendations.
