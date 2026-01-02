//! Episode output implementations

use crate::output::Output;
use super::types::{EpisodeSummary, EpisodeList, EpisodeDetail, EpisodeSearchResult};

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
            let status_color = if step.success { Color::Green } else { Color::Red };

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
        writeln!(writer, "  Status: {} | Created: {}", self.status, self.created_at)?;

        if !self.matched_terms.is_empty() {
            writeln!(writer, "  Matched: {}", self.matched_terms.join(", "))?;
        }

        Ok(())
    }
}
