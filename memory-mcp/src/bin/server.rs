//! MCP Server Binary
//!
//! This binary implements the Model Context Protocol (MCP) server for the
//! self-learning memory system with OAuth 2.1 authorization support.
//! It communicates over stdio using JSON-RPC.

use anyhow::Context;
use memory_core::{Error, MemoryConfig, SelfLearningMemory};
use memory_mcp::jsonrpc::{
    read_next_message, write_response_with_length, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
};
use memory_mcp::mcp::tools::quality_metrics::QualityMetricsInput;
use memory_mcp::{MemoryMCPServer, SandboxConfig};
use memory_storage_redb::{CacheConfig, RedbStorage};
use memory_storage_turso::{TursoConfig, TursoStorage};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, Write};

use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// OAuth 2.1 Configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct OAuthConfig {
    /// Whether authorization is enabled
    enabled: bool,
    /// Expected audience for tokens
    audience: Option<String>,
    /// Expected issuer for tokens
    issuer: Option<String>,
    /// Supported scopes
    scopes: Vec<String>,
    /// JWKS URI for token validation
    jwks_uri: Option<String>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for local development
            audience: None,
            issuer: None,
            scopes: vec!["mcp:read".to_string(), "mcp:write".to_string()],
            jwks_uri: None,
        }
    }
}

/// Protected Resource Metadata (RFC 9728)
#[derive(Debug, Serialize)]
struct ProtectedResourceMetadata {
    #[serde(rename = "authorizationServers", skip_serializing_if = "Vec::is_empty")]
    authorization_servers: Vec<String>,
    resource: String,
    #[serde(rename = "scopesSupported", skip_serializing_if = "Vec::is_empty")]
    scopes_supported: Vec<String>,
    #[serde(rename = "resourceMetadata")]
    resource_metadata: Option<String>,
}

/// Bearer token claims (simplified JWT structure)
#[derive(Debug)]
#[allow(dead_code)]
struct TokenClaims {
    /// Subject (user/client ID)
    sub: String,
    /// Issuer
    iss: Option<String>,
    /// Audience
    aud: Option<String>,
    /// Expiration time
    exp: Option<u64>,
    /// Issued at
    iat: Option<u64>,
    /// Scopes
    scope: Option<String>,
}

/// Authorization result
#[derive(Debug)]
#[allow(dead_code)]
enum AuthorizationResult {
    Authorized,
    MissingToken,
    InvalidToken(String),
    InsufficientScope(Vec<String>),
}

/// MCP Initialize response payload
#[derive(Debug, Serialize)]
struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    capabilities: Value,
    #[serde(rename = "serverInfo")]
    server_info: Value,
}

/// MCP Tool structure for listing
#[derive(Debug, Serialize)]
struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// MCP ListTools response
#[derive(Debug, Serialize)]
struct ListToolsResult {
    tools: Vec<McpTool>,
}

/// MCP CallTool request parameters
#[derive(Debug, Deserialize)]
struct CallToolParams {
    name: String,
    arguments: Option<Value>,
}

/// MCP CallTool response
#[derive(Debug, Serialize)]
struct CallToolResult {
    content: Vec<Content>,
}

/// MCP Content structure
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum Content {
    #[serde(rename = "text")]
    Text { text: String },
}

/// Completion reference types (MCP 2025-11-25)
#[allow(dead_code)]
#[derive(Debug, Serialize)]
enum CompletionRef {
    #[serde(rename = "ref/prompt")]
    Prompt { name: String },
    #[serde(rename = "ref/resource")]
    Resource { uri: String },
}

/// Completion argument for completion/complete request
#[derive(Debug, Deserialize)]
struct CompletionArgument {
    name: String,
    value: String,
}

/// Completion context (optional additional context)
#[derive(Debug, Deserialize)]
struct CompletionContext {
    #[serde(default)]
    arguments: std::collections::HashMap<String, Value>,
}

/// Completion request parameters (ref parsed manually due to external tagging)
#[derive(Debug, Deserialize)]
struct CompletionParams {
    #[serde(rename = "ref")]
    reference: Value,
    argument: CompletionArgument,
    #[serde(default)]
    context: Option<CompletionContext>,
}

/// Completion result
#[derive(Debug, Serialize)]
struct CompletionResult {
    completion: CompletionValues,
}

/// Completion values response
#[derive(Debug, Serialize)]
struct CompletionValues {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<u64>,
    #[serde(rename = "hasMore", skip_serializing_if = "Option::is_none")]
    has_more: Option<bool>,
}

// ============================================================
// Elicitation Structures (MCP 2025-11-25)
// ============================================================

/// Elicitation request type - what kind of input is requested
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
enum ElicitationType {
    Text,
    Select,
    Confirm,
}

/// Prompt for the user in an elicitation request
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ElicitationPrompt {
    /// The prompt text to display to the user
    r#type: String,
    /// Human-readable description of what input is needed
    description: Option<String>,
    /// Additional data for select type elicitation
    options: Option<Vec<ElicitationOption>>,
}

