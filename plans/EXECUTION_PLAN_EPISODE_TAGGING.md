# GOAP Execution Plan: Episode Tagging Implementation & Code Quality

**Date**: 2026-01-28
**Coordinator**: GOAP Agent
**Status**: Ready to Execute

## Executive Summary

This plan addresses three priorities:

1. **P0 - Critical Blocker**: Fix compilation errors preventing any development work
2. **P1 - Primary Feature**: Implement Episode Tagging Phase 1 (Core Data Model & Storage)
3. **P2 - Code Quality**: Split 4 oversized files to maintain 500 LOC compliance

## Status Assessment

### Completed (No Action Needed)
- ✅ Prepared Statement Cache (`memory-storage-turso/src/prepared/`)
- ✅ Metrics Module (`memory-storage-turso/src/metrics/`)
- ✅ Batch Operations (`memory-storage-turso/src/storage/batch/`)
- ✅ Files mentioned in GOAP_EXECUTION_PLAN.md already split (lib.rs, episodes.rs, keepalive.rs, compression.rs, cache/tests.rs)

### Critical Blocker
- ❌ Compilation errors in `memory-mcp/src/mcp/tools/embeddings/tool/execute.rs`
  - `EmbeddingProviderType` import doesn't exist (should be `EmbeddingProvider`)
  - `provider_config` field access outdated (should be `provider`)

### Ready to Start
- 🚀 Episode Tagging Phase 1 (v0.1.13)
  - Feature spec complete: `plans/EPISODE_TAGGING_FEATURE_SPEC.md`
  - Implementation roadmap: `plans/EPISODE_TAGGING_IMPLEMENTATION_ROADMAP.md`

### Code Quality Issues
- ⚠️ 4 files exceed 500 LOC:
  1. `memory-storage-turso/src/cache/adaptive_ttl.rs` (645 lines)
  2. `memory-storage-turso/src/storage/mod.rs` (562 lines)
  3. `memory-storage-turso/src/pool/adaptive.rs` (526 lines)
  4. `memory-storage-turso/src/storage/tag_operations.rs` (517 lines)

## Phase 1: Fix Compilation Errors (P0 - Critical Path)

**Duration**: 30 minutes
**Agent**: rust-specialist
**Priority**: BLOCKING - Must complete before any other work

### Tasks

#### 1.1 Fix Import Errors
**File**: `memory-mcp/src/mcp/tools/embeddings/tool/execute.rs`

**Changes Required**:
```rust
// Line 13: Remove EmbeddingProviderType, add EmbeddingProvider
- EmbeddingConfig, EmbeddingProviderType, ProviderConfig,
+ EmbeddingConfig, EmbeddingProvider, ProviderConfig,

// Lines 11-12: Remove unused imports
- mistral::MistralConfig, openai::OpenAIConfig, AzureOpenAIConfig, CustomConfig, LocalConfig,
+ // These are now inside ProviderConfig
```

#### 1.2 Fix Field Access Errors
**File**: `memory-mcp/src/mcp/tools/embeddings/tool/execute.rs`

**Changes Required**:
```rust
// Line 154: Change provider_config to provider
- embedding_config.provider_config.model_name()
+ embedding_config.provider.model_name()

// Line 283: Change provider_config to provider
- config.provider_config.effective_dimension()
+ config.provider.effective_dimension()

// Line 378: Change provider_config to provider
- config.provider_config.model_name().to_string()
+ config.provider.model_name().to_string()
```

#### 1.3 Verify Compilation
```bash
cargo build --all
cargo clippy --all -- -D warnings
```

**Success Criteria**:
- ✅ Zero compilation errors
- ✅ Zero clippy warnings
- ✅ All tests pass

**Deliverables**:
- Fixed `memory-mcp/src/mcp/tools/embeddings/tool/execute.rs`
- Atomic git commit: `fix(mcp): update embedding config API usage`

---

## Phase 2: Episode Tagging Phase 1 Implementation (P1 - Primary Feature)

**Duration**: 12 hours
**Agent**: feature-implementer
**Dependencies**: Phase 1 complete (compilation working)

### Overview
Implement core data model and storage layer for episode tagging feature.

### Task Breakdown

