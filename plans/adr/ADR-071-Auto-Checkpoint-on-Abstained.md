# ADR-071: Auto-checkpoint on TaskOutcome::Abstained

## Status
Proposed

## Context
When an agent detects infeasibility and sets `TaskOutcome::Abstained`, the episode should automatically create a `CheckpointMeta` snapshot before terminating. This preserves all partial findings, tool outputs, and the abstention reason so another agent or session can resume from the last valid state rather than restarting from zero.

The paper Agentic Abstention (arXiv:2606.28733) notes that abstention events should be recoverable: a different model, a retried session with refreshed context, or a human-delegated sub-task may be able to complete what the original agent could not.

## Decision
We will extend `CheckpointMeta` with an `is_abstention_checkpoint` flag and implement an automatic checkpointing mechanism during episode completion.

1.  **Extend `CheckpointMeta`**: Add `is_abstention_checkpoint: bool` and ensure backward compatibility using `#[serde(default)]` and field aliases for `timestamp` (aliased from `created_at`) and `label` (aliased from `reason`).
2.  **Auto-checkpoint Logic**: Implement `maybe_create_abstention_checkpoint` to be called during `complete_episode`. It will create a checkpoint if the outcome is `Abstained` and at least one step was performed.
3.  **Handoff Support**: Provide `list_abstention_checkpoints` to retrieve these snapshots for resuming work.
4.  **CLI Visibility**: Update `memory-cli` to distinguish between manual and abstention checkpoints in output.

## Consequences
- Partial work performed before an agent realizes a task is infeasible is preserved automatically.
- Future agents or humans can resume from these "failure-adjacent" states using the handoff mechanism.
- Episode storage size increases slightly for abstained episodes that performed work.
