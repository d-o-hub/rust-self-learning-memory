//! Tag command output implementations

use super::types::{
    TagAddResult, TagListResult, TagRemoveResult, TagRenameResult, TagSearchResult, TagSetResult,
    TagShowResult, TagStatsDetailedResult, TagStatsResult,
};
use crate::output::Output;

impl Output for TagAddResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        if self.success {
            writeln!(
                writer,
                "{}",
                format!(
                    "Added {} tag(s) to episode {}",
                    self.tags_added, self.episode_id
                )
                .green()
            )?;
        } else {
            writeln!(writer, "{}", "No tags added".yellow())?;
        }

        if !self.current_tags.is_empty() {
            writeln!(writer, "\nCurrent tags:")?;
            for tag in &self.current_tags {
                writeln!(writer, "  • {}", tag.cyan())?;
            }
        } else {
            writeln!(writer, "\nNo tags on this episode")?;
        }

        Ok(())
    }
}

impl Output for TagRemoveResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        if self.success {
            writeln!(
                writer,
                "{}",
                format!(
                    "Removed {} tag(s) from episode {}",
                    self.tags_removed, self.episode_id
                )
                .green()
            )?;
        } else {
            writeln!(writer, "{}", "No tags removed".yellow())?;
        }

        if !self.current_tags.is_empty() {
            writeln!(writer, "\nCurrent tags:")?;
            for tag in &self.current_tags {
                writeln!(writer, "  • {}", tag.cyan())?;
            }
        } else {
            writeln!(writer, "\nNo tags remaining on this episode")?;
        }

        Ok(())
    }
}

impl Output for TagSetResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        if self.success {
            writeln!(
                writer,
                "{}",
                format!(
                    "Set {} tag(s) on episode {}",
                    self.tags_set, self.episode_id
                )
                .green()
            )?;
        } else {
            writeln!(writer, "{}", "Failed to set tags".red())?;
        }

        if !self.current_tags.is_empty() {
            writeln!(writer, "\nCurrent tags:")?;
            for tag in &self.current_tags {
                writeln!(writer, "  • {}", tag.cyan())?;
            }
        } else {
            writeln!(writer, "\nNo tags on this episode")?;
        }

        Ok(())
    }
}

impl Output for TagListResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(
            writer,
            "{}",
            format!("Tags for episode {}", self.episode_id).bold()
        )?;
        writeln!(writer, "{}", "─".repeat(60))?;

        writeln!(writer, "Total: {} tag(s)", self.count)?;

        if !self.tags.is_empty() {
            writeln!(writer)?;
            for tag in &self.tags {
                writeln!(writer, "  • {}", tag.cyan())?;
            }
        }

        Ok(())
    }
}

impl Output for TagSearchResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(
            writer,
            "{}",
            format!("Found {} episode(s)", self.count).bold()
        )?;
        writeln!(writer, "Search: {}", self.search_criteria.dimmed())?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for episode in &self.episodes {
            let id_display = &episode.episode_id[..episode.episode_id.len().min(8)];
            let task_display = episode
                .task_description
                .chars()
                .take(50)
                .collect::<String>();

            writeln!(
                writer,
                "{} {} {}",
                id_display.dimmed(),
                task_display,
                format!("[{} tags]", episode.tags.len()).yellow()
            )?;

            if !episode.tags.is_empty() {
                let tags_str = episode
                    .tags
                    .iter()
                    .map(|t| t.cyan().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(writer, "     Tags: {}", tags_str)?;
            }
        }

        Ok(())
    }
}

impl Output for TagShowResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Episode Details".bold())?;
        writeln!(writer, "{}", "─".repeat(60))?;

        writeln!(writer, "ID: {}", self.episode_id)?;
        writeln!(writer, "Task: {}", self.task_description)?;

        let status_color = match self.status.as_str() {
            "completed" => Color::Green,
            "in_progress" => Color::Yellow,
            _ => Color::Red,
        };
        writeln!(writer, "Status: {}", self.status.color(status_color))?;

        writeln!(writer, "Created: {}", self.created_at)?;

        if let Some(ref completed_at) = self.completed_at {
            writeln!(writer, "Completed: {}", completed_at)?;
        }

        if let Some(duration) = self.duration_ms {
            writeln!(writer, "Duration: {}ms", duration)?;
        }

        if let Some(ref outcome) = self.outcome {
            let outcome_color = match outcome.as_str() {
                "Success" => Color::Green,
                "PartialSuccess" => Color::Yellow,
                "Failure" => Color::Red,
                _ => Color::White,
            };
            writeln!(writer, "Outcome: {}", outcome.color(outcome_color))?;
        }

        writeln!(writer)?;
        writeln!(
            writer,
            "{} ({})",
            "Tags:".bold(),
            format!("{}", self.tags_count).cyan()
        )?;

        if !self.tags.is_empty() {
            for tag in &self.tags {
                writeln!(writer, "  • {}", tag.cyan())?;
            }
        } else {
            writeln!(writer, "  (none)")?;
        }

        Ok(())
    }
}

