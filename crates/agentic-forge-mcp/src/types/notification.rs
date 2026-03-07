//! MCP notification types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotification {
    pub level: String,
    pub logger: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNotification {
    #[serde(rename = "progressToken")]
    pub progress_token: String,
    pub progress: f64,
    pub total: Option<f64>,
}
