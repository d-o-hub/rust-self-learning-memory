use crate::cache::query_cache::{QueryKey, QueryType, TableDependency};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InvalidationStrategy {
    TimeBased,
    DependencyBased,
    EventDriven,
    #[default]
    Combined,
    Custom,
}

#[derive(Debug, Clone)]
pub struct InvalidationRule {
    pub pattern: String,
    pub dependencies: Vec<TableDependency>,
    pub ttl_override: Option<Duration>,
    pub priority: u32,
}

impl InvalidationRule {
    pub fn new(pattern: impl Into<String>, dependencies: Vec<TableDependency>) -> Self {
        Self {
            pattern: pattern.into(),
            dependencies,
            ttl_override: None,
            priority: 0,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl_override = Some(ttl);
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn matches(&self, sql: &str) -> bool {
        let sql_lower = sql.to_lowercase();
        let pattern_lower = self.pattern.to_lowercase();

        if pattern_lower.contains('%') {
            self.like_match(&sql_lower, &pattern_lower)
        } else {
            sql_lower.contains(&pattern_lower)
        }
    }

    fn like_match(&self, text: &str, pattern: &str) -> bool {
        let parts: Vec<&str> = pattern.split('%').collect();

        if parts.is_empty() {
            return true;
        }

        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            if i == 0 && !text.starts_with(part) {
                return false;
            }

            if let Some(found) = text[pos..].find(part) {
                pos += found + part.len();
            } else {
                return false;
            }
        }

        if !pattern.ends_with('%') && pos < text.len() {
            return parts
                .last()
                .map(|last_part| text.ends_with(last_part))
                .unwrap_or(false);
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct InvalidationConfig {
    pub strategy: InvalidationStrategy,
    pub enable_background_cleanup: bool,
    pub cleanup_interval: Duration,
    pub enable_event_listening: bool,
    pub max_rules: usize,
    pub batch_size: usize,
    pub enable_metrics: bool,
}

impl Default for InvalidationConfig {
    fn default() -> Self {
        Self {
            strategy: InvalidationStrategy::Combined,
            enable_background_cleanup: true,
            cleanup_interval: Duration::from_secs(60),
            enable_event_listening: true,
            max_rules: 100,
            batch_size: 100,
            enable_metrics: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidationEvent {
    TableModified {
        table: TableDependency,
        operation: CrudOperation,
        affected_rows: u64,
    },
    BatchCompleted {
        tables: Vec<TableDependency>,
        operation_count: u64,
    },
    SchemaChanged {
        table: TableDependency,
        change_type: SchemaChangeType,
    },
    ManualInvalidation {
        target: InvalidationTarget,
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrudOperation {
    Insert,
    Update,
    Delete,
    Upsert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaChangeType {
    Create,
    Alter,
    Drop,
    Index,
}

#[derive(Debug, Clone)]
pub enum InvalidationTarget {
    All,
    Table(TableDependency),
    Query(QueryKey),
    Pattern(String),
    Type(QueryType),
}

#[derive(Debug, Clone, Default)]
pub struct InvalidationMetrics {
    pub total_invalidations: u64,
    pub by_table: HashMap<TableDependency, u64>,
    pub by_operation: HashMap<CrudOperation, u64>,
    pub batch_count: u64,
    pub avg_invalidation_time_us: u64,
    pub entries_invalidated: u64,
}

impl InvalidationMetrics {
    pub fn record(&mut self, table: &TableDependency, operation: CrudOperation, entries: u64) {
        self.total_invalidations += 1;
        *self.by_table.entry(table.clone()).or_default() += 1;
        *self.by_operation.entry(operation).or_default() += 1;
        self.entries_invalidated += entries;
    }

    pub fn record_batch(&mut self, count: u64) {
        self.batch_count += 1;
        self.entries_invalidated += count;
    }
}

pub struct InvalidationRuleBuilder {
    pattern: String,
    dependencies: Vec<TableDependency>,
    ttl_override: Option<Duration>,
    priority: u32,
}

impl InvalidationRuleBuilder {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            dependencies: Vec::new(),
            ttl_override: None,
            priority: 0,
        }
    }

    pub fn depends_on(mut self, table: TableDependency) -> Self {
        self.dependencies.push(table);
        self
    }

    pub fn depends_on_many(mut self, tables: Vec<TableDependency>) -> Self {
        self.dependencies.extend(tables);
        self
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl_override = Some(ttl);
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn build(self) -> InvalidationRule {
        InvalidationRule {
            pattern: self.pattern,
            dependencies: self.dependencies,
            ttl_override: self.ttl_override,
            priority: self.priority,
        }
    }
}

pub mod utils {
    use super::*;

    pub fn episode_queries_rule() -> InvalidationRule {
        InvalidationRuleBuilder::new("%episodes%")
            .depends_on(TableDependency::Episodes)
            .depends_on(TableDependency::Steps)
            .with_priority(10)
            .build()
    }

    pub fn pattern_queries_rule() -> InvalidationRule {
        InvalidationRuleBuilder::new("%patterns%")
            .depends_on(TableDependency::Patterns)
            .with_priority(10)
            .build()
    }

    pub fn statistics_queries_rule() -> InvalidationRule {
        InvalidationRuleBuilder::new("%count%")
            .depends_on(TableDependency::Episodes)
            .depends_on(TableDependency::Patterns)
            .depends_on(TableDependency::Steps)
            .with_ttl(Duration::from_secs(30))
            .with_priority(5)
            .build()
    }

    pub fn search_queries_rule() -> InvalidationRule {
        InvalidationRuleBuilder::new("%search%")
            .depends_on(TableDependency::Episodes)
            .depends_on(TableDependency::Patterns)
            .depends_on(TableDependency::Embeddings)
            .with_ttl(Duration::from_secs(120))
            .with_priority(8)
            .build()
    }

    pub fn default_rules() -> Vec<InvalidationRule> {
        vec![
            episode_queries_rule(),
            pattern_queries_rule(),
            statistics_queries_rule(),
            search_queries_rule(),
        ]
    }
}
