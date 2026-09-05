# API Reference

**Version**: v0.1.38 (current workspace release)
**Last Updated**: 2026-08-07
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

- `query_memory` (optional `with_provenance: true` for F4.1 redacted retrieval provenance)
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

#### Attributed recommendation contract (ADR-080/081)

`recommend_patterns` and `recommend_playbook` accept an **optional** `episode_id`
(`type: string`, `format: uuid`). It is never required — omitting it selects the
legacy unattributed path, whose response shape is unchanged.

When `episode_id` is supplied and valid:

- The episode must already exist; a nil, malformed, or nonexistent episode ID is
  an error and never creates a session.
- The response wraps the recommendations in an **attribution envelope**:
  `attribution` containing `session_id` and a `receipt` with `state`
  (`persisted` | `partially_persisted` | `memory_only` | `persistence_failed`),
  the episode ID, and (for partial/failed states) the stable `failed_backends`
  identifiers (`turso`, `redb`).
- `success` is `false` when the receipt is `persistence_failed`; manual session
  and feedback commands report the same receipt truthfully.

Receipt states and restart implications:

| state | meaning | restart-safe feedback |
|-------|---------|-----------------------|
| `persisted` | every configured capable backend wrote the record | yes — feedback submitted after a process restart resolves the session from storage |
| `partially_persisted` | at least one capable backend wrote it, at least one failed (`failed_backends` lists the failures in try order `turso`→`redb`) | only if the surviving backend holds it |
| `memory_only` | no configured backend advertises attribution capability (including configured backends that advertise `false`) | no — the record exists only in this process |
| `persistence_failed` | every configured capable backend failed to write | no |

**Scope — feedback-to-ranking adaptation (ADR-082):** attribution feedback now
derives a per-pattern learned weight (the Wilson lower-bound success rate at
`z = 1.96` on success-after-application, over `(applied, succeeded)` evidence) and
the recommendation path re-ranks its candidate pool by base relevance plus that
weight. This affects `recommend_patterns` only; generic search, discovery, and
retrieval are unchanged, and `get_recommendation_stats` remains exact attribution
capture. The learned weight is a deterministic reduction of the in-process tracker
plus capability-gated durable history — the tracker is authoritative for
"latest feedback wins", and after a cold restart the index is a pure function of
durable history (idempotent, replacement-safe, rollback-safe by rebuild). Backends
that do not advertise `supports_ranking_adaptation` contribute nothing to the
durable read, so behavior is identical to pre-ADR-082 when unconfigured
(in-process feedback still tracks within a run).

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

#### Runtime activation contract (ADR-077)

- `configure_embeddings` is an **activation** operation, not a declarative config
  write. A `success: true` response means the requested provider is live in the
  server process; subsequent `embedding_provider_status`, `generate_embedding`,
  `search_by_embedding`, and `query_semantic_memory` calls use it immediately.
- Selectable providers: `local`, `openai`, `mistral`. `azure`, `azure_openai`,
  `custom`, and `cohere` are rejected with an error naming the provider before any
  credential or network work (no runtime adapter yet).
- Cloud providers require a credential environment-variable **name** via
  `api_key_env` (for example `OPENAI_API_KEY`, `MISTRAL_API_KEY`). Credentials are
  read at activation time only; they are never stored, persisted, or echoed in
  responses, warnings, errors, or audit fields.
- Activation is atomic: a failed request preserves the previously active provider
  and its revision unchanged.
- Successful responses add `activation_revision` (strictly increasing per
  successful activation), `reindex_required`
  (`true` when the provider/model/dimension identity changed), and
  `provider_health` (`"active"`). Activation state is runtime-only and is not
  persisted across process restarts.

### Unavailable / Fail-Closed Tool

- `execute_agent_code` — **unavailable / fail-closed**. The WASM sandbox was removed; there is no `wasmtime-backend` feature and no working code-execution backend. Calls are rejected (fail closed).

### Provenance diagnostics (F4.1 / ADR-074)

- `query_memory` accepts optional `with_provenance: true` — response may include a redacted `provenance` object (fingerprint, cache_hit, index_generation, latency). Raw query text is never returned in provenance.

---

## Deferred Batch Tools (Intentionally Absent)

Per WG-053 and parity tests, these tool names are intentionally **not advertised** in `tools/list` and should not resolve:

- `batch_query_episodes`
- `batch_pattern_analysis`
- `batch_compare_episodes`

Status: **Deferred / absent from active MCP tool contract** (R-C7) until handlers are implemented and wired. Do not advertise them as available.

---

---

## Notes

- This file is a contract index, not a full schema dump.
- For exact argument schemas, use runtime `tools/list` from the server build you are running.
- When updating tool names, update this file and `memory-mcp/tests/tool_contract_parity.rs` in the same change.

---

## See Also

- [Playbooks and Checkpoints](./PLAYBOOKS_AND_CHECKPOINTS.md)
- [do-memory-mcp README](../memory-mcp/README.md)
- [Current project status](../plans/STATUS/CURRENT.md)
