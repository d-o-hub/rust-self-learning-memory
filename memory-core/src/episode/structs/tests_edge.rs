use super::*;

#[test]
fn test_tag_whitespace_variations() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    // Add tag with leading whitespace
    episode.add_tag("  tag1".to_string()).unwrap();
    assert_eq!(episode.tags[0], "tag1");

    // Add tag with trailing whitespace
    episode.add_tag("tag2  ".to_string()).unwrap();
    assert_eq!(episode.tags[1], "tag2");

    // Add tag with both
    episode.add_tag("  tag3  ".to_string()).unwrap();
    assert_eq!(episode.tags[2], "tag3");

    // All should be found without whitespace
    assert!(episode.has_tag("tag1"));
    assert!(episode.has_tag("tag2"));
    assert!(episode.has_tag("tag3"));

    // Verify no duplicates from whitespace variations
    episode.add_tag(" tag1 ".to_string()).unwrap();
    assert_eq!(episode.tags.len(), 3);
}

#[test]
fn test_clear_tags_on_empty_episode() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test task".to_string(), context, TaskType::Analysis);

    assert_eq!(episode.tags.len(), 0);
    episode.clear_tags(); // Should not panic
    assert_eq!(episode.tags.len(), 0);
}
