//! MCP Server module
//!
//! This module contains the MCP server implementation organized into logical submodules:
//! - `types`: All type definitions (OAuth, MCP protocol, tasks, elicitation, etc.)
//! - `storage`: Storage backend initialization functions
//! - `oauth`: OAuth 2.1 security and authentication functions
//! - `jsonrpc`: JSON-RPC server loop and request routing
//! - `core`: Core MCP protocol handlers (initialize, list_tools, call_tool, shutdown)
//! - `tools`: Memory tool handlers (query_memory, analyze_patterns, etc.)
//! - `mcp`: MCP 2025-11-25 protocol handlers (completion, elicitation, tasks)
//! - `embedding`: Embedding configuration and handlers

mod core;
#[cfg(feature = "embeddings")]
mod embedding;
mod handlers;
mod jsonrpc;
mod mcp;
mod oauth; // Always available (load_oauth_config is unconditionally compiled)
mod storage;
mod tools;
mod types;

// Re-export JSON-RPC types from memory_mcp for convenient access
#[allow(unused)]
pub use memory_mcp::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

// Re-export OAuthConfig from library protocol module
#[allow(unused)]
pub use memory_mcp::protocol::OAuthConfig;

// Re-export types needed by other modules
#[allow(unused)]
pub use types::{
    ActiveElicitation, ActiveTask, CallToolParams, CallToolResult, Content, EmbeddingEnvConfig,
    RateLimitEnvConfig,
};

// Re-export all types and functions for convenient access (may be used by external consumers)
#[allow(unused)]
pub use core::*;
#[cfg(feature = "embeddings")]
#[allow(unused)]
pub use embedding::*;
#[allow(unused)]
pub use jsonrpc::*;
// Always export load_oauth_config (it's available without the oauth feature)
#[allow(unused)]
pub use oauth::load_oauth_config;
// Other oauth functions are gated behind the feature
#[cfg(feature = "oauth")]
#[allow(unused)]
pub use oauth::{
    check_scopes, create_www_authenticate_header, extract_bearer_token, validate_bearer_token,
};
#[allow(unused)]
pub use storage::*;
#[allow(unused)]
pub use types::*;
