# API Reference

**Version**: v0.1.22 (current workspace release)
**Last Updated**: 2026-03-24
**Protocol**: MCP over JSON-RPC 2.0 (protocol negotiation supports `2025-11-25` and `2024-11-05`)

---

## Contract Source (Truth Source)

This document is intentionally aligned to the MCP contract parity test:

- `memory-mcp/tests/tool_contract_parity.rs`

If this document and runtime behavior diverge, treat the parity test + `tools/list` runtime output as authoritative.

---

## Overview

The Memory MCP server exposes tools for:

- episodic memory lifecycle
- pattern analysis and recommendations
- playbook/checkpoint/handoff workflows
- recommendation attribution/feedback tracking
- tagging and relationships
- embeddings and semantic search
- health and metrics

All calls use MCP `tools/call` payloads over JSON-RPC 2.0.

---

## Request/Response Shape

### Tool call request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "query_memory",
    "arguments": {
      "query": "how to resume interrupted work",
      "domain": "agent-ops"
    }
  }
}
```

### Tool call response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"ok\":true}"
      }
    ],
    "isError": false
  }
}
```

---

## MCP Tool Contract (Current)

The following tool names are the current contract tracked by parity tests.

### Core and Monitoring

- `query_memory`
- `analyze_patterns`
- `health_check`
- `get_metrics`

### Pattern / Recommendation / Explainability

- `advanced_pattern_analysis`
- `quality_metrics`
- `search_patterns`
- `recommend_patterns`
- `recommend_playbook`
- `explain_pattern`

### Recommendation Attribution / Feedback

- `record_recommendation_session`
- `record_recommendation_feedback`
- `get_recommendation_stats`

### Playbook / Checkpoint / Handoff

- `checkpoint_episode`
- `get_handoff_pack`
- `resume_from_handoff`

### Episode Lifecycle

- `bulk_episodes`
- `create_episode`
- `add_episode_step`
- `complete_episode`
- `get_episode`
- `delete_episode`
- `update_episode`
- `get_episode_timeline`

### Episode Tags

- `add_episode_tags`
- `remove_episode_tags`
- `set_episode_tags`
- `get_episode_tags`
- `search_episodes_by_tags`

### Episode Relationships

- `add_episode_relationship`
- `remove_episode_relationship`
- `get_episode_relationships`
- `find_related_episodes`
- `check_relationship_exists`
- `get_dependency_graph`
- `validate_no_cycles`
- `get_topological_order`

### Embeddings

- `configure_embeddings`
- `query_semantic_memory`
- `test_embeddings`
- `generate_embedding`
- `search_by_embedding`
- `embedding_provider_status`

### Conditional Tool

- `execute_agent_code` — conditionally available based on sandbox/runtime configuration (WASM availability and `MCP_USE_WASM` behavior).

---

## Deferred Batch Tools (Intentionally Absent)

Per WG-053 and parity tests, these tool names are intentionally **not advertised** in `tools/list` and should not resolve:

- `batch_query_episodes`
- `batch_pattern_analysis`
- `batch_compare_episodes`

Status: **Deferred / absent from active MCP tool contract** until handlers are implemented and wired.

---

## Notes

- This file is a contract index, not a full schema dump.
- For exact argument schemas, use runtime `tools/list` from the server build you are running.
- When updating tool names, update this file and `memory-mcp/tests/tool_contract_parity.rs` in the same change.

---

## See Also

- [Playbooks and Checkpoints](./PLAYBOOKS_AND_CHECKPOINTS.md)
- [memory-mcp README](../memory-mcp/README.md)
- [Current project status](../plans/STATUS/CURRENT.md)
