---
name: checkpoint-handoff
description: Checkpoint episodes and create handoff packs for multi-agent session continuity
---

# Checkpoint & Handoff

## When to Use

- Saving progress mid-episode for session recovery
- Creating handoff packs for agent-to-agent context transfer
- Resuming work from a previous agent's checkpoint
- Long-running tasks that may be interrupted

## CLI Commands

| Command | Purpose |
|---------|---------|
| `do-memory-cli episode checkpoint <EPISODE_ID>` | Create checkpoint of current episode state |
| `do-memory-cli episode checkpoint <EPISODE_ID> --note "reason"` | Checkpoint with annotation |

## MCP Tools

| Tool | Parameters | Purpose |
|------|-----------|---------|
| `checkpoint_episode` | episode_id, note (optional) | Save episode state snapshot |
| `get_handoff_pack` | episode_id | Generate complete context pack for handoff |
| `resume_from_handoff` | handoff_pack | Resume episode from a handoff pack |

## Workflow: Agent Handoff

```
Agent A: Working on episode...
  1. checkpoint_episode(episode_id, note: "completed API design")
  2. get_handoff_pack(episode_id)
     → Returns: { episode state, steps, patterns applied, context }

Agent B: Resuming work...
  1. resume_from_handoff(handoff_pack)
     → Restores context, continues episode
```

## Workflow: Session Recovery

```
1. Agent hits context limit or session ends
2. checkpoint_episode(episode_id, note: "context limit reached")
3. New session starts
4. get_handoff_pack(episode_id)
5. resume_from_handoff(pack)
6. Continue adding steps to the same episode
```

## Abstention Checkpoints

The system automatically creates checkpoints when an agent abstains (decides not to act). These are queryable via `list_abstention_checkpoints()` for analyzing decision patterns.

## Best Practices

- Checkpoint before expensive operations (in case of failure)
- Include meaningful notes describing what was accomplished
- Use handoff packs for cross-agent context, not just resumption
- Handoff packs contain patterns applied + episode steps — sufficient for cold-start
