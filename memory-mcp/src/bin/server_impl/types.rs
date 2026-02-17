//! Type definitions for MCP server
//!
//! This module contains all struct and enum definitions used by the MCP server,
//! including OAuth 2.1, MCP protocol, completion, elicitation, tasks, and embedding types.
//!
//! Note: Core protocol types (OAuthConfig, ProtectedResourceMetadata, InitializeResult,
//! McpTool, ListToolsResult) are now defined in the library's protocol module and re-exported
//! from core.rs for backward compatibility.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================
// OAuth 2.1 Types (deprecated - use library version)
// ============================================================

/// Authorization result
#[derive(Debug)]
pub enum AuthorizationResult {
    Authorized,
    #[allow(dead_code)] // TODO: Implement missing token handling
    MissingToken,
    #[allow(dead_code)] // Error message available for logging/debugging
    InvalidToken(String),
    #[allow(dead_code)] // Required scopes available for logging/debugging
    InsufficientScope(Vec<String>),
}

// ============================================================
// MCP Core Protocol Types (deprecated - use library versions)
// ============================================================

/// MCP CallTool request parameters
#[derive(Debug, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    pub arguments: Option<Value>,
}

/// MCP CallTool response
#[derive(Debug, Serialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl CallToolResult {
    /// Create a successful result
    pub fn success(content: Vec<Content>) -> Self {
        Self {
            content,
            is_error: None,
        }
    }

    /// Create an error result
    pub fn error(content: Vec<Content>) -> Self {
        Self {
            content,
            is_error: Some(true),
        }
    }
}

/// MCP Content structure
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
}

// ============================================================
// Completion Types (MCP 2025-11-25)
// ============================================================

/// Completion reference types (MCP 2025-11-25)
///
/// TODO: Implement completion support in protocol handlers
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub enum CompletionRef {
    #[serde(rename = "ref/prompt")]
    Prompt { name: String },
    #[serde(rename = "ref/resource")]
    Resource { uri: String },
}

/// Completion argument for completion/complete request
#[derive(Debug, Deserialize)]
pub struct CompletionArgument {
    pub name: String,
    pub value: String,
}

/// Completion context (optional additional context)
#[derive(Debug, Deserialize)]
pub struct CompletionContext {
    #[serde(default)]
    pub arguments: std::collections::HashMap<String, Value>,
}

/// Completion request parameters (ref parsed manually due to external tagging)
#[derive(Debug, Deserialize)]
pub struct CompletionParams {
    #[serde(rename = "ref")]
    pub reference: Value,
    pub argument: CompletionArgument,
    #[serde(default)]
    pub context: Option<CompletionContext>,
}

/// Completion result
#[derive(Debug, Serialize)]
pub struct CompletionResult {
    pub completion: CompletionValues,
}

/// Completion values response
#[derive(Debug, Serialize)]
pub struct CompletionValues {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(rename = "hasMore", skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

// ============================================================
// Elicitation Types (MCP 2025-11-25)
// ============================================================

/// Elicitation request type - what kind of input is requested
///
/// TODO: Implement elicitation support in protocol handlers
#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ElicitationType {
    Text,
    Select,
    Confirm,
}

/// Prompt for the user in an elicitation request
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationPrompt {
    /// The prompt text to display to the user
    pub r#type: String,
    /// Human-readable description of what input is needed
    pub description: Option<String>,
    /// Additional data for select type elicitation
    pub options: Option<Vec<ElicitationOption>>,
}

/// Option for select type elicitation
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationOption {
    /// Label displayed to the user
    pub label: String,
    /// Value returned when selected
    pub value: Value,
}

/// Elicitation request parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationParams {
    /// Unique identifier for this elicitation
    pub elicitation_id: String,
    /// The prompt to send to the user
    pub prompt: ElicitationPrompt,
    /// Name of the tool that triggered this elicitation
    pub trigger: String,
}

/// Elicitation response parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationDataParams {
    /// The elicitation being responded to
    pub elicitation_id: String,
    /// The user's response data
    pub data: Value,
}