/// Option for select type elicitation
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ElicitationOption {
    /// Label displayed to the user
    label: String,
    /// Value returned when selected
    value: Value,
}

/// Elicitation request parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ElicitationParams {
    /// Unique identifier for this elicitation
    elicitation_id: String,
    /// The prompt to send to the user
    prompt: ElicitationPrompt,
    /// Name of the tool that triggered this elicitation
    trigger: String,
}

/// Elicitation response parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ElicitationDataParams {
    /// The elicitation being responded to
    elicitation_id: String,
    /// The user's response data
    data: Value,
}

/// Elicitation result response
#[derive(Debug, Serialize)]
struct ElicitationResult {
    /// The elicitation that was resolved
    elicitation_id: String,
    /// The received data
    data: Value,
}

/// Parameters for cancelling an elicitation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ElicitationCancelParams {
    /// The elicitation to cancel
    elicitation_id: String,
}

/// Active elicitation tracker
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ActiveElicitation {
    id: String,
    prompt: ElicitationPrompt,
    trigger: String,
    created_at: std::time::Instant,
}

/// Initialize the memory system with appropriate storage backends
async fn initialize_memory_system() -> anyhow::Result<Arc<SelfLearningMemory>> {
    // Try Turso local first (default behavior)
    if let Ok(memory) = initialize_turso_local().await {
        info!("Memory system initialized with Turso local database (default)");
        return Ok(memory);
    }

    // If Turso local fails, try dual storage (Turso cloud + redb)
    if let Ok(memory) = initialize_dual_storage().await {
        info!("Memory system initialized with persistent storage (Turso cloud + redb)");
        return Ok(memory);
    }

    // If dual storage fails, try redb-only storage
    if let Ok(memory) = initialize_redb_only_storage().await {
        info!("Memory system initialized with redb cache storage (Turso unavailable)");
        return Ok(memory);
    }

    // Final fallback to in-memory storage
    warn!("Failed to initialize any persistent storage, falling back to in-memory");
    info!("To enable persistence:");
    info!("  - Default: Turso local database (no configuration needed)");
    info!("  - Cloud: set TURSO_DATABASE_URL and TURSO_AUTH_TOKEN");
    info!("  - Cache-only: ensure REDB_CACHE_PATH is accessible");
    Ok(Arc::new(SelfLearningMemory::new()))
}

/// Initialize memory system with redb cache storage only (fallback when Turso is unavailable)
async fn initialize_redb_only_storage() -> anyhow::Result<Arc<SelfLearningMemory>> {
    info!("Attempting to initialize redb-only storage...");

    // Initialize redb cache storage
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    // Create data directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::Storage(format!("Failed to create data directory: {}", e)))?;
    }

    let cache_config = CacheConfig {
        max_size: std::env::var("REDB_MAX_CACHE_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000),
        default_ttl_secs: 1800,     // 30 minutes
        cleanup_interval_secs: 600, // 10 minutes
        enable_background_cleanup: true,
    };

    let redb_storage = RedbStorage::new_with_cache_config(cache_path, cache_config).await?;
    info!(
        "Successfully initialized redb storage at {}",
        cache_path.display()
    );

    // Create memory system with redb cache and in-memory fallbacks for Turso
    // Note: We use the same redb instance for both turso and cache since we only have redb
    let memory_config = MemoryConfig::default();
    let redb_arc: Arc<dyn memory_core::StorageBackend> = Arc::new(redb_storage);
    let memory = SelfLearningMemory::with_storage(memory_config, Arc::clone(&redb_arc), redb_arc);

    Ok(Arc::new(memory))
}

