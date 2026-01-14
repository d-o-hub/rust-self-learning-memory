//! Episode output implementations

use super::types::{
    EpisodeDetail, EpisodeList, EpisodeListFiltered, EpisodeSearchResult, EpisodeSummary,
    FilterList, SavedFilter,
};
use crate::output::Output;

impl Output for EpisodeSummary {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(writer, "Episode: {}", self.episode_id)?;
        writeln!(writer, "Task: {}", self.task_description)?;
        writeln!(writer, "Status: {}", self.status)?;
        writeln!(writer, "Created: {}", self.created_at)?;
        if let Some(duration) = self.duration_ms {
            writeln!(writer, "Duration: {}ms", duration)?;
        }
        writeln!(writer, "Steps: {}", self.steps_count)?;
        Ok(())
    }
}

impl Output for EpisodeList {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(
            writer,
            "{} episodes (showing {})",
            self.total_count,
            self.episodes.len()
        )?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for episode in &self.episodes {
            let (status_color, status_icon) = match episode.status.as_str() {
                "completed" => (Color::Green, "✓"),
                "in_progress" => (Color::Yellow, "⟳"),
                _ => (Color::Red, "✗"),
            };

            let id_display = format!(
                "{:<8}",
                &episode.episode_id[..episode.episode_id.len().min(8)]
            );
            let task_display = episode
                .task_description
                .chars()
                .take(50)
                .collect::<String>();
            let status_display = format!("{} {}", status_icon, episode.status);

            writeln!(
                writer,
                "{} {} {}",
                id_display.dimmed(),
                task_display,
                status_display.color(status_color).bold()
            )?;
        }

        Ok(())
    }
}

impl Output for EpisodeListFiltered {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(
            writer,
            "{} episodes (showing {}/{})",
            self.total_count, self.filtered_count, self.total_count
        )?;
        writeln!(writer, "{}", "─".repeat(80))?;

        writeln!(writer, "Applied Filters:")?;
        if let Some(ref task_type) = self.applied_filters.task_type {
            writeln!(writer, "  • Task Type: {}", task_type)?;
        }
        if let Some(ref status) = self.applied_filters.status {
            writeln!(writer, "  • Status: {}", status)?;
        }
        if let Some(ref since) = self.applied_filters.since {
            writeln!(writer, "  • Since: {}", since)?;
        }
        if let Some(ref until) = self.applied_filters.until {
            writeln!(writer, "  • Until: {}", until)?;
        }
        if let Some(ref domain) = self.applied_filters.domain {
            writeln!(writer, "  • Domain: {}", domain)?;
        }
        if let Some(ref tags) = self.applied_filters.tags {
            writeln!(writer, "  • Tags: {}", tags)?;
        }
        if let Some(ref outcome) = self.applied_filters.outcome {
            writeln!(writer, "  • Outcome: {}", outcome)?;
        }
        writeln!(writer, "  • Sort: {}", self.applied_filters.sort)?;
        writeln!(writer, "  • Offset: {}", self.applied_filters.offset)?;
        writeln!(writer, "  • Limit: {}", self.applied_filters.limit)?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for episode in &self.episodes {
            let (status_color, status_icon) = match episode.status.as_str() {
                "completed" => (Color::Green, "✓"),
                "in_progress" => (Color::Yellow, "⟳"),
                _ => (Color::Red, "✗"),
            };

            let id_display = format!(
                "{:<8}",
                &episode.episode_id[..episode.episode_id.len().min(8)]
            );
            let task_display = episode
                .task_description
                .chars()
                .take(50)
                .collect::<String>();
            let status_display = format!("{} {}", status_icon, episode.status);

            writeln!(
                writer,
                "{} {} {}",
                id_display.dimmed(),
                task_display,
                status_display.color(status_color).bold()
            )?;
        }

        Ok(())
    }
}

