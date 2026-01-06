# Sprint Execution Detail - MCP Server Split

**File**: memory-mcp/src/bin/server.rs (2,368 LOC)  
**Target**: 9 modules (~100-400 LOC each)  
**Strategy**: Option C - Minimal bin/server.rs with extracted modules

## Module Breakdown

### Module 1: server/types.rs (~270 LOC)
**Content**: All type definitions
- OAuthConfig, TokenClaims, AuthorizationResult
- InitializeResult, McpTool, ListToolsResult, CallToolParams, CallToolResult
- Content enum, CompletionRef enum
- CompletionArgument, CompletionContext, CompletionParams, CompletionResult, CompletionValues
- ElicitationType enum, ElicitationPrompt, ElicitationOption, ElicitationParams
- ElicitationDataParams, ElicitationResult, ElicitationCancelParams, ActiveElicitation
- TaskStatus enum, TaskParams, TaskResult, ActiveTask, TaskMetadata
- EmbeddingEnvConfig

**Dependencies**: serde, serde_json

### Module 2: server/storage.rs (~180 LOC)
**Content**: Storage initialization functions
- initialize_memory_system()
- initialize_redb_only_storage()
- initialize_dual_storage()
- initialize_turso_local()

**Dependencies**: memory_core, memory_storage_redb, memory_storage_turso, types

### Module 3: server/oauth.rs (~174 LOC)
**Content**: OAuth 2.1 security functions
- load_oauth_config()
- validate_bearer_token()
- base64url_decode()
- check_scopes()
- extract_bearer_token()
- create_www_authenticate_header()

**Dependencies**: base64, types

### Module 4: server/jsonrpc.rs (~175 LOC)
**Content**: JSON-RPC server infrastructure
- run_jsonrpc_server() - main server loop
- handle_request() - request routing and method normalization

**Dependencies**: tokio, serde_json, memory_mcp::jsonrpc, types, all handler modules

### Module 5: server/core.rs (~400 LOC)
**Content**: Core MCP protocol handlers
- handle_initialize()
- handle_protected_resource_metadata()
- handle_list_tools()
- handle_call_tool()
- handle_shutdown()

**Dependencies**: memory_mcp, types, oauth

### Module 6: server/tools.rs (~400 LOC)
**Content**: Memory tool handlers
- handle_query_memory()
- handle_execute_code()
- handle_start_episode()
- handle_log_step()
- handle_complete_episode()
- handle_extract_patterns()
- Plus other tool-specific handlers

**Dependencies**: memory_core, memory_mcp, types

### Module 7: server/mcp.rs (~400 LOC)
**Content**: MCP 2025-11-25 protocol handlers
- handle_completion_complete()
- handle_elicitation_request()
- handle_elicitation_data()
- handle_elicitation_cancel()
- handle_task_create()
- handle_task_update()
- handle_task_complete()
- handle_task_cancel()
- handle_task_list()

**Dependencies**: types, Arc<Mutex<>> for state tracking

### Module 8: server/embedding.rs (~80 LOC)
**Content**: Embedding configuration handler
- handle_embedding_config()
- parse_embedding_env_config()

**Dependencies**: types, std::env

### Module 9: server/mod.rs (~50 LOC)
**Content**: Module declarations and re-exports
```rust
mod types;
mod storage;
mod oauth;
mod jsonrpc;
mod core;
mod tools;
mod mcp;
mod embedding;

pub use types::*;
pub use storage::*;
pub use oauth::*;
pub use jsonrpc::*;
pub use core::*;
pub use tools::*;
pub use mcp::*;
pub use embedding::*;
```

### Module 10: bin/server.rs (~100 LOC)
**Content**: Entry point
```rust
//! MCP Server Binary
mod server;

use server::*;
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing setup
    // Initialize memory system
    // Initialize OAuth config
    // Run JSON-RPC server
}
```

## Extraction Order (Low Risk → High Risk)

1. **types.rs** (SAFEST) - Pure data structures, no logic
2. **storage.rs** (SAFE) - Self-contained initialization functions
3. **oauth.rs** (SAFE) - Self-contained security functions
4. **embedding.rs** (SAFE) - Single handler, minimal dependencies
5. **core.rs** (MEDIUM) - Core handlers, moderate coupling
6. **tools.rs** (MEDIUM) - Tool handlers, moderate coupling
7. **mcp.rs** (MEDIUM) - MCP handlers, state tracking
8. **jsonrpc.rs** (HIGHER) - Server loop, imports all handlers
9. **mod.rs** (EASY) - Just re-exports
10. **server.rs** (EASY) - Just main() + imports

## Testing Strategy

After each extraction:
1. Run `cargo check -p memory-mcp`
2. Fix any import errors
3. Run `cargo test -p memory-mcp --lib`
4. Run `cargo clippy -p memory-mcp -- -D warnings`

After all extractions:
1. Run full test suite: `cargo test -p memory-mcp`
2. Build binary: `cargo build --bin memory-mcp-server`
3. Test binary runs: `./target/debug/memory-mcp-server --help` (if applicable)
4. Run integration tests
