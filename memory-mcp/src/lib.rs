// Clippy suppressions for memory-mcp
#![allow(clippy::useless_attribute)]
#![allow(clippy::excessive_nesting)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::panic)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::float_cmp)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unused_self)]
#![allow(clippy::unused_async)]
#![allow(clippy::similar_names)]
#![allow(clippy::to_string_in_format_args)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::format_push_string)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::option_option)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::doc_link_with_quotes)]
#![allow(clippy::single_match)]
#![allow(clippy::neg_cmp_op_on_partial_ord)]
#![allow(clippy::inefficient_to_string)]
#![allow(clippy::bool_comparison)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::cloned_instead_of_copied)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::let_and_return)]
#![allow(clippy::unused_rounding)]
#![allow(clippy::if_not_else)]
#![allow(clippy::needless_continue)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::ignore_without_reason)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ref_option)]
#![allow(clippy::single_match_else)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::if_then_some_else_none)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::redundant_else)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(missing_docs)]
#![allow(rust_2024_compatibility)]
#![allow(tail_expr_drop_order)]
#![allow(unknown_lints)]

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
