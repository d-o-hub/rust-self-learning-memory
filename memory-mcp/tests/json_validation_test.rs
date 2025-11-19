use memory_core::SelfLearningMemory;
use memory_mcp::{ExecutionContext, MemoryMCPServer, SandboxConfig};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_json_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing MCP Tools JSON Response Validation\n");

    // Create MCP server
    let memory = Arc::new(SelfLearningMemory::new());
    let server = MemoryMCPServer::new(SandboxConfig::default(), memory).await?;

    let mut test_results = Vec::new();

    // Test 1: query_memory tool
    println!("🔍 Testing query_memory tool...");
    match test_query_memory(&server).await {
        Ok(_) => test_results.push(("query_memory", "PASS", "Valid JSON response".to_string())),
        Err(e) => test_results.push(("query_memory", "FAIL", format!("Error: {}", e))),
    }

    // Test 2: execute_agent_code tool
    println!("⚙️ Testing execute_agent_code tool...");
    match test_execute_agent_code(&server).await {
        Ok(_) => test_results.push((
            "execute_agent_code",
            "PASS",
            "Valid JSON response".to_string(),
        )),
        Err(e) => test_results.push(("execute_agent_code", "FAIL", format!("Error: {}", e))),
    }

    // Test 3: analyze_patterns tool
    println!("📊 Testing analyze_patterns tool...");
    match test_analyze_patterns(&server).await {
        Ok(_) => test_results.push((
            "analyze_patterns",
            "PASS",
            "Valid JSON response".to_string(),
        )),
        Err(e) => test_results.push(("analyze_patterns", "FAIL", format!("Error: {}", e))),
    }

    // Test 4: list_tools
    println!("📋 Testing list_tools...");
    match test_list_tools(&server).await {
        Ok(_) => test_results.push(("list_tools", "PASS", "Valid JSON response".to_string())),
        Err(e) => test_results.push(("list_tools", "FAIL", format!("Error: {}", e))),
    }

    // Report results
    println!("\n📊 Test Results Summary");
    println!("{}", "=".repeat(50));

    let passed = test_results
        .iter()
        .filter(|(_, status, _)| *status == "PASS")
        .count();
    let failed = test_results
        .iter()
        .filter(|(_, status, _)| *status == "FAIL")
        .count();
    let total = test_results.len();

    println!("Total Tests: {}", total);
    println!("✅ Passed: {}", passed);
    println!("❌ Failed: {}", failed);

    if failed > 0 {
        println!("\n❌ Failed Tests:");
        for (test, status, details) in &test_results {
            if *status == "FAIL" {
                println!("  - {}: {}", test, details);
            }
        }
    }

    println!("\n{}", "=".repeat(50));

    if failed == 0 {
        println!("🎉 All MCP tools return valid JSON responses!");
    } else {
        println!("⚠️ Some tests failed. Check the details above.");
        panic!("Tests failed");
    }

    Ok(())
}

async fn test_query_memory(server: &MemoryMCPServer) -> Result<(), Box<dyn std::error::Error>> {
    let result = server
        .query_memory(
            "test query".to_string(),
            "testing".to_string(),
            Some("code_generation".to_string()),
            5,
        )
        .await?;

    // Validate JSON structure
    validate_json_structure(&result, "query_memory")?;

    // Check expected fields
    if result.get("episodes").is_none() {
        return Err("Missing 'episodes' field".into());
    }
    if result.get("patterns").is_none() {
        return Err("Missing 'patterns' field".into());
    }
    if result.get("insights").is_none() {
        return Err("Missing 'insights' field".into());
    }

    Ok(())
}

async fn test_execute_agent_code(
    server: &MemoryMCPServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let code = "return { success: true, value: 42 };";
    let context = ExecutionContext::new("test execution".to_string(), json!({ "test": "data" }));

    let result = server.execute_agent_code(code.to_string(), context).await?;

    // Validate JSON serialization
    let serialized = serde_json::to_string(&result)?;
    let _: serde_json::Value = serde_json::from_str(&serialized)?;

    Ok(())
}

async fn test_analyze_patterns(server: &MemoryMCPServer) -> Result<(), Box<dyn std::error::Error>> {
    let result = server
        .analyze_patterns("code_generation".to_string(), 0.7, 10)
        .await?;

    // Validate JSON structure
    validate_json_structure(&result, "analyze_patterns")?;

    // Check expected fields
    if result.get("patterns").is_none() {
        return Err("Missing 'patterns' field".into());
    }
    if result.get("statistics").is_none() {
        return Err("Missing 'statistics' field".into());
    }

    Ok(())
}

async fn test_list_tools(server: &MemoryMCPServer) -> Result<(), Box<dyn std::error::Error>> {
    let tools = server.list_tools().await;

    // Validate each tool has required fields
    for tool in &tools {
        if tool.name.is_empty() {
            return Err("Tool missing name".into());
        }
        if tool.description.is_empty() {
            return Err("Tool missing description".into());
        }

        // Validate input schema is valid JSON
        let _: serde_json::Value = serde_json::from_value(tool.input_schema.clone())?;
    }

    // Should have at least the 3 default tools
    if tools.len() < 3 {
        return Err(format!("Expected at least 3 tools, got {}", tools.len()).into());
    }

    Ok(())
}

fn validate_json_structure(
    value: &serde_json::Value,
    test_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to serialize and deserialize to ensure it's valid JSON
    let serialized = serde_json::to_string(value)?;
    let _: serde_json::Value = serde_json::from_str(&serialized)?;

    // Check it's an object
    if !value.is_object() {
        return Err(format!("{}: Expected JSON object, got {}", test_name, value).into());
    }

    Ok(())
}
