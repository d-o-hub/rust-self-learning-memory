# Memory MCP Integration

MCP (Model Context Protocol) server integration for the self-learning memory
system. The server exposes episode lifecycle, memory retrieval, pattern
analysis, embeddings, and monitoring tools over JSON-RPC on stdio.

> **Agent code execution is fail-closed.** `execute_agent_code` is **not** a
> working execution backend. It is absent from tool discovery and direct calls
> are rejected. There is no WASM/Wasmtime/Javy sandbox in production. See
> [ADR-073](../plans/adr/ADR-073-Capability-Enforced-Agent-Code-Execution.md)
> and [ADR-052](../plans/adr/ADR-052-Comprehensive-Analysis-v0.1.29.md).

## Features

- **MCP Server**: JSON-RPC over stdio with tool discovery and a shared tool registry
- **Episode Lifecycle Management**: Programmatic episode creation, tracking, and completion
- **Episode Relationships, Tags, and Handoffs**: Knowledge-graph links, tag filtering, checkpoints
- **Memory Integration**: Query episodic and semantic memory; analyze learned patterns
- **Pattern Analysis**: Pattern extraction, search, recommendations, and playbooks
- **Embeddings Support**: Multiple providers (local, OpenAI, Mistral)
- **Progressive Tool Disclosure**: Tools prioritized based on usage patterns
- **Monitoring**: Health checks, metrics, rate limiting, and audit logging
- **Fail-Closed Code Execution**: `execute_agent_code` is unavailable in production

## Code Execution Status

`execute_agent_code` is a **fail-closed, unavailable** tool in production:

- It is not advertised by `tools/list` (no working backend is registered).
- Direct `tools/call` for it is rejected by the dispatcher in
  `src/bin/server_impl/handlers/call_tool.rs` with a JSON-RPC error and the
  detail `"execute_agent_code tool is not available due to WASM sandbox
  compilation issues"`.
- The batch dispatcher rejects it identically
  (`src/bin/server_impl/handlers/batch_execute.rs`).
- The handler in `src/bin/server_impl/tools/memory_handlers.rs` additionally
  audit-logs the attempt and returns
  `"Code execution is no longer available. The WASM sandbox was removed in v0.1.29."`
- The WASM/Wasmtime/Javy/rquickjs dependencies and the `wasmtime-backend`,
  `javy-backend`, and `wasm-rquickjs` feature names were removed in v0.1.29
  (ADR-052). They do not exist in this crate today.

Use the supported episode and memory tools instead (see
[Available Tools](#available-tools)), or run agent code in an external runner
that you control outside this MCP server.

### `sandbox-dev` (trusted local experimentation only)

A legacy Node.js `CodeSandbox` still lives in `src/sandbox/`, but it is
compiled **only** when the non-default `sandbox-dev` feature is enabled:

```bash
cargo test -p do-memory-mcp --features sandbox-dev
```

`sandbox-dev` is **not** a production sandbox and MUST NOT be enabled for
untrusted input. It exists solely for trusted local experimentation.

**Limits that ARE enforced in this path:**

- Execution timeout (`max_execution_time_ms`) via a Tokio timeout; the child
  process is killed on drop (`kill_on_drop(true)`).
- Maximum code length (100 KB) checked before execution.
- Static regex screening of the source for filesystem, network, subprocess, and
  obvious malicious patterns (see `src/sandbox/mod.rs`).
- Global shadowing/deletion in the JavaScript wrapper (`process`, `require`,
  `module`, `__dirname`, `__filename`) when those capabilities are disabled.

**Limits that are NOT enforced (do not rely on them):**

- OS-level isolation: `src/sandbox/isolation.rs` provides `apply_isolation`
  (privilege drop, `ulimit`, namespaces), but the execution path does **not**
  call it.
- Memory limits (`max_memory_mb`) and CPU limits (`max_cpu_percent` /
  `max_cpu_seconds`) are configuration-only; they are not applied to the child
  process.
- Output sanitization: stdout/stderr are returned as-is.
- The Node.js process itself is a full interpreter; regex screening is
  heuristic and can be bypassed by runtime obfuscation.
- There is no runtime capability enforcement for network/filesystem/subprocess;
  the settings only drive source-pattern checks.

## Usage

The server is constructed with a memory system and a sandbox configuration, then
served over stdio:

```rust
use do_memory_core::SelfLearningMemory;
use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
use std::sync::Arc;
use tokio::sync::Mutex;

async fn build_server(memory: Arc<SelfLearningMemory>) -> anyhow::Result<Arc<Mutex<MemoryMCPServer>>> {
    let server = MemoryMCPServer::new(SandboxConfig::restrictive(), memory).await?;
    Ok(Arc::new(Mutex::new(server)))
}
```

`SelfLearningMemory` and its storage backends are initialized by
`src/bin/server_impl`; see `src/bin/memory-mcp-server.rs` for the full startup
path and stdio JSON-RPC loop.

### Calling a supported tool

Use episode and memory tools rather than code execution:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "query_memory",
    "arguments": {
      "query": "implement REST API",
      "domain": "web-api",
      "task_type": "code_generation",
      "limit": 10
    }
  }
}
```

### Calling `execute_agent_code` (fails closed)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "execute_agent_code",
    "arguments": { "code": "console.log('hello')" }
  }
}
```