#### Task 2.1: Update Episode Structure (2 hours)
**File**: `memory-core/src/episode/structs.rs`

**Implementation**:
```rust
// Add tags field to Episode struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub episode_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: EpisodeStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,

    // NEW: Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

// Add helper methods
impl Episode {
    /// Add a tag to the episode (normalized)
    pub fn add_tag(&mut self, tag: String) -> bool {
        let normalized = Self::normalize_tag(&tag);
        if self.tags.contains(&normalized) {
            false
        } else {
            self.tags.push(normalized);
            true
        }
    }

    /// Remove a tag from the episode
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let normalized = Self::normalize_tag(tag);
        let pos = self.tags.iter().position(|t| t == &normalized);
        match pos {
            Some(idx) => {
                self.tags.remove(idx);
                true
            }
            None => false,
        }
    }

    /// Check if episode has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&Self::normalize_tag(tag))
    }

    /// Clear all tags from the episode
    pub fn clear_tags(&mut self) {
        self.tags.clear();
    }

    /// Normalize tag name (lowercase, trim)
    fn normalize_tag(tag: &str) -> String {
        tag.trim().to_lowercase()
    }

    /// Validate tag name (alphanumeric, hyphens, underscores)
    fn validate_tag(tag: &str) -> Result<()> {
        let normalized = Self::normalize_tag(tag);
        if normalized.is_empty() {
            return Err(anyhow!("Tag name cannot be empty"));
        }
        if normalized.len() > 100 {
            return Err(anyhow!("Tag name too long (max 100 characters)"));
        }
        if !normalized.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(anyhow!("Tag name contains invalid characters (only alphanumeric, hyphens, underscores allowed)"));
        }
        Ok(())
    }
}
```

