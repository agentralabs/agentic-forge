//! MCP protocol handler.

pub mod compact;

use crate::session::SessionManager;
use crate::tools::registry::ToolRegistry;
use crate::types::*;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ProtocolHandler {
    session: Arc<Mutex<SessionManager>>,
}

impl ProtocolHandler {
    pub fn new(session: Arc<Mutex<SessionManager>>) -> Self {
        Self { session }
    }

    pub async fn handle_message(&self, msg: Value) -> McpResult<Value> {
        let method = msg
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidRequest("missing method".into()))?;
        let id = msg
            .get("id")
            .cloned()
            .and_then(|v| serde_json::from_value::<RequestId>(v).ok())
            .unwrap_or(RequestId::Null);
        let params = msg.get("params").cloned();

        match method {
            "initialize" => self.handle_initialize(id, params).await,
            "initialized" | "notifications/initialized" => Ok(json!(null)),
            "tools/list" => self.handle_tools_list(id).await,
            "tools/call" => self.handle_tools_call(id, params).await,
            "resources/list" => self.handle_resources_list(id).await,
            "prompts/list" => self.handle_prompts_list(id).await,
            "ping" => Ok(json!({"jsonrpc": "2.0", "id": id, "result": {}})),
            _ => Err(McpError::MethodNotFound(method.to_string())),
        }
    }

    async fn handle_initialize(&self, id: RequestId, _params: Option<Value>) -> McpResult<Value> {
        let mut session = self.session.lock().await;
        session.initialize();

        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "subscribe": false, "listChanged": false },
                    "prompts": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "agentic-forge-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        }))
    }

    async fn handle_tools_list(&self, id: RequestId) -> McpResult<Value> {
        if compact::mcp_tool_surface_is_compact() {
            let tools = compact::compact_tool_definitions();
            return Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "tools": tools }
            }));
        }

        let tools = ToolRegistry::list_tools();
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": tools }
        }))
    }

    async fn handle_tools_call(&self, id: RequestId, params: Option<Value>) -> McpResult<Value> {
        let params = params.ok_or_else(|| McpError::InvalidParams("missing params".into()))?;
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("missing tool name".into()))?;
        let arguments = params.get("arguments").cloned();

        // Normalize compact facade calls to canonical tool names
        let (name, arguments) = match compact::normalize_compact_tool_call(
            name,
            arguments.clone().unwrap_or_else(|| json!({})),
        ) {
            Ok((n, a)) => (n, Some(a)),
            Err(msg) => return Err(McpError::InvalidParams(msg)),
        };

        match ToolRegistry::call(&name, arguments, &self.session).await {
            Ok(result) => Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            })),
            Err(e) if e.is_protocol_error() => Err(e),
            Err(e) => Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": ToolCallResult::error(e.to_string())
            })),
        }
    }

    async fn handle_resources_list(&self, id: RequestId) -> McpResult<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "resources": [] }
        }))
    }

    async fn handle_prompts_list(&self, id: RequestId) -> McpResult<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "prompts": [] }
        }))
    }
}