/// Initialize memory system with both Turso (durable) and redb (cache) storage
async fn initialize_dual_storage() -> anyhow::Result<Arc<SelfLearningMemory>> {
    // Read Turso configuration from environment
    let turso_url = std::env::var("TURSO_DATABASE_URL")
        .context("TURSO_DATABASE_URL environment variable not set")?;
    let turso_token = std::env::var("TURSO_AUTH_TOKEN")
        .context("TURSO_AUTH_TOKEN environment variable not set")?;

    info!("Connecting to Turso database at {}", turso_url);

    // Initialize Turso storage with connection pooling
    let turso_config = TursoConfig {
        max_retries: 3,
        retry_base_delay_ms: 100,
        retry_max_delay_ms: 5000,
        enable_pooling: true,
    };

    let turso_storage = TursoStorage::with_config(&turso_url, &turso_token, turso_config).await?;
    turso_storage.initialize_schema().await?;

    // Initialize redb cache storage
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    let cache_config = CacheConfig {
        max_size: std::env::var("REDB_MAX_CACHE_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000),
        default_ttl_secs: 1800,     // 30 minutes
        cleanup_interval_secs: 600, // 10 minutes
        enable_background_cleanup: true,
    };

    let redb_storage = RedbStorage::new_with_cache_config(cache_path, cache_config).await?;

    // Create memory system with both storage backends
    let memory_config = MemoryConfig::default();
    let memory = SelfLearningMemory::with_storage(
        memory_config,
        Arc::new(turso_storage),
        Arc::new(redb_storage),
    );

    Ok(Arc::new(memory))
}

/// Initialize memory system with Turso local database (default behavior)
async fn initialize_turso_local() -> anyhow::Result<Arc<SelfLearningMemory>> {
    info!("Attempting to initialize Turso local database (default)...");

    // Use local Turso database file
    let turso_url =
        std::env::var("TURSO_DATABASE_URL").unwrap_or_else(|_| "file:./data/memory.db".to_string());

    // For local files, no token is needed
    let turso_token = if turso_url.starts_with("file:") {
        "".to_string()
    } else {
        std::env::var("TURSO_AUTH_TOKEN").unwrap_or_default()
    };

    info!("Connecting to Turso database at {}", turso_url);

    // Initialize Turso storage with basic config for local use
    let turso_config = TursoConfig {
        max_retries: 1, // Fewer retries for local
        retry_base_delay_ms: 50,
        retry_max_delay_ms: 1000,
        enable_pooling: false, // No pooling needed for local
    };

    let turso_storage = TursoStorage::with_config(&turso_url, &turso_token, turso_config).await?;
    turso_storage.initialize_schema().await?;

    // Initialize redb cache storage for performance
    let cache_path_str =
        std::env::var("REDB_CACHE_PATH").unwrap_or_else(|_| "./data/cache.redb".to_string());
    let cache_path = Path::new(&cache_path_str);

    let cache_config = CacheConfig {
        max_size: std::env::var("REDB_MAX_CACHE_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000),
        default_ttl_secs: 1800,     // 30 minutes
        cleanup_interval_secs: 600, // 10 minutes
        enable_background_cleanup: true,
    };

    let redb_storage = RedbStorage::new_with_cache_config(cache_path, cache_config).await?;

    // Create memory system with both storage backends
    let memory_config = MemoryConfig::default();
    let memory = SelfLearningMemory::with_storage(
        memory_config,
        Arc::new(turso_storage),
        Arc::new(redb_storage),
    );

    info!("Successfully initialized Turso local + redb cache storage");
    Ok(Arc::new(memory))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    info!("Starting Memory MCP Server in JSON-RPC mode");

    // Initialize memory system with storage backends
    let memory = initialize_memory_system().await?;

    // Create MCP server with restrictive sandbox
    let sandbox_config = SandboxConfig::restrictive();
    let mcp_server = Arc::new(Mutex::new(
        MemoryMCPServer::new(sandbox_config, memory).await?,
    ));

    info!("MCP Server initialized successfully");

    // Initialize OAuth config from environment
    let oauth_config = load_oauth_config();
    if oauth_config.enabled {
        info!("OAuth 2.1 authorization enabled");
        if let Some(ref issuer) = oauth_config.issuer {
            info!("Expected token issuer: {}", issuer);
        }
    }

    run_jsonrpc_server(mcp_server, oauth_config).await
}

/// Load OAuth 2.1 configuration from environment variables
fn load_oauth_config() -> OAuthConfig {
    let enabled = std::env::var("MCP_OAUTH_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();

    OAuthConfig {
        enabled: enabled == "true" || enabled == "1" || enabled == "yes",
        audience: std::env::var("MCP_OAUTH_AUDIENCE").ok(),
        issuer: std::env::var("MCP_OAUTH_ISSUER").ok(),
        scopes: std::env::var("MCP_OAUTH_SCOPES")
            .unwrap_or_else(|_| "mcp:read,mcp:write".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        jwks_uri: std::env::var("MCP_OAUTH_JWKS_URI").ok(),
    }
}

/// Validate Bearer token (simplified JWT parsing)
#[allow(dead_code)]
fn validate_bearer_token(token: &str, config: &OAuthConfig) -> AuthorizationResult {
    // Split JWT into parts
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return AuthorizationResult::InvalidToken("Invalid token format".to_string());
    }

    // Decode payload (base64url)
    let payload = match base64url_decode(parts[1]) {
        Ok(p) => p,
        Err(e) => {
            return AuthorizationResult::InvalidToken(format!("Invalid token payload: {}", e))
        }
    };

    // Parse JSON payload - convert bytes to string first
    let payload_str = match String::from_utf8(payload) {
        Ok(s) => s,
        Err(e) => {
            return AuthorizationResult::InvalidToken(format!("Invalid token encoding: {}", e))
        }
    };

    let claims: serde_json::Value = match serde_json::from_str(&payload_str) {
        Ok(c) => c,
        Err(e) => return AuthorizationResult::InvalidToken(format!("Invalid token JSON: {}", e)),
    };

    // Validate issuer if configured
    if let Some(expected_iss) = &config.issuer {
        let token_iss = claims.get("iss").and_then(|v| v.as_str()).unwrap_or("");
        if !token_iss.is_empty() && token_iss != expected_iss {
            return AuthorizationResult::InvalidToken(format!(
                "Invalid token issuer: expected {}, got {}",
                expected_iss, token_iss
            ));
        }
    }

    // Validate audience if configured
    if let Some(expected_aud) = &config.audience {
        let token_aud = claims.get("aud").and_then(|v| v.as_str()).unwrap_or("");
        if !token_aud.is_empty() && token_aud != expected_aud {
            return AuthorizationResult::InvalidToken(format!(
                "Invalid token audience: expected {}, got {}",
                expected_aud, token_aud
            ));
        }
    }

    // Check expiration if present
    if let Some(exp) = claims.get("exp").and_then(|v| v.as_u64()) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if exp < now {
            return AuthorizationResult::InvalidToken("Token expired".to_string());
        }
    }

    // Validate required subject claim
    let sub = claims.get("sub").and_then(|v| v.as_str()).unwrap_or("");
    if sub.is_empty() {
        return AuthorizationResult::InvalidToken("Token missing subject claim".to_string());
    }

    debug!("Token validated for subject: {}", sub);
    AuthorizationResult::Authorized
}

/// Base64url decode (RFC 4648)
#[allow(dead_code)]
fn base64url_decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    // For simplicity, we'll do basic base64 decoding
    // In production, use a proper base64url crate
    let filtered: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    // Pad if necessary
    let padded = match filtered.len() % 4 {
        2 => filtered + "==",
        3 => filtered + "=",
        _ => filtered,
    };

    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &padded)
}

