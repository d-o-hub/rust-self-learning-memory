//! Tag operations for episode management
//!
//! Provides functions for managing episode tags including CRUD operations,
//! tag-based queries, and tag statistics.

use crate::{Error, Result};
use libsql::Connection;
use std::collections::HashMap;
use uuid::Uuid;

/// Tag statistics
#[derive(Debug, Clone)]
pub struct TagStats {
    pub tag: String,
    pub usage_count: i64,
    pub first_used: i64,
    pub last_used: i64,
}

/// Save tags for an episode (replaces existing tags)
///
/// This function:
/// 1. Deletes all existing tags for the episode
/// 2. Inserts new tags
/// 3. Updates tag metadata (usage counts, timestamps)
///
/// All operations are performed in a transaction for atomicity.
pub async fn save_episode_tags(
    conn: &Connection,
    episode_id: &Uuid,
    tags: &[String],
) -> Result<()> {
    let episode_id_str = episode_id.to_string();

    // Start transaction
    conn.execute("BEGIN", ())
        .await
        .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;

    // Delete existing tags
    conn.execute(
        "DELETE FROM episode_tags WHERE episode_id = ?",
        libsql::params![episode_id_str.clone()],
    )
    .await
    .map_err(|e| Error::Storage(format!("Failed to delete existing tags: {}", e)))?;

    // Insert new tags and update metadata
    let now = chrono::Utc::now().timestamp();
    for tag in tags {
        // Insert tag association
        conn.execute(
            "INSERT INTO episode_tags (episode_id, tag, created_at) VALUES (?, ?, ?)",
            libsql::params![episode_id_str.clone(), tag.clone(), now],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to insert tag: {}", e)))?;

        // Update tag metadata (insert or update)
        conn.execute(
            r#"
            INSERT INTO tag_metadata (tag, usage_count, first_used, last_used)
            VALUES (?, 1, ?, ?)
            ON CONFLICT(tag) DO UPDATE SET
                usage_count = usage_count + 1,
                last_used = ?
            "#,
            libsql::params![tag.clone(), now, now, now],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to update tag metadata: {}", e)))?;
    }

    // Commit transaction
    conn.execute("COMMIT", ())
        .await
        .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

    Ok(())
}

/// Get all tags for an episode
pub async fn get_episode_tags(conn: &Connection, episode_id: &Uuid) -> Result<Vec<String>> {
    let episode_id_str = episode_id.to_string();

    let stmt = conn
        .prepare("SELECT tag FROM episode_tags WHERE episode_id = ? ORDER BY tag")
        .await
        .map_err(|e| Error::Storage(format!("Failed to prepare query: {}", e)))?;

    let mut rows = stmt
        .query(libsql::params![episode_id_str])
        .await
        .map_err(|e| Error::Storage(format!("Failed to query tags: {}", e)))?;

    let mut tags = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| Error::Storage(format!("Failed to fetch row: {}", e)))?
    {
        let tag: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get tag: {}", e)))?;
        tags.push(tag);
    }

    Ok(tags)
}

/// Delete specific tags from an episode
pub async fn delete_episode_tags(
    conn: &Connection,
    episode_id: &Uuid,
    tags: &[String],
) -> Result<()> {
    if tags.is_empty() {
        return Ok(());
    }

    let episode_id_str = episode_id.to_string();

    // Build placeholders for IN clause
    let placeholders = tags.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
        "DELETE FROM episode_tags WHERE episode_id = ? AND tag IN ({})",
        placeholders
    );

    // Build params: episode_id + tags
    let mut params: Vec<libsql::Value> = vec![episode_id_str.into()];
    params.extend(tags.iter().map(|t| t.clone().into()));

    conn.execute(&query, libsql::params_from_iter(params))
        .await
        .map_err(|e| Error::Storage(format!("Failed to delete tags: {}", e)))?;

    // Note: We don't decrement usage_count in tag_metadata to keep historical stats

    Ok(())
}

