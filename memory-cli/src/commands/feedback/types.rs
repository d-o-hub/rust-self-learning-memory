//! Feedback command types

/// Feedback subcommands
#[derive(Debug, Clone, clap::Subcommand)]
pub enum FeedbackCommands {
    /// Record a recommendation session when patterns are suggested
    RecordSession {
        /// Episode ID for which recommendations are made
        #[arg(short, long)]
        episode_id: String,
        /// Pattern IDs that were recommended (comma-separated)
        #[arg(short = 'p', long, value_delimiter = ',')]
        patterns: Vec<String>,
        /// Playbook IDs that were recommended (comma-separated)
        #[arg(short = 'P', long, value_delimiter = ',')]
        playbooks: Vec<String>,
    },
    /// Record feedback about which recommendations were used
    RecordFeedback {
        /// Session ID from the recommendation session
        #[arg(short, long)]
        session: String,
        /// Pattern IDs that were applied (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        applied: Vec<String>,
        /// Episode IDs that were consulted (comma-separated)
        #[arg(short = 'e', long, value_delimiter = ',')]
        consulted: Vec<String>,
        /// Outcome: success, partial, or failure
        #[arg(short, long)]
        outcome: String,
        /// Verdict or reason message
        #[arg(short, long)]
        message: String,
        /// Agent rating (0.0-1.0)
        #[arg(short, long)]
        rating: Option<f32>,
    },
    /// Show recommendation statistics
    Stats,
}
