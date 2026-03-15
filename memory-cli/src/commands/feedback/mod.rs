//! Feedback commands for recording recommendation sessions and feedback
//!
//! This module provides CLI commands for:
//! - `feedback record-session`: Record when recommendations are made
//! - `feedback record-feedback`: Record what was used and outcome
//! - `feedback stats`: Show recommendation effectiveness statistics

mod core;
mod types;

pub use core::{RecordFeedbackResult, RecordSessionResult, StatsResult, handle_feedback_command};
pub use types::FeedbackCommands;
