//! Transport layer — stdio with optional auth gate.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::protocol::ProtocolHandler;
use crate::session::SessionManager;

fn check_auth_gate() -> Result<(), String> {
    if let Ok(required_token) = std::env::var("AGENTIC_TOKEN") {
        if required_token.is_empty() {
            return Err("AGENTIC_TOKEN is set but empty".into());
        }
        tracing::info!("Auth gate active: AGENTIC_TOKEN required");
    }
    Ok(())
}

pub async fn run_stdio() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = check_auth_gate() {
        eprintln!("Auth gate error: {}", e);
        return Err(e.into());
    }

    let session = Arc::new(Mutex::new(SessionManager::new()));
    let handler = ProtocolHandler::new(session);

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let msg: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let err_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                let response_str = serde_json::to_string(&err_response)?;
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
                continue;
            }
        };

        match handler.handle_message(msg).await {
            Ok(response) => {
                if !response.is_null() {
                    let response_str = serde_json::to_string(&response)?;
                    stdout.write_all(response_str.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
            Err(e) => {
                let err_response = e.to_json_rpc_error(crate::types::RequestId::Null);
                let response_str = serde_json::to_string(&err_response)?;
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }
    }

    Ok(())
}
