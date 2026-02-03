//! Episode update command implementation

use crate::config::Config;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct UpdateResult {
    pub success: bool,
    pub episode_id: String,
    pub message: String,
    pub updated_fields: Vec<String>,
}

impl crate::output::Output for UpdateResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        if self.success {
            writeln!(writer, "✓ {}", self.message)?;
            if !self.updated_fields.is_empty() {
                writeln!(writer, "Updated fields: {}", self.updated_fields.join(", "))?;
            }
        } else {
            writeln!(writer, "✗ {}", self.message)?;
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn update_episode(
    episode_id: String,
    description: Option<String>,
    add_tags: Option<Vec<String>>,
    remove_tags: Option<Vec<String>>,
    set_tags: Option<Vec<String>>,
    metadata: Option<Vec<String>>,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    let uuid = Uuid::parse_str(&episode_id)
        .map_err(|_| anyhow::anyhow!("Invalid episode ID format: {}", episode_id))?;

    if dry_run {
        println!("[DRY RUN] Would update episode: {}", episode_id);
        if let Some(desc) = &description {
            println!("  Would set description to: {}", desc);
        }
        if let Some(tags) = &add_tags {
            println!("  Would add tags: {}", tags.join(", "));
        }
        if let Some(tags) = &remove_tags {
            println!("  Would remove tags: {}", tags.join(", "));
        }
        if let Some(tags) = &set_tags {
            println!("  Would set tags to: {}", tags.join(", "));
        }
        if let Some(meta) = &metadata {
            println!("  Would add metadata: {}", meta.join(", "));
        }
        return Ok(());
    }

    let mut updated_fields = Vec::new();

    // Update description
    if let Some(desc) = description {
        memory
            .update_episode(uuid, Some(desc), None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update description: {}", e))?;
        updated_fields.push("description".to_string());
    }

    // Parse metadata from key=value pairs
    let metadata_map = if let Some(meta_pairs) = metadata {
        let mut map = HashMap::new();
        for pair in meta_pairs {
            if let Some((key, value)) = pair.split_once('=') {
                map.insert(key.to_string(), value.to_string());
            } else {
                anyhow::bail!("Invalid metadata format: '{}'. Expected 'key=value'", pair);
            }
        }
        Some(map)
    } else {
        None
    };

    // Update metadata if provided (without description to avoid double update)
    if metadata_map.is_some() {
        memory
            .update_episode(uuid, None, metadata_map)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update metadata: {}", e))?;
        updated_fields.push("metadata".to_string());
    }

    // Add tags
    if let Some(tags) = add_tags {
        memory
            .add_episode_tags(uuid, tags)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add tags: {}", e))?;
        updated_fields.push("tags (added)".to_string());
    }

    // Remove tags
    if let Some(tags) = remove_tags {
        memory
            .remove_episode_tags(uuid, tags)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to remove tags: {}", e))?;
        updated_fields.push("tags (removed)".to_string());
    }

    // Set tags (replace all)
    if let Some(tags) = set_tags {
        memory
            .set_episode_tags(uuid, tags)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set tags: {}", e))?;
        updated_fields.push("tags (set)".to_string());
    }

    let result = UpdateResult {
        success: true,
        episode_id: episode_id.clone(),
        message: format!("Successfully updated episode: {}", episode_id),
        updated_fields,
    };

    format.print_output(&result)
}
