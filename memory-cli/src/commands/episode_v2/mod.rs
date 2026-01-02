//! Episode command module
//!
//! This module provides commands for managing episodes in the memory system.
//! It is split into submodules to keep each file under 500 LOC.

pub mod episode;

pub use episode::EpisodeCommands;
pub use episode::EpisodeDetail;
pub use episode::EpisodeList;
pub use episode::EpisodeSearchResult;
pub use episode::EpisodeStatus;
pub use episode::EpisodeStep;
pub use episode::EpisodeSummary;
pub use episode::TaskOutcome;

pub use episode::complete_episode;
pub use episode::create_episode;
pub use episode::list_episodes;
pub use episode::log_step;
pub use episode::search_episodes;
pub use episode::view_episode;
