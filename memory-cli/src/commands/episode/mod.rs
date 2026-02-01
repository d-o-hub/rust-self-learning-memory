//! Episode command module
//!
//! This module provides commands for managing episodes in the memory system.
//! It is split into submodules to keep each file under 500 LOC.

pub mod core;
pub mod relationships;

pub use core::AppliedFilters;
pub use core::DeletionResult;
pub use core::EpisodeCommands;
pub use core::EpisodeDetail;
pub use core::EpisodeList;
pub use core::EpisodeListFiltered;
pub use core::EpisodeSearchResult;
pub use core::EpisodeSortOrder;
pub use core::EpisodeStatus;
pub use core::EpisodeStep;
pub use core::EpisodeSummary;
pub use core::FilterCommands;
pub use core::FilterList;
pub use core::SavedFilter;
pub use core::TaskOutcome;

pub use core::bulk_get_episodes;
pub use core::complete_episode;
pub use core::create_episode;
pub use core::delete;
pub use core::delete_episode;
pub use core::filter;
pub use core::handle_filter_command;
pub use core::list_episodes;
pub use core::log_step;
pub use core::search_episodes;
pub use core::view_episode;

// Re-export relationship commands and types
pub use relationships::add_relationship;
pub use relationships::dependency_graph;
pub use relationships::find_related;
pub use relationships::list_relationships;
pub use relationships::remove_relationship;
pub use relationships::topological_sort;
pub use relationships::validate_cycles;
pub use relationships::AddRelationshipResult;
pub use relationships::DependencyGraphResult;
pub use relationships::DirectionArg;
pub use relationships::FindRelatedResult;
pub use relationships::GraphFormat;
pub use relationships::ListFormat;
pub use relationships::ListRelationshipsResult;
pub use relationships::RelatedEpisodeItem;
pub use relationships::RelationshipCommands;
pub use relationships::RelationshipListItem;
pub use relationships::RelationshipTypeArg;
pub use relationships::RemoveRelationshipResult;
pub use relationships::TopologicalSortResult;
pub use relationships::ValidateCyclesResult;

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
