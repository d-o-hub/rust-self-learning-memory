//! Test fixtures and builder patterns for common test data structures

use memory_core::{ComplexityLevel, ExecutionResult, ExecutionStep, TaskContext, TaskType};
use serde_json::json;
use std::collections::HashMap;

/// Builder pattern for creating TaskContext instances in tests
///
/// # Examples
///
/// ```ignore
/// let context = ContextBuilder::new("web-api")
///     .language("rust")
///     .framework("tokio")
///     .complexity(ComplexityLevel::Moderate)
///     .tag("async")
///     .tag("rest")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    language: Option<String>,
    framework: Option<String>,
    complexity: ComplexityLevel,
    domain: String,
    tags: Vec<String>,
}

impl ContextBuilder {
    /// Create a new ContextBuilder with the specified domain
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            language: None,
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: domain.into(),
            tags: Vec::new(),
        }
    }

    /// Set the programming language
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the framework
    pub fn framework(mut self, framework: impl Into<String>) -> Self {
        self.framework = Some(framework.into());
        self
    }

    /// Set the complexity level
    pub fn complexity(mut self, complexity: ComplexityLevel) -> Self {
        self.complexity = complexity;
        self
    }

    /// Add a single tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set all tags at once
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Build the TaskContext
    pub fn build(self) -> TaskContext {
        TaskContext {
            language: self.language,
            framework: self.framework,
            complexity: self.complexity,
            domain: self.domain,
            tags: self.tags,
        }
    }
}

/// Builder pattern for creating ExecutionStep instances in tests
///
/// # Examples
///
/// ```ignore
/// let step = StepBuilder::new(1, "file_reader", "Read config file")
///     .latency_ms(150)
///     .tokens_used(100)
///     .success("Config loaded successfully")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct StepBuilder {
    step_number: usize,
    tool: String,
    action: String,
    parameters: serde_json::Value,
    result: Option<ExecutionResult>,
    latency_ms: u64,
    tokens_used: Option<usize>,
    metadata: HashMap<String, String>,
}

impl StepBuilder {
    /// Create a new StepBuilder
    pub fn new(
        step_number: usize,
        tool: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            step_number,
            tool: tool.into(),
            action: action.into(),
            parameters: json!({}),
            result: None,
            latency_ms: 10,
            tokens_used: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the parameters as JSON
    pub fn parameters(mut self, parameters: serde_json::Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Set a successful result
    pub fn success(mut self, output: impl Into<String>) -> Self {
        self.result = Some(ExecutionResult::Success {
            output: output.into(),
        });
        self
    }

    /// Set an error result
    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.result = Some(ExecutionResult::Error {
            message: message.into(),
        });
        self
    }

    /// Set the latency in milliseconds
    pub fn latency_ms(mut self, latency: u64) -> Self {
        self.latency_ms = latency;
        self
    }

    /// Set the number of tokens used
    pub fn tokens_used(mut self, tokens: usize) -> Self {
        self.tokens_used = Some(tokens);
        self
    }

    /// Add a metadata entry
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the ExecutionStep
    pub fn build(self) -> ExecutionStep {
        ExecutionStep {
            step_number: self.step_number,
            timestamp: chrono::Utc::now(),
            tool: self.tool,
            action: self.action,
            parameters: self.parameters,
            result: self.result,
            latency_ms: self.latency_ms,
            tokens_used: self.tokens_used,
            metadata: self.metadata,
        }
    }
}

/// Pre-configured test contexts for common scenarios

/// Standard Rust/Tokio test context
pub fn rust_context() -> TaskContext {
    ContextBuilder::new("testing")
        .language("rust")
        .framework("tokio")
        .complexity(ComplexityLevel::Moderate)
        .tag("test")
        .build()
}

/// API testing context
pub fn api_context() -> TaskContext {
    ContextBuilder::new("api-testing")
        .language("rust")
        .framework("tokio")
        .complexity(ComplexityLevel::Moderate)
        .tag("api")
        .tag("integration")
        .build()
}

/// Data processing context
pub fn data_processing_context() -> TaskContext {
    ContextBuilder::new("data-processing")
        .language("rust")
        .complexity(ComplexityLevel::Simple)
        .tag("batch")
        .build()
}

/// Error handling context
pub fn error_handling_context() -> TaskContext {
    ContextBuilder::new("error-handling")
        .language("rust")
        .complexity(ComplexityLevel::Moderate)
        .tag("retry")
        .tag("recovery")
        .build()
}

/// Generic test context (for backwards compatibility)
pub fn test_context() -> TaskContext {
    rust_context()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let context = ContextBuilder::new("web-api")
            .language("rust")
            .framework("axum")
            .complexity(ComplexityLevel::Complex)
            .tag("rest")
            .tag("async")
            .build();

        assert_eq!(context.domain, "web-api");
        assert_eq!(context.language, Some("rust".to_string()));
        assert_eq!(context.framework, Some("axum".to_string()));
        assert_eq!(context.complexity, ComplexityLevel::Complex);
        assert_eq!(context.tags.len(), 2);
    }

    #[test]
    fn test_step_builder() {
        let step = StepBuilder::new(1, "test_tool", "test action")
            .latency_ms(100)
            .tokens_used(50)
            .success("OK")
            .metadata("key", "value")
            .build();

        assert_eq!(step.step_number, 1);
        assert_eq!(step.tool, "test_tool");
        assert_eq!(step.latency_ms, 100);
        assert_eq!(step.tokens_used, Some(50));
        assert!(step.is_success());
        assert_eq!(step.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_predefined_contexts() {
        let rust_ctx = rust_context();
        assert_eq!(rust_ctx.language, Some("rust".to_string()));

        let api_ctx = api_context();
        assert_eq!(api_ctx.domain, "api-testing");

        let data_ctx = data_processing_context();
        assert_eq!(data_ctx.domain, "data-processing");

        let error_ctx = error_handling_context();
        assert!(error_ctx.tags.contains(&"retry".to_string()));
    }
}
