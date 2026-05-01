//! MCP Server Binary
//!
//! This binary implements the Model Context Protocol (MCP) server for the
//! self-learning memory system with OAuth 2.1 authorization support.
//! It communicates over stdio using JSON-RPC.

// Clippy suppressions for MCP server binary
#![allow(clippy::doc_markdown)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::single_match_else)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::similar_names)]
#![allow(clippy::unused_async)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::inefficient_to_string)]
#![allow(clippy::manual_string_new)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::format_push_string)]
#![allow(clippy::items_after_statements)]

mod server_impl;

use do_memory_mcp::SandboxConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    info!("Starting Memory MCP Server in JSON-RPC mode");

    // Initialize memory system with storage backends
    let memory = server_impl::initialize_memory_system().await?;

    // Create MCP server with restrictive sandbox
    let sandbox_config = SandboxConfig::restrictive();
    let mcp_server = Arc::new(Mutex::new(
        do_memory_mcp::MemoryMCPServer::new(sandbox_config, memory).await?,
    ));

    info!("MCP Server initialized successfully");

    // Initialize OAuth config from environment
    let oauth_config = server_impl::load_oauth_config();
    if oauth_config.enabled {
        info!("OAuth 2.1 authorization enabled");
        if let Some(ref issuer) = oauth_config.issuer {
            info!("Expected token issuer: {}", issuer);
        }
    }

    server_impl::run_jsonrpc_server(mcp_server, oauth_config).await
}
