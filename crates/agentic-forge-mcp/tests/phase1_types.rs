//! Phase 1: MCP type tests.

use agentic_forge_mcp::types::*;
use serde_json::json;

#[test]
fn test_request_id_string() {
    let id = RequestId::String("test-123".to_string());
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "\"test-123\"");
    let parsed: RequestId = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, id);
}

#[test]
fn test_request_id_number() {
    let id = RequestId::Number(42);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "42");
    let parsed: RequestId = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, id);
}

#[test]
fn test_request_id_null() {
    let id = RequestId::Null;
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "null");
}

#[test]
fn test_request_id_display() {
    assert_eq!(RequestId::String("a".into()).to_string(), "a");
    assert_eq!(RequestId::Number(1).to_string(), "1");
    assert_eq!(RequestId::Null.to_string(), "null");
}

#[test]
fn test_mcp_error_codes() {
    assert_eq!(error_codes::PARSE_ERROR, -32700);
    assert_eq!(error_codes::INVALID_REQUEST, -32600);
    assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
    assert_eq!(error_codes::INVALID_PARAMS, -32602);
    assert_eq!(error_codes::INTERNAL_ERROR, -32603);
}

#[test]
fn test_mcp_specific_error_codes() {
    assert_eq!(mcp_error_codes::TOOL_NOT_FOUND, -32803);
    assert_eq!(mcp_error_codes::BLUEPRINT_NOT_FOUND, -32850);
    assert_eq!(mcp_error_codes::ENTITY_NOT_FOUND, -32851);
}

#[test]
fn test_mcp_error_code_mapping() {
    let err = McpError::ParseError("bad json".into());
    assert_eq!(err.code(), error_codes::PARSE_ERROR);

    let err = McpError::ToolNotFound("bar".into());
    assert_eq!(err.code(), mcp_error_codes::TOOL_NOT_FOUND);

    let err = McpError::BlueprintNotFound("bp-1".into());
    assert_eq!(err.code(), mcp_error_codes::BLUEPRINT_NOT_FOUND);
}

#[test]
fn test_mcp_error_is_protocol() {
    assert!(McpError::ParseError("x".into()).is_protocol_error());
    assert!(McpError::InvalidRequest("x".into()).is_protocol_error());
    assert!(McpError::MethodNotFound("x".into()).is_protocol_error());
    assert!(!McpError::ToolNotFound("x".into()).is_protocol_error());
    assert!(!McpError::BlueprintNotFound("x".into()).is_protocol_error());
}

#[test]
fn test_mcp_error_to_json_rpc() {
    let err = McpError::ToolNotFound("missing".into());
    let json = err.to_json_rpc_error(RequestId::Number(1));
    assert_eq!(json["error"]["code"], -32803);
}

#[test]
fn test_tool_call_result_text() {
    let result = ToolCallResult::text("hello".into());
    assert!(result.is_error.is_none());
    assert_eq!(result.content.len(), 1);
}

#[test]
fn test_tool_call_result_error() {
    let result = ToolCallResult::error("fail".into());
    assert_eq!(result.is_error, Some(true));
}

#[test]
fn test_tool_call_result_json() {
    let result = ToolCallResult::json(&json!({"key": "value"}));
    assert!(result.is_error.is_none());
    let text = match &result.content[0] {
        ToolContent::Text { text } => text.clone(),
    };
    assert!(text.contains("key"));
}

#[test]
fn test_tool_definition_serialization() {
    let td = ToolDefinition {
        name: "test_tool".into(),
        description: Some("A test tool".into()),
        input_schema: json!({"type": "object"}),
    };
    let json = serde_json::to_value(&td).unwrap();
    assert_eq!(json["name"], "test_tool");
    assert_eq!(json["inputSchema"]["type"], "object");
}

#[test]
fn test_json_rpc_request_serialization() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: RequestId::Number(1),
        method: "test".into(),
        params: Some(json!({"key": "value"})),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["method"], "test");
}

#[test]
fn test_json_rpc_response_success() {
    let resp = JsonRpcResponse::success(RequestId::Number(1), json!({"ok": true}));
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[test]
fn test_json_rpc_response_error() {
    let resp = JsonRpcResponse::error(RequestId::Number(1), -32600, "bad request".into());
    assert!(resp.result.is_none());
    assert!(resp.error.is_some());
    assert_eq!(resp.error.unwrap().code, -32600);
}

#[test]
fn test_server_capabilities_default() {
    let caps = ServerCapabilities::default();
    assert!(caps.tools.is_some());
    assert!(caps.resources.is_some());
    assert!(caps.prompts.is_some());
}

#[test]
fn test_initialize_result_serialization() {
    let result = InitializeResult {
        protocol_version: "2024-11-05".into(),
        capabilities: ServerCapabilities::default(),
        server_info: ServerInfo {
            name: "test".into(),
            version: "0.1.0".into(),
        },
    };
    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["protocolVersion"], "2024-11-05");
    assert_eq!(json["serverInfo"]["name"], "test");
}
