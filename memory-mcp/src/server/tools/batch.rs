// Batch execution support for MCP server
//!
//! This module re-exports batch execution types and provides integration
//! with the MCP server for batch operations.
//!
//! The core batch execution logic is in `crate::batch`.
//! This module provides server-level integration.

pub use crate::batch::{
    BatchExecutor, BatchMode, BatchOperation, BatchRequest, BatchResponse, BatchStats,
    DependencyGraph, OperationError, OperationResult,
};
