//! Tests for spatiotemporal indexing.

#[cfg(test)]
mod tests {
    use crate::episode::Episode;
    use crate::indexing::spatiotemporal::{SpatiotemporalIndex, TimeBucket};
    use crate::types::{TaskContext, TaskType};
    use chrono::{DateTime, Datelike, Duration, Timelike, Utc};

    fn create_test_episode_with_time(
        domain: &str,
        task_type: TaskType,
        timestamp: DateTime<Utc>,
    ) -> Episode {
        let context = TaskContext {
            domain: domain.to_string(),
            complexity: crate::types::ComplexityLevel::Simple,
            tags: vec![],
            ..Default::default()
        };
        let mut episode = Episode::new("Test episode".to_string(), context, task_type);
        episode.start_time = timestamp;
        episode
    }

    #[test]
    fn test_index_creation() {
        let index = SpatiotemporalIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert_eq!(index.year_count(), 0);
    }

    #[test]
    fn test_insert_and_query_hour() {
        let mut index = SpatiotemporalIndex::new();
        let timestamp = Utc::now();

        let episode =
            create_test_episode_with_time("test-domain", TaskType::CodeGeneration, timestamp);
        let episode_id = episode.episode_id;

        index.insert(&episode);

        assert_eq!(index.len(), 1);
        assert_eq!(index.year_count(), 1);

        // Query by hour
        let results = index.query_hour(
            timestamp.year() as u32,
            timestamp.month() as u8,
            timestamp.day() as u8,
            timestamp.hour() as u8,
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], episode_id);
    }

    #[test]
    fn test_query_range() {
        let mut index = SpatiotemporalIndex::new();
        let now = Utc::now();

        // Insert episodes at different times
        for i in 0..5 {
            let timestamp = now - Duration::hours(i);
            let episode =
                create_test_episode_with_time("test-domain", TaskType::CodeGeneration, timestamp);
            index.insert(&episode);
        }

        assert_eq!(index.len(), 5);

        // Query last 3 hours
        let start = now - Duration::hours(3);
        let results = index.query_range(start, now, 100);

        // Should find episodes from hours 0, 1, 2, 3
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_query_bucket() {
        let mut index = SpatiotemporalIndex::new();
        let now = Utc::now();

        let episode = create_test_episode_with_time("test-domain", TaskType::CodeGeneration, now);
        index.insert(&episode);

        // Query by year bucket
        let bucket = TimeBucket::Year(now.year() as u32);
        let results = index.query_bucket(&bucket);
        assert_eq!(results.len(), 1);

        // Query by month bucket
        let bucket = TimeBucket::Month {
            year: now.year() as u32,
            month: now.month() as u8,
        };
        let results = index.query_bucket(&bucket);
        assert_eq!(results.len(), 1);

        // Query by non-existent year
        let bucket = TimeBucket::Year(1999);
        let results = index.query_bucket(&bucket);
        assert!(results.is_empty());
    }

    #[test]
    fn test_remove() {
        let mut index = SpatiotemporalIndex::new();
        let now = Utc::now();

        let episode = create_test_episode_with_time("test-domain", TaskType::CodeGeneration, now);
        let episode_id = episode.episode_id;

        index.insert(&episode);
        assert_eq!(index.len(), 1);

        let removed = index.remove(episode_id, now);
        assert!(removed);
        assert_eq!(index.len(), 0);

        // Remove non-existent episode
        let removed = index.remove(episode_id, now);
        assert!(!removed);
    }

    #[test]
    fn test_clear() {
        let mut index = SpatiotemporalIndex::new();
        let now = Utc::now();

        for _ in 0..10 {
            let episode =
                create_test_episode_with_time("test-domain", TaskType::CodeGeneration, now);
            index.insert(&episode);
        }

        assert_eq!(index.len(), 10);

        index.clear();

        assert!(index.is_empty());
        assert_eq!(index.year_count(), 0);
    }

    #[test]
    fn test_memory_usage_estimate() {
        let mut index = SpatiotemporalIndex::new();
        let now = Utc::now();

        let base_usage = index.memory_usage_estimate();

        for _ in 0..100 {
            let episode =
                create_test_episode_with_time("test-domain", TaskType::CodeGeneration, now);
            index.insert(&episode);
        }

        let usage_with_data = index.memory_usage_estimate();

        // Memory usage should increase with data
        assert!(usage_with_data > base_usage);
    }

    #[test]
    fn test_query_limit() {
        let mut index = SpatiotemporalIndex::new();
        let now = Utc::now();

        // Insert 10 episodes in the same hour
        for _ in 0..10 {
            let episode =
                create_test_episode_with_time("test-domain", TaskType::CodeGeneration, now);
            index.insert(&episode);
        }

        // Query with limit of 5
        let start = now - Duration::hours(1);
        let results = index.query_range(start, now + Duration::hours(1), 5);

        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_multiple_episodes_same_hour() {
        let mut index = SpatiotemporalIndex::new();
        let now = Utc::now();

        let episode1 = create_test_episode_with_time("test-domain", TaskType::CodeGeneration, now);
        let episode2 = create_test_episode_with_time("test-domain", TaskType::Debugging, now);

        let id1 = episode1.episode_id;
        let id2 = episode2.episode_id;

        index.insert(&episode1);
        index.insert(&episode2);

        let results = index.query_hour(
            now.year() as u32,
            now.month() as u8,
            now.day() as u8,
            now.hour() as u8,
        );

        assert_eq!(results.len(), 2);
        assert!(results.contains(&id1));
        assert!(results.contains(&id2));
    }
}