Observed response (production build):

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": null,
  "error": {
    "code": -32000,
    "message": "Tool execution failed",
    "data": {
      "details": "execute_agent_code tool is not available due to WASM sandbox compilation issues"
    }
  }
}
```

### Sandbox configuration

`SandboxConfig` is still accepted at construction (the server is built with
`SandboxConfig::restrictive()` by default) but, without a registered execution
backend, it no longer governs any production code path. Treat the timeout,
memory, and CPU fields as advisory configuration only.

## Available Tools

The MCP server exposes tools grouped into categories, defined in the shared
registry (`src/server/tools/registry/`). Consult the registry and
[docs/API_REFERENCE.md](../docs/API_REFERENCE.md) for the authoritative,
current surface.

### Episode Lifecycle Management

- **`create_episode`** - Start tracking a new task with metadata
- **`add_episode_step`** - Log execution steps to track progress
- **`complete_episode`** - Finalize episode and trigger learning cycle
- **`get_episode`** - Retrieve complete episode details
- **`get_episode_timeline`** - Visualize chronological task progression
- **`update_episode`** - Update episode details
- **`delete_episode`** - Remove episodes permanently (with safeguards)
- **`checkpoint_episode`** - Create mid-task checkpoints
- **`get_handoff_pack`** / **`resume_from_handoff`** - Multi-agent handoff

📖 **[Complete Episode Lifecycle Documentation](EPISODE_LIFECYCLE_TOOLS.md)**

### Episode Relationships & Tags

- **`add_episode_relationship`**, **`get_episode_relationships`**, **`remove_episode_relationship`**
- **`find_related_episodes`**, **`check_relationship_exists`**
- **`add_episode_tags`**, **`get_episode_tags`**, **`set_episode_tags`**, **`remove_episode_tags`**
- **`search_episodes_by_tags`**
- **`get_dependency_graph`**, **`get_topological_order`**, **`validate_no_cycles`**

📖 **[Episode Tags Tools](EPISODE_TAGS_TOOLS.md)**

### Memory & Query Tools

- **`query_memory`** - Query episodic memory for relevant past experiences
- **`query_semantic_memory`** - Semantic search using embeddings
- **`bulk_episodes`** - Retrieve multiple episodes efficiently

### Pattern Analysis

- **`analyze_patterns`** - Analyze patterns from past episodes
- **`advanced_pattern_analysis`** - Deep pattern analysis with statistical methods
- **`search_patterns`** - Search for specific patterns
- **`recommend_patterns`** - Get pattern recommendations for tasks
- **`recommend_playbook`** - Get actionable playbooks for tasks
- **`explain_pattern`** - Explain a specific pattern

### Embeddings & Configuration

- **`configure_embeddings`** - Configure embedding providers (local, OpenAI, Mistral)
- **`test_embeddings`** - Test embedding generation
- **`generate_embedding`** - Generate an embedding for supplied text
- **`search_by_embedding`** - Search by embedding vector
- **`embedding_provider_status`** - Report active provider health

### Monitoring & Health

- **`health_check`** - Server health and status
- **`get_metrics`** - Performance metrics and statistics
- **`quality_metrics`** - Episode quality assessment
- **`record_recommendation_session`**, **`record_recommendation_feedback`**, **`get_recommendation_stats`**

### Code Execution (fail-closed)

- **`execute_agent_code`** - **Unavailable / fail-closed.** Not a working
  execution backend; calls are rejected. Use the tools above instead.

### Batch Operations Contract Status

The MCP JSON-RPC endpoint supports `batch/execute` (multi-operation transport).
Tool-level batch analytics names are intentionally **deferred and not
advertised**:

- `batch_query_episodes`
- `batch_pattern_analysis`
- `batch_compare_episodes`

These names return `Tool not found` until dedicated handlers are implemented.

📖 **[Batch Tool Status (WG-053)](BATCH_OPERATIONS_TOOLS.md)**

## Security

Production security boundaries, authentication, isolation, and audit logging
are described in [SECURITY.md](SECURITY.md). In brief:

- Transport is JSON-RPC over stdio; there is no network listener by default.
- OAuth 2.1 bearer-token validation is opt-in via the `oauth` feature and
  `MCP_OAUTH_*` environment variables; when disabled the server logs that OAuth
  is disabled.
- Per-client token-bucket rate limiting protects against DoS.
- Structured audit logging records security-relevant events, including rejected
  `execute_agent_code` attempts.
- Agent code execution is fail-closed; the only in-tree executor is the
  non-default, trusted-local `sandbox-dev` path documented above.

## Testing

```bash
# Run the crate tests
cargo test -p do-memory-mcp

# Include the trusted-development sandbox path (local experimentation only)
cargo test -p do-memory-mcp --features sandbox-dev
```

## Contributing

1. **Security first**: keep `execute_agent_code` fail-closed unless an approved
   backend with enforced controls exists (ADR-073).
2. **Test coverage**: add tests for both success and failure cases.
3. **Documentation**: update this README and inline docs; do not reintroduce
   WASM/Javy production claims.
4. **Performance**: profile memory/query paths.

## License

MIT License - See the repository `LICENSE` file for details.
