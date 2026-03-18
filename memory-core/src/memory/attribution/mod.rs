//! Recommendation Attribution & Online Effectiveness
//!
//! This module provides types and tracking for recommendation sessions and feedback,
//! enabling the system to learn which recommendations actually help agents succeed.
//!
//! # Overview
//!
//! When the system recommends patterns or playbooks to an agent, it creates a
//! `RecommendationSession` to record what was recommended. After the agent completes
//! or abandons the task, it provides `RecommendationFeedback` indicating which
//! recommendations were used and the outcome.
//!
//! This closes the feedback loop and enables:
//! - Pattern adoption rate tracking (recommended vs. applied)
//! - Success-after-adoption rate tracking
//! - Recommendation precision metrics
//! - Improved pattern ranking based on actual effectiveness
//!
//! # Example
//!
//! ```no_run
//! use memory_core::memory::attribution::{
//!     RecommendationTracker, RecommendationSession, RecommendationFeedback,
//! };
//! use memory_core::TaskOutcome;
//! use uuid::Uuid;
//! use chrono::Utc;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let tracker = RecommendationTracker::new();
//!
//! // 1. Record a recommendation session when suggesting patterns
//! let session = RecommendationSession {
//!     session_id: Uuid::new_v4(),
//!     episode_id: Uuid::new_v4(),
//!     timestamp: Utc::now(),
//!     recommended_pattern_ids: vec!["pattern-123".to_string()],
//!     recommended_playbook_ids: vec![],
//! };
//! let session_id = session.session_id;
//! tracker.record_session(session).await;
//!
//! // 2. Later, record feedback when the agent completes the task
//! let feedback = RecommendationFeedback {
//!     session_id,
//!     applied_pattern_ids: vec!["pattern-123".to_string()],
//!     consulted_episode_ids: vec![],
//!     outcome: TaskOutcome::Success {
//!         verdict: "Task completed".to_string(),
//!         artifacts: vec![],
//!     },
//!     agent_rating: Some(0.9),
//! };
//! tracker.record_feedback(feedback).await.unwrap();
//!
//! // 3. Get statistics on recommendation effectiveness
//! let stats = tracker.get_stats().await;
//! println!("Adoption rate: {:.1}%", stats.adoption_rate * 100.0);
//! # }
//! ```

mod tracker;
mod types;

pub use tracker::RecommendationTracker;
pub use types::{
    RecommendationFeedback, RecommendationSession, RecommendationStats, SessionWithFeedback,
};