/// Check if request has required scopes
#[allow(dead_code)]
fn check_scopes(token_scope: Option<&str>, required_scopes: &[String]) -> AuthorizationResult {
    let token_scopes: Vec<String> = match token_scope {
        Some(s) => s
            .split(' ')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        None => vec![],
    };

    // If no required scopes, allow access
    if required_scopes.is_empty() {
        return AuthorizationResult::Authorized;
    }

    // If token has no scopes and required scopes exist, deny
    if token_scopes.is_empty() {
        return AuthorizationResult::InsufficientScope(required_scopes.to_vec());
    }

    // Check if token has all required scopes
    let missing: Vec<String> = required_scopes
        .iter()
        .filter(|r| !token_scopes.contains(r))
        .cloned()
        .collect();

    if missing.is_empty() {
        AuthorizationResult::Authorized
    } else {
        AuthorizationResult::InsufficientScope(missing)
    }
}

/// Extract Bearer token from Authorization header
#[allow(dead_code)]
fn extract_bearer_token(_headers: &str) -> Option<String> {
    // For stdio mode, we can't access headers directly
    // This would be used for HTTP transport mode
    None
}

/// Create WWW-Authenticate challenge header value (RFC 6750)
#[allow(dead_code)]
fn create_www_authenticate_header(
    error: &str,
    resource_metadata: Option<&str>,
    scopes: Option<&str>,
) -> String {
    let mut params = vec![format!("error=\"{}\"", error)];

    if let Some(rm) = resource_metadata {
        params.push(format!("resource_metadata=\"{}\"", rm));
    }

    if let Some(s) = scopes {
        params.push(format!("scope=\"{}\"", s));
    }

    format!("Bearer {}", params.join(", "))
}

