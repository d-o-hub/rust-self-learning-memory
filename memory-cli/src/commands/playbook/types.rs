//! Playbook command types

use clap::Subcommand;
use serde::Serialize;

/// Playbook commands for generating and explaining actionable playbooks
#[derive(Subcommand)]
pub enum PlaybookCommands {
    /// Generate a playbook for a task
    Recommend {
        /// Task description for playbook generation
        #[arg(value_name = "TASK")]
        task: Option<String>,

        /// Domain context for the playbook
        #[arg(short = 'D', long, default_value = "default")]
        domain: String,

        /// Task type (code_generation, debugging, refactoring, testing, analysis, documentation)
        #[arg(short = 't', long, default_value = "code_generation")]
        task_type: String,

        /// Maximum number of steps in the playbook
        #[arg(short = 's', long, default_value = "5")]
        max_steps: usize,

        /// Programming language context
        #[arg(short = 'l', long)]
        language: Option<String>,

        /// Framework context
        #[arg(short = 'f', long)]
        framework: Option<String>,

        /// Context tags (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },

    /// Explain a pattern in human-readable form
    Explain {
        /// Pattern ID to explain
        #[arg(value_name = "PATTERN_ID")]
        pattern_id: String,
    },
}

/// Summary of a recommended playbook
#[derive(Debug, Serialize)]
pub struct PlaybookSummary {
    pub playbook_id: String,
    pub task_match_score: f32,
    pub confidence: f32,
    pub why_relevant: String,
    pub step_count: usize,
    pub steps: Vec<PlaybookStepSummary>,
    pub pitfalls: Vec<String>,
    pub when_to_apply: Vec<String>,
    pub when_not_to_apply: Vec<String>,
    pub expected_outcome: String,
}

/// Summary of a playbook step
#[derive(Debug, Serialize)]
pub struct PlaybookStepSummary {
    pub order: usize,
    pub action: String,
    pub tool_hint: Option<String>,
    pub expected_result: Option<String>,
}

/// Pattern explanation result
#[derive(Debug, Serialize)]
pub struct PatternExplanation {
    pub pattern_id: String,
    pub explanation: String,
}
