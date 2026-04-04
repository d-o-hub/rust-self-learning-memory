#![allow(clippy::useless_attribute)]
#![allow(clippy::excessive_nesting)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::ifs_same_cond)]

//! # Memory MCP (Model Context Protocol) Integration
//!
//! This crate provides MCP server integration for the self-learning memory system,
//! enabling memory queries and pattern analysis through standardized tool interfaces.
//!
//! ## Features
//!
//! - **MCP Server**: Standard MCP server implementation with tool definitions
//! - **Memory Integration**: Query episodic memory and analyze patterns
//! - **Progressive Disclosure**: Tools are prioritized based on usage patterns
//! - **Execution Monitoring**: Detailed statistics and logging
//!
//! ## Example
//!
//! ```no_run
//! use do_memory_core::SelfLearningMemory;
//! use do_memory_mcp::server::MemoryMCPServer;
//! use do_memory_mcp::types::SandboxConfig;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create memory and server
//!     let memory = Arc::new(SelfLearningMemory::new());
//!     let server = MemoryMCPServer::new(SandboxConfig::default(), memory).await?;
//!
//!     // List available tools
//!     let tools = server.list_tools().await;
//!     for tool in tools {
//!         println!("Tool: {} - {}", tool.name, tool.description);
//!     }
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
//! │  │ Query Memory │  │ Analyze      │  │ Execute      │ │
//! │  │              │  │ Patterns     │  │ Operations   │ │
//! │  └──────────────┘  └──────────────┘  └──────────────┘ │
//! └───────────────────────┬─────────────────────────────────┘
//!                         │
//!          ┌──────────────┴──────────────┐
//!          ▼                             ▼
//! ┌─────────────────┐          ┌──────────────────┐
//! │  Memory System  │          │  Monitoring      │
//! │  - Episodes     │          │  - Metrics       │
//! │  - Patterns     │          │  - Health        │
//! │  - Heuristics   │          │                  │
//! └─────────────────┘          └──────────────────┘
//! ```

pub mod batch;
pub mod cache;
pub mod error;
pub mod jsonrpc;
pub mod mcp;
pub mod monitoring;
pub mod patterns;
pub mod protocol;
pub mod sandbox;
pub mod server;
pub mod types;

// Re-export commonly used types
pub use batch::{
    BatchExecutor, BatchMode, BatchOperation, BatchRequest, BatchResponse, BatchStats,
    DependencyGraph, OperationError, OperationResult,
};
pub use cache::{CacheConfig, CacheStats, QueryCache};
pub use error::{Error, Result};
pub use sandbox::CodeSandbox;
pub use sandbox::{FileSystemRestrictions, IsolationConfig, NetworkRestrictions};
pub use server::MemoryMCPServer;
pub use server::audit::{AuditConfig, AuditDestination, AuditLogEntry, AuditLogLevel, AuditLogger};
pub use types::{
    ErrorType, ExecutionContext, ExecutionResult, ExecutionStats, ResourceLimits, SandboxConfig,
    SecurityViolationType, Tool,
};
