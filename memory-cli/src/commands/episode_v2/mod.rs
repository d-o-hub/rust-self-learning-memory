//! Episode command module
//!
//! This module provides commands for managing episodes in the memory system.
//! It is split into submodules to keep each file under 500 LOC.

pub mod episode;

pub use episode::AppliedFilters;
pub use episode::DeletionResult;
pub use episode::EpisodeCommands;
pub use episode::EpisodeDetail;
pub use episode::EpisodeList;
pub use episode::EpisodeListFiltered;
pub use episode::EpisodeSearchResult;
pub use episode::EpisodeSortOrder;
pub use episode::EpisodeStatus;
pub use episode::EpisodeStep;
pub use episode::EpisodeSummary;
pub use episode::FilterCommands;
pub use episode::FilterList;
pub use episode::SavedFilter;
pub use episode::TaskOutcome;

pub use episode::bulk_get_episodes;
pub use episode::complete_episode;
pub use episode::create_episode;
pub use episode::delete;
pub use episode::delete_episode;
pub use episode::filter;
pub use episode::handle_filter_command;
pub use episode::list_episodes;
pub use episode::log_step;
pub use episode::search_episodes;
pub use episode::view_episode;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{Output, OutputFormat};

    #[test]
    fn test_episode_status_enum() {
        assert_eq!(EpisodeStatus::InProgress, EpisodeStatus::InProgress);
        assert_eq!(EpisodeStatus::Completed, EpisodeStatus::Completed);
    }

    #[test]
    fn test_task_outcome_enum() {
        assert_eq!(TaskOutcome::Success, TaskOutcome::Success);
        assert_eq!(TaskOutcome::PartialSuccess, TaskOutcome::PartialSuccess);
        assert_eq!(TaskOutcome::Failure, TaskOutcome::Failure);
    }

    #[test]
    fn test_episode_summary_output() {
        let summary = EpisodeSummary {
            episode_id: "test-id".to_string(),
            task_description: "Test task".to_string(),
            status: "completed".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            duration_ms: Some(1000),
            steps_count: 5,
        };

        let mut buffer = Vec::new();
        summary.write_human(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Episode: test-id"));
        assert!(output.contains("Task: Test task"));
        assert!(output.contains("Status: completed"));
        assert!(output.contains("Duration: 1000ms"));
        assert!(output.contains("Steps: 5"));
    }

    #[test]
    fn test_episode_list_output() {
        let summaries = vec![
            EpisodeSummary {
                episode_id: "id1".to_string(),
                task_description: "Task 1".to_string(),
                status: "completed".to_string(),
                created_at: "2023-01-01T00:00:00Z".to_string(),
                duration_ms: Some(500),
                steps_count: 3,
            },
            EpisodeSummary {
                episode_id: "id2".to_string(),
                task_description: "Task 2".to_string(),
                status: "in_progress".to_string(),
                created_at: "2023-01-01T01:00:00Z".to_string(),
                duration_ms: None,
                steps_count: 2,
            },
        ];

        let list = EpisodeList {
            episodes: summaries,
            total_count: 2,
        };

        let mut buffer = Vec::new();
        list.write_human(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("2 episodes"));
        assert!(output.contains("id1"));
        assert!(output.contains("Task 1"));
        assert!(output.contains("id2"));
        assert!(output.contains("Task 2"));
    }
}
