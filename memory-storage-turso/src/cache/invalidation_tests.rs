use super::*;

#[test]
fn test_invalidation_rule_matching() {
    let rule = InvalidationRule::new("%episodes%", vec![TableDependency::Episodes]);

    assert!(rule.matches("SELECT * FROM episodes"));
    assert!(rule.matches("SELECT * FROM episodes WHERE id = 1"));
    assert!(!rule.matches("SELECT * FROM patterns"));
}

#[test]
fn test_invalidation_rule_priority() {
    let rule = InvalidationRule::new("test", vec![]).with_priority(5);
    assert_eq!(rule.priority, 5);
}

#[test]
fn test_invalidation_rule_ttl() {
    let ttl = Duration::from_secs(60);
    let rule = InvalidationRule::new("test", vec![]).with_ttl(ttl);
    assert_eq!(rule.ttl_override, Some(ttl));
}

#[test]
fn test_rule_builder() {
    let rule = InvalidationRuleBuilder::new("%episodes%")
        .depends_on(TableDependency::Episodes)
        .depends_on(TableDependency::Steps)
        .with_ttl(Duration::from_secs(300))
        .with_priority(10)
        .build();

    assert_eq!(rule.pattern, "%episodes%");
    assert_eq!(rule.dependencies.len(), 2);
    assert_eq!(rule.ttl_override, Some(Duration::from_secs(300)));
    assert_eq!(rule.priority, 10);
}

#[test]
fn test_default_rules() {
    let rules = utils::default_rules();
    assert!(!rules.is_empty());

    let episode_rule = rules.iter().find(|r| r.pattern == "%episodes%");
    assert!(episode_rule.is_some());
}

#[test]
fn test_invalidation_metrics() {
    let mut metrics = InvalidationMetrics::default();

    metrics.record(&TableDependency::Episodes, CrudOperation::Insert, 5);
    metrics.record(&TableDependency::Patterns, CrudOperation::Update, 3);
    metrics.record_batch(10);

    assert_eq!(metrics.total_invalidations, 2);
    assert_eq!(metrics.entries_invalidated, 18);
    assert_eq!(metrics.batch_count, 1);
}

#[test]
fn test_invalidation_target() {
    let target_all = InvalidationTarget::All;
    let target_table = InvalidationTarget::Table(TableDependency::Episodes);
    let target_type = InvalidationTarget::Type(QueryType::Episode);

    // Just verify they can be created
    assert!(matches!(target_all, InvalidationTarget::All));
    assert!(matches!(target_table, InvalidationTarget::Table(_)));
    assert!(matches!(target_type, InvalidationTarget::Type(_)));
}

#[test]
fn test_crud_operations() {
    assert_eq!(CrudOperation::Insert, CrudOperation::Insert);
    assert_ne!(CrudOperation::Insert, CrudOperation::Update);
}

#[test]
fn test_schema_change_types() {
    assert_eq!(SchemaChangeType::Create, SchemaChangeType::Create);
    assert_ne!(SchemaChangeType::Create, SchemaChangeType::Drop);
}

#[test]
fn test_invalidation_strategy_default() {
    let strategy: InvalidationStrategy = Default::default();
    assert_eq!(strategy, InvalidationStrategy::Combined);
}

#[test]
fn test_invalidation_config_default() {
    let config = InvalidationConfig::default();
    assert_eq!(config.strategy, InvalidationStrategy::Combined);
    assert!(config.enable_background_cleanup);
    assert_eq!(config.batch_size, 100);
}

#[test]
fn test_like_pattern_matching() {
    let rule = InvalidationRule::new("%episodes%", vec![]);

    // Test various patterns
    assert!(rule.matches("SELECT * FROM episodes"));
    assert!(rule.matches("UPDATE episodes SET x = 1"));
    assert!(rule.matches("DELETE FROM episodes WHERE id = 1"));
    assert!(!rule.matches("SELECT * FROM patterns"));

    // Test prefix pattern
    let prefix_rule = InvalidationRule::new("SELECT%", vec![]);
    assert!(prefix_rule.matches("SELECT * FROM episodes"));
    assert!(!prefix_rule.matches("INSERT INTO episodes"));

    // Test suffix pattern
    let suffix_rule = InvalidationRule::new("%episodes", vec![]);
    assert!(suffix_rule.matches("FROM episodes"));
    assert!(!suffix_rule.matches("FROM episodes WHERE"));
}
