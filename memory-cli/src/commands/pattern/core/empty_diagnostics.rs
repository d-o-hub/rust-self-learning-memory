//! ADR-076 empty-result diagnostics for pattern list/search (human format).

use std::io::Write;

/// Human-mode diagnostic footer when pattern list/search returns zero results.
///
/// Explains the learning contract: patterns come from episode complete (with
/// steps for tool sequences), not from `storage sync`. JSON/YAML callers must
/// not invoke this — empty arrays stay machine-stable.
pub fn write_empty_pattern_diagnostics<W: Write>(mut writer: W) -> std::io::Result<()> {
    writeln!(writer)?;
    writeln!(writer, "No durable patterns found.")?;
    writeln!(writer)?;
    writeln!(writer, "Hints:")?;
    writeln!(
        writer,
        "  • Patterns are created on episode complete (not on `storage sync`)"
    )?;
    writeln!(
        writer,
        "  • Tool-sequence patterns need ≥1 step (`episode log-step`) before complete"
    )?;
    writeln!(
        writer,
        "  • Confirm the same `--db-path` / config as the process that completed episodes"
    )?;
    writeln!(
        writer,
        "  • Inspect effective config: `do-memory-cli config show`"
    )?;
    Ok(())
}

/// Print empty-pattern diagnostics to stdout (human CLI path).
pub fn print_empty_pattern_diagnostics() {
    let _ = write_empty_pattern_diagnostics(std::io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_diagnostics_cover_adr076_contract() {
        let mut buf = Vec::new();
        write_empty_pattern_diagnostics(&mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();

        assert!(text.contains("No durable patterns found"));
        assert!(text.contains("episode complete"));
        assert!(text.contains("storage sync"));
        assert!(text.contains("log-step"));
        assert!(text.contains("--db-path"));
        assert!(text.contains("config show"));
    }
}