#[allow(clippy::excessive_nesting)]
async fn run_jsonrpc_server(
    mcp_server: Arc<Mutex<MemoryMCPServer>>,
    oauth_config: OAuthConfig,
) -> anyhow::Result<()> {
    // Main message loop for JSON-RPC
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut handle = stdin.lock();

    // Track active elicitation requests (MCP 2025-11-25)
    let elicitation_tracker: Arc<Mutex<Vec<ActiveElicitation>>> = Arc::new(Mutex::new(Vec::new()));

    // Track last input framing to respond with the same framing style
    #[allow(unused_assignments)]
    let mut last_input_was_lsp = false;
    loop {
        match read_next_message(&mut handle) {
            Ok(None) => {
                // EOF reached
                info!("Received EOF, shutting down");
                break;
            }
            Ok(Some((line, is_lsp))) => {
                last_input_was_lsp = is_lsp;
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Parse JSON-RPC request
                #[allow(clippy::excessive_nesting)]
                match serde_json::from_str::<JsonRpcRequest>(line) {
                    Ok(request) => {
                        let response = handle_request(
                            request,
                            &mcp_server,
                            &oauth_config,
                            &elicitation_tracker,
                        )
                        .await;

                        // Send response, matching the input framing style
                        if let Some(response_json) = response {
                            let response_str = serde_json::to_string(&response_json)?;
                            if last_input_was_lsp {
                                write_response_with_length(&mut stdout, &response_str)?;
                            } else {
                                writeln!(stdout, "{}", response_str)?;
                                stdout.flush()?;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse JSON-RPC request: {}", e);
                        // Send error response
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: None, // We don't have an ID if parsing failed
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32700,
                                message: "Parse error".to_string(),
                                data: Some(json!({"details": e.to_string()})),
                            }),
                        };
                        let response_str = serde_json::to_string(&error_response)?;
                        if last_input_was_lsp {
                            write_response_with_length(&mut stdout, &response_str)?;
                        } else {
                            writeln!(stdout, "{}", response_str)?;
                            stdout.flush()?;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    info!("Memory MCP Server shutting down");
    Ok(())
}

// jsonrpc helpers moved to memory_mcp::jsonrpc
/// Handle a JSON-RPC request
async fn handle_request(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
    oauth_config: &OAuthConfig,
    elicitation_tracker: &Arc<Mutex<Vec<ActiveElicitation>>>,
) -> Option<JsonRpcResponse> {
    // Notifications (no id) must not produce a response per JSON-RPC
    // Treat both missing id and explicit null id as notifications (no response)
    if request.id.is_none() || matches!(request.id, Some(serde_json::Value::Null)) {
        return None;
    }

    // Check authorization for protected methods
    if oauth_config.enabled {
        // For HTTP transport, extract Bearer token from Authorization header
        // For stdio, token would need to be passed via request metadata
        // This is a placeholder for HTTP transport mode
        debug!("OAuth enabled - authorization check would occur here for HTTP transport");
    }

    // Normalize method name if compatibility aliases are enabled
    // Enable compatibility aliases by default for broader client support.
    // Can be disabled by setting MCP_COMPAT_ALIASES to "false"/"0"/"no".
    let compat_env = std::env::var("MCP_COMPAT_ALIASES").unwrap_or_else(|_| "true".to_string());
    let compat = compat_env.to_lowercase();
    let compat_enabled = !(compat == "false" || compat == "0" || compat == "no");

    let mut method = request.method.clone();
    if compat_enabled {
        method = match request.method.as_str() {
            // Common variants observed in some clients
            "tools.get" | "tools/get" | "list_tools" | "list-tools" => "tools/list".to_string(),
            "call_tool" | "tool/call" | "tools.call" => "tools/call".to_string(),
            // Pass through known methods unchanged
            _ => request.method.clone(),
        };
    }

    match method.as_str() {
        "initialize" => handle_initialize(request, oauth_config).await,
        "tools/list" => handle_list_tools(request, mcp_server).await,
        "tools/call" => handle_call_tool(request, mcp_server).await,
        "shutdown" => handle_shutdown(request).await,
        "completion/complete" => handle_completion_complete(request).await,
        // Elicitation handlers (MCP 2025-11-25)
        "elicitation/request" => handle_elicitation_request(request, elicitation_tracker).await,
        "elicitation/data" => handle_elicitation_data(request, elicitation_tracker).await,
        "elicitation/cancel" => handle_elicitation_cancel(request, elicitation_tracker).await,
        // OAuth 2.1 protected resource metadata endpoint (MCP specification)
        ".well-known/oauth-protected-resource" => {
            handle_protected_resource_metadata(request, oauth_config).await
        }
        _ => {
            warn!("Unknown method: {}", method);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            })
        }
    }
}

/// Handle initialize request
async fn handle_initialize(
    request: JsonRpcRequest,
    oauth_config: &OAuthConfig,
) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling initialize request");

    // Build capabilities object
    let mut capabilities = json!({
        "tools": {
            "listChanged": false
        },
        "completions": {},
        "elicitation": {}
    });

    // Add OAuth 2.1 authorization capability if enabled
    if oauth_config.enabled {
        capabilities["authorization"] = json!({
            "enabled": true,
            "issuer": oauth_config.issuer.clone().unwrap_or_default(),
            "audience": oauth_config.audience.clone().unwrap_or_default(),
            "scopes": oauth_config.scopes
        });
    }

    let result = InitializeResult {
        protocol_version: "2025-11-25".to_string(),
        capabilities,
        server_info: json!({
            "name": "memory-mcp-server",
            "version": env!("CARGO_PKG_VERSION")
        }),
    };

    match serde_json::to_value(result) {
        Ok(value) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        }),
        Err(e) => {
            error!("Failed to serialize initialize response: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(json!({"details": format!("Response serialization failed: {}", e)})),
                }),
            })
        }
    }
}

/// Handle protected resource metadata request (RFC 9728)
async fn handle_protected_resource_metadata(
    request: JsonRpcRequest,
    oauth_config: &OAuthConfig,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling protected resource metadata request");

    // RFC 9728: Protected Resource Metadata
    let resource_uri = std::env::var("MCP_RESOURCE_URI")
        .unwrap_or_else(|_| "https://memory-mcp.example.com".to_string());

    let resource_uri_clone = resource_uri.clone();
    let metadata = ProtectedResourceMetadata {
        authorization_servers: oauth_config
            .issuer
            .clone()
            .map(|iss| vec![iss])
            .unwrap_or_default(),
        resource: resource_uri,
        scopes_supported: oauth_config.scopes.clone(),
        resource_metadata: Some(format!(
            "{}/.well-known/oauth-protected-resource",
            resource_uri_clone
        )),
    };

    match serde_json::to_value(metadata) {
        Ok(value) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        }),
        Err(e) => {
            error!("Failed to serialize protected resource metadata: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(json!({"details": format!("Failed to serialize metadata: {}", e)})),
                }),
            })
        }
    }
}

