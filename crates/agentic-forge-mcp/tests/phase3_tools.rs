//! Phase 3: MCP tool tests.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use agentic_forge_mcp::tools::registry::ToolRegistry;
use agentic_forge_mcp::session::SessionManager;

fn create_session() -> Arc<Mutex<SessionManager>> {
    Arc::new(Mutex::new(SessionManager::new()))
}

#[test]
fn test_tool_count() {
    let tools = ToolRegistry::list_tools();
    assert_eq!(tools.len(), 15);
}

#[test]
fn test_tool_names() {
    let tools = ToolRegistry::list_tools();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"forge_blueprint_create"));
    assert!(names.contains(&"forge_blueprint_get"));
    assert!(names.contains(&"forge_blueprint_update"));
    assert!(names.contains(&"forge_blueprint_validate"));
    assert!(names.contains(&"forge_blueprint_list"));
    assert!(names.contains(&"forge_entity_add"));
    assert!(names.contains(&"forge_entity_infer"));
    assert!(names.contains(&"forge_dependency_resolve"));
    assert!(names.contains(&"forge_dependency_add"));
    assert!(names.contains(&"forge_structure_generate"));
    assert!(names.contains(&"forge_skeleton_create"));
    assert!(names.contains(&"forge_test_generate"));
    assert!(names.contains(&"forge_import_graph"));
    assert!(names.contains(&"forge_wiring_create"));
    assert!(names.contains(&"forge_export"));
}

#[test]
fn test_tool_descriptions_verb_first() {
    let tools = ToolRegistry::list_tools();
    for tool in &tools {
        if let Some(ref desc) = tool.description {
            let first_word = desc.split_whitespace().next().unwrap_or("");
            let first_char = first_word.chars().next().unwrap_or(' ');
            assert!(first_char.is_uppercase(), "Tool {} description should start with uppercase verb: {}", tool.name, desc);
            assert!(!desc.ends_with('.'), "Tool {} description should not end with period", tool.name);
        }
    }
}

#[test]
fn test_all_tools_have_input_schema() {
    let tools = ToolRegistry::list_tools();
    for tool in &tools {
        assert!(tool.input_schema.is_object(), "Tool {} should have object input_schema", tool.name);
        assert_eq!(tool.input_schema["type"], "object", "Tool {} input_schema type should be object", tool.name);
    }
}

#[tokio::test]
async fn test_blueprint_create_tool() {
    let session = create_session();
    let result = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "TestProject",
        "description": "A test",
        "domain": "api"
    })), &session).await.unwrap();
    assert!(result.is_error.is_none());
}

#[tokio::test]
async fn test_blueprint_create_missing_name() {
    let session = create_session();
    let result = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "description": "A test",
        "domain": "api"
    })), &session).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_blueprint_list_tool() {
    let session = create_session();
    let result = ToolRegistry::call("forge_blueprint_list", Some(json!({})), &session).await.unwrap();
    assert!(result.is_error.is_none());
}

#[tokio::test]
async fn test_entity_add_tool() {
    let session = create_session();
    // First create a blueprint
    let create_result = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "Test", "description": "Test", "domain": "api"
    })), &session).await.unwrap();
    let text = match &create_result.content[0] { agentic_forge_mcp::types::ToolContent::Text { text } => text.clone() };
    let bp_data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let bp_id = bp_data["blueprint_id"].as_str().unwrap();

    let result = ToolRegistry::call("forge_entity_add", Some(json!({
        "blueprint_id": bp_id,
        "name": "User",
        "description": "A user entity"
    })), &session).await.unwrap();
    assert!(result.is_error.is_none());
}

#[tokio::test]
async fn test_entity_infer_tool() {
    let session = create_session();
    let create_result = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "Test", "description": "Test", "domain": "api"
    })), &session).await.unwrap();
    let text = match &create_result.content[0] { agentic_forge_mcp::types::ToolContent::Text { text } => text.clone() };
    let bp_data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let bp_id = bp_data["blueprint_id"].as_str().unwrap();

    let result = ToolRegistry::call("forge_entity_infer", Some(json!({
        "blueprint_id": bp_id,
        "description": "A system with users and posts"
    })), &session).await.unwrap();
    assert!(result.is_error.is_none());
}

