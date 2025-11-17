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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    impl Output for TestData {
        fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
            writeln!(writer, "Name: {}", self.name)?;
            writeln!(writer, "Value: {}", self.value)?;
            Ok(())
        }
    }

    #[test]
    fn test_output_format_enum() {
        assert_eq!(OutputFormat::Human, OutputFormat::Human);
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_eq!(OutputFormat::Yaml, OutputFormat::Yaml);
    }

    #[test]
    fn test_write_human_output() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let mut buffer = Vec::new();
        data.write_human(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Name: test"));
        assert!(output.contains("Value: 42"));
    }

    #[test]
    fn test_write_json_output() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let mut buffer = Vec::new();
        data.write_json(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["name"], "test");
        assert_eq!(parsed["value"], 42);
    }

    #[test]
    fn test_write_yaml_output() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let mut buffer = Vec::new();
        data.write_yaml(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&output).unwrap();
        assert_eq!(parsed["name"].as_str().unwrap(), "test");
        assert_eq!(parsed["value"].as_i64().unwrap(), 42);
    }

    #[test]
    fn test_output_format_write_output() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let format = OutputFormat::Json;
        let mut buffer = Vec::new();
        format.write_output(&data, &mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["name"], "test");
    }

    #[test]
    fn test_output_format_print_output() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let format = OutputFormat::Json;
        // This would normally print to stdout, but we can't easily test that
        // In a real test, we'd capture stdout
        let result = format.print_output(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clap_value_enum() {
        // Test that OutputFormat implements ValueEnum
        let variants = OutputFormat::value_variants();
        assert_eq!(variants.len(), 3);
        assert!(variants.contains(&OutputFormat::Human));
        assert!(variants.contains(&OutputFormat::Json));
        assert!(variants.contains(&OutputFormat::Yaml));
    }
}