/// Handle tools/list request
async fn handle_list_tools(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling tools/list request");

    let server = mcp_server.lock().await;
    let tools = server.list_tools().await;

    let mcp_tools: Vec<McpTool> = tools
        .into_iter()
        .map(|tool| McpTool {
            name: tool.name,
            description: tool.description,
            input_schema: tool.input_schema,
        })
        .collect();

    let result = ListToolsResult { tools: mcp_tools };

    match serde_json::to_value(result) {
        Ok(value) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        }),
        Err(e) => {
            error!("Failed to serialize list_tools response: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(json!({"details": format!("Response serialization failed: {}", e)})),
                }),
            })
        }
    }
}

/// Handle tools/call request
async fn handle_call_tool(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    let params: CallToolParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: Some(json!({"details": e.to_string()})),
                    }),
                });
            }
        },
        None => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            });
        }
    };

    info!("Handling tools/call request for tool: {}", params.name);

    let mut server = mcp_server.lock().await;
    let result = match params.name.as_str() {
        "query_memory" => handle_query_memory(&mut server, params.arguments).await,
        "execute_agent_code" => {
            // Check if execute_agent_code tool is available
            let tools = server.list_tools().await;
            if tools.iter().any(|t| t.name == "execute_agent_code") {
                handle_execute_code(&mut server, params.arguments).await
            } else {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32000,
                        message: "Tool execution failed".to_string(),
                        data: Some(json!({
                            "details": "execute_agent_code tool is not available due to WASM sandbox compilation issues"
                        })),
                    }),
                });
            }
        }
        "analyze_patterns" => handle_analyze_patterns(&mut server, params.arguments).await,
        "advanced_pattern_analysis" => {
            handle_advanced_pattern_analysis(&mut server, params.arguments).await
        }
        "health_check" => handle_health_check(&mut server, params.arguments).await,
        "get_metrics" => handle_get_metrics(&mut server, params.arguments).await,
        "quality_metrics" => handle_quality_metrics(&mut server, params.arguments).await,
        _ => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Tool not found".to_string(),
                    data: Some(json!({"tool": params.name})),
                }),
            });
        }
    };

    // Process the tool result
    let response = match result {
        Ok(content) => {
            let call_result = CallToolResult { content };
            match serde_json::to_value(call_result) {
                Ok(value) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(value),
                    error: None,
                }),
                Err(e) => {
                    error!("Failed to serialize call_tool response: {}", e);
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: "Internal error".to_string(),
                            data: Some(
                                json!({"details": format!("Response serialization failed: {}", e)}),
                            ),
                        }),
                    })
                }
            }
        }
        Err(e) => {
            error!("Tool execution failed: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: "Tool execution failed".to_string(),
                    data: Some(json!({"details": e.to_string()})),
                }),
            })
        }
    };

    response
}

/// Handle query_memory tool
async fn handle_query_memory(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let task_type = args
        .get("task_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let result = server.query_memory(query, domain, task_type, limit).await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
}

/// Handle execute_agent_code tool
async fn handle_execute_code(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::ExecutionContext;

    let args: Value = arguments.unwrap_or(json!({}));
    let code = args
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'code' parameter"))?
        .to_string();

    let context_obj = args
        .get("context")
        .ok_or_else(|| anyhow::anyhow!("Missing 'context' parameter"))?;

    let task = context_obj
        .get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'task' in context"))?
        .to_string();

    let input = context_obj.get("input").cloned().unwrap_or(json!({}));

    let context = ExecutionContext::new(task, input);

    // Check if WASM sandbox is available by attempting a simple test
    // If it fails, return a proper error instead of crashing
    match server
        .execute_agent_code(
            "console.log('test');".to_string(),
            ExecutionContext::new("test".to_string(), json!({})),
        )
        .await
    {
        Ok(_) => {
            // WASM sandbox is working, proceed with actual execution
            let result = server.execute_agent_code(code, context).await?;
            let content = vec![Content::Text {
                text: serde_json::to_string_pretty(&result)?,
            }];
            Ok(content)
        }
        Err(e) => {
            // WASM sandbox is not available, return proper error
            Err(anyhow::anyhow!("Code execution is currently unavailable due to WASM sandbox compilation issues. Error: {}", e))
        }
    }
}

/// Handle analyze_patterns tool
async fn handle_analyze_patterns(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let task_type = args
        .get("task_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'task_type' parameter"))?
        .to_string();
    let min_success_rate = args
        .get("min_success_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7) as f32;
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

    let result = server
        .analyze_patterns(task_type, min_success_rate, limit)
        .await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
}

/// Handle advanced_pattern_analysis tool
async fn handle_advanced_pattern_analysis(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));

    // Parse analysis type
    let analysis_type_str = args
        .get("analysis_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'analysis_type' parameter"))?;

    let analysis_type = match analysis_type_str {
        "statistical" => {
            memory_mcp::mcp::tools::advanced_pattern_analysis::AnalysisType::Statistical
        }
        "predictive" => memory_mcp::mcp::tools::advanced_pattern_analysis::AnalysisType::Predictive,
        "comprehensive" => {
            memory_mcp::mcp::tools::advanced_pattern_analysis::AnalysisType::Comprehensive
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid analysis_type: {}",
                analysis_type_str
            ))
        }
    };

    // Parse time series data
    let time_series_data_value = args
        .get("time_series_data")
        .ok_or_else(|| anyhow::anyhow!("Missing 'time_series_data' parameter"))?;

    let time_series_data: std::collections::HashMap<String, Vec<f64>> =
        serde_json::from_value(time_series_data_value.clone())?;

    // Parse optional config
    let config = args
        .get("config")
        .and_then(|c| serde_json::from_value(c.clone()).ok());

    let input = memory_mcp::mcp::tools::advanced_pattern_analysis::AdvancedPatternAnalysisInput {
        analysis_type,
        time_series_data,
        config,
    };

    let result = server.execute_advanced_pattern_analysis(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
}

