//! Allowlisted, parameterized SQL query construction.
//!
//! The raw entry points ([`crate::storage::RawEpisodeQuery::query`],
//! [`crate::storage::RawPatternQuery::query`]) execute caller-provided SQL and
//! cannot enforce that the text is trusted. This builder is the supported,
//! nonbreaking alternative: column names come exclusively from the per-table
//! enums below and every caller-supplied value is emitted as a `?` placeholder,
//! so generated statements cannot carry injected SQL.
//!
//! ```no_run
//! use do_memory_storage_turso::storage::query_builder::{
//!     EpisodeColumn, FilterOp, QueryBuilder,
//! };
//! # fn example() {
//! let (sql, _params) = QueryBuilder::episodes()
//!     .filter(EpisodeColumn::Domain, FilterOp::Eq, "test-domain")
//!     .order_by(EpisodeColumn::StartTime, true)
//!     .limit(100)
//!     .into_parts();
//! # let _ = sql;
//! # }
//! ```

use libsql::Value;

use crate::storage::episodes::raw_query::EPISODE_SELECT_COLUMNS;
use crate::storage::patterns::PATTERN_SELECT_COLUMNS;

/// Comparison operators permitted in a built query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    /// `=`
    Eq,
    /// `<>`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `LIKE`
    Like,
}

impl FilterOp {
    /// SQL text for this operator.
    pub const fn as_sql(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "<>",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Like => "LIKE",
        }
    }
}

/// A column of the `episodes` table that built queries may reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodeColumn {
    /// `episode_id`
    EpisodeId,
    /// `task_type`
    TaskType,
    /// `domain`
    Domain,
    /// `language`
    Language,
    /// `start_time`
    StartTime,
    /// `end_time`
    EndTime,
    /// `created_at`
    CreatedAt,
    /// `archived_at`
    ArchivedAt,
}

impl EpisodeColumn {
    /// The exact SQL column name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EpisodeId => "episode_id",
            Self::TaskType => "task_type",
            Self::Domain => "domain",
            Self::Language => "language",
            Self::StartTime => "start_time",
            Self::EndTime => "end_time",
            Self::CreatedAt => "created_at",
            Self::ArchivedAt => "archived_at",
        }
    }
}

/// A column of the `patterns` table that built queries may reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternColumn {
    /// `pattern_id`
    PatternId,
    /// `pattern_type`
    PatternType,
    /// `context_domain`
    ContextDomain,
    /// `context_language`
    ContextLanguage,
    /// `occurrence_count`
    OccurrenceCount,
    /// `created_at`
    CreatedAt,
    /// `updated_at`
    UpdatedAt,
}

impl PatternColumn {
    /// The exact SQL column name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PatternId => "pattern_id",
            Self::PatternType => "pattern_type",
            Self::ContextDomain => "context_domain",
            Self::ContextLanguage => "context_language",
            Self::OccurrenceCount => "occurrence_count",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}

/// A column type permitted in a [`QueryBuilder`].
///
/// Implementations are exhaustive enums, so a builder can never emit an
/// arbitrary caller-supplied identifier.
pub trait QueryColumn {
    /// The exact SQL column name for this variant.
    fn column_sql(self) -> &'static str;
}

impl QueryColumn for EpisodeColumn {
    fn column_sql(self) -> &'static str {
        self.as_str()
    }
}

impl QueryColumn for PatternColumn {
    fn column_sql(self) -> &'static str {
        self.as_str()
    }
}

/// Parameterized `SELECT` builder restricted to an allowlisted table/columns.
///
/// Obtain one via [`QueryBuilder::episodes`] or [`QueryBuilder::patterns`], then
/// execute it through the corresponding `query_with_params` /
/// `query_built` path.
#[derive(Debug, Clone)]
pub struct QueryBuilder<C: QueryColumn> {
    table: &'static str,
    select_columns: &'static str,
    filters: Vec<String>,
    order: Vec<String>,
    limit: Option<u32>,
    params: Vec<Value>,
    marker: std::marker::PhantomData<C>,
}

