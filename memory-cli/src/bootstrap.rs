//! Commands that must work before configuration and storage are available.

use clap::CommandFactory;

use crate::commands::{ConfigCommands, config};
use crate::{Cli, Commands};

pub async fn try_run(command: &Commands) -> Option<anyhow::Result<()>> {
    Some(match command {
        Commands::Completion { shell } => {
            clap_complete::generate(
                *shell,
                &mut Cli::command(),
                "do-memory-cli",
                &mut std::io::stdout(),
            );
            Ok(())
        }
        Commands::Config { command } => match command {
            ConfigCommands::Init { path } => config::init_config(path).await,
            ConfigCommands::ShowTemplate => config::show_config_template().await,
            ConfigCommands::Wizard => config::run_wizard().await,
            _ => return None,
        },
        _ => return None,
    })
}
