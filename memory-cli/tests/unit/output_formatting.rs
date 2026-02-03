//! Unit tests for output formatting and serialization.
//!
//! These tests verify that different output formats (human, JSON, YAML)
//! work correctly and produce valid output.

use memory_cli::output::{Output, OutputFormat};
use serde::Serialize;
use std::io::Cursor;

#[cfg(test)]
mod output_formatting_tests {
    use super::*;

    #[derive(Debug, Serialize, Clone)]
    struct TestData {
        name: String,
        value: i32,
        nested: NestedData,
        list: Vec<String>,
    }

    #[derive(Debug, Serialize, Clone)]
    struct NestedData {
        enabled: bool,
        count: usize,
    }

    impl Output for TestData {
        fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
            writeln!(writer, "Test Data:")?;
            writeln!(writer, "  Name: {}", self.name)?;
            writeln!(writer, "  Value: {}", self.value)?;
            writeln!(writer, "  Nested:")?;
            writeln!(writer, "    Enabled: {}", self.nested.enabled)?;
            writeln!(writer, "    Count: {}", self.nested.count)?;
            writeln!(writer, "  List:")?;
            for item in &self.list {
                writeln!(writer, "    - {}", item)?;
            }
            Ok(())
        }
    }

    fn create_test_data() -> TestData {
        TestData {
            name: "test item".to_string(),
            value: 42,
            nested: NestedData {
                enabled: true,
                count: 5,
            },
            list: vec![
                "item1".to_string(),
                "item2".to_string(),
                "item3".to_string(),
            ],
        }
    }

    #[test]
    fn test_human_output_format() {
        let data = create_test_data();
        let mut buffer = Vec::new();

        data.write_human(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Test Data:"));
        assert!(output.contains("Name: test item"));
        assert!(output.contains("Value: 42"));
        assert!(output.contains("Enabled: true"));
        assert!(output.contains("Count: 5"));
        assert!(output.contains("- item1"));
        assert!(output.contains("- item2"));
        assert!(output.contains("- item3"));
    }

    #[test]
    fn test_json_output_format() {
        let data = create_test_data();
        let mut buffer = Vec::new();

        data.write_json(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["name"], "test item");
        assert_eq!(parsed["value"], 42);
        assert_eq!(parsed["nested"]["enabled"], true);
        assert_eq!(parsed["nested"]["count"], 5);
        assert_eq!(parsed["list"].as_array().unwrap().len(), 3);
        assert_eq!(parsed["list"][0], "item1");
    }

    #[test]
    fn test_yaml_output_format() {
        let data = create_test_data();
        let mut buffer = Vec::new();

        data.write_yaml(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Verify it's valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&output).unwrap();
        assert_eq!(parsed["name"].as_str().unwrap(), "test item");
        assert_eq!(parsed["value"].as_i64().unwrap(), 42);
        assert_eq!(parsed["nested"]["enabled"].as_bool().unwrap(), true);
        assert_eq!(parsed["nested"]["count"].as_i64().unwrap(), 5);
        assert_eq!(parsed["list"].as_sequence().unwrap().len(), 3);
        assert_eq!(parsed["list"][0].as_str().unwrap(), "item1");
    }

    #[test]
    fn test_output_format_write_output() {
        let data = create_test_data();

        // Test JSON via OutputFormat
        let mut buffer = Vec::new();
        let format = OutputFormat::Json;
        format.write_output(&data, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        let _: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Test YAML via OutputFormat
        let mut buffer = Vec::new();
        let format = OutputFormat::Yaml;
        format.write_output(&data, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        let _: serde_yaml::Value = serde_yaml::from_str(&output).unwrap();

        // Test human via OutputFormat
        let mut buffer = Vec::new();
        let format = OutputFormat::Human;
        format.write_output(&data, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Test Data:"));
    }

    #[test]
    fn test_output_format_enum_values() {
        assert_eq!(OutputFormat::Human, OutputFormat::Human);
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_eq!(OutputFormat::Yaml, OutputFormat::Yaml);

        // Test that they are different
        assert_ne!(OutputFormat::Human, OutputFormat::Json);
        assert_ne!(OutputFormat::Json, OutputFormat::Yaml);
        assert_ne!(OutputFormat::Yaml, OutputFormat::Human);
    }

    #[test]
    fn test_clap_value_enum_implementation() {
        // Test that OutputFormat implements ValueEnum for clap
        let variants = OutputFormat::value_variants();
        assert_eq!(variants.len(), 3);
        assert!(variants.contains(&OutputFormat::Human));
        assert!(variants.contains(&OutputFormat::Json));
        assert!(variants.contains(&OutputFormat::Yaml));
    }

    #[test]
    fn test_output_with_empty_data() {
        #[derive(Debug, Serialize)]
        struct EmptyData {
            empty_list: Vec<String>,
            empty_string: String,
        }

        impl Output for EmptyData {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                writeln!(writer, "Empty Data:")?;
                writeln!(writer, "  List: {} items", self.empty_list.len())?;
                writeln!(writer, "  String: '{}'", self.empty_string)?;
                Ok(())
            }
        }

        let data = EmptyData {
            empty_list: vec![],
            empty_string: "".to_string(),
        };

        // Test all formats with empty data
        let mut buffer = Vec::new();
        data.write_human(&mut buffer).unwrap();
        let human_output = String::from_utf8(buffer).unwrap();
        assert!(human_output.contains("List: 0 items"));
        assert!(human_output.contains("String: ''"));

        let mut buffer = Vec::new();
        data.write_json(&mut buffer).unwrap();
        let json_output = String::from_utf8(buffer).unwrap();
        let _: serde_json::Value = serde_json::from_str(&json_output).unwrap();

        let mut buffer = Vec::new();
        data.write_yaml(&mut buffer).unwrap();
        let yaml_output = String::from_utf8(buffer).unwrap();
        let _: serde_yaml::Value = serde_yaml::from_str(&yaml_output).unwrap();
    }

    #[test]
    fn test_output_with_special_characters() {
        #[derive(Debug, Serialize)]
        struct SpecialData {
            special_string: String,
            unicode: String,
        }

        impl Output for SpecialData {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                writeln!(writer, "Special: {}", self.special_string)?;
                writeln!(writer, "Unicode: {}", self.unicode)?;
                Ok(())
            }
        }

        let data = SpecialData {
            special_string: "Special chars: <>&\"'".to_string(),
            unicode: "Unicode: 🚀 🔥 💯".to_string(),
        };

        // Test that special characters are handled properly
        let mut buffer = Vec::new();
        data.write_human(&mut buffer).unwrap();
        let human_output = String::from_utf8(buffer).unwrap();
        assert!(human_output.contains("<>&\"'"));
        assert!(human_output.contains("🚀 🔥 💯"));

        // Test JSON serialization with special chars
        let mut buffer = Vec::new();
        data.write_json(&mut buffer).unwrap();
        let json_output = String::from_utf8(buffer).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert_eq!(parsed["special_string"], "Special chars: <>&\"'");
        assert_eq!(parsed["unicode"], "Unicode: 🚀 🔥 💯");

        // Test YAML serialization with special chars
        let mut buffer = Vec::new();
        data.write_yaml(&mut buffer).unwrap();
        let yaml_output = String::from_utf8(buffer).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml_output).unwrap();
        assert_eq!(
            parsed["special_string"].as_str().unwrap(),
            "Special chars: <>&\"'"
        );
        assert_eq!(parsed["unicode"].as_str().unwrap(), "Unicode: 🚀 🔥 💯");
    }

    #[test]
    fn test_output_format_print_output() {
        let data = create_test_data();

        // Test that print_output doesn't panic (we can't easily capture stdout in unit tests)
        let format = OutputFormat::Json;
        let result = format.print_output(&data);
        assert!(result.is_ok());

        let format = OutputFormat::Yaml;
        let result = format.print_output(&data);
        assert!(result.is_ok());

        let format = OutputFormat::Human;
        let result = format.print_output(&data);
        assert!(result.is_ok());
    }
}
