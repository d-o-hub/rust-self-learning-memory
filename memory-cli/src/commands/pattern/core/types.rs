//! Pattern command types and structures

#[cfg(feature = "turso")]
use super::batch::PatternBatchCommands;
use clap::{Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Subcommand)]
pub enum PatternCommands {
    /// List patterns
    List {
        /// Minimum confidence threshold
        #[arg(long, default_value = "0.0")]
        min_confidence: f32,

        /// Filter by pattern type
        #[arg(short, long)]
        pattern_type: Option<PatternType>,

        /// Maximum number of patterns to return
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// View pattern details
    View {
        /// Pattern ID
        #[arg(value_name = "PATTERN_ID")]
        pattern_id: String,
    },

    /// Analyze pattern effectiveness
    Analyze {
        /// Pattern ID
        #[arg(value_name = "PATTERN_ID")]
        pattern_id: String,

        /// Number of episodes to analyze
        #[arg(short, long, default_value = "100")]
        episodes: usize,
    },

    /// Show pattern effectiveness rankings
    Effectiveness {
        /// Show top N patterns
        #[arg(short, long, default_value = "10")]
        top: usize,

        /// Minimum number of uses
        #[arg(long, default_value = "1")]
        min_uses: usize,
    },

    /// Apply pattern decay
    Decay {
        /// Show what would be done without executing
        #[arg(long)]
        dry_run: bool,

        /// Force decay without confirmation
        #[arg(long)]
        force: bool,
    },

    /// Batch pattern operations (4-6x faster)
    #[cfg(feature = "turso")]
    Batch {
        /// Batch operation to perform
        #[command(subcommand)]
        command: PatternBatchCommands,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PatternType {
    /// Tool sequence patterns
    ToolSequence,
    /// Decision point patterns
    DecisionPoint,
    /// Error recovery patterns
    ErrorRecovery,
    /// Context patterns
    ContextPattern,
}

#[derive(Debug, Serialize)]
pub struct PatternSummary {
    pub pattern_id: String,
    pub pattern_type: String,
    pub confidence: f32,
    pub effectiveness: f32,
    pub use_count: usize,
    pub last_used: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct PatternList {
    pub patterns: Vec<PatternSummary>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct PatternAnalysis {
    pub pattern_id: String,
    pub analysis: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct PatternAnalysisData {
    pub success_rate: f32,
    pub average_improvement: f32,
    pub episodes_analyzed: usize,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PatternAnalysisResult {
    pub pattern_id: String,
    pub analysis: PatternAnalysisData,
}

#[derive(Debug, Serialize)]
pub struct EffectivenessRankings {
    pub rankings: Vec<EffectivenessRanking>,
    pub total_patterns_analyzed: usize,
}

#[derive(Debug, Serialize)]
pub struct EffectivenessRanking {
    pub rank: usize,
    pub pattern_id: String,
    pub effectiveness_score: f32,
    pub use_count: usize,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct PatternDetail {
    pub id: String,
    pub pattern_type: String,
    pub confidence: f32,
    pub context: serde_json::Value,
    pub effectiveness_data: serde_json::Value,
    pub extracted_at: String,
}

#[derive(Debug, Serialize)]
pub struct PatternDecayResult {
    pub pattern_id: String,
    pub old_confidence: f32,
    pub new_confidence: f32,
    pub decay_applied: bool,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct PatternDecaySummary {
    pub patterns_affected: usize,
    pub total_decay: f32,
    pub dry_run: bool,
    pub results: Vec<PatternDecayResult>,
}
