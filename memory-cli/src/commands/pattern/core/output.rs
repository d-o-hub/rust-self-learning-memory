//! Pattern output implementations

use super::decay::DecayResult;
use super::types::{
    EffectivenessRankings, PatternAnalysisResult, PatternDetail, PatternList,
    PatternSummary,
};
use crate::output::Output;

impl Output for PatternSummary {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        let confidence_color = match self.confidence {
            c if c >= 0.8 => Color::Green,
            c if c >= 0.6 => Color::Yellow,
            _ => Color::Red,
        };

        let effectiveness_color = match self.effectiveness {
            e if e >= 0.8 => Color::Green,
            e if e >= 0.6 => Color::Yellow,
            _ => Color::Red,
        };

        writeln!(
            writer,
            "{} ({})",
            self.pattern_id[..8].to_string().dimmed(),
            self.pattern_type
        )?;
        writeln!(writer, "  Description: {}", self.description)?;
        writeln!(
            writer,
            "  Confidence: {:.2} {}",
            self.confidence,
            "●".color(confidence_color)
        )?;
        writeln!(
            writer,
            "  Effectiveness: {:.2} {}",
            self.effectiveness,
            "●".color(effectiveness_color)
        )?;
        writeln!(
            writer,
            "  Uses: {}, Last: {}",
            self.use_count, self.last_used
        )?;
        Ok(())
    }
}

impl Output for PatternList {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(
            writer,
            "{} patterns (showing {})",
            self.total_count,
            self.patterns.len()
        )?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for pattern in &self.patterns {
            let (confidence_color, confidence_icon) = match pattern.confidence {
                c if c >= 0.8 => (Color::Green, "●"),
                c if c >= 0.6 => (Color::Yellow, "○"),
                _ => (Color::Red, "○"),
            };

            let confidence_display = format!("{:.2} {}", pattern.confidence, confidence_icon);

            writeln!(
                writer,
                "{} {} {} {} uses",
                pattern.pattern_id[..8].to_string().dimmed(),
                confidence_display.color(confidence_color).bold(),
                pattern.pattern_type.dimmed(),
                pattern.use_count.to_string().color(confidence_color)
            )?;
        }

        Ok(())
    }
}

impl Output for EffectivenessRankings {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(
            writer,
            "{}",
            "Pattern Effectiveness Rankings".bold().underline()
        )?;
        writeln!(
            writer,
            "Analyzed {} patterns\n",
            self.total_patterns_analyzed
        )?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for ranking in &self.rankings {
            let effectiveness_color = match ranking.effectiveness_score {
                e if e >= 0.8 => Color::Green,
                e if e >= 0.6 => Color::Yellow,
                _ => Color::Red,
            };

            let rank_display = format!("#{:>2}", ranking.rank);
            writeln!(
                writer,
                "{} {} [{:.2}] {}",
                rank_display.bold(),
                ranking.pattern_id[..8].to_string().dimmed(),
                format!("{:.2}", ranking.effectiveness_score).color(effectiveness_color),
                ranking.description
            )?;
            writeln!(
                writer,
                "      Uses: {}",
                ranking.use_count.to_string().color(effectiveness_color)
            )?;
        }

        Ok(())
    }
}

impl Output for PatternDetail {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Pattern Details".bold().underline())?;
        writeln!(writer, "ID: {}", self.id)?;
        writeln!(writer, "Type: {}", self.pattern_type)?;
        writeln!(writer, "Confidence: {:.2}", self.confidence)?;
        writeln!(writer, "Extracted: {}", self.extracted_at)?;
        Ok(())
    }
}

impl Output for PatternAnalysisResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Pattern Analysis".bold().underline())?;
        writeln!(writer, "Pattern ID: {}", self.pattern_id)?;
        writeln!(writer, "Success Rate: {:.2}", self.analysis.success_rate)?;
        writeln!(
            writer,
            "Avg Improvement: {:.2}",
            self.analysis.average_improvement
        )?;
        writeln!(
            writer,
            "Episodes Analyzed: {}",
            self.analysis.episodes_analyzed
        )?;
        writeln!(writer)?;
        writeln!(writer, "Recommendations:")?;
        for (i, rec) in self.analysis.recommendations.iter().enumerate() {
            writeln!(writer, "  {}. {}", i + 1, rec)?;
        }
        Ok(())
    }
}

impl Output for DecayResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Pattern Decay Results".bold().underline())?;
        writeln!(
            writer,
            "Analyzed {} patterns, would decay {}",
            self.total_patterns_analyzed, self.would_decay_count
        )?;
        writeln!(writer, "Dry run: {}", self.dry_run)?;
        Ok(())
    }
}