/// Handle health_check tool
async fn handle_health_check(
    server: &mut MemoryMCPServer,
    _arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let result = server.health_check().await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
}

/// Handle get_metrics tool
async fn handle_get_metrics(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let metric_type = args
        .get("metric_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let result = server.get_metrics(metric_type).await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
}

/// Handle quality_metrics tool
async fn handle_quality_metrics(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let input: QualityMetricsInput = serde_json::from_value(args)?;
    let result = server.execute_quality_metrics(input).await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle shutdown request
async fn handle_shutdown(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling shutdown request");

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!(null)),
        error: None,
    })
}

/// Handle completion/complete request (MCP 2025-11-25)
async fn handle_completion_complete(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling completion/complete request");

    // Parse completion params
    let params: CompletionParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: Some(json!({"details": e.to_string()})),
                    }),
                })
            }
        },
        None => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            })
        }
    };

    // Generate completions based on reference type and argument
    let completions = generate_completions(&params).await;

    let result = CompletionResult {
        completion: completions,
    };

    match serde_json::to_value(result) {
        Ok(value) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        }),
        Err(e) => {
            error!("Failed to serialize completion response: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(json!({"details": format!("Response serialization failed: {}", e)})),
                }),
            })
        }
    }
}

/// Generate completions based on reference type and argument value
async fn generate_completions(params: &CompletionParams) -> CompletionValues {
    let argument_value = params.argument.value.clone();

    // Get context arguments if available
    let _context_args = params.context.as_ref().map(|c| &c.arguments);

    // Parse the reference Value to extract prompt name or resource URI
    // External tagging format: {"ref/prompt": {"name": "..."}} or {"ref/resource": {"uri": "..."}}
    let prompt_name = if let Some(prompt_ref) = params
        .reference
        .as_object()
        .and_then(|o| o.get("ref/prompt"))
    {
        prompt_ref
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    // Generate domain completions for query_memory tool
    if let Some(ref name) = prompt_name {
        // Handle prompt argument completions
        match name.as_str() {
            "query_memory" => {
                // Common domains for query_memory
                let domains = [
                    "web-api",
                    "data-processing",
                    "code-generation",
                    "debugging",
                    "refactoring",
                    "testing",
                    "analysis",
                    "documentation",
                    "infrastructure",
                    "security",
                ];
                let filtered: Vec<String> = domains
                    .iter()
                    .filter(|d| d.starts_with(&argument_value))
                    .map(|s| s.to_string())
                    .collect();
                return CompletionValues {
                    values: filtered.clone(),
                    total: Some(filtered.len() as u64),
                    has_more: Some(false),
                };
            }
            "analyze_patterns" => {
                // Task types for analyze_patterns
                let task_types = [
                    "code_generation",
                    "debugging",
                    "refactoring",
                    "testing",
                    "analysis",
                    "documentation",
                ];
                let filtered: Vec<String> = task_types
                    .iter()
                    .filter(|t| t.starts_with(&argument_value))
                    .map(|s| s.to_string())
                    .collect();
                return CompletionValues {
                    values: filtered.clone(),
                    total: Some(filtered.len() as u64),
                    has_more: Some(false),
                };
            }
            "advanced_pattern_analysis" => {
                // Analysis types
                let analysis_types = ["statistical", "predictive", "comprehensive"];
                let filtered: Vec<String> = analysis_types
                    .iter()
                    .filter(|a| a.starts_with(&argument_value))
                    .map(|s| s.to_string())
                    .collect();
                return CompletionValues {
                    values: filtered.clone(),
                    total: Some(filtered.len() as u64),
                    has_more: Some(false),
                };
            }
            _ => {
                // Generic completions based on argument name
                let arg_name = params.argument.name.as_str();
                let completions: Vec<&str> = match arg_name {
                    "domain" => vec!["web-api", "data-processing", "testing"],
                    "task_type" => vec!["code_generation", "debugging", "refactoring"],
                    "metric_type" => vec!["all", "performance", "episodes", "system"],
                    "analysis_type" => vec!["statistical", "predictive", "comprehensive"],
                    "time_range" => vec!["24h", "7d", "30d", "90d", "all"],
                    "provider" => vec!["openai", "local", "mistral", "azure", "cohere"],
                    _ => vec![],
                };
                let filtered: Vec<String> = completions
                    .iter()
                    .filter(|s| s.starts_with(&argument_value))
                    .map(|s| s.to_string())
                    .collect();
                return CompletionValues {
                    values: filtered.clone(),
                    total: Some(filtered.len() as u64),
                    has_more: Some(false),
                };
            }
        }
    }

    // Handle resource completions
    let _is_resource_ref = params
        .reference
        .as_object()
        .and_then(|o| o.get("ref/resource"))
        .is_some();

    // For resource URI completions, return empty for now
    CompletionValues {
        values: vec![],
        total: Some(0),
        has_more: Some(false),
    }
}

// ============================================================
// Elicitation Handlers (MCP 2025-11-25)
// ============================================================

/// Handle elicitation/request - server asks client for user input
async fn handle_elicitation_request(
    request: JsonRpcRequest,
    elicitation_tracker: &Arc<Mutex<Vec<ActiveElicitation>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling elicitation/request");

    let params: ElicitationParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: Some(json!({"details": e.to_string()})),
                    }),
                })
            }
        },
        None => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            })
        }
    };

    // Store the active elicitation
    let active = ActiveElicitation {
        id: params.elicitation_id.clone(),
        prompt: params.prompt.clone(),
        trigger: params.trigger.clone(),
        created_at: std::time::Instant::now(),
    };

    let mut tracker = elicitation_tracker.lock().await;
    tracker.push(active);

    info!(
        "Elicitation {} created for tool: {}",
        params.elicitation_id, params.trigger
    );

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!({
            "elicitationId": params.elicitation_id,
            "status": "pending"
        })),
        error: None,
    })
}

