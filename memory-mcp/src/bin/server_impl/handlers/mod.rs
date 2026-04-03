//! JSON-RPC request handlers
//!
//! This module contains the tool call and batch execution handlers:
//! - handle_call_tool: Route tools/call requests to appropriate handlers
//! - handle_batch_execute: Handle batch/execute requests

mod batch;
mod tool_call;

pub use batch::handle_batch_execute;
pub use tool_call::handle_call_tool;
