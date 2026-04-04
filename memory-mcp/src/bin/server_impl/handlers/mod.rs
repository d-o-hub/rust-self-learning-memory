//! JSON-RPC request handlers
//!
//! This module contains the tool call and batch execution handlers:
//! - handle_call_tool: Route tools/call requests to appropriate handlers
//! - handle_batch_execute: Handle batch/execute requests

mod batch_execute;
mod call_tool;

pub use batch_execute::handle_batch_execute;
pub use call_tool::handle_call_tool;
