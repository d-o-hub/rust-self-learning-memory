//! MCP Server Binary
//!
//! This binary implements the Model Context Protocol (MCP) server for the
//! self-learning memory system. It communicates over stdio using JSON-RPC.

use anyhow::Context;
use memory_core::{Error, MemoryConfig, SelfLearningMemory};
use memory_mcp::{MemoryMCPServer, SandboxConfig};
use memory_storage_redb::{CacheConfig, RedbStorage};
use memory_storage_turso::{TursoConfig, TursoStorage};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// MCP Initialize response
#[derive(Debug, Serialize)]
struct InitializeResult {
    protocol_version: String,
    capabilities: Value,
    server_info: Value,
}

/// MCP Tool structure for listing
#[derive(Debug, Serialize)]
struct McpTool {
    name: String,
    description: String,
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

/// Initialize the memory system with appropriate storage backends
async fn initialize_memory_system() -> anyhow::Result<Arc<SelfLearningMemory>> {
    // Try to initialize with dual storage (Turso + redb) first
    if let Ok(memory) = initialize_dual_storage().await {
        info!("Memory system initialized with persistent storage (Turso + redb)");
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
    info!("  - For full persistence: set TURSO_DATABASE_URL and TURSO_AUTH_TOKEN");
    info!("  - For cache-only persistence: ensure REDB_CACHE_PATH is accessible");
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
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

    run_jsonrpc_server(mcp_server).await
}

async fn run_jsonrpc_server(mcp_server: Arc<Mutex<MemoryMCPServer>>) -> anyhow::Result<()> {
    // Main message loop for JSON-RPC
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut handle = stdin.lock();

    loop {
        let mut line = String::new();
        match handle.read_line(&mut line) {
            Ok(0) => {
                // EOF reached
                info!("Received EOF, shutting down");
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Parse JSON-RPC request
                match serde_json::from_str::<JsonRpcRequest>(line) {
                    Ok(request) => {
                        let response = handle_request(request, &mcp_server).await;

                        // Send response
                        if let Some(response_json) = response {
                            let response_str = serde_json::to_string(&response_json)?;
                            writeln!(stdout, "{}", response_str)?;
                            stdout.flush()?;
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
                        writeln!(stdout, "{}", response_str)?;
                        stdout.flush()?;
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

/// Handle a JSON-RPC request
async fn handle_request(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    match request.method.as_str() {
        "initialize" => handle_initialize(request).await,
        "tools/list" => handle_list_tools(request, mcp_server).await,
        "tools/call" => handle_call_tool(request, mcp_server).await,
        "shutdown" => handle_shutdown(request).await,
        _ => {
            warn!("Unknown method: {}", request.method);
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
async fn handle_initialize(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    info!("Handling initialize request");

    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: json!({
            "tools": {
                "listChanged": false
            }
        }),
        server_info: json!({
            "name": "memory-mcp-server",
            "version": env!("CARGO_PKG_VERSION")
        }),
    };

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(serde_json::to_value(result).unwrap()),
        error: None,
    })
}

/// Handle tools/list request
async fn handle_list_tools(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
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

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(serde_json::to_value(result).unwrap()),
        error: None,
    })
}

/// Handle tools/call request
async fn handle_call_tool(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
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
        "execute_agent_code" => handle_execute_code(&mut server, params.arguments).await,
        "analyze_patterns" => handle_analyze_patterns(&mut server, params.arguments).await,
        "health_check" => handle_health_check(&mut server, params.arguments).await,
        "get_metrics" => handle_get_metrics(&mut server, params.arguments).await,
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

    match result {
        Ok(content) => {
            let result = CallToolResult { content };
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::to_value(result).unwrap()),
                error: None,
            })
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
    }
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

    let result = server.execute_agent_code(code, context).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
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

/// Handle shutdown request
async fn handle_shutdown(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    info!("Handling shutdown request");

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!(null)),
        error: None,
    })
}
