#![deny(unsafe_code)]
// Clippy suppressions for memory-mcp
#![expect(clippy::excessive_nesting, reason = "deeply nested control flow")]
#![expect(
    clippy::missing_errors_doc,
    reason = "error variants documented separately"
)]
#![expect(
    clippy::missing_panics_doc,
    reason = "panic paths documented separately"
)]
#![expect(
    clippy::cognitive_complexity,
    reason = "long-standing complex functions"
)]
#![expect(
    clippy::must_use_candidate,
    reason = "not every public value is must_use"
)]
#![expect(clippy::doc_markdown, reason = "identifier backticks noisy in prose")]
#![cfg_attr(
    test,
    expect(clippy::panic, reason = "panic used for invariant violations")
)]
#![cfg_attr(
    not(test),
    expect(clippy::wildcard_imports, reason = "glob imports used for preludes")
)]
#![expect(
    clippy::cast_precision_loss,
    reason = "precision loss accepted in metric math"
)]
#![expect(
    clippy::cast_possible_truncation,
    reason = "integer narrowing bounded by checks"
)]
#![expect(
    clippy::cast_sign_loss,
    reason = "non-negative values cast to unsigned"
)]
#![expect(
    clippy::cast_possible_wrap,
    reason = "range validated before wrapping cast"
)]
#![expect(
    clippy::unreadable_literal,
    reason = "literals kept verbatim for clarity"
)]
#![expect(
    clippy::struct_excessive_bools,
    reason = "config struct expressed as flags"
)]
#![expect(clippy::default_trait_access, reason = "explicit Default::default")]
#![expect(clippy::match_same_arms, reason = "explicit arms aid maintenance")]
#![expect(
    clippy::redundant_closure_for_method_calls,
    reason = "explicit closures aid readability"
)]
#![expect(clippy::map_unwrap_or, reason = "explicit pattern clearer than map_or")]
#![expect(
    clippy::needless_pass_by_value,
    reason = "owned params kept for ergonomics"
)]
#![expect(clippy::unused_self, reason = "method kept for API symmetry")]
#![expect(clippy::unused_async, reason = "async kept for API symmetry")]
#![expect(clippy::similar_names, reason = "short names common in math code")]
#![expect(
    clippy::cast_lossless,
    reason = "explicit widening casts document intent"
)]
#![expect(
    clippy::format_push_string,
    reason = "push_str formatting kept explicit"
)]
#![expect(clippy::explicit_iter_loop, reason = "explicit iterator loop kept")]
#![expect(clippy::implicit_hasher, reason = "concrete hash maps in public API")]
#![expect(
    clippy::doc_link_with_quotes,
    reason = "quoted doc links kept readable"
)]
#![expect(
    clippy::inefficient_to_string,
    reason = "to_string kept for uniformity"
)]
#![expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "small types passed by ref uniformly"
)]
#![expect(clippy::manual_let_else, reason = "match kept for clarity")]
#![expect(clippy::unnecessary_wraps, reason = "Result kept for API consistency")]
#![expect(
    clippy::cloned_instead_of_copied,
    reason = "cloned() kept for uniformity"
)]
#![expect(
    clippy::used_underscore_binding,
    reason = "underscore-prefixed binding is read"
)]
#![expect(clippy::if_not_else, reason = "positive-first form reads better")]
#![expect(clippy::needless_continue, reason = "continue kept for explicitness")]
#![expect(clippy::uninlined_format_args, reason = "format args kept explicit")]
#![cfg_attr(
    test,
    expect(
        clippy::items_after_statements,
        reason = "helpers declared next to use"
    )
)]
#![expect(clippy::ref_option, reason = "&Option kept for API consistency")]
#![expect(clippy::single_match_else, reason = "explicit arms aid maintenance")]
#![expect(clippy::if_then_some_else_none, reason = "explicit if/else clearer")]
#![expect(clippy::expect_used, reason = "infallible expect in known-good paths")]
#![expect(missing_docs, reason = "public docs still incomplete")]

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
pub mod constants;
pub mod error;
pub mod jsonrpc;
pub mod mcp;
pub mod monitoring;
pub mod patterns;
pub mod protocol;
/// Legacy Node sandbox executor — **not compiled by default** (S1.1b).
///
/// Enable with `--features sandbox-dev` for trusted local experimentation only.
/// Production MCP does not register a working `execute_agent_code` tool.
#[cfg(feature = "sandbox-dev")]
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
#[cfg(feature = "sandbox-dev")]
pub use sandbox::CodeSandbox;
#[cfg(feature = "sandbox-dev")]
pub use sandbox::{FileSystemRestrictions, IsolationConfig, NetworkRestrictions};
pub use server::MemoryMCPServer;
pub use server::audit::{
    AuditConfig, AuditDestination, AuditFileWriter, AuditLogEntry, AuditLogLevel, AuditLogger,
    DEFAULT_AUDIT_WRITE_QUEUE_CAPACITY, WriterConfig, redact_sensitive_data,
};
pub use server::tools::core::QueryMemoryRequest;
pub use types::{
    ErrorType, ExecutionContext, ExecutionResult, ExecutionStats, ResourceLimits, SandboxConfig,
    SecurityViolationType, Tool,
};
