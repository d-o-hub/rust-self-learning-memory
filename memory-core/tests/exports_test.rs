//! Test to verify that symbols used in README.md examples are exported.
use do_memory_core::{SelfLearningMemory, TaskContext, ComplexityLevel, TaskType};

#[test]
fn verify_readme_symbols_exported() {
    // This test ensures that the symbols used in the README.md examples
    // are correctly exported from the root of do_memory_core.
    // We just need to verify they can be referenced.
    let _ = TaskType::CodeGeneration;
    let _ = ComplexityLevel::Moderate;

    // We instantiate TaskContext to prove it's usable as documented
    let _context = TaskContext {
        domain: "test".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec!["test".to_string()],
    };

    // SelfLearningMemory is an async struct, we just verify its type exists
    let _verify_type: Option<SelfLearningMemory> = None;
}
