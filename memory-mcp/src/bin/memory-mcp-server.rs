//! MCP Server Binary
//!
//! This binary implements the Model Context Protocol (MCP) server for the
//! self-learning memory system with OAuth 2.1 authorization support.
//! It communicates over stdio using JSON-RPC.

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
        // Fail fast: Require MCP_OAUTH_TOKEN_SECRET in production when OAuth is enabled
        if oauth_config.token_secret.is_none() {
            panic!(
                "SECURITY ERROR: MCP_OAUTH_ENABLED=true but MCP_OAUTH_TOKEN_SECRET is not set. \
                 JWT signature verification requires a secret key. Set MCP_OAUTH_TOKEN_SECRET \
                 before starting the server."
            );
        }
    }

    server_impl::run_jsonrpc_server(mcp_server, oauth_config).await
}