#[tokio::test]
async fn test_unknown_tool() {
    let session = create_session();
    let result = ToolRegistry::call("nonexistent_tool", Some(json!({})), &session).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dependency_add_tool() {
    let session = create_session();
    let create_result = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "Test", "description": "Test", "domain": "api"
    })), &session).await.unwrap();
    let text = match &create_result.content[0] { agentic_forge_mcp::types::ToolContent::Text { text } => text.clone() };
    let bp_data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let bp_id = bp_data["blueprint_id"].as_str().unwrap();

    let result = ToolRegistry::call("forge_dependency_add", Some(json!({
        "blueprint_id": bp_id,
        "name": "serde",
        "version": "1.0"
    })), &session).await.unwrap();
    assert!(result.is_error.is_none());
}

/// Query tools that should have token conservation params.
const QUERY_TOOLS: &[&str] = &[
    "forge_blueprint_get",
    "forge_blueprint_list",
    "forge_blueprint_validate",
    "forge_entity_infer",
    "forge_skeleton_create",
    "forge_test_generate",
    "forge_import_graph",
    "forge_wiring_create",
    "forge_export",
];

/// Mutation tools that must NOT have token conservation params.
const MUTATION_TOOLS: &[&str] = &[
    "forge_blueprint_create",
    "forge_blueprint_update",
    "forge_entity_add",
    "forge_dependency_add",
    "forge_dependency_resolve",
    "forge_structure_generate",
];

const CONSERVATION_PARAMS: &[&str] = &[
    "include_content",
    "intent",
    "since",
    "token_budget",
    "max_results",
    "cursor",
];

#[test]
fn test_query_tools_have_conservation_params() {
    let tools = ToolRegistry::list_tools();
    for tool in &tools {
        if QUERY_TOOLS.contains(&tool.name.as_str()) {
            let props = tool.input_schema["properties"].as_object()
                .unwrap_or_else(|| panic!("Tool {} has no properties", tool.name));
            for param in CONSERVATION_PARAMS {
                assert!(props.contains_key(*param),
                    "Query tool {} missing conservation param '{}'", tool.name, param);
            }
        }
    }
}

#[test]
fn test_mutation_tools_no_conservation_params() {
    let tools = ToolRegistry::list_tools();
    for tool in &tools {
        if MUTATION_TOOLS.contains(&tool.name.as_str()) {
            let props = tool.input_schema["properties"].as_object()
                .unwrap_or_else(|| panic!("Tool {} has no properties", tool.name));
            for param in CONSERVATION_PARAMS {
                assert!(!props.contains_key(*param),
                    "Mutation tool {} should not have conservation param '{}'", tool.name, param);
            }
        }
    }
}

#[test]
fn test_conservation_param_types() {
    let tools = ToolRegistry::list_tools();
    let tool = tools.iter().find(|t| t.name == "forge_blueprint_get").unwrap();
    let props = tool.input_schema["properties"].as_object().unwrap();

    assert_eq!(props["include_content"]["type"], "boolean");
    assert_eq!(props["intent"]["type"], "string");
    assert_eq!(props["since"]["type"], "integer");
    assert_eq!(props["token_budget"]["type"], "integer");
    assert_eq!(props["max_results"]["type"], "integer");
    assert_eq!(props["cursor"]["type"], "string");

    // Check intent enum values
    let intent_enum = props["intent"]["enum"].as_array().unwrap();
    let intent_values: Vec<&str> = intent_enum.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(intent_values.contains(&"exists"));
    assert!(intent_values.contains(&"ids"));
    assert!(intent_values.contains(&"summary"));
    assert!(intent_values.contains(&"full"));
}
