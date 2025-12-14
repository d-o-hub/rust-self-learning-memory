//! # Memory MCP (Model Context Protocol) Integration
//!
//! This crate provides MCP server integration for the self-learning memory system,
//! enabling secure code execution and memory queries through standardized tool interfaces.
//!
//! ## Features
//!
//! - **MCP Server**: Standard MCP server implementation with tool definitions
//! - **Secure Sandbox**: TypeScript/JavaScript code execution with comprehensive security
//! - **Memory Integration**: Query episodic memory and analyze patterns
//! - **Progressive Disclosure**: Tools are prioritized based on usage patterns
//! - **Execution Monitoring**: Detailed statistics and logging
//!
//! ## Security
//!
//! The sandbox implements defense-in-depth security:
//!
//! 1. **Input Validation**: All code is scanned for malicious patterns
//! 2. **Process Isolation**: Code runs in separate Node.js processes
//! 3. **Resource Limits**: CPU and memory usage are constrained
//! 4. **Timeout Enforcement**: Long-running code is terminated
//! 5. **Access Controls**: Network and filesystem access denied by default
//! 6. **Malicious Code Detection**: Common attack patterns are blocked
//!
//! ## Example
//!
//! ```no_run
//! use memory_core::SelfLearningMemory;
//! use memory_mcp::server::MemoryMCPServer;
//! use memory_mcp::types::{SandboxConfig, ExecutionContext};
//! use serde_json::json;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create memory and server with restrictive sandbox
//!     let memory = Arc::new(SelfLearningMemory::new());
//!     let server = MemoryMCPServer::new(SandboxConfig::restrictive(), memory).await?;
//!
//!     // List available tools
//!     let tools = server.list_tools().await;
//!     for tool in tools {
//!         println!("Tool: {} - {}", tool.name, tool.description);
//!     }
//!
//!     // Execute code securely
//!     let code = r#"
//!         const result = {
//!             sum: 1 + 1,
//!             message: "Hello from sandbox"
//!         };
//!         return result;
//!     "#;
//!
//!     let context = ExecutionContext::new(
//!         "Calculate sum".to_string(),
//!         json!({"a": 1, "b": 1}),
//!     );
//!
//!     let result = server.execute_agent_code(code.to_string(), context).await?;
//!     println!("Result: {:?}", result);
//!
//!     // Get execution statistics
//!     let stats = server.get_stats().await;
//!     println!(
//!         "Executed {} times, success rate: {:.1}%",
//!         stats.total_executions,
//!         stats.success_rate()
//!     );
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    MCP Server                           │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
//! │  │ Query Memory │  │Execute Code  │  │Analyze       │ │
//! │  │              │  │              │  │Patterns      │ │
//! │  └──────────────┘  └──────────────┘  └──────────────┘ │
//! └───────────────────────┬─────────────────────────────────┘
//!                         │
//!          ┌──────────────┴──────────────┐
//!          ▼                             ▼
//! ┌─────────────────┐          ┌──────────────────┐
//! │  Code Sandbox   │          │ Memory System    │
//! │  - Validation   │          │ - Episodes       │
//! │  - Isolation    │          │ - Patterns       │
//! │  - Timeout      │          │ - Heuristics     │
//! │  - Limits       │          │                  │
//! └─────────────────┘          └──────────────────┘
//! ```

pub mod cache;
pub mod error;
pub mod jsonrpc;
pub mod mcp;
pub mod monitoring;
pub mod patterns;
pub mod sandbox;
pub mod server;
pub mod types;
pub mod unified_sandbox;
pub mod javy_compiler;
pub mod wasm_sandbox;
pub mod wasmtime_sandbox;

// Re-export commonly used types
pub use cache::{CacheConfig, CacheStats, QueryCache};
pub use error::{Error, Result};
pub use sandbox::CodeSandbox;
pub use server::MemoryMCPServer;
pub use types::{
    ErrorType, ExecutionContext, ExecutionResult, ExecutionStats, ResourceLimits, SandboxConfig,
    SecurityViolationType, Tool,
};

// Re-export security modules
pub use sandbox::{FileSystemRestrictions, IsolationConfig, NetworkRestrictions};

// Re-export new WASM modules
pub use unified_sandbox::{BackendHealth, SandboxBackend, UnifiedMetrics, UnifiedSandbox};
#[cfg(feature = "wasm-rquickjs")]
pub use wasm_sandbox::{WasmConfig, WasmMetrics, WasmSandbox};
#[cfg(feature = "javy-backend")]
pub use javy_compiler::{JavyConfig, JavyMetrics, JavyCompiler};
pub use wasmtime_sandbox::{WasmtimeConfig, WasmtimeMetrics, WasmtimeSandbox};