**Tests** (`memory-core/src/episode/structs_tests.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_normalization() {
        assert_eq!(Episode::normalize_tag("  Test  "), "test");
        assert_eq!(Episode::normalize_tag("MixedCase"), "mixedcase");
    }

    #[test]
    fn test_tag_validation() {
        assert!(Episode::validate_tag("valid-tag").is_ok());
        assert!(Episode::validate_tag("valid_tag").is_ok());
        assert!(Episode::validate_tag("").is_err());
        assert!(Episode::validate_tag("a".repeat(101).as_str()).is_err());
        assert!(Episode::validate_tag("invalid!").is_err());
    }

    #[test]
    fn test_add_tag() {
        let mut episode = Episode::default();
        assert!(episode.add_tag("test".to_string()));
        assert!(!episode.add_tag("TEST".to_string())); // Duplicate
        assert_eq!(episode.tags, vec!["test"]);
    }

    #[test]
    fn test_remove_tag() {
        let mut episode = Episode::default();
        episode.tags = vec!["test".to_string(), "other".to_string()];
        assert!(episode.remove_tag("TEST".to_string()));
        assert_eq!(episode.tags, vec!["other"]);
    }
}
```

**Success Criteria**:
- ✅ Tags field added to Episode
- ✅ Helper methods implemented
- ✅ All unit tests pass
- ✅ File <500 LOC

**Deliverable**:
- Modified: `memory-core/src/episode/structs.rs`
- Modified: `memory-core/src/episode/mod.rs` (if needed)
- Git commit: `feat(episode): add tags field and helper methods`

---

#### Task 2.2: Database Schema Updates (2 hours)
**File**: `memory-storage-turso/src/schema.rs`

**Implementation**:
```rust
/// Initialize database schema with episode tagging support
pub async fn initialize_schema(conn: &Connection) -> Result<()> {
    // Existing tables...
    create_episodes_table(conn).await?;
    create_steps_table(conn).await?;
    create_reflection_table(conn).await?;
    create_pattern_table(conn).await?;

    // NEW: Episode tagging tables
    create_episode_tags_table(conn).await?;
    create_tag_metadata_table(conn).await?;

    Ok(())
}

/// Create episode_tags table (many-to-many relationship)
async fn create_episode_tags_table(conn: &Connection) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS episode_tags (
            episode_id TEXT NOT NULL,
            tag TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            PRIMARY KEY (episode_id, tag),
            FOREIGN KEY (episode_id) REFERENCES episodes(episode_id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(conn)
    .await?;

    // Create indexes for efficient queries
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_episode_tags_tag
        ON episode_tags(tag);
        "#,
    )
    .execute(conn)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_episode_tags_episode
        ON episode_tags(episode_id);
        "#,
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// Create tag_metadata table for statistics
async fn create_tag_metadata_table(conn: &Connection) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tag_metadata (
            tag TEXT PRIMARY KEY NOT NULL,
            usage_count INTEGER NOT NULL DEFAULT 0,
            first_used INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            last_used INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );
        "#,
    )
    .execute(conn)
    .await?;

    Ok(())
}
```

**Migration Test** (`memory-storage-turso/tests/schema_migration_test.rs`):
```rust
#[tokio::test]
async fn test_episode_tags_schema_migration() {
    let conn = create_test_connection().await;

    // Run schema initialization
    initialize_schema(&conn).await.unwrap();

    // Verify tables exist
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('episode_tags', 'tag_metadata')"
    )
    .fetch_all(&conn)
    .await
    .unwrap();

    assert!(tables.contains(&"episode_tags".to_string()));
    assert!(tables.contains(&"tag_metadata".to_string()));

    // Verify indexes exist
    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_episode_tags_%'"
    )
    .fetch_all(&conn)
    .await
    .unwrap();

    assert!(indexes.contains(&"idx_episode_tags_tag".to_string()));
    assert!(indexes.contains(&"idx_episode_tags_episode".to_string()));
}
```

**Success Criteria**:
- ✅ Tables created successfully
- ✅ Indexes created for performance
- ✅ Foreign key constraints work
- ✅ Migration tests pass

**Deliverables**:
- Modified: `memory-storage-turso/src/schema.rs`
- New: `memory-storage-turso/tests/schema_migration_test.rs`
- Git commit: `feat(schema): add episode_tags and tag_metadata tables`

---

#### Task 2.3: Turso Storage Backend Implementation (4 hours)
**New File**: `memory-storage-turso/src/storage/tag_operations.rs`

**Implementation**:
```rust
//! Episode tagging operations for Turso storage backend

use anyhow::{Context, Result};
use sqlx::{Connection, SqliteConnection};
use uuid::Uuid;

/// Save tags for an episode
///
/// This replaces all existing tags for the episode with the provided list.
/// It uses a transaction to ensure atomicity and updates tag metadata.
pub async fn save_episode_tags(
    conn: &mut SqliteConnection,
    episode_id: &Uuid,
    tags: &[String],
) -> Result<()> {
    let mut tx = conn.begin().await.context("Failed to begin transaction")?;

    let episode_id_str = episode_id.to_string();

    // Delete existing tags for this episode
    sqlx::query("DELETE FROM episode_tags WHERE episode_id = ?1")
        .bind(&episode_id_str)
        .execute(&mut *tx)
        .await
        .context("Failed to delete existing tags")?;

    // Insert new tags
    for tag in tags {
        sqlx::query(
            "INSERT INTO episode_tags (episode_id, tag) VALUES (?1, ?2)
             ON CONFLICT(episode_id, tag) DO NOTHING",
        )
        .bind(&episode_id_str)
        .bind(tag)
        .execute(&mut *tx)
        .await
        .context("Failed to insert tag")?;

        // Update tag metadata
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO tag_metadata (tag, usage_count, first_used, last_used)
             VALUES (?1, 1, ?2, ?2)
             ON CONFLICT(tag) DO UPDATE SET
                 usage_count = usage_count + 1,
                 last_used = ?2",
        )
        .bind(tag)
        .bind(now)
        .execute(&mut *tx)
        .await
        .context("Failed to update tag metadata")?;
    }

    tx.commit().await.context("Failed to commit transaction")?;
    Ok(())
}

/// Get tags for a specific episode
pub async fn get_episode_tags(
    conn: &mut SqliteConnection,
    episode_id: &Uuid,
) -> Result<Vec<String>> {
    let episode_id_str = episode_id.to_string();

    let tags: Vec<String> = sqlx::query_scalar(
        "SELECT tag FROM episode_tags WHERE episode_id = ?1 ORDER BY tag",
    )
    .bind(&episode_id_str)
    .fetch_all(conn)
    .await
    .context("Failed to fetch episode tags")?;

    Ok(tags)
}

/// Delete specific tags from an episode (or all if None)
pub async fn delete_episode_tags(
    conn: &mut SqliteConnection,
    episode_id: &Uuid,
    tags: Option<&[String]>,
) -> Result<()> {
    let episode_id_str = episode_id.to_string();

    match tags {
        Some(tag_list) => {
            // Delete specific tags and decrement their usage counts
            let mut tx = conn.begin().await.context("Failed to begin transaction")?;

            for tag in tag_list {
                sqlx::query("DELETE FROM episode_tags WHERE episode_id = ?1 AND tag = ?2")
                    .bind(&episode_id_str)
                    .bind(tag)
                    .execute(&mut *tx)
                    .await
                    .context("Failed to delete tag")?;

                // Update tag metadata (decrement usage count)
                sqlx::query(
                    "UPDATE tag_metadata
                     SET usage_count = usage_count - 1
                     WHERE tag = ?1 AND usage_count > 0",
                )
                .bind(tag)
                .execute(&mut *tx)
                .await
                .context("Failed to update tag metadata")?;
            }

            tx.commit().await.context("Failed to commit transaction")?;
        }
        None => {
            // Delete all tags for the episode
            sqlx::query("DELETE FROM episode_tags WHERE episode_id = ?1")
                .bind(&episode_id_str)
                .execute(conn)
                .await
                .context("Failed to delete all tags")?;
        }
    }

    Ok(())
}

/// Find episodes by tags with AND/OR logic
///
/// - `tags`: List of tags to search for
/// - `match_all`: If true, episodes must have ALL tags (AND), otherwise any tag (OR)
/// - `limit`: Maximum number of results
pub async fn find_episodes_by_tags(
    conn: &mut SqliteConnection,
    tags: &[String],
    match_all: bool,
    limit: Option<usize>,
) -> Result<Vec<Uuid>> {
    if tags.is_empty() {
        return Ok(vec![]);
    }

    let episode_ids: Vec<String> = if match_all {
        // AND logic: Episodes must have all specified tags
        let placeholders = tags.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let query = format!(
            "SELECT episode_id
             FROM episode_tags
             WHERE tag IN ({})
             GROUP BY episode_id
             HAVING COUNT(DISTINCT tag) = ?
             ORDER BY episode_id
             LIMIT ?",
            placeholders
        );

        let mut query_builder = sqlx::query_as(&query);
        for tag in tags {
            query_builder = query_builder.bind(tag);
        }
        query_builder = query_builder.bind(tags.len()).bind(limit.unwrap_or(1000) as i64);

        query_builder
            .fetch_all(conn)
            .await
            .context("Failed to find episodes by tags (AND)")?
    } else {
        // OR logic: Episodes must have at least one of the tags
        let placeholders = tags.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let query = format!(
            "SELECT DISTINCT episode_id
             FROM episode_tags
             WHERE tag IN ({})
             ORDER BY episode_id
             LIMIT ?",
            placeholders
        );

        let mut query_builder = sqlx::query_as(&query);
        for tag in tags {
            query_builder = query_builder.bind(tag);
        }
        query_builder = query_builder.bind(limit.unwrap_or(1000) as i64);

        query_builder
            .fetch_all(conn)
            .await
            .context("Failed to find episodes by tags (OR)")?
    };

    Ok(episode_ids
        .into_iter()
        .filter_map(|id| Uuid::parse_str(&id).ok())
        .collect())
}

/// Get all unique tags in the system
pub async fn get_all_tags(conn: &mut SqliteConnection) -> Result<Vec<String>> {
    let tags: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT tag FROM tag_metadata ORDER BY tag",
    )
    .fetch_all(conn)
    .await
    .context("Failed to fetch all tags")?;

    Ok(tags)
}

/// Get tag statistics
#[derive(Debug, Clone)]
pub struct TagStats {
    pub tag: String,
    pub usage_count: usize,
    pub first_used: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

pub async fn get_tag_statistics(
    conn: &mut SqliteConnection,
) -> Result<Vec<TagStats>> {
    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
        "SELECT tag, usage_count, first_used, last_used
         FROM tag_metadata
         ORDER BY usage_count DESC"
    )
    .fetch_all(conn)
    .await
    .context("Failed to fetch tag statistics")?;

    let stats = rows
        .into_iter()
        .map(|(tag, usage_count, first_used, last_used)| TagStats {
            tag,
            usage_count: usage_count as usize,
            first_used: chrono::DateTime::from_timestamp(first_used, 0).unwrap(),
            last_used: chrono::DateTime::from_timestamp(last_used, 0).unwrap(),
        })
        .collect();

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::initialize_schema;

    #[tokio::test]
    async fn test_save_and_get_episode_tags() {
        let mut conn = create_test_connection().await;
        initialize_schema(&conn).await.unwrap();

        let episode_id = Uuid::new_v4();
        let tags = vec!["test".to_string(), "example".to_string()];

        save_episode_tags(&mut conn, &episode_id, &tags)
            .await
            .unwrap();

        let fetched = get_episode_tags(&mut conn, &episode_id).await.unwrap();
        assert_eq!(fetched, tags);
    }

    #[tokio::test]
    async fn test_delete_episode_tags() {
        let mut conn = create_test_connection().await;
        initialize_schema(&conn).await.unwrap();

        let episode_id = Uuid::new_v4();
        let tags = vec!["test".to_string(), "example".to_string()];

        save_episode_tags(&mut conn, &episode_id, &tags).await.unwrap();

        delete_episode_tags(&mut conn, &episode_id, Some(&["test".to_string()]))
            .await
            .unwrap();

        let fetched = get_episode_tags(&mut conn, &episode_id).await.unwrap();
        assert_eq!(fetched, vec!["example"]);
    }

    #[tokio::test]
    async fn test_find_episodes_by_tags_and() {
        let mut conn = create_test_connection().await;
        initialize_schema(&conn).await.unwrap();

        let episode1 = Uuid::new_v4();
        let episode2 = Uuid::new_v4();
        let episode3 = Uuid::new_v4();

        save_episode_tags(&mut conn, &episode1, &["test".to_string(), "example".to_string()]).await.unwrap();
        save_episode_tags(&mut conn, &episode2, &["test".to_string()]).await.unwrap();
        save_episode_tags(&mut conn, &episode3, &["example".to_string()]).await.unwrap();

        // Find episodes with both tags (AND)
        let found = find_episodes_by_tags(
            &mut conn,
            &["test".to_string(), "example".to_string()],
            true,
            None,
        )
        .await
        .unwrap();

        assert_eq!(found.len(), 1);
        assert!(found.contains(&episode1));
    }

    #[tokio::test]
    async fn test_find_episodes_by_tags_or() {
        let mut conn = create_test_connection().await;
        initialize_schema(&conn).await.unwrap();

        let episode1 = Uuid::new_v4();
        let episode2 = Uuid::new_v4();

        save_episode_tags(&mut conn, &episode1, &["test".to_string()]).await.unwrap();
        save_episode_tags(&mut conn, &episode2, &["example".to_string()]).await.unwrap();

        // Find episodes with either tag (OR)
        let found = find_episodes_by_tags(
            &mut conn,
            &["test".to_string(), "example".to_string()],
            false,
            None,
        )
        .await
        .unwrap();

        assert_eq!(found.len(), 2);
    }

    #[tokio::test]
    async fn test_tag_statistics() {
        let mut conn = create_test_connection().await;
        initialize_schema(&conn).await.unwrap();

        let episode1 = Uuid::new_v4();
        let episode2 = Uuid::new_v4();

        save_episode_tags(&mut conn, &episode1, &["test".to_string()]).await.unwrap();
        save_episode_tags(&mut conn, &episode2, &["test".to_string(), "example".to_string()]).await.unwrap();

        let stats = get_tag_statistics(&mut conn).await.unwrap();
        assert_eq!(stats.len(), 2);

        let test_stats = stats.iter().find(|s| s.tag == "test").unwrap();
        assert_eq!(test_stats.usage_count, 2);
    }
}
```

**Success Criteria**:
- ✅ All tag operations implemented
- ✅ Transactions used for atomicity
- ✅ SQL injection protection (parameterized queries)
- ✅ All tests pass
- ✅ File <500 LOC (may need split)

**Deliverables**:
- New: `memory-storage-turso/src/storage/tag_operations.rs`
- Git commit: `feat(turso): implement episode tag operations`

---

#### Task 2.4: Redb Cache Integration (2 hours)
**File**: `memory-storage-redb/src/storage.rs`

**Implementation**:
```rust
// Tags are automatically handled via Episode serialization (postcard)
// Just need to ensure cache invalidation when tags change

impl StorageBackend for RedbStorage {
    // ... existing methods ...

    /// Invalidate episode cache when tags change
    async fn invalidate_episode_cache(&self, episode_id: &Uuid) -> Result<()> {
        let db = self.db.lock().await;

        let write_txn = db
            .begin_write()
            .map_err(|e| anyhow!("Failed to begin transaction: {}", e))?;

        {
            let mut episodes_table = write_txn
                .open_table(self.episodes_table_def())
                .map_err(|e| anyhow!("Failed to open episodes table: {}", e))?;

            episodes_table
                .remove(&episode_id.as_bytes())
                .map_err(|e| anyhow!("Failed to invalidate episode cache: {}", e))?;
        }

        write_txn
            .commit()
            .map_err(|e| anyhow!("Failed to commit cache invalidation: {}", e))?;

        Ok(())
    }
}
```

**Test** (`memory-storage-redb/tests/cache_invalidation_test.rs`):
```rust
#[tokio::test]
async fn test_tag_cache_invalidation() {
    let storage = create_test_storage().await;
    let episode = create_test_episode();

    // Save episode with initial tags
    let mut episode = episode;
    episode.tags = vec!["test".to_string()];
    storage.save_episode(&episode).await.unwrap();

    // Update tags
    episode.tags = vec!["updated".to_string()];
    storage.save_episode(&episode).await.unwrap();

    // Invalidate cache
    storage
        .invalidate_episode_cache(&episode.episode_id)
        .await
        .unwrap();

    // Verify updated tags
    let fetched = storage.get_episode(&episode.episode_id).await.unwrap();
    assert_eq!(fetched.tags, vec!["updated"]);
}
```

**Success Criteria**:
- ✅ Tags cached with episodes
- ✅ Cache invalidation on tag changes
- ✅ All tests pass

**Deliverables**:
- Modified: `memory-storage-redb/src/storage.rs`
- New: `memory-storage-redb/tests/cache_invalidation_test.rs`
- Git commit: `feat(redb): add tag cache invalidation`

---

#### Task 2.5: Storage Integration Tests (2 hours)
**New File**: `memory-storage-turso/tests/tag_integration_test.rs`

**Implementation**:
```rust
//! Integration tests for episode tagging across Turso and redb

use memory_storage_turso::TursoStorage;
use memory_core::Episode;
use uuid::Uuid;
use tokio;

#[tokio::test]
async fn test_end_to_end_tag_lifecycle() {
    let storage = create_test_storage().await;
    let episode_id = Uuid::new_v4();

    // Create episode with tags
    let mut episode = Episode::default();
    episode.episode_id = episode_id;
    episode.tags = vec!["initial".to_string()];

    storage.save_episode(&episode).await.unwrap();

    // Verify tags persist
    let fetched = storage.get_episode(&episode_id).await.unwrap();
    assert_eq!(fetched.tags, vec!["initial"]);

    // Update tags
    let mut updated = episode;
    updated.tags = vec!["updated".to_string(), "another".to_string()];
    storage.save_episode(&updated).await.unwrap();

    let fetched = storage.get_episode(&episode_id).await.unwrap();
    assert_eq!(fetched.tags, vec!["updated", "another"]);
}

#[tokio::test]
async fn test_tag_filtering_with_retrieval() {
    let storage = create_test_storage().await;

    let episode1 = create_test_episode_with_tags(vec!["test".to_string(), "bug".to_string()]);
    let episode2 = create_test_episode_with_tags(vec!["test".to_string(), "feature".to_string()]);
    let episode3 = create_test_episode_with_tags(vec!["bug".to_string()]);

    storage.save_episode(&episode1).await.unwrap();
    storage.save_episode(&episode2).await.unwrap();
    storage.save_episode(&episode3).await.unwrap();

    // Find episodes with "test" tag
    let found = storage
        .find_episodes_by_tags(&["test".to_string()], true, None)
        .await
        .unwrap();

    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|e| e.episode_id == episode1.episode_id));
    assert!(found.iter().any(|e| e.episode_id == episode2.episode_id));
}

#[tokio::test]
async fn test_concurrent_tag_operations() {
    let storage = Arc::new(create_test_storage().await);
    let episode_id = Uuid::new_v4();

    let mut episode = Episode::default();
    episode.episode_id = episode_id;
    storage.save_episode(&episode).await.unwrap();

    // Concurrent tag additions
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let storage = Arc::clone(&storage);
            let episode_id = episode_id;
            tokio::spawn(async move {
                let mut episode = storage.get_episode(&episode_id).await.unwrap();
                let tag = format!("tag{}", i);
                episode.add_tag(tag);
                storage.save_episode(&episode).await
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let fetched = storage.get_episode(&episode_id).await.unwrap();
    assert_eq!(fetched.tags.len(), 10);
}

#[tokio::test]
async fn test_tag_statistics_accuracy() {
    let storage = create_test_storage().await;

    // Create episodes with tags
    for _ in 0..5 {
        let mut episode = Episode::default();
        episode.tags = vec!["common".to_string()];
        storage.save_episode(&episode).await.unwrap();
    }

    for _ in 0..3 {
        let mut episode = Episode::default();
        episode.tags = vec!["less-common".to_string()];
        storage.save_episode(&episode).await.unwrap();
    }

    // Get statistics
    let stats = storage.get_tag_statistics().await.unwrap();

    let common_stats = stats.iter().find(|s| s.tag == "common").unwrap();
    assert_eq!(common_stats.usage_count, 5);

    let less_common_stats = stats.iter().find(|s| s.tag == "less-common").unwrap();
    assert_eq!(less_common_stats.usage_count, 3);
}
```

**Success Criteria**:
- ✅ All integration tests pass
- ✅ Coverage >90% for tagging code
- ✅ Edge cases covered (concurrent, empty, duplicates)

**Deliverables**:
- New: `memory-storage-turso/tests/tag_integration_test.rs`
- Git commit: `test(turso): add tag integration tests`

---

### Phase 2 Summary

**Total Duration**: 12 hours
**Files Modified**: 4
**Files Created**: 5
**Commits**: 6 (atomic per task)

**Quality Gates**:
- ✅ All tests pass (>95%)
- ✅ Coverage >90% for new code
- ✅ Zero clippy warnings
- ✅ All files <500 LOC
- ✅ SQL injection tests pass

---

## Phase 3: Split Oversized Files (P2 - Code Quality)

**Duration**: 4 hours
**Agent**: refactorer
**Priority**: Can run parallel to Phase 2 after Phase 1

### Files to Split

#### 3.1 Split `memory-storage-turso/src/cache/adaptive_ttl.rs` (645 lines)

**Current Structure**:
- Adaptive TTL cache implementation
- Contains logic for TTL calculation, caching, eviction

**Proposed Split**:
- `adaptive_ttl.rs` (core cache logic, ~300 lines)
- `ttl_calculator.rs` (TTL calculation logic, ~200 lines)
- `eviction.rs` (eviction strategies, ~150 lines)

**Success Criteria**:
- ✅ All files <500 LOC
- ✅ Functionality unchanged
- ✅ Tests still pass

**Git Commit**: `refactor(cache): split adaptive_ttl into smaller modules`

---

#### 3.2 Split `memory-storage-turso/src/storage/mod.rs` (562 lines)

**Current Structure**:
- Storage module declarations
- Trait implementations

**Proposed Split**:
- `mod.rs` (module exports, ~100 lines)
- `trait_impls/mod.rs` (trait implementations, ~462 lines)

**Success Criteria**:
- ✅ All files <500 LOC
- ✅ Module structure logical
- ✅ Tests pass

**Git Commit**: `refactor(storage): split mod.rs into smaller files`

---

#### 3.3 Split `memory-storage-turso/src/pool/adaptive.rs` (526 lines)

**Current Structure**:
- Adaptive pool sizing logic

**Proposed Split**:
- `mod.rs` (exports, ~50 lines)
- `sizing.rs` (size calculation, ~250 lines)
- `pool_impl.rs` (pool implementation, ~226 lines)

**Success Criteria**:
- ✅ All files <500 LOC
- ✅ Functionality preserved
- ✅ Tests pass

**Git Commit**: `refactor(pool): split adaptive module into components`

---

#### 3.4 Split `memory-storage-turso/src/storage/tag_operations.rs` (517 lines)

**Current Structure**:
- Tag CRUD operations
- Tag query operations
- Tag statistics
- Tests

**Proposed Split**:
- `mod.rs` (exports, ~50 lines)
- `crud.rs` (create/read/update/delete, ~200 lines)
- `queries.rs` (search and filtering, ~150 lines)
- `stats.rs` (statistics, ~100 lines)
- Tests move to separate file: `tests/tag_operations_test.rs`

**Success Criteria**:
- ✅ All files <500 LOC
- ✅ Clear separation of concerns
- ✅ Tests pass

**Git Commit**: `refactor(tags): split tag_operations into focused modules`

---

### Phase 3 Summary

**Total Duration**: 4 hours
**Files Split**: 4
**Total Files Created**: ~10 new modules

**Quality Gates**:
- ✅ All files <500 LOC
- ✅ No functionality changes
- ✅ All tests pass
- ✅ Zero clippy warnings

---

## Coordination Strategy

### Execution Timeline

```
Hour 0-0.5: Phase 1 - Fix Compilation Errors (BLOCKING)
Hour 0.5-4.5: Phase 2.1 - Episode Structure (feature-implementer)
Hour 0.5-4.5: Phase 3.1 - Split adaptive_ttl (refactorer) [PARALLEL]
Hour 4.5-6.5: Phase 2.2 - Schema Updates (feature-implementer)
Hour 4.5-6.5: Phase 3.2 - Split storage/mod.rs (refactorer) [PARALLEL]
Hour 6.5-8.5: Phase 2.3 - Turso Backend (feature-implementer)
Hour 8.5-10.5: Phase 2.4 - Redb Cache (feature-implementer)
Hour 6.5-8.5: Phase 3.3 - Split pool/adaptive.rs (refactorer) [PARALLEL]
Hour 10.5-12.5: Phase 2.5 - Integration Tests (feature-implementer)
Hour 8.5-10.5: Phase 3.4 - Split tag_operations.rs (refactorer) [PARALLEL]
Hour 12.5-13.0: Final validation and reporting
```

### Dependencies

- **Phase 1** must complete before any other work
- **Phase 2** tasks are sequential (must complete in order)
- **Phase 3** tasks can run parallel to Phase 2 after Phase 1

### Quality Gates

After each phase:
1. Run `cargo fmt`
2. Run `cargo clippy --all -- -D warnings`
3. Run `cargo test --all`
4. Verify file sizes (<500 LOC)
5. Create atomic git commit

## Success Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Compilation Errors | 0 | 3 | ❌ (Phase 1) |
| Episode Tagging Phase 1 | 100% | 0% | ⏳ (Phase 2) |
| Files >500 LOC | 0 | 4 | ⏳ (Phase 3) |
| Test Coverage | >90% | 92.5% | ✅ |
| Clippy Warnings | 0 | 0+ | ❌ (Phase 1) |

## Risk Mitigation

| Risk | Severity | Mitigation |
|------|----------|------------|
| Compilation fix introduces new errors | Low | Run tests after each change |
| Tag operations performance issues | Medium | Benchmark early, add indexes |
| File splitting breaks functionality | Low | Comprehensive test coverage |
| Phase 2/3 coordination conflicts | Low | Clear file ownership defined |

## Rollback Plan

- Each phase has atomic git commits
- Can rollback any phase independently
- Compilation errors must be fixed first (no rollback option)

---

**Plan Generated**: 2026-01-28
**Next Review**: After Phase 1 completion
**Estimated Completion**: 13 hours after Phase 1 start