/// Find episodes that have any of the specified tags (OR logic)
pub async fn find_episodes_by_tags_or(
    conn: &Connection,
    tags: &[String],
    limit: Option<usize>,
) -> Result<Vec<Uuid>> {
    if tags.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = tags.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();

    let query = format!(
        "SELECT DISTINCT episode_id FROM episode_tags WHERE tag IN ({}) ORDER BY created_at DESC{}",
        placeholders, limit_clause
    );

    let params: Vec<libsql::Value> = tags.iter().map(|t| t.clone().into()).collect();

    let stmt = conn
        .prepare(&query)
        .await
        .map_err(|e| Error::Storage(format!("Failed to prepare query: {}", e)))?;

    let mut rows = stmt
        .query(libsql::params_from_iter(params))
        .await
        .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

    let mut episode_ids = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| Error::Storage(format!("Failed to fetch row: {}", e)))?
    {
        let id_str: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get episode_id: {}", e)))?;
        let id =
            Uuid::parse_str(&id_str).map_err(|e| Error::Storage(format!("Invalid UUID: {}", e)))?;
        episode_ids.push(id);
    }

    Ok(episode_ids)
}

/// Find episodes that have all of the specified tags (AND logic)
pub async fn find_episodes_by_tags_and(
    conn: &Connection,
    tags: &[String],
    limit: Option<usize>,
) -> Result<Vec<Uuid>> {
    if tags.is_empty() {
        return Ok(Vec::new());
    }

    let tag_count = tags.len();
    let placeholders = tags.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();

    // Query: Find episodes that have all specified tags
    let query = format!(
        r#"
        SELECT episode_id
        FROM episode_tags
        WHERE tag IN ({})
        GROUP BY episode_id
        HAVING COUNT(DISTINCT tag) = ?
        ORDER BY MAX(created_at) DESC
        {}
        "#,
        placeholders, limit_clause
    );

    let mut params: Vec<libsql::Value> = tags.iter().map(|t| t.clone().into()).collect();
    params.push((tag_count as i64).into());

    let stmt = conn
        .prepare(&query)
        .await
        .map_err(|e| Error::Storage(format!("Failed to prepare query: {}", e)))?;

    let mut rows = stmt
        .query(libsql::params_from_iter(params))
        .await
        .map_err(|e| Error::Storage(format!("Failed to query episodes: {}", e)))?;

    let mut episode_ids = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| Error::Storage(format!("Failed to fetch row: {}", e)))?
    {
        let id_str: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get episode_id: {}", e)))?;
        let id =
            Uuid::parse_str(&id_str).map_err(|e| Error::Storage(format!("Invalid UUID: {}", e)))?;
        episode_ids.push(id);
    }

    Ok(episode_ids)
}

/// Get all unique tags in the system
pub async fn get_all_tags(conn: &Connection) -> Result<Vec<String>> {
    let stmt = conn
        .prepare("SELECT tag FROM tag_metadata ORDER BY tag")
        .await
        .map_err(|e| Error::Storage(format!("Failed to prepare query: {}", e)))?;

    let mut rows = stmt
        .query(())
        .await
        .map_err(|e| Error::Storage(format!("Failed to query tags: {}", e)))?;

    let mut tags = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| Error::Storage(format!("Failed to fetch row: {}", e)))?
    {
        let tag: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get tag: {}", e)))?;
        tags.push(tag);
    }

    Ok(tags)
}

/// Get statistics for all tags
pub async fn get_tag_statistics(conn: &Connection) -> Result<HashMap<String, TagStats>> {
    let stmt = conn
        .prepare("SELECT tag, usage_count, first_used, last_used FROM tag_metadata ORDER BY usage_count DESC")
        .await
        .map_err(|e| Error::Storage(format!("Failed to prepare query: {}", e)))?;

    let mut rows = stmt
        .query(())
        .await
        .map_err(|e| Error::Storage(format!("Failed to query tag statistics: {}", e)))?;

    let mut stats = HashMap::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| Error::Storage(format!("Failed to fetch row: {}", e)))?
    {
        let tag: String = row
            .get(0)
            .map_err(|e| Error::Storage(format!("Failed to get tag: {}", e)))?;
        let usage_count: i64 = row
            .get(1)
            .map_err(|e| Error::Storage(format!("Failed to get usage_count: {}", e)))?;
        let first_used: i64 = row
            .get(2)
            .map_err(|e| Error::Storage(format!("Failed to get first_used: {}", e)))?;
        let last_used: i64 = row
            .get(3)
            .map_err(|e| Error::Storage(format!("Failed to get last_used: {}", e)))?;

        stats.insert(
            tag.clone(),
            TagStats {
                tag,
                usage_count,
                first_used,
                last_used,
            },
        );
    }

    Ok(stats)
}

#[cfg(test)]
#[path = "tag_operations_tests.rs"]
mod tests;
