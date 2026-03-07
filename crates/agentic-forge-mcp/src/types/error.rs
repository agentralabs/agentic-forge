//! MCP error types and error codes.

use serde::{Deserialize, Serialize};

pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

pub mod mcp_error_codes {
    pub const REQUEST_CANCELLED: i32 = -32800;
    pub const CONTENT_TOO_LARGE: i32 = -32801;
    pub const RESOURCE_NOT_FOUND: i32 = -32802;
    pub const TOOL_NOT_FOUND: i32 = -32803;
    pub const PROMPT_NOT_FOUND: i32 = -32804;
    pub const BLUEPRINT_NOT_FOUND: i32 = -32850;
    pub const ENTITY_NOT_FOUND: i32 = -32851;
    pub const INVALID_BLUEPRINT: i32 = -32852;
    pub const UNAUTHORIZED: i32 = -32900;
    pub const RATE_LIMITED: i32 = -32902;
}

#[derive(thiserror::Error, Debug)]
pub enum McpError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    #[error("Invalid params: {0}")]
    InvalidParams(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Request cancelled")]
    RequestCancelled,
    #[error("Content too large: {size} bytes exceeds {max} bytes")]
    ContentTooLarge { size: usize, max: usize },
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Prompt not found: {0}")]
    PromptNotFound(String),
    #[error("Blueprint not found: {0}")]
    BlueprintNotFound(String),
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    #[error("Invalid blueprint: {0}")]
    InvalidBlueprint(String),
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Forge error: {0}")]
    Forge(String),
    #[error("Unauthorized")]
    Unauthorized,
}

impl McpError {
    pub fn is_protocol_error(&self) -> bool {
        matches!(self, Self::ParseError(_) | Self::InvalidRequest(_) | Self::MethodNotFound(_) | Self::InvalidParams(_) | Self::InternalError(_))
    }

    pub fn code(&self) -> i32 {
        match self {
            Self::ParseError(_) => error_codes::PARSE_ERROR,
            Self::InvalidRequest(_) => error_codes::INVALID_REQUEST,
            Self::MethodNotFound(_) => error_codes::METHOD_NOT_FOUND,
            Self::InvalidParams(_) => error_codes::INVALID_PARAMS,
            Self::InternalError(_) => error_codes::INTERNAL_ERROR,
            Self::RequestCancelled => mcp_error_codes::REQUEST_CANCELLED,
            Self::ContentTooLarge { .. } => mcp_error_codes::CONTENT_TOO_LARGE,
            Self::ResourceNotFound(_) => mcp_error_codes::RESOURCE_NOT_FOUND,
            Self::ToolNotFound(_) => mcp_error_codes::TOOL_NOT_FOUND,
            Self::PromptNotFound(_) => mcp_error_codes::PROMPT_NOT_FOUND,
            Self::BlueprintNotFound(_) => mcp_error_codes::BLUEPRINT_NOT_FOUND,
            Self::EntityNotFound(_) => mcp_error_codes::ENTITY_NOT_FOUND,
            Self::InvalidBlueprint(_) => mcp_error_codes::INVALID_BLUEPRINT,
            Self::Transport(_) | Self::Io(_) | Self::Json(_) | Self::Forge(_) => error_codes::INTERNAL_ERROR,
            Self::Unauthorized => mcp_error_codes::UNAUTHORIZED,
        }
    }

    pub fn to_json_rpc_error(&self, id: RequestId) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": self.code(),
                "message": self.to_string()
            }
        })
    }
}

pub type McpResult<T> = Result<T, McpError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
    Null,
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Null => write!(f, "null"),
        }
    }
}
