//! # External Signal Provider MCP Tools
//!
//! This module contains MCP tools for configuring and managing external
//! signal providers (like AgentFS) that feed into the reward system.

mod tool;
mod types;

pub use tool::{configure_agentfs_tool, external_signal_status_tool, test_agentfs_connection_tool};
pub use types::{
    ConfigureAgentFsInput, ConfigureAgentFsOutput, ExternalSignalStatusInput,
    ExternalSignalStatusOutput, ProviderStatus, TestAgentFsConnectionInput,
    TestAgentFsConnectionOutput,
};
