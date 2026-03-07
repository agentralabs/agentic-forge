//! Phase 4: MCP edge case tests — malformed input, missing params, error paths.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use agentic_forge_mcp::tools::registry::ToolRegistry;
use agentic_forge_mcp::protocol::ProtocolHandler;
use agentic_forge_mcp::session::SessionManager;
use agentic_forge_mcp::types::*;

fn session() -> Arc<Mutex<SessionManager>> {
    Arc::new(Mutex::new(SessionManager::new()))
}

fn handler() -> ProtocolHandler {
    ProtocolHandler::new(session())
}

// ── Missing/invalid params ───────────────────────────────────────────

#[tokio::test]
async fn test_blueprint_create_missing_all_params() {
    let s = session();
    let result = ToolRegistry::call("forge_blueprint_create", Some(json!({})), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_blueprint_create_missing_description() {
    let s = session();
    let result = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "Test", "domain": "api"
    })), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_blueprint_create_missing_domain() {
    let s = session();
    let result = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "Test", "description": "test"
    })), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_blueprint_create_invalid_domain() {
    let s = session();
    let result = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "Test", "description": "test", "domain": "nonexistent_domain"
    })), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_blueprint_get_invalid_id() {
    let s = session();
    let result = ToolRegistry::call("forge_blueprint_get", Some(json!({
        "blueprint_id": "not-a-valid-uuid"
    })), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_blueprint_get_nonexistent_id() {
    let s = session();
    let result = ToolRegistry::call("forge_blueprint_get", Some(json!({
        "blueprint_id": "bp-550e8400-e29b-41d4-a716-446655440000"
    })), &s).await;
    // This should return an error result (tool execution error)
    assert!(result.is_err() || result.unwrap().is_error == Some(true));
}

#[tokio::test]
async fn test_blueprint_update_invalid_status() {
    let s = session();
    // Create first
    let create = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "Test", "description": "test", "domain": "api"
    })), &s).await.unwrap();
    let text = match &create.content[0] { ToolContent::Text { text } => text.clone() };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let bp_id = data["blueprint_id"].as_str().unwrap();

    let result = ToolRegistry::call("forge_blueprint_update", Some(json!({
        "blueprint_id": bp_id, "status": "INVALID_STATUS_VALUE"
    })), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_blueprint_list_invalid_status_filter() {
    let s = session();
    let result = ToolRegistry::call("forge_blueprint_list", Some(json!({
        "status": "bogus"
    })), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_entity_add_missing_name() {
    let s = session();
    let create = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "T", "description": "t", "domain": "api"
    })), &s).await.unwrap();
    let text = match &create.content[0] { ToolContent::Text { text } => text.clone() };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let bp_id = data["blueprint_id"].as_str().unwrap();

    let result = ToolRegistry::call("forge_entity_add", Some(json!({
        "blueprint_id": bp_id, "description": "no name"
    })), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dependency_add_missing_version() {
    let s = session();
    let create = ToolRegistry::call("forge_blueprint_create", Some(json!({
        "name": "T", "description": "t", "domain": "api"
    })), &s).await.unwrap();
    let text = match &create.content[0] { ToolContent::Text { text } => text.clone() };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let bp_id = data["blueprint_id"].as_str().unwrap();

    let result = ToolRegistry::call("forge_dependency_add", Some(json!({
        "blueprint_id": bp_id, "name": "serde"
    })), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_call_with_null_arguments() {
    let s = session();
    let result = ToolRegistry::call("forge_blueprint_list", None, &s).await;
    // Should work with None args (defaults to empty object)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tool_call_with_wrong_type_arguments() {
    let s = session();
    let result = ToolRegistry::call("forge_blueprint_create", Some(json!("string_not_object")), &s).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unknown_tool_returns_32803() {
    let s = session();
    let result = ToolRegistry::call("nonexistent_tool", Some(json!({})), &s).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), mcp_error_codes::TOOL_NOT_FOUND);
}

// ── Protocol edge cases ──────────────────────────────────────────────

#[tokio::test]
async fn test_protocol_no_jsonrpc_field() {
    let h = handler();
    let msg = json!({"id": 1, "method": "ping"});
    // Should still work since we only check method
    let result = h.handle_message(msg).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_protocol_null_id() {
    let h = handler();
    let msg = json!({"jsonrpc": "2.0", "id": null, "method": "ping"});
    let result = h.handle_message(msg).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_protocol_string_id() {
    let h = handler();
    let msg = json!({"jsonrpc": "2.0", "id": "abc-123", "method": "ping"});
    let result = h.handle_message(msg).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_protocol_tools_call_no_params() {
    let h = handler();
    let msg = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call"});
    let result = h.handle_message(msg).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_protocol_tools_call_no_tool_name() {
    let h = handler();
    let msg = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"arguments": {}}
    });
    let result = h.handle_message(msg).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_protocol_empty_object() {
    let h = handler();
    let msg = json!({});
    let result = h.handle_message(msg).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_protocol_method_case_sensitive() {
    let h = handler();
    // "PING" != "ping"
    let msg = json!({"jsonrpc": "2.0", "id": 1, "method": "PING"});
    let result = h.handle_message(msg).await;
    assert!(result.is_err());
}

// ── Tool count invariant ─────────────────────────────────────────────

#[test]
fn test_tool_count_is_15() {
    let tools = ToolRegistry::list_tools();
    assert_eq!(tools.len(), 15, "Must have exactly 15 MCP tools, got {}", tools.len());
}

#[test]
fn test_all_tools_have_descriptions() {
    for tool in ToolRegistry::list_tools() {
        assert!(tool.description.is_some(), "Tool {} lacks description", tool.name);
        let desc = tool.description.unwrap();
        assert!(!desc.is_empty(), "Tool {} has empty description", tool.name);
    }
}

#[test]
fn test_no_tool_description_ends_with_period() {
    for tool in ToolRegistry::list_tools() {
        if let Some(desc) = &tool.description {
            assert!(!desc.ends_with('.'), "Tool {} description ends with period: {}", tool.name, desc);
        }
    }
}

#[test]
fn test_all_tool_schemas_have_type_object() {
    for tool in ToolRegistry::list_tools() {
        assert_eq!(tool.input_schema["type"], "object",
            "Tool {} schema type must be 'object'", tool.name);
    }
}

#[test]
fn test_tool_names_are_unique() {
    let tools = ToolRegistry::list_tools();
    let mut seen = std::collections::HashSet::new();
    for tool in &tools {
        assert!(seen.insert(&tool.name), "Duplicate tool name: {}", tool.name);
    }
}

#[test]
fn test_tool_names_follow_forge_prefix() {
    for tool in ToolRegistry::list_tools() {
        assert!(tool.name.starts_with("forge_"), "Tool {} must start with 'forge_'", tool.name);
    }
}