impl Output for EpisodeDetail {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "Episode: {}", self.episode_id)?;
        writeln!(writer, "Task: {}", self.task_description)?;
        writeln!(writer, "Status: {}", self.status)?;

        let status_color = match self.status.as_str() {
            "completed" => Color::Green,
            "in_progress" => Color::Yellow,
            _ => Color::Red,
        };
        writeln!(writer, "Created: {}", self.created_at)?;

        if let Some(completed_at) = &self.completed_at {
            writeln!(writer, "Completed: {}", completed_at)?;
        }

        if let Some(duration) = self.duration_ms {
            writeln!(writer, "Duration: {}ms", duration)?;
        }

        if let Some(outcome) = &self.outcome {
            writeln!(writer, "Outcome: {}", outcome.color(status_color))?;
        }

        writeln!(writer, "\nSteps:")?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for step in &self.steps {
            let status_icon = if step.success { "✓" } else { "✗" };
            let status_color = if step.success {
                Color::Green
            } else {
                Color::Red
            };

            writeln!(
                writer,
                "[{:>3}] {} {} - {}",
                step.step_number,
                status_icon.color(status_color),
                step.tool,
                step.action
            )?;

            if let Some(obs) = &step.observation {
                writeln!(writer, "     Observation: {}", obs)?;
            }

            let mut meta = Vec::new();
            if let Some(latency) = step.latency_ms {
                meta.push(format!("{}ms", latency));
            }
            if let Some(tokens) = step.tokens {
                meta.push(format!("{} tokens", tokens));
            }
            if !meta.is_empty() {
                writeln!(writer, "     [{}]", meta.join(", "))?;
            }
        }

        Ok(())
    }
}

impl Output for EpisodeSearchResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        let relevance_pct = (self.relevance_score * 100.0) as u8;
        let relevance_color = match relevance_pct {
            80..=100 => Color::Green,
            60..=79 => Color::Yellow,
            40..=59 => Color::Magenta,
            _ => Color::Red,
        };

        writeln!(
            writer,
            "{} [{}%] {}",
            &self.episode_id[..8],
            format!("{}", relevance_pct).color(relevance_color).bold(),
            self.task_description
        )?;
        writeln!(
            writer,
            "  Status: {} | Created: {}",
            self.status, self.created_at
        )?;

        if !self.matched_terms.is_empty() {
            writeln!(writer, "  Matched: {}", self.matched_terms.join(", "))?;
        }

        Ok(())
    }
}

impl Output for SavedFilter {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "Filter: {}", self.name.bold())?;
        writeln!(writer, "{}", "─".repeat(60))?;

        if let Some(ref task_type) = self.task_type {
            writeln!(writer, "  Task Type: {}", task_type)?;
        }
        if let Some(ref status) = self.status {
            writeln!(writer, "  Status: {}", status)?;
        }
        if let Some(ref since) = self.since {
            writeln!(writer, "  Since: {}", since)?;
        }
        if let Some(ref until) = self.until {
            writeln!(writer, "  Until: {}", until)?;
        }
        if let Some(ref domain) = self.domain {
            writeln!(writer, "  Domain: {}", domain)?;
        }
        if let Some(ref tags) = self.tags {
            writeln!(writer, "  Tags: {}", tags)?;
        }
        if let Some(ref outcome) = self.outcome {
            writeln!(writer, "  Outcome: {}", outcome)?;
        }
        if let Some(limit) = self.limit {
            writeln!(writer, "  Default Limit: {}", limit)?;
        }
        writeln!(writer, "  Created: {}", self.created_at)?;

        Ok(())
    }
}

impl Output for FilterList {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "Saved Filters ({})", self.total_count)?;
        writeln!(writer, "{}", "─".repeat(60))?;

        for filter in &self.filters {
            writeln!(writer, "  {}", filter.name.bold())?;
            if let Some(ref task_type) = filter.task_type {
                writeln!(writer, "    Task Type: {}", task_type)?;
            }
            if let Some(ref status) = filter.status {
                writeln!(writer, "    Status: {}", status)?;
            }
            if let Some(limit) = filter.limit {
                writeln!(writer, "    Default Limit: {}", limit)?;
            }
            writeln!(writer)?;
        }

        writeln!(writer, "{}", "─".repeat(60))?;
        writeln!(writer, "Total: {} filters", self.total_count)?;

        Ok(())
    }
}