impl<C: QueryColumn> QueryBuilder<C> {
    /// Start a builder for `table` selecting `select_columns`.
    ///
    /// Prefer [`QueryBuilder::episodes`] / [`QueryBuilder::patterns`], which
    /// pin the table and the canonical select column list.
    pub const fn new(table: &'static str, select_columns: &'static str) -> Self {
        Self {
            table,
            select_columns,
            filters: Vec::new(),
            order: Vec::new(),
            limit: None,
            params: Vec::new(),
            marker: std::marker::PhantomData,
        }
    }

    /// Add an allowlisted equality/range/pattern filter.
    ///
    /// The value is always bound as a `?` placeholder.
    #[must_use]
    pub fn filter(mut self, column: C, op: FilterOp, value: impl Into<Value>) -> Self {
        self.filters
            .push(format!("{} {} ?", column.column_sql(), op.as_sql()));
        self.params.push(value.into());
        self
    }

    /// Add an allowlisted `IS NULL` / `IS NOT NULL` predicate.
    #[must_use]
    pub fn filter_null(mut self, column: C, is_null: bool) -> Self {
        let predicate = if is_null { "IS NULL" } else { "IS NOT NULL" };
        self.filters
            .push(format!("{} {}", column.column_sql(), predicate));
        self
    }

    /// Order by an allowlisted column.
    #[must_use]
    pub fn order_by(mut self, column: C, descending: bool) -> Self {
        self.order.push(format!(
            "{} {}",
            column.column_sql(),
            if descending { "DESC" } else { "ASC" }
        ));
        self
    }

    /// Cap the number of returned rows.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Render the `SELECT` statement with `?` placeholders.
    pub fn sql(&self) -> String {
        let mut sql = format!("SELECT {} FROM {}", self.select_columns, self.table);
        if !self.filters.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.filters.join(" AND "));
        }
        if !self.order.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order.join(", "));
        }
        if let Some(limit) = self.limit {
            // `limit` is a u32, so this interpolation cannot carry SQL.
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        sql
    }

    /// The bound parameter values, in placeholder order.
    pub fn params(&self) -> &[Value] {
        &self.params
    }

    /// Consume the builder, returning the SQL and its bound parameters.
    pub fn into_parts(self) -> (String, Vec<Value>) {
        let sql = self.sql();
        (sql, self.params)
    }
}

impl QueryBuilder<EpisodeColumn> {
    /// Builder over the `episodes` table with the canonical select columns.
    pub const fn episodes() -> Self {
        Self::new("episodes", EPISODE_SELECT_COLUMNS)
    }
}

impl QueryBuilder<PatternColumn> {
    /// Builder over the `patterns` table with the canonical select columns.
    pub const fn patterns() -> Self {
        Self::new("patterns", PATTERN_SELECT_COLUMNS)
    }
}

/// Allowlisted builder specialized for episode rows.
pub type EpisodeQueryBuilder = QueryBuilder<EpisodeColumn>;

