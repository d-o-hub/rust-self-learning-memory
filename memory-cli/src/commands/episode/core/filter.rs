//! Filter management command implementation

use super::types::{FilterCommands, SavedFilter};
use crate::config::Config;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;
use std::fs;
use std::path::{Path, PathBuf};

fn get_filter_path() -> PathBuf {
    let base_dir = dirs::data_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        })
        .join("memory-cli");
    base_dir.join("filters")
}

fn load_filters(filter_dir: &PathBuf) -> Vec<SavedFilter> {
    if !filter_dir.exists() {
        return Vec::new();
    }

    let entries = match fs::read_dir(filter_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut filters = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        if entry.path().extension().is_some_and(|ext| ext == "json") {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(filter) = serde_json::from_str::<SavedFilter>(&content) {
                    filters.push(filter);
                }
            }
        }
    }
    filters
}

fn save_filter(filter_dir: &Path, filter: &SavedFilter) -> anyhow::Result<()> {
    if !filter_dir.exists() {
        fs::create_dir_all(filter_dir)?;
    }

    let path = filter_dir.join(format!("{}.json", filter.name));
    let content = serde_json::to_string_pretty(filter)?;
    fs::write(path, content)?;
    Ok(())
}

fn delete_filter_file(filter_dir: &Path, name: &str) -> anyhow::Result<()> {
    let path = filter_dir.join(format!("{}.json", name));
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub async fn handle_filter_command(
    command: FilterCommands,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    let filter_dir = get_filter_path();

    match command {
        FilterCommands::Save {
            name,
            task_type,
            status,
            since,
            until,
            domain,
            tags,
            outcome,
            limit,
        } => {
            let filter = SavedFilter {
                name,
                task_type,
                status: status.map(|s| format!("{:?}", s)),
                since,
                until,
                domain,
                tags,
                outcome: outcome.map(|o| format!("{:?}", o)),
                limit,
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            save_filter(&filter_dir, &filter)?;
            println!("Filter '{}' saved successfully", filter.name);
            Ok(())
        }
        FilterCommands::List => {
            let filters = load_filters(&filter_dir);
            let total_count = filters.len();
            println!("Saved Filters:");
            println!("{}", "─".repeat(60));
            for filter in &filters {
                println!("  {}", filter.name);
                if let Some(task_type) = &filter.task_type {
                    println!("    Task Type: {}", task_type);
                }
                if let Some(status) = &filter.status {
                    println!("    Status: {}", status);
                }
                if let Some(limit) = filter.limit {
                    println!("    Default Limit: {}", limit);
                }
            }
            println!("{}", "─".repeat(60));
            println!("Total: {} filters", total_count);
            Ok(())
        }
        FilterCommands::Delete { filter_name } => {
            delete_filter_file(&filter_dir, &filter_name)?;
            println!("Filter '{}' deleted", filter_name);
            Ok(())
        }
        FilterCommands::Show { filter_name } => {
            let filters = load_filters(&filter_dir);
            let filter = filters
                .iter()
                .find(|f| f.name == filter_name)
                .ok_or_else(|| anyhow::anyhow!("Filter '{}' not found", filter_name))?;

            println!("Filter: {}", filter.name);
            println!("{}", "─".repeat(60));
            if let Some(task_type) = &filter.task_type {
                println!("  Task Type: {}", task_type);
            }
            if let Some(status) = &filter.status {
                println!("  Status: {}", status);
            }
            if let Some(since) = &filter.since {
                println!("  Since: {}", since);
            }
            if let Some(until) = &filter.until {
                println!("  Until: {}", until);
            }
            if let Some(domain) = &filter.domain {
                println!("  Domain: {}", domain);
            }
            if let Some(tags) = &filter.tags {
                println!("  Tags: {}", tags);
            }
            if let Some(outcome) = &filter.outcome {
                println!("  Outcome: {}", outcome);
            }
            if let Some(limit) = filter.limit {
                println!("  Default Limit: {}", limit);
            }
            println!("  Created: {}", filter.created_at);
            Ok(())
        }
        FilterCommands::Apply {
            filter_name,
            limit,
            offset: _,
        } => {
            let filters = load_filters(&filter_dir);
            let filter = filters
                .iter()
                .find(|f| f.name == filter_name)
                .ok_or_else(|| anyhow::anyhow!("Filter '{}' not found", filter_name))?;

            println!("Applying filter '{}':", filter_name);
            if let Some(task_type) = &filter.task_type {
                println!("  --task-type {}", task_type);
            }
            if let Some(status) = &filter.status {
                println!("  --status {}", status.to_lowercase());
            }
            if let Some(since) = &filter.since {
                println!("  --since {}", since);
            }
            if let Some(until) = &filter.until {
                println!("  --until {}", until);
            }
            if let Some(domain) = &filter.domain {
                println!("  --domain {}", domain);
            }
            if let Some(tags) = &filter.tags {
                println!("  --tags {}", tags);
            }
            if let Some(outcome) = &filter.outcome {
                println!("  --outcome {}", outcome.to_lowercase());
            }
            if let Some(l) = limit.or(filter.limit) {
                println!("  --limit {}", l);
            }
            println!("\nTo use this filter, run:");
            println!(
                "  memory-cli episode list {}",
                build_filter_args(filter, limit)
            );
            Ok(())
        }
    }
}

fn build_filter_args(filter: &SavedFilter, override_limit: Option<usize>) -> String {
    let mut args = Vec::new();

    if let Some(task_type) = &filter.task_type {
        args.push(format!("--task-type {}", task_type));
    }
    if let Some(status) = &filter.status {
        args.push(format!("--status {}", status.to_lowercase()));
    }
    if let Some(since) = &filter.since {
        args.push(format!("--since {}", since));
    }
    if let Some(until) = &filter.until {
        args.push(format!("--until {}", until));
    }
    if let Some(domain) = &filter.domain {
        args.push(format!("--domain {}", domain));
    }
    if let Some(tags) = &filter.tags {
        args.push(format!("--tags {}", tags));
    }
    if let Some(outcome) = &filter.outcome {
        args.push(format!("--outcome {}", outcome.to_lowercase()));
    }
    if let Some(limit) = override_limit.or(filter.limit) {
        args.push(format!("--limit {}", limit));
    }

    args.join(" ")
}