/// Elicitation result response
#[derive(Debug, Serialize)]
pub struct ElicitationResult {
    /// The elicitation that was resolved
    pub elicitation_id: String,
    /// The received data
    pub data: Value,
}

/// Parameters for cancelling an elicitation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationCancelParams {
    /// The elicitation to cancel
    pub elicitation_id: String,
}

/// Active elicitation tracker
#[derive(Debug, Clone)]
pub struct ActiveElicitation {
    pub id: String,
    #[allow(dead_code)] // Not currently used, available for future timeout handling
    pub prompt: ElicitationPrompt,
    pub trigger: String,
    #[allow(dead_code)] // Not currently used, available for future timeout handling
    pub created_at: std::time::Instant,
}

// ============================================================
// Task Types (MCP 2025-11-25)
// ============================================================

/// Task status enumeration
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Task result type
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TaskResultType {
    Text,
    Json,
    Error,
}

/// Task result
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskResult {
    pub r#type: TaskResultType,
    pub content: Value,
}

/// Task input parameters
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub name: String,
    pub input: Option<Value>,
    pub metadata: Option<std::collections::HashMap<String, Value>>,
}

/// Task creation parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCreateParams {
    pub task_id: String,
    pub task: TaskInput,
}

/// Task status update parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdateParams {
    pub task_id: String,
    pub status: TaskStatus,
    pub progress: Option<u32>,
    pub partial_result: Option<Value>,
}

/// Task completion parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCompleteParams {
    pub task_id: String,
    pub result: TaskResult,
}

/// Task cancellation parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCancelParams {
    pub task_id: String,
    pub reason: Option<String>,
}

/// Active task tracker
#[derive(Debug, Clone)]
pub struct ActiveTask {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    #[allow(dead_code)] // Available for future task execution
    pub input: Option<Value>,
    #[allow(dead_code)] // Available for future task metadata
    pub metadata: Option<std::collections::HashMap<String, Value>>,
    pub progress: u32,
    pub result: Option<TaskResult>,
    pub created_at: std::time::Instant,
}

// ============================================================
// Embedding Configuration Types
// ============================================================

/// Embedding configuration from environment
#[allow(dead_code)] // TODO: Implement embedding config in production
#[derive(Debug, Clone)]
pub struct EmbeddingEnvConfig {
    pub provider: String,
    pub api_key: Option<String>,
    #[allow(dead_code)]
    pub api_key_env: String,
    pub model: Option<String>,
    pub similarity_threshold: f32,
    pub batch_size: usize,
}

// ============================================================
// Rate Limiting Types
// ============================================================

/// Rate limit configuration from environment
#[allow(dead_code)] // TODO: Implement rate limiting in production
#[derive(Debug, Clone)]
pub struct RateLimitEnvConfig {
    pub enabled: bool,
    pub read_rps: u32,
    pub read_burst: u32,
    pub write_rps: u32,
    pub write_burst: u32,
    pub cleanup_interval_secs: u64,
    pub client_id_header: String,
}

impl Default for RateLimitEnvConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            read_rps: 100,
            read_burst: 150,
            write_rps: 20,
            write_burst: 30,
            cleanup_interval_secs: 60,
            client_id_header: "X-Client-ID".to_string(),
        }
    }
}

impl RateLimitEnvConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            enabled: std::env::var("MCP_RATE_LIMIT_ENABLED")
                .ok()
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(true),
            read_rps: std::env::var("MCP_RATE_LIMIT_READ_RPS")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(100),
            read_burst: std::env::var("MCP_RATE_LIMIT_READ_BURST")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(150),
            write_rps: std::env::var("MCP_RATE_LIMIT_WRITE_RPS")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(20),
            write_burst: std::env::var("MCP_RATE_LIMIT_WRITE_BURST")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(30),
            cleanup_interval_secs: std::env::var("MCP_RATE_LIMIT_CLEANUP_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60),
            client_id_header: std::env::var("MCP_RATE_LIMIT_CLIENT_ID_HEADER")
                .unwrap_or_else(|_| "X-Client-ID".to_string()),
        }
    }
}
