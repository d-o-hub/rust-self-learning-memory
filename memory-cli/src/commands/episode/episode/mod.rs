//! Episode command submodules
//!
//! This module contains the types, output implementations, and command functions
//! for the episode commands.

pub mod bulk;
pub mod complete;
pub mod create;
pub mod delete;
pub mod filter;
pub mod list;
pub mod log_step;
pub mod output;
pub mod search;
pub mod types;
pub mod view;

pub use bulk::bulk_get_episodes;
pub use complete::complete_episode;
pub use create::create_episode;
pub use delete::delete_episode;
pub use delete::DeletionResult;
pub use filter::handle_filter_command;
pub use list::list_episodes;
pub use log_step::log_step;
pub use output::*;
pub use search::search_episodes;
pub use types::*;
pub use view::view_episode;
