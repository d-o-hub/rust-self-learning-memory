//! # MCP 2025-11-25 Protocol Handlers
//!
//! Handler modules for MCP 2025-11-25 protocol features:
//! - Completion handlers (completion/complete)
//! - Elicitation handlers (elicitation/request, data, cancel)
//! - Task handlers (task/create, update, complete, cancel, list)

pub mod completion;
pub mod elicitation;
pub mod tasks;

// Re-export all handler functions for convenience
pub use completion::handle_completion_complete;
pub use elicitation::{
    handle_elicitation_cancel, handle_elicitation_data, handle_elicitation_request,
};
pub use tasks::{
    handle_task_cancel, handle_task_complete, handle_task_create, handle_task_list,
    handle_task_update,
};
