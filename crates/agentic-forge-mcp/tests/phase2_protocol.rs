//! Phase 2: MCP protocol handler tests.

use agentic_forge_mcp::protocol::ProtocolHandler;
use agentic_forge_mcp::session::SessionManager;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

fn create_handler() -> ProtocolHandler {
    let session = Arc::new(Mutex::new(SessionManager::new()));
    ProtocolHandler::new(session)
}

#[tokio::test]
async fn test_initialize() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        }
    });
    let response = handler.handle_message(msg).await.unwrap();
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
}

#[tokio::test]
async fn test_tools_list() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });
    let response = handler.handle_message(msg).await.unwrap();
    let tools = response["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 15);
}

#[tokio::test]
async fn test_tools_call_create_blueprint() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "forge_blueprint_create",
            "arguments": {
                "name": "TestProject",
                "description": "A test project",
                "domain": "api"
            }
        }
    });
    let response = handler.handle_message(msg).await.unwrap();
    assert!(response["result"]["content"].is_array());
}

#[tokio::test]
async fn test_tools_call_unknown_tool() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "nonexistent_tool",
            "arguments": {}
        }
    });
    let response = handler.handle_message(msg).await.unwrap();
    assert_eq!(response["result"]["isError"], true);
}

#[tokio::test]
async fn test_method_not_found() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "nonexistent/method"
    });
    let result = handler.handle_message(msg).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ping() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping"
    });
    let response = handler.handle_message(msg).await.unwrap();
    assert!(response["result"].is_object());
}

#[tokio::test]
async fn test_resources_list() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/list"
    });
    let response = handler.handle_message(msg).await.unwrap();
    assert!(response["result"]["resources"].is_array());
}

#[tokio::test]
async fn test_prompts_list() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "prompts/list"
    });
    let response = handler.handle_message(msg).await.unwrap();
    assert!(response["result"]["prompts"].is_array());
}

#[tokio::test]
async fn test_initialized_notification() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "method": "initialized"
    });
    let response = handler.handle_message(msg).await.unwrap();
    assert!(response.is_null());
}

#[tokio::test]
async fn test_missing_method() {
    let handler = create_handler();
    let msg = json!({
        "jsonrpc": "2.0",
        "id": 1
    });
    let result = handler.handle_message(msg).await;
    assert!(result.is_err());
}