/// Allowlisted builder specialized for pattern rows.
pub type PatternQueryBuilder = QueryBuilder<PatternColumn>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episode_builder_uses_placeholders_for_values() {
        let (sql, params) = EpisodeQueryBuilder::episodes()
            .filter(
                EpisodeColumn::Domain,
                FilterOp::Eq,
                "x'; DROP TABLE episodes; --",
            )
            .limit(5)
            .into_parts();

        assert!(sql.starts_with("SELECT "));
        assert!(sql.contains("FROM episodes"));
        assert!(sql.contains("domain = ?"));
        assert!(sql.contains("LIMIT 5"));
        assert!(!sql.contains("DROP TABLE"), "payload must not reach SQL");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_pattern_builder_orders_and_nulls() {
        let (sql, params) = PatternQueryBuilder::patterns()
            .filter_null(PatternColumn::ContextLanguage, false)
            .order_by(PatternColumn::OccurrenceCount, true)
            .into_parts();

        assert!(sql.contains("context_language IS NOT NULL"));
        assert!(sql.contains("ORDER BY occurrence_count DESC"));
        assert_eq!(params.len(), 0, "filter_null must bind no parameters");
    }

    /// Every operator renders its own SQL text and binds exactly one value.
    #[test]
    fn test_all_filter_ops_emit_their_operator() {
        for (op, expected) in [
            (FilterOp::Eq, "="),
            (FilterOp::Ne, "<>"),
            (FilterOp::Lt, "<"),
            (FilterOp::Le, "<="),
            (FilterOp::Gt, ">"),
            (FilterOp::Ge, ">="),
            (FilterOp::Like, "LIKE"),
        ] {
            assert_eq!(op.as_sql(), expected);
            let (sql, params) = EpisodeQueryBuilder::episodes()
                .filter(EpisodeColumn::Domain, op, "value")
                .into_parts();
            assert!(
                sql.contains(&format!("domain {expected} ?")),
                "{op:?} must render '{expected}' with a placeholder: {sql}"
            );
            assert_eq!(params.len(), 1);
        }
    }

    /// Every episode column maps to its exact allowlisted name.
    #[test]
    fn test_all_episode_columns_are_allowlisted_names() {
        for (column, expected) in [
            (EpisodeColumn::EpisodeId, "episode_id"),
            (EpisodeColumn::TaskType, "task_type"),
            (EpisodeColumn::Domain, "domain"),
            (EpisodeColumn::Language, "language"),
            (EpisodeColumn::StartTime, "start_time"),
            (EpisodeColumn::EndTime, "end_time"),
            (EpisodeColumn::CreatedAt, "created_at"),
            (EpisodeColumn::ArchivedAt, "archived_at"),
        ] {
            assert_eq!(column.as_str(), expected);
            assert_eq!(QueryColumn::column_sql(column), expected);
        }
    }

    /// Every pattern column maps to its exact allowlisted name.
    #[test]
    fn test_all_pattern_columns_are_allowlisted_names() {
        for (column, expected) in [
            (PatternColumn::PatternId, "pattern_id"),
            (PatternColumn::PatternType, "pattern_type"),
            (PatternColumn::ContextDomain, "context_domain"),
            (PatternColumn::ContextLanguage, "context_language"),
            (PatternColumn::OccurrenceCount, "occurrence_count"),
            (PatternColumn::CreatedAt, "created_at"),
            (PatternColumn::UpdatedAt, "updated_at"),
        ] {
            assert_eq!(column.as_str(), expected);
            assert_eq!(QueryColumn::column_sql(column), expected);
        }
    }

    /// A builder with no clauses renders a bare `SELECT` and binds nothing.
    #[test]
    fn test_builder_without_clauses_has_no_where_order_or_limit() {
        let (sql, params) = EpisodeQueryBuilder::episodes().into_parts();

        assert!(sql.contains("FROM episodes"));
        assert!(!sql.contains("WHERE"));
        assert!(!sql.contains("ORDER BY"));
        assert!(!sql.contains("LIMIT"));
        assert!(params.is_empty());
    }

    /// `filter_null(true)` renders `IS NULL` and ascending order renders `ASC`.
    #[test]
    fn test_null_filter_and_ascending_order() {
        let (sql, params) = EpisodeQueryBuilder::episodes()
            .filter_null(EpisodeColumn::EndTime, true)
            .order_by(EpisodeColumn::StartTime, false)
            .limit(7)
            .into_parts();

        assert!(sql.contains("end_time IS NULL"));
        assert!(sql.contains("ORDER BY start_time ASC"));
        assert!(sql.contains("LIMIT 7"));
        assert!(params.is_empty());
    }

    /// Several filters are conjoined and their values keep placeholder order.
    #[test]
    fn test_multiple_filters_join_with_and() {
        let (sql, params) = EpisodeQueryBuilder::episodes()
            .filter(EpisodeColumn::Domain, FilterOp::Eq, "d")
            .filter(EpisodeColumn::Language, FilterOp::Like, "rust%")
            .into_parts();

        assert!(sql.contains("domain = ? AND language LIKE ?"), "{sql}");
        assert_eq!(params.len(), 2);
    }
}
