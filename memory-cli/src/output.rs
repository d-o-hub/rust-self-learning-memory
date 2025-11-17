use clap::ValueEnum;
use serde::Serialize;
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output with colors and formatting
    Human,
    /// JSON output for scripting
    Json,
    /// YAML output for configuration
    Yaml,
}

pub trait Output {
    fn write<W: Write>(&self, writer: W, format: &OutputFormat) -> anyhow::Result<()>
    where
        Self: Serialize,
    {
        match format {
            OutputFormat::Human => self.write_human(writer),
            OutputFormat::Json => self.write_json(writer),
            OutputFormat::Yaml => self.write_yaml(writer),
        }
    }

    fn write_human<W: Write>(&self, writer: W) -> anyhow::Result<()>;
    fn write_json<W: Write>(&self, writer: W) -> anyhow::Result<()>
    where
        Self: Serialize,
    {
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    fn write_yaml<W: Write>(&self, writer: W) -> anyhow::Result<()>
    where
        Self: Serialize,
    {
        serde_yaml::to_writer(writer, self)?;
        Ok(())
    }
}

impl OutputFormat {
    pub fn write_output<T: Output + Serialize, W: Write>(
        &self,
        data: &T,
        writer: W,
    ) -> anyhow::Result<()> {
        data.write(writer, self)
    }

    pub fn print_output<T: Output + Serialize>(&self, data: &T) -> anyhow::Result<()> {
        self.write_output(data, io::stdout())
    }
}