impl Output for TagStatsResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "All Tags".bold())?;
        writeln!(writer, "{}", "─".repeat(80))?;

        writeln!(
            writer,
            "Total: {} unique tag(s), {} total usage(s)",
            self.total_tags, self.total_usage
        )?;
        writeln!(writer, "Sorted by: {}", self.sort_by.cyan())?;
        writeln!(writer)?;

        if self.tags.is_empty() {
            writeln!(writer, "{}", "No tags found in the system.".yellow())?;
            return Ok(());
        }

        // Table header
        writeln!(
            writer,
            "{:<20} {:>8} {:>20} {:>20}",
            "Tag".bold(),
            "Count".bold(),
            "First Used".bold(),
            "Last Used".bold()
        )?;
        writeln!(writer, "{}", "─".repeat(80))?;

        // Table rows
        for entry in &self.tags {
            writeln!(
                writer,
                "{:<20} {:>8} {:>20} {:>20}",
                entry.tag.cyan(),
                entry.usage_count,
                entry.first_used.dimmed(),
                entry.last_used.dimmed()
            )?;
        }

        Ok(())
    }
}

impl Output for TagRenameResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        if self.success {
            writeln!(
                writer,
                "{}",
                format!(
                    "Renamed tag '{}' to '{}' across {} episode(s)",
                    self.old_tag, self.new_tag, self.episodes_affected
                )
                .green()
            )?;
        } else if self.episodes_affected == 0 {
            writeln!(
                writer,
                "{}",
                format!("Tag '{}' not found in any episodes", self.old_tag).yellow()
            )?;
        } else {
            writeln!(writer, "{}", "Failed to rename tag".red())?;
        }

        Ok(())
    }
}

impl Output for TagStatsDetailedResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Tag Statistics".bold())?;
        writeln!(writer, "{}", "═".repeat(80))?;

        // Summary section
        writeln!(writer)?;
        writeln!(writer, "{}", "Summary".bold())?;
        writeln!(
            writer,
            "  Total Tags: {}",
            self.total_tags.to_string().cyan()
        )?;
        writeln!(
            writer,
            "  Total Usage: {}",
            self.total_usage.to_string().cyan()
        )?;
        writeln!(
            writer,
            "  Total Episodes: {}",
            self.total_episodes.to_string().cyan()
        )?;
        writeln!(
            writer,
            "  Avg Tags/Episode: {:.2}",
            self.avg_tags_per_episode
        )?;

        if let Some(ref most) = self.most_used_tag {
            writeln!(writer, "  Most Used: {}", most.cyan())?;
        }
        if let Some(ref least) = self.least_used_tag {
            writeln!(writer, "  Least Used: {}", least.cyan())?;
        }

        writeln!(writer)?;
        writeln!(writer, "Sorted by: {}", self.sort_by.cyan())?;
        writeln!(writer)?;

        if self.tags.is_empty() {
            writeln!(writer, "{}", "No tags found in the system.".yellow())?;
            return Ok(());
        }

        // Table header
        writeln!(
            writer,
            "{:<20} {:>8} {:>10} {:>20} {:>20}",
            "Tag".bold(),
            "Count".bold(),
            "%".bold(),
            "First Used".bold(),
            "Last Used".bold()
        )?;
        writeln!(writer, "{}", "─".repeat(80))?;

        // Table rows
        for entry in &self.tags {
            writeln!(
                writer,
                "{:<20} {:>8} {:>9.1}% {:>20} {:>20}",
                entry.tag.cyan(),
                entry.usage_count,
                entry.percentage,
                entry.first_used.dimmed(),
                entry.last_used.dimmed()
            )?;
        }

        Ok(())
    }
}