/// Handle elicitation/data - client responds with user input
async fn handle_elicitation_data(
    request: JsonRpcRequest,
    elicitation_tracker: &Arc<Mutex<Vec<ActiveElicitation>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling elicitation/data");

    let params: ElicitationDataParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: Some(json!({"details": e.to_string()})),
                    }),
                })
            }
        },
        None => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            })
        }
    };

    // Find and remove the matching elicitation
    let mut tracker = elicitation_tracker.lock().await;
    let index = tracker.iter().position(|e| e.id == params.elicitation_id);

    match index {
        Some(i) => {
            let elicitation = tracker.remove(i);
            info!(
                "Elicitation {} resolved from tool: {}",
                params.elicitation_id, elicitation.trigger
            );

            let result = ElicitationResult {
                elicitation_id: params.elicitation_id,
                data: params.data,
            };

            match serde_json::to_value(result) {
                Ok(value) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(value),
                    error: None,
                }),
                Err(e) => {
                    error!("Failed to serialize elicitation response: {}", e);
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: "Internal error".to_string(),
                            data: Some(
                                json!({"details": format!("Response serialization failed: {}", e)}),
                            ),
                        }),
                    })
                }
            }
        }
        None => {
            warn!("Elicitation {} not found", params.elicitation_id);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Elicitation not found".to_string(),
                    data: Some(
                        json!({"details": format!("No active elicitation with id: {}", params.elicitation_id)}),
                    ),
                }),
            })
        }
    }
}

/// Cancel an active elicitation
async fn handle_elicitation_cancel(
    request: JsonRpcRequest,
    elicitation_tracker: &Arc<Mutex<Vec<ActiveElicitation>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling elicitation/cancel");

    let params: ElicitationCancelParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: Some(json!({"details": e.to_string()})),
                    }),
                })
            }
        },
        None => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            })
        }
    };

    let mut tracker = elicitation_tracker.lock().await;
    let index = tracker.iter().position(|e| e.id == params.elicitation_id);

    match index {
        Some(i) => {
            let elicitation = tracker.remove(i);
            info!(
                "Elicitation {} cancelled for tool: {}",
                params.elicitation_id, elicitation.trigger
            );

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "elicitationId": params.elicitation_id,
                    "status": "cancelled"
                })),
                error: None,
            })
        }
        None => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "Elicitation not found".to_string(),
                data: Some(
                    json!({"details": format!("No active elicitation with id: {}", params.elicitation_id)}),
                ),
            }),
        }),
    }
}
