//! Tag command output implementations

use super::types::{
    TagAddResult, TagListResult, TagRemoveResult, TagSearchResult, TagSetResult, TagShowResult,
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
