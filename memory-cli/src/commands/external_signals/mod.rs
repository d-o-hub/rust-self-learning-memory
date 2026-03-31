//! External signal management module.
//!
//! This module provides commands for managing external signal providers
//! that contribute additional context and signals to the memory system.
//!
//! # Commands
//!
//! - `configure` - Configure external signal providers (AgentFS, custom)
//! - `status` - Show provider status and connectivity information
//! - `test` - Test connections to configured providers
//! - `list` - List all configured providers
//!
//! # Examples
//!
//! Configure AgentFS:
//! ```bash
//! memory-cli external-signal configure agentfs --db-path ~/.agentfs/db.sqlite --weight 0.3
//! ```
//!
//! Show status:
//! ```bash
//! memory-cli external-signal status
//! memory-cli external-signal status --provider agentfs
//! ```
//!
//! Test connections:
//! ```bash
//! memory-cli external-signal test
//! memory-cli external-signal test --provider agentfs
//! ```
//!
//! List providers:
//! ```bash
//! memory-cli external-signal list
//! memory-cli external-signal list --detailed
//! ```

mod configure;
mod list;
mod status;
mod test;
pub mod types;

pub use types::ExternalSignalCommands;

use crate::config::Config;
use crate::output::OutputFormat;

/// Handle external signal commands
pub async fn handle_external_signal_command(
    command: ExternalSignalCommands,
    memory: &do_memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    match command {
        ExternalSignalCommands::Configure { provider } => match provider {
            types::ConfigureProviderArgs::AgentFs(args) => {
                configure::configure_agentfs(args, config, format).await
            }
            types::ConfigureProviderArgs::Custom(args) => {
                configure::configure_custom(args, config, format).await
            }
        },
        ExternalSignalCommands::Status { provider } => {
            status::show_status(provider, memory, config, format).await
        }
        ExternalSignalCommands::Test { provider } => {
            test::test_providers(provider, memory, config, format).await
        }
        ExternalSignalCommands::List { detailed } => {
            list::list_providers(detailed, memory, config, format).await
        }
    }
}
